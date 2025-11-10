//! HTTP client module for API clients

use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Serialize};

use crate::Result;

mod client;
mod middleware;

pub use self::client::DefaultHttpClient;
pub use self::middleware::Middleware;

/// HTTP client interface (native async - Phase 4 migration)
pub trait HttpClient: Send + Sync + 'static {
    /// Create a GET request
    fn get(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send;

    /// Create a POST request
    fn post(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send;

    /// Create a PUT request
    fn put(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send;

    /// Create a DELETE request
    fn delete(&self, path: &str) -> impl std::future::Future<Output = Result<RequestBuilder>> + Send;
}

/// HTTP client extension trait (native async - Phase 4 migration)
pub trait HttpClientExt: HttpClient {
    /// Send a request and parse the response as JSON
    fn send_json<T: DeserializeOwned + Send + 'static>(
        &self,
        request: RequestBuilder,
    ) -> impl std::future::Future<Output = Result<T>> + Send;

    /// Send a GET request and parse the response as JSON
    fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> impl std::future::Future<Output = Result<T>> + Send;

    /// Send a POST request with a JSON body and parse the response as JSON
    fn post_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync + 'static>(
        &self,
        url: &str,
        body: &B,
    ) -> impl std::future::Future<Output = Result<T>> + Send;

    /// Send a PUT request with a JSON body and parse the response as JSON
    fn put_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync + 'static>(
        &self,
        url: &str,
        body: &B,
    ) -> impl std::future::Future<Output = Result<T>> + Send;
}
