//! HTTP client configuration

use std::collections::HashMap;

use super::middleware::{Middleware, RateLimitMiddleware};

/// Configuration for the HTTP client
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Base URL for the API
    pub base_url: Option<String>,
    /// Timeout in seconds for requests
    pub timeout_seconds: u64,
    /// Connection timeout in seconds
    pub connect_timeout_seconds: u64,
    /// Default headers to include with all requests
    pub default_headers: HashMap<String, String>,
    /// Proxy URL if needed
    pub proxy: Option<String>,
    /// Middleware to apply to requests and responses
    pub middleware: Vec<Box<dyn Middleware>>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            default_headers: HashMap::new(),
            proxy: None,
            middleware: Vec::new(),
        }
    }
}

impl HttpClientConfig {
    /// Create a new configuration with the given base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Some(base_url.into()),
            ..Default::default()
        }
    }

    /// Set the timeout for requests
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, seconds: u64) -> Self {
        self.connect_timeout_seconds = seconds;
        self
    }

    /// Add a default header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Set the proxy URL
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Add a middleware
    pub fn with_middleware(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Add rate limiting middleware
    pub fn with_rate_limit(self, requests_per_minute: u32) -> Self {
        self.with_middleware(Box::new(RateLimitMiddleware::new(requests_per_minute)))
    }
} 