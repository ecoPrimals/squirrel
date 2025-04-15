//! HTTP client module for API clients

use async_trait::async_trait;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};

use crate::Result;

mod client;
mod middleware;

pub use self::client::DefaultHttpClient;
pub use self::middleware::Middleware;

/// HTTP client interface
#[async_trait]
pub trait HttpClient: Send + Sync + 'static {
    /// Create a GET request
    async fn get(&self, path: &str) -> Result<RequestBuilder>;

    /// Create a POST request
    async fn post(&self, path: &str) -> Result<RequestBuilder>;

    /// Create a PUT request
    async fn put(&self, path: &str) -> Result<RequestBuilder>;

    /// Create a DELETE request
    async fn delete(&self, path: &str) -> Result<RequestBuilder>;
}

/// HTTP client extension trait
#[async_trait]
pub trait HttpClientExt: HttpClient {
    /// Send a request and parse the response as JSON
    async fn send_json<T: DeserializeOwned + Send + 'static>(
        &self,
        request: RequestBuilder,
    ) -> Result<T>;

    /// Send a GET request and parse the response as JSON
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> Result<T>;

    /// Send a POST request with a JSON body and parse the response as JSON
    async fn post_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync + 'static>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T>;

    /// Send a PUT request with a JSON body and parse the response as JSON
    async fn put_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync + 'static>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T>;
} 