//! Default HTTP client implementation

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

impl HttpClient for DefaultHttpClient {
    fn get(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send {
        let url = self.build_url(path);
        let client = self.client.clone();
        async move { Ok(client.get(url)) }
    }

    fn post(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send {
        let url = self.build_url(path);
        let client = self.client.clone();
        async move { Ok(client.post(url)) }
    }

    fn put(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send {
        let url = self.build_url(path);
        let client = self.client.clone();
        async move { Ok(client.put(url)) }
    }

    fn delete(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send {
        let url = self.build_url(path);
        let client = self.client.clone();
        async move { Ok(client.delete(url)) }
    }
}

impl HttpClientExt for DefaultHttpClient {
    fn send_json<T: DeserializeOwned + Send + 'static>(
        &self,
        request: RequestBuilder,
    ) -> impl std::future::Future<Output = Result<T>> + Send {
        let self_clone = self.clone();
        async move {
            let response = self_clone.send_request(request).await?.send().await?;
            Ok(response.json().await?)
        }
    }

    fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> impl std::future::Future<Output = Result<T>> + Send {
        let self_clone = self.clone();
        let url = url.to_string();
        async move {
            let request = self_clone.get(&url).await?;
            self_clone.send_json(request).await
        }
    }

    fn post_json<
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Send + Sync + 'static,
    >(
        &self,
        url: &str,
        body: &B,
    ) -> impl std::future::Future<Output = Result<T>> + Send {
        let self_clone = self.clone();
        let url = url.to_string();
        let body = serde_json::to_value(body).expect("Failed to serialize body");
        async move {
            let request = self_clone.post(&url).await?.json(&body);
            self_clone.send_json(request).await
        }
    }

    fn put_json<
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Send + Sync + 'static,
    >(
        &self,
        url: &str,
        body: &B,
    ) -> impl std::future::Future<Output = Result<T>> + Send {
        let self_clone = self.clone();
        let url = url.to_string();
        let body = serde_json::to_value(body).expect("Failed to serialize body");
        async move {
            let request = self_clone.put(&url).await?.json(&body);
            self_clone.send_json(request).await
        }
    }
}
