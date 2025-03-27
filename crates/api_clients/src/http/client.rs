//! Default HTTP client implementation

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authenticator;
use crate::{Error, Result};

use super::{HttpClient, HttpClientConfig, Middleware};

/// Default implementation of the HttpClient trait
pub struct DefaultHttpClient {
    /// The underlying reqwest client
    client: Client,
    /// The base URL for the API
    base_url: Option<String>,
    /// Authenticator for the client
    authenticator: Option<Arc<dyn Authenticator>>,
    /// Middleware for the client
    middleware: Vec<Box<dyn Middleware>>,
}

impl DefaultHttpClient {
    /// Create a new HTTP client
    pub fn new(
        config: HttpClientConfig,
        authenticator: Option<Arc<dyn Authenticator>>,
    ) -> Self {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .user_agent(format!("squirrel-api-client/{}", env!("CARGO_PKG_VERSION")));

        if let Some(proxy) = config.proxy.as_ref() {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy).unwrap());
        }

        // Add other configuration options as needed
        if !config.default_headers.is_empty() {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in config.default_headers {
                headers.insert(
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    reqwest::header::HeaderValue::from_str(&value).unwrap(),
                );
            }
            client_builder = client_builder.default_headers(headers);
        }

        Self {
            client: client_builder.build().unwrap(),
            base_url: config.base_url,
            authenticator,
            middleware: config.middleware,
        }
    }

    /// Build the full URL from the base URL and path
    fn build_url(&self, path: &str) -> Result<String> {
        if path.starts_with("http://") || path.starts_with("https://") {
            Ok(path.to_string())
        } else if let Some(base_url) = &self.base_url {
            let base = base_url.trim_end_matches('/');
            let path = path.trim_start_matches('/');
            Ok(format!("{base}/{path}"))
        } else {
            Err(Error::Url(url::ParseError::RelativeUrlWithoutBase))
        }
    }

    /// Apply all middleware to the request builder
    async fn apply_middleware(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        let mut current_builder = builder;
        for middleware in &self.middleware {
            current_builder = middleware.process_request(current_builder).await?;
        }
        Ok(current_builder)
    }

    /// Process the response through middleware
    async fn process_response(&self, response: Response) -> Result<Response> {
        let mut current_response = response;
        for middleware in &self.middleware {
            current_response = middleware.process_response(current_response).await?;
        }
        Ok(current_response)
    }

    /// Check if the response indicates an error
    fn check_response(&self, response: &Response) -> Result<()> {
        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            Err(Error::ApiError {
                status: status.as_u16(),
                message: format!("API returned error status: {}", status),
            })
        }
    }
}

#[async_trait]
impl HttpClient for DefaultHttpClient {
    fn request(&self, method: Method, url: &str) -> Result<RequestBuilder> {
        let full_url = self.build_url(url)?;
        let mut builder = self.client.request(method, full_url);
        
        // Apply authentication if available
        if let Some(auth) = &self.authenticator {
            builder = auth.authenticate(builder);
        }
        
        Ok(builder)
    }
    
    async fn send_raw(&self, builder: RequestBuilder) -> Result<Response> {
        let builder = self.apply_middleware(builder).await?;
        let response = builder.send().await.map_err(Error::Http)?;
        let response = self.process_response(response).await?;
        
        self.check_response(&response)?;
        
        Ok(response)
    }
    
    async fn send_json<T: DeserializeOwned + Send + 'static>(
        &self, 
        builder: RequestBuilder
    ) -> Result<T> {
        let response = self.send_raw(builder).await?;
        let json = response.json::<T>().await.map_err(Error::Http)?;
        Ok(json)
    }
    
    async fn send_bytes(&self, builder: RequestBuilder) -> Result<Bytes> {
        let response = self.send_raw(builder).await?;
        let bytes = response.bytes().await.map_err(Error::Http)?;
        Ok(bytes)
    }
    
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> Result<T> {
        let builder = self.request(Method::GET, url)?;
        self.send_json(builder).await
    }
    
    async fn post_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let builder = self.request(Method::POST, url)?.json(body);
        self.send_json(builder).await
    }
    
    async fn put_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let builder = self.request(Method::PUT, url)?.json(body);
        self.send_json(builder).await
    }
    
    async fn patch_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let builder = self.request(Method::PATCH, url)?.json(body);
        self.send_json(builder).await
    }
    
    async fn delete(&self, url: &str) -> Result<()> {
        let builder = self.request(Method::DELETE, url)?;
        self.send_raw(builder).await?;
        Ok(())
    }
} 