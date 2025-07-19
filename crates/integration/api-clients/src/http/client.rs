//! Default HTTP client implementation

use async_trait::async_trait;
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

use super::{HttpClient, HttpClientExt};
use crate::auth::Authenticator;
use crate::Result;

/// Default HTTP client implementation
#[derive(Clone)]
pub struct DefaultHttpClient {
    client: Client,
    base_url: Option<String>,
    authenticator: Option<Arc<dyn Authenticator>>,
}

impl Default for DefaultHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHttpClient {
    /// Create a new HTTP client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: None,
            authenticator: None,
        }
    }

    /// Set the base URL for all requests
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the authenticator for all requests
    pub fn with_authenticator(mut self, authenticator: Arc<dyn Authenticator>) -> Self {
        self.authenticator = Some(authenticator);
        self
    }

    /// Build the full URL for a request
    fn build_url(&self, path: &str) -> String {
        if let Some(base_url) = &self.base_url {
            if path.starts_with('/') {
                format!("{}{}", base_url.trim_end_matches('/'), path)
            } else {
                format!("{}/{}", base_url.trim_end_matches('/'), path)
            }
        } else {
            path.to_string()
        }
    }

    /// Send a request and return the response
    async fn send_request(&self, mut request: RequestBuilder) -> Result<RequestBuilder> {
        if let Some(auth) = &self.authenticator {
            let mut headers = std::collections::HashMap::new();
            auth.authenticate(&mut headers);

            // Apply headers to request
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        Ok(request)
    }
}

#[async_trait]
impl HttpClient for DefaultHttpClient {
    async fn get(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.build_url(path);
        Ok(self.client.get(url))
    }

    async fn post(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.build_url(path);
        Ok(self.client.post(url))
    }

    async fn put(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.build_url(path);
        Ok(self.client.put(url))
    }

    async fn delete(&self, path: &str) -> Result<RequestBuilder> {
        let url = self.build_url(path);
        Ok(self.client.delete(url))
    }
}

#[async_trait]
impl HttpClientExt for DefaultHttpClient {
    async fn send_json<T: DeserializeOwned + Send + 'static>(
        &self,
        request: RequestBuilder,
    ) -> Result<T> {
        let response = self.send_request(request).await?.send().await?;
        Ok(response.json().await?)
    }

    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> Result<T> {
        let request = self.get(url).await?;
        self.send_json(request).await
    }

    async fn post_json<
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Send + Sync + 'static,
    >(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let request = self.post(url).await?.json(body);
        self.send_json(request).await
    }

    async fn put_json<
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Send + Sync + 'static,
    >(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T> {
        let request = self.put(url).await?.json(body);
        self.send_json(request).await
    }
}
