//! HTTP client middleware
//!
//! This module provides middleware components that can be applied to HTTP requests
//! and responses to add functionality like rate limiting, caching, etc.

use async_trait::async_trait;
use governor::{Quota, RateLimiter};
use reqwest::{RequestBuilder, Response};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use crate::{Error, Result};

/// Middleware for HTTP requests and responses
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Process a request before it is sent
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder>;
    
    /// Process a response after it is received
    async fn process_response(&self, response: Response) -> Result<Response>;
}

/// Rate limiting middleware to prevent hitting API rate limits
pub struct RateLimitMiddleware {
    /// The rate limiter
    limiter: Arc<RateLimiter<String, governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl RateLimitMiddleware {
    /// Create a new rate limiter with the given requests per minute
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap());
        let limiter = RateLimiter::direct(quota);
        
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        // Use a key based on the request URL (when we can extract it)
        let key = "default".to_string();
        
        // Wait for the rate limiter to allow the request
        match self.limiter.check_key(&key) {
            Ok(_) => Ok(request),
            Err(negative) => {
                let wait_time = negative.wait_time_from(governor::clock::DefaultClock::default().now());
                tokio::time::sleep(wait_time).await;
                Ok(request)
            }
        }
    }
    
    async fn process_response(&self, response: Response) -> Result<Response> {
        // Check for rate limit headers and adjust rate limiting if needed
        // This is an example and would need to be customized for each API
        
        if response.status().as_u16() == 429 {
            return Err(Error::RateLimit("Rate limit exceeded".to_string()));
        }
        
        Ok(response)
    }
}

/// Retry middleware for retrying failed requests
pub struct RetryMiddleware {
    /// Maximum number of retries
    max_retries: u32,
    /// Initial backoff time in milliseconds
    initial_backoff_ms: u64,
}

impl RetryMiddleware {
    /// Create a new retry middleware
    pub fn new(max_retries: u32, initial_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
        }
    }
}

#[async_trait]
impl Middleware for RetryMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        // We don't need to do anything with the request
        Ok(request)
    }
    
    async fn process_response(&self, response: Response) -> Result<Response> {
        // In a real implementation, we would need to clone the request and retry
        // if the response indicates a retryable error. This is a simplified example.
        
        let status = response.status();
        if status.is_server_error() || status.as_u16() == 429 {
            // In a real implementation, we would retry here
            // For now, we just return the response
        }
        
        Ok(response)
    }
}

/// Logging middleware for logging requests and responses
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    /// Create a new logging middleware
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        // In a real implementation, we would log the request details
        // For now, we just return the request
        Ok(request)
    }
    
    async fn process_response(&self, response: Response) -> Result<Response> {
        // In a real implementation, we would log the response details
        // For now, we just return the response
        Ok(response)
    }
} 