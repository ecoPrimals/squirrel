//! HTTP client implementation
//!
//! This module provides a common HTTP client interface that other API clients can build upon.
//! It handles common concerns like rate limiting, timeouts, and authentication.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authenticator;
use crate::{Error, Result};

mod client;
mod config;
mod middleware;

pub use client::DefaultHttpClient;
pub use config::HttpClientConfig;
pub use middleware::{Middleware, RateLimitMiddleware};

/// HTTP client trait that provides common functionality for API clients
#[async_trait]
pub trait HttpClient: Send + Sync + 'static {
    /// Create a new HTTP request
    fn request(&self, method: Method, url: &str) -> Result<RequestBuilder>;
    
    /// Send a request and get the raw response
    async fn send_raw(&self, builder: RequestBuilder) -> Result<Response>;
    
    /// Send a request and parse the response as JSON
    async fn send_json<T: DeserializeOwned + Send + 'static>(
        &self, 
        builder: RequestBuilder
    ) -> Result<T>;
    
    /// Send a request and get the response bytes
    async fn send_bytes(&self, builder: RequestBuilder) -> Result<Bytes>;
    
    /// Execute a GET request and parse the response as JSON
    async fn get_json<T: DeserializeOwned + Send + 'static>(&self, url: &str) -> Result<T>;
    
    /// Execute a POST request with a JSON body and parse the response as JSON
    async fn post_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T>;
    
    /// Execute a PUT request with a JSON body and parse the response as JSON
    async fn put_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T>;
    
    /// Execute a PATCH request with a JSON body and parse the response as JSON
    async fn patch_json<T: DeserializeOwned + Send + 'static, B: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &B,
    ) -> Result<T>;
    
    /// Execute a DELETE request
    async fn delete(&self, url: &str) -> Result<()>;
}

/// Create a new HTTP client with the given configuration and authenticator
pub fn new_client(
    config: HttpClientConfig,
    authenticator: Option<Arc<dyn Authenticator>>,
) -> Arc<dyn HttpClient> {
    Arc::new(DefaultHttpClient::new(config, authenticator))
}

/// Create a new HTTP client with default configuration
pub fn default_client() -> Arc<dyn HttpClient> {
    new_client(HttpClientConfig::default(), None)
} 