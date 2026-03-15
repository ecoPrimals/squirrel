// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Connection Pool Types
//!
//! Additional type definitions and utilities for the HTTP connection pool system,
//! including request wrappers, response handlers, and connection management types.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use reqwest::{Client, Method, Response, RequestBuilder};
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::error::{Result, types::MCPError};

/// HTTP request wrapper for connection pool usage
#[derive(Debug, Clone)]
pub struct PooledRequest {
    /// Request ID for tracking
    pub id: String,
    
    /// Provider name
    pub provider_name: String,
    
    /// HTTP method
    pub method: Method,
    
    /// Request URL
    pub url: String,
    
    /// Request headers
    pub headers: HashMap<String, String>,
    
    /// Request body
    pub body: Option<Vec<u8>>,
    
    /// Request timeout override
    pub timeout: Option<Duration>,
    
    /// Request metadata
    pub metadata: HashMap<String, String>,
    
    /// Request creation timestamp
    pub created_at: Instant,
}

/// HTTP response wrapper with tracking information
#[derive(Debug)]
pub struct PooledResponse {
    /// Request ID that generated this response
    pub request_id: String,
    
    /// Provider name
    pub provider_name: String,
    
    /// Underlying HTTP response
    pub response: Response,
    
    /// Response processing start time
    pub processing_started: Instant,
    
    /// Total request duration
    pub request_duration: Duration,
    
    /// Response metadata
    pub metadata: HashMap<String, String>,
}

/// Connection state for tracking individual connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection is idle and available
    Idle,
    
    /// Connection is currently in use
    Active,
    
    /// Connection is being established
    Connecting,
    
    /// Connection is being closed
    Closing,
    
    /// Connection failed and needs cleanup
    Failed,
    
    /// Connection is in maintenance mode
    Maintenance,
}

/// Connection lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionEvent {
    /// New connection established
    Connected {
        provider_name: String,
        connection_id: String,
        timestamp: Instant,
    },
    
    /// Connection became active (request started)
    RequestStarted {
        provider_name: String,
        connection_id: String,
        request_id: String,
        timestamp: Instant,
    },
    
    /// Request completed successfully
    RequestCompleted {
        provider_name: String,
        connection_id: String,
        request_id: String,
        duration: Duration,
        bytes_sent: u64,
        bytes_received: u64,
        timestamp: Instant,
    },
    
    /// Request failed
    RequestFailed {
        provider_name: String,
        connection_id: String,
        request_id: String,
        error: String,
        duration: Option<Duration>,
        timestamp: Instant,
    },
    
    /// Connection became idle
    ConnectionIdle {
        provider_name: String,
        connection_id: String,
        timestamp: Instant,
    },
    
    /// Connection was closed
    ConnectionClosed {
        provider_name: String,
        connection_id: String,
        reason: String,
        timestamp: Instant,
    },
    
    /// Health check performed
    HealthCheck {
        provider_name: String,
        is_healthy: bool,
        response_time: Duration,
        timestamp: Instant,
    },
}

/// Request priority levels for connection pool management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestPriority {
    /// Low priority request (can wait for available connection)
    Low = 1,
    
    /// Normal priority request (default)
    Normal = 2,
    
    /// High priority request (should get connection quickly)
    High = 3,
    
    /// Critical priority request (must get immediate connection)
    Critical = 4,
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl RequestPriority {
    /// Get numeric priority value for comparison
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    
    /// Check if this priority is higher than another
    pub fn is_higher_than(self, other: RequestPriority) -> bool {
        self.as_u8() > other.as_u8()
    }
}

/// Request queue entry for priority-based scheduling
#[derive(Debug)]
pub struct QueuedRequest {
    /// The pooled request
    pub request: PooledRequest,
    
    /// Request priority
    pub priority: RequestPriority,
    
    /// Queue entry timestamp
    pub queued_at: Instant,
    
    /// Response sender for async completion
    pub response_sender: tokio::sync::oneshot::Sender<Result<PooledResponse>>,
}

/// Connection pool statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolStatistics {
    /// Total connections in pool
    pub total_connections: usize,
    
    /// Active (in-use) connections
    pub active_connections: usize,
    
    /// Idle (available) connections
    pub idle_connections: usize,
    
    /// Connections being established
    pub connecting: usize,
    
    /// Failed connections needing cleanup
    pub failed_connections: usize,
    
    /// Average connection age
    pub avg_connection_age: Duration,
    
    /// Pool utilization percentage
    pub utilization_percentage: f64,
    
    /// Requests waiting in queue
    pub queued_requests: usize,
    
    /// Peak concurrent connections
    pub peak_connections: usize,
    
    /// Connection churn rate (connections/minute)
    pub connection_churn_rate: f64,
}

/// Rate limiter for controlling request frequency
#[derive(Debug)]
pub struct RateLimiter {
    /// Token bucket for rate limiting
    tokens: Arc<RwLock<f64>>,
    
    /// Maximum tokens (burst capacity)
    max_tokens: f64,
    
    /// Token refill rate per second
    refill_rate: f64,
    
    /// Last token refill timestamp
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests_per_second: f64, burst_capacity: u32) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(burst_capacity as f64)),
            max_tokens: burst_capacity as f64,
            refill_rate: max_requests_per_second,
            last_refill: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    /// Try to acquire a token for rate limiting
    pub async fn try_acquire(&self) -> bool {
        self.refill_tokens().await;
        
        let mut tokens = self.tokens.write().await;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    /// Wait until a token becomes available
    pub async fn acquire(&self) -> Result<()> {
        loop {
            if self.try_acquire().await {
                return Ok(());
            }
            
            // Calculate wait time based on current token count
            let tokens = *self.tokens.read().await;
            let wait_time = if tokens > 0.0 {
                Duration::from_millis(100) // Short wait if we have partial tokens
            } else {
                Duration::from_millis((1000.0 / self.refill_rate) as u64)
            };
            
            tokio::time::sleep(wait_time).await;
        }
    }
    
    /// Refill tokens based on elapsed time
    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);
        
        if elapsed.as_millis() > 0 {
            let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate).min(self.max_tokens);
            let mut tokens = self.tokens.write().await;
            *tokens = (*tokens + tokens_to_add).min(self.max_tokens);
            *last_refill = now;
        }
    }
    
    /// Get current token count
    pub async fn get_token_count(&self) -> f64 {
        self.refill_tokens().await;
        *self.tokens.read().await
    }
}

/// Connection retry policy for handling failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Base delay between retries
    pub base_delay: Duration,
    
    /// Maximum delay between retries
    pub max_delay: Duration,
    
    /// Backoff strategy
    pub backoff_strategy: BackoffStrategy,
    
    /// Which HTTP status codes should trigger retries
    pub retryable_status_codes: Vec<u16>,
    
    /// Whether to retry on network errors
    pub retry_on_network_error: bool,
}

/// Backoff strategy for retry delays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    
    /// Exponential backoff (delay doubles each time)
    Exponential,
    
    /// Linear backoff (delay increases linearly)
    Linear,
    
    /// Exponential backoff with jitter
    ExponentialWithJitter,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_strategy: BackoffStrategy::ExponentialWithJitter,
            retryable_status_codes: vec![429, 500, 502, 503, 504],
            retry_on_network_error: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for a specific retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_millis = self.base_delay.as_millis() as u64;
        
        let delay_millis = match self.backoff_strategy {
            BackoffStrategy::Fixed => base_millis,
            BackoffStrategy::Linear => base_millis * (attempt as u64),
            BackoffStrategy::Exponential => base_millis * 2_u64.pow(attempt.saturating_sub(1)),
            BackoffStrategy::ExponentialWithJitter => {
                let exp_delay = base_millis * 2_u64.pow(attempt.saturating_sub(1));
                let jitter = fastrand::u64(0..=exp_delay / 4); // Add up to 25% jitter
                exp_delay + jitter
            }
        };
        
        Duration::from_millis(delay_millis.min(self.max_delay.as_millis() as u64))
    }
    
    /// Check if a status code is retryable
    pub fn is_retryable_status(&self, status_code: u16) -> bool {
        self.retryable_status_codes.contains(&status_code)
    }
}

impl PooledRequest {
    /// Create a new GET request
    pub fn get(provider_name: String, url: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            provider_name,
            method: Method::GET,
            url,
            headers: HashMap::new(),
            body: None,
            timeout: None,
            metadata: HashMap::new(),
            created_at: Instant::now(),
        }
    }
    
    /// Create a new POST request
    pub fn post(provider_name: String, url: String, body: Vec<u8>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            provider_name,
            method: Method::POST,
            url,
            headers: HashMap::new(),
            body: Some(body),
            timeout: None,
            metadata: HashMap::new(),
            created_at: Instant::now(),
        }
    }
    
    /// Add a header to the request
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    /// Add metadata to the request
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Build a reqwest RequestBuilder from this request
    pub fn to_request_builder(&self, client: &Client) -> RequestBuilder {
        let mut builder = client.request(self.method.clone(), &self.url);
        
        // Add headers
        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }
        
        // Add body if present
        if let Some(body) = &self.body {
            builder = builder.body(body.clone());
        }
        
        // Set timeout if specified
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        
        builder
    }
    
    /// Get request age since creation
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

impl PooledResponse {
    /// Create a new pooled response
    pub fn new(request_id: String, provider_name: String, response: Response, request_duration: Duration) -> Self {
        Self {
            request_id,
            provider_name,
            response,
            processing_started: Instant::now(),
            request_duration,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the response
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get response processing duration
    pub fn processing_duration(&self) -> Duration {
        self.processing_started.elapsed()
    }
    
    /// Get total duration including request and processing time
    pub fn total_duration(&self) -> Duration {
        self.request_duration + self.processing_duration()
    }
}

impl PoolStatistics {
    /// Calculate utilization percentage
    pub fn calculate_utilization(&mut self) {
        if self.total_connections > 0 {
            self.utilization_percentage = (self.active_connections as f64 / self.total_connections as f64) * 100.0;
        } else {
            self.utilization_percentage = 0.0;
        }
    }
    
    /// Check if pool is under-utilized
    pub fn is_under_utilized(&self) -> bool {
        self.utilization_percentage < 30.0 && self.total_connections > 5
    }
    
    /// Check if pool is over-utilized
    pub fn is_over_utilized(&self) -> bool {
        self.utilization_percentage > 90.0
    }
    
    /// Get efficiency score (0.0 to 1.0)
    pub fn efficiency_score(&self) -> f64 {
        if self.total_connections == 0 {
            return 1.0;
        }
        
        let utilization_score = (self.utilization_percentage / 100.0).min(1.0);
        let queue_penalty = if self.queued_requests > 0 {
            0.8 // Penalize for queued requests
        } else {
            1.0
        };
        
        utilization_score * queue_penalty
    }
} 