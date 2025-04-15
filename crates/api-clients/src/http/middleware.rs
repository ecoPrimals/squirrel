//! HTTP client middleware
//!
//! This module provides middleware components that can be applied to HTTP requests
//! and responses to add functionality like rate limiting, caching, etc.

use async_trait::async_trait;
use governor::{
    clock::{Clock, DefaultClock},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter
};
use nonzero_ext::nonzero;
use reqwest::{RequestBuilder, Response};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn, Level};

use crate::Result;

/// HTTP client middleware
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Process a request before it is sent
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder>;
    
    /// Process a response after it is received
    async fn process_response(&self, response: Response) -> Result<Response>;
}

/// Rate limit middleware
pub struct RateLimitMiddleware {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
    key: String,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware with the given number of requests per minute
    pub fn new(requests_per_minute: u32, key: impl Into<String>) -> Self {
        // Convert to requests per second for the rate limiter
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap_or(nonzero!(60u32)));
        let limiter = Arc::new(GovernorRateLimiter::direct(quota));
        
        Self {
            limiter,
            key: key.into(),
        }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        // Clone the request to extract information for rate limiting
        let req_clone = match request.try_clone() {
            Some(req) => {
                match req.build() {
                    Ok(built_req) => {
                        let key = &self.key;
                        
                        // Check if we can proceed with the request
                        match self.limiter.check() {
                            Ok(_) => {
                                debug!("Rate limit check passed for {}", key);
                                request
                            }
                            Err(negative) => {
                                // Wait for the required time and try again
                                let wait_time = negative.wait_time_from(DefaultClock::default().now());
                                warn!("Rate limit exceeded for {}. Waiting {:?}", key, wait_time);
                                tokio::time::sleep(wait_time).await;
                                
                                // Try again after waiting
                                match self.limiter.check() {
                                    Ok(_) => {
                                        info!("Rate limit check passed after waiting for {}", key);
                                        request
                                    }
                                    Err(_) => {
                                        error!("Rate limit still exceeded after waiting for {}", key);
                                        request
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        warn!("Failed to build request for rate limiting");
                        request
                    }
                }
            }
            None => {
                warn!("Failed to clone request for rate limiting");
                request
            }
        };
        
        Ok(req_clone)
    }
    
    async fn process_response(&self, response: Response) -> Result<Response> {
        // We could implement response-based rate limiting here if needed
        Ok(response)
    }
}

/// Logging middleware
pub struct LoggingMiddleware {
    level: Level,
}

impl LoggingMiddleware {
    /// Create a new logging middleware with the given tracing level
    pub fn new(level: Level) -> Self {
        Self { level }
    }
    
    /// Log at the appropriate level
    fn log_request(&self, method: &str, url: &str) {
        match self.level {
            Level::TRACE => trace!("Sending {} request to {}", method, url),
            Level::DEBUG => debug!("Sending {} request to {}", method, url),
            Level::INFO => info!("Sending {} request to {}", method, url),
            Level::WARN => warn!("Sending {} request to {}", method, url),
            Level::ERROR => error!("Sending {} request to {}", method, url),
        }
    }
    
    /// Log at the appropriate level
    fn log_response(&self, status: &str, url: &str) {
        match self.level {
            Level::TRACE => trace!("Received response {} from {}", status, url),
            Level::DEBUG => debug!("Received response {} from {}", status, url),
            Level::INFO => info!("Received response {} from {}", status, url),
            Level::WARN => warn!("Received response {} from {}", status, url),
            Level::ERROR => error!("Received response {} from {}", status, url),
        }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process_request(&self, request: RequestBuilder) -> Result<RequestBuilder> {
        // Clone the request for logging
        let request_clone = request.try_clone().unwrap_or_else(|| {
            warn!("Failed to clone request for logging");
            request.try_clone().expect("Failed to clone request twice")
        });
        
        // Only build the request for logging if debug or more verbose
        if let Ok(req) = request_clone.build() {
            let method = req.method().as_str();
            let url = req.url().as_str();
            self.log_request(method, url);
        } else {
            warn!("Failed to build request for logging");
        }
        
        // Return the original request
        Ok(request)
    }
    
    async fn process_response(&self, response: Response) -> Result<Response> {
        let status = response.status().to_string();
        let url = response.url().to_string();
        self.log_response(&status, &url);
        
        Ok(response)
    }
} 