//! HTTP client configuration

use std::time::Duration;
use std::collections::HashMap;

use crate::http::middleware::{LoggingMiddleware, Middleware, RateLimitMiddleware};

/// Configuration for HTTP clients
pub struct HttpClientConfig {
    /// The base URL for all requests
    pub base_url: Option<String>,
    /// Default timeout for requests
    pub timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Default headers to include in requests
    pub default_headers: HashMap<String, String>,
    /// Middlewares to apply to requests and responses
    pub middlewares: Vec<Box<dyn Middleware>>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            default_headers: HashMap::new(),
            middlewares: Vec::new(),
        }
    }
}

impl HttpClientConfig {
    /// Create a new HTTP client configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a new configuration with the given base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
    
    /// Set the timeout for requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Add a default header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }
    
    /// Add middleware to the client
    pub fn with_middleware(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }
    
    /// Add a logging middleware for easier debugging
    pub fn with_logging(self, level: tracing::Level) -> Self {
        self.with_middleware(Box::new(LoggingMiddleware::new(level)))
    }
    
    /// Add a rate limiting middleware to prevent API throttling
    pub fn with_rate_limit(self, requests_per_minute: u32) -> Self {
        self.with_middleware(Box::new(RateLimitMiddleware::new(
            requests_per_minute,
            "default".to_string()
        )))
    }
}

/// Builder for HttpClientConfig
pub struct HttpClientConfigBuilder {
    base_url: String,
    timeout: Duration,
    connect_timeout: Duration,
    default_headers: HashMap<String, String>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl Default for HttpClientConfigBuilder {
    fn default() -> Self {
        Self {
            base_url: "".to_string(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            default_headers: HashMap::new(),
            middleware: Vec::new(),
        }
    }
}

impl HttpClientConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL
    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
    
    /// Set the timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set the connect timeout
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }
    
    /// Add a default header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }
    
    /// Add middleware
    pub fn middleware(mut self, middleware: Box<dyn Middleware>) -> Self {
        self.middleware.push(middleware);
        self
    }
    
    /// Build the HttpClientConfig
    pub fn build(self) -> HttpClientConfig {
        HttpClientConfig {
            base_url: Some(self.base_url),
            timeout: self.timeout,
            connect_timeout: self.connect_timeout,
            default_headers: self.default_headers,
            middlewares: self.middleware,
        }
    }
} 