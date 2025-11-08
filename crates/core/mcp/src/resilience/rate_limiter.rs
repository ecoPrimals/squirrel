//! Rate Limiter implementation for the MCP resilience framework
//! 
//! This module provides a rate limiting implementation to protect services from
//! excessive load by limiting the number of operations in a given time period.

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::error::Error;
use tokio::sync::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::future::Future;
use tokio::task::block_in_place;

use crate::resilience::{ResilienceError, Result};

/// Error type for rate limiter operations
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// Rate limit exceeded
    #[error("Rate limit exceeded, limit: {limit} per {period:?}")]
    LimitExceeded {
        /// Maximum number of operations allowed in the period
        limit: u64,
        /// Time period for the rate limit
        period: Duration,
    },
    
    /// Operation timed out while waiting for rate limit
    #[error("Operation timed out after {duration:?} while waiting for rate limit")]
    Timeout {
        /// Duration after which the operation timed out
        duration: Duration,
    },
    
    /// Operation failed with an error
    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

impl From<RateLimitError> for ResilienceError {
    fn from(value: RateLimitError) -> Self {
        match value {
            RateLimitError::LimitExceeded { limit, period } => {
                Self::RateLimit(format!(
                    "Rate limit exceeded, limit: {limit} per {:?}", period
                ))
            },
            RateLimitError::Timeout { duration } => {
                Self::Timeout(format!(
                    "Operation timed out after {:?} while waiting for rate limit", duration
                ))
            },
            RateLimitError::OperationFailed(msg) => {
                Self::General(format!("Rate limited operation failed: {}", msg))
            }
        }
    }
}

/// Configuration for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Name of the rate limiter for identification
    pub name: String,
    
    /// Maximum number of operations allowed in the refresh period
    pub limit_for_period: u64,
    
    /// Period after which the limit refreshes
    pub limit_refresh_period: Duration,
    
    /// Timeout for waiting for rate limit
    pub timeout_duration: Option<Duration>,
    
    /// Whether to wait for permits when limit is exceeded
    pub wait_for_permits: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .and_then(|loaded| loaded.try_into_config().ok());
        
        let (refresh_period, timeout) = if let Some(cfg) = config {
            // Use custom rate limiter timeouts if configured
            let refresh = cfg.timeouts.get_custom_timeout("rate_limit_refresh")
                .unwrap_or_else(|| Duration::from_secs(1));
            let timeout_val = cfg.timeouts.get_custom_timeout("rate_limit_timeout")
                .unwrap_or_else(|| Duration::from_secs(1));
            (refresh, Some(timeout_val))
        } else {
            // Fallback to sensible defaults
            (Duration::from_secs(1), Some(Duration::from_secs(1)))
        };
        
        Self {
            name: "default".to_string(),
            limit_for_period: 100,
            limit_refresh_period: refresh_period,
            timeout_duration: timeout,
            wait_for_permits: true,
        }
    }
}

/// Metrics for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterMetrics {
    /// Number of available permits
    pub available_permits: u64,
    
    /// Number of waiting threads
    pub waiting_threads: usize,
    
    /// Number of operations rejected due to rate limiting
    pub rejection_count: u64,
    
    /// Number of operations that timed out while waiting
    pub timeout_count: u64,
    
    /// Total number of operations attempted
    pub total_operations: u64,
    
    /// Total number of operations executed successfully
    pub successful_operations: u64,
}

/// Entry in the token bucket tracking operation timestamps
struct TokenBucketEntry {
    /// When the operation was executed
    timestamp: Instant,
}

/// The rate limiter implementation using token bucket algorithm
pub struct RateLimiter {
    /// Configuration for this rate limiter
    config: RateLimiterConfig,
    
    /// Token bucket tracking executed operations
    token_bucket: Arc<Mutex<VecDeque<TokenBucketEntry>>>,
    
    /// Counter for rejected operations
    rejection_count: Arc<AtomicU64>,
    
    /// Counter for timed out operations
    timeout_count: Arc<AtomicU64>,
    
    /// Counter for total operations
    total_operations: Arc<AtomicU64>,
    
    /// Counter for successful operations
    successful_operations: Arc<AtomicU64>,
    
    /// Number of waiting threads (approximation)
    waiting_threads: Arc<AtomicU64>,
    
    /// Last refresh time
    last_refresh: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified configuration
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            token_bucket: Arc::new(Mutex::new(VecDeque::with_capacity(
                config.limit_for_period as usize
            ))),
            rejection_count: Arc::new(AtomicU64::new(0)),
            timeout_count: Arc::new(AtomicU64::new(0)),
            total_operations: Arc::new(AtomicU64::new(0)),
            successful_operations: Arc::new(AtomicU64::new(0)),
            waiting_threads: Arc::new(AtomicU64::new(0)),
            last_refresh: Arc::new(Mutex::new(Instant::now())),
            config,
        }
    }
    
    /// Create a new rate limiter with default configuration
    pub fn default() -> Self {
        Self::new(RateLimiterConfig::default())
    }
    
    /// Get the current metrics for this rate limiter
    pub async fn metrics(&self) -> RateLimiterMetrics {
        let token_bucket = self.token_bucket.lock().await;
        let available_permits = self.config.limit_for_period.saturating_sub(token_bucket.len() as u64);
        
        RateLimiterMetrics {
            available_permits,
            waiting_threads: self.waiting_threads.load(Ordering::Relaxed) as usize,
            rejection_count: self.rejection_count.load(Ordering::Relaxed),
            timeout_count: self.timeout_count.load(Ordering::Relaxed),
            total_operations: self.total_operations.load(Ordering::Relaxed),
            successful_operations: self.successful_operations.load(Ordering::Relaxed),
        }
    }
    
    /// Check if the operation is allowed to proceed without consuming a token
    pub async fn is_allowed(&self) -> bool {
        let token_bucket = self.token_bucket.lock().await;
        token_bucket.len() < self.config.limit_for_period as usize
    }
    
    /// Get the number of available permits
    pub async fn available_permits(&self) -> u64 {
        let token_bucket = self.token_bucket.lock().await;
        self.config.limit_for_period.saturating_sub(token_bucket.len() as u64)
    }
    
    /// Clean the token bucket by removing expired entries
    async fn clean_token_bucket(&self) {
        let mut token_bucket = self.token_bucket.lock().await;
        let now = Instant::now();
        
        // Remove entries that are older than the refresh period
        while !token_bucket.is_empty() {
            if let Some(entry) = token_bucket.front() {
                if now.duration_since(entry.timestamp) > self.config.limit_refresh_period {
                    token_bucket.pop_front();
                    continue;
                }
            }
            break;
        }
    }
    
    /// Acquire a permit to execute an operation, with optional waiting
    async fn acquire_permit(&self) -> Result<()> {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Clean expired entries first
        self.clean_token_bucket().await;
        
        // Check if permit is available
        let is_allowed;
        {
            let token_bucket = self.token_bucket.lock().await;
            is_allowed = token_bucket.len() < self.config.limit_for_period as usize;
        }
        
        if is_allowed {
            // Add entry to the bucket
            let mut token_bucket = self.token_bucket.lock().await;
            token_bucket.push_back(TokenBucketEntry {
                timestamp: Instant::now(),
            });
            return Ok(());
        }
        
        // If we don't wait, reject immediately
        if !self.config.wait_for_permits {
            self.rejection_count.fetch_add(1, Ordering::Relaxed);
            return Err(RateLimitError::LimitExceeded {
                limit: self.config.limit_for_period,
                period: self.config.limit_refresh_period,
            }.into());
        }
        
        // Wait for a permit with timeout
        self.waiting_threads.fetch_add(1, Ordering::Relaxed);
        let wait_result = match self.config.timeout_duration {
            Some(timeout) => {
                match tokio::time::timeout(timeout, self.wait_for_permit()).await {
                    Ok(result) => result,
                    Err(_) => {
                        // Timeout waiting for a permit
                        self.timeout_count.fetch_add(1, Ordering::Relaxed);
                        self.waiting_threads.fetch_sub(1, Ordering::Relaxed);
                        return Err(RateLimitError::Timeout {
                            duration: timeout,
                        }.into());
                    }
                }
            },
            None => {
                // No timeout, simply wait
                self.wait_for_permit().await
            }
        };
        
        self.waiting_threads.fetch_sub(1, Ordering::Relaxed);
        
        if wait_result {
            Ok(())
        } else {
            self.rejection_count.fetch_add(1, Ordering::Relaxed);
            Err(RateLimitError::LimitExceeded {
                limit: self.config.limit_for_period,
                period: self.config.limit_refresh_period,
            }.into())
        }
    }
    
    /// Wait for a permit to become available
    async fn wait_for_permit(&self) -> bool {
        let start = Instant::now();
        let refresh_period = self.config.limit_refresh_period;
        
        loop {
            // First check if we should abort the wait due to timeout
            if let Some(timeout) = self.config.timeout_duration {
                if start.elapsed() >= timeout {
                    return false;
                }
            }
            
            // Clean and check
            self.clean_token_bucket().await;
            
            // Check if permit is available
            let is_allowed;
            {
                let token_bucket = self.token_bucket.lock().await;
                is_allowed = token_bucket.len() < self.config.limit_for_period as usize;
            }
            
            if is_allowed {
                // Add entry to the bucket
                let mut token_bucket = self.token_bucket.lock().await;
                token_bucket.push_back(TokenBucketEntry {
                    timestamp: Instant::now(),
                });
                return true;
            }
            
            // Calculate wait time until the oldest entry expires
            let wait_duration = {
                let token_bucket = self.token_bucket.lock().await;
                if let Some(entry) = token_bucket.front() {
                    let elapsed = Instant::now().duration_since(entry.timestamp);
                    if elapsed >= refresh_period {
                        // The oldest entry has already expired, try again immediately
                        Duration::from_millis(1)
                    } else {
                        // Wait until the oldest entry expires
                        refresh_period.saturating_sub(elapsed)
                    }
                } else {
                    // No entries in the bucket, try again immediately
                    Duration::from_millis(1)
                }
            };
            
            // Wait for the calculated duration
            tokio::time::sleep(wait_duration).await;
        }
    }
    
    /// Execute an operation with rate limiting
    /// 
    /// This method:
    /// 1. Acquires a permit from the rate limiter
    /// 2. Executes the operation if permit is granted
    /// 3. Returns the result or appropriate error
    /// 
    /// # Arguments
    /// 
    /// * `operation` - Async operation to execute
    /// 
    /// # Returns
    /// 
    /// Result of the operation if successful
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// * The rate limit is exceeded and waiting is disabled
    /// * The operation times out while waiting for a permit
    /// * The operation itself fails
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T> 
    where
        F: Future<Output = std::result::Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + std::fmt::Debug + 'static,
    {
        // Acquire a permit first
        self.acquire_permit().await?;
        
        // Execute the operation
        match operation.await {
            Ok(value) => {
                self.successful_operations.fetch_add(1, Ordering::Relaxed);
                Ok(value)
            },
            Err(error) => {
                Err(RateLimitError::OperationFailed(format!("{:?}", error)).into())
            }
        }
    }

    /// Get the configuration of this rate limiter
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }
    
    /// Check if there are at least the specified number of permits available
    /// without actually acquiring them
    pub fn has_permits(&self, count: usize) -> bool {
        block_in_place(|| futures::executor::block_on(async {
            self.clean_token_bucket().await;
            let token_bucket = self.token_bucket.lock().await;
            let available = self.config.limit_for_period as usize - token_bucket.len();
            available >= count
        }))
    }

    /// Refresh the available permits based on the time elapsed since the last refresh
    async fn refresh_permits(&self) {
        let mut token_bucket = self.token_bucket.lock().await;
        let now = Instant::now();
        
        // Remove entries that are older than the refresh period
        while !token_bucket.is_empty() {
            if let Some(entry) = token_bucket.front() {
                if now.duration_since(entry.timestamp) > self.config.limit_refresh_period {
                    token_bucket.pop_front();
                    continue;
                }
            }
            break;
        }
    }

    /// Try to acquire a permit without waiting
    /// 
    /// This method checks if the rate limit allows another operation to proceed.
    /// If a permit is available, it consumes one and returns true.
    /// If no permits are available, it returns false without waiting.
    pub async fn try_acquire(&self) -> bool {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Clean up expired tokens
        self.clean_token_bucket().await;
        
        // Check if we have capacity
        let mut token_bucket = self.token_bucket.lock().await;
        if token_bucket.len() >= self.config.limit_for_period as usize {
            self.rejection_count.fetch_add(1, Ordering::Relaxed);
            false
        } else {
            // Add new token to bucket
            token_bucket.push_back(TokenBucketEntry {
                timestamp: Instant::now(),
            });
            
            self.successful_operations.fetch_add(1, Ordering::Relaxed);
            true
        }
    }
}

/// Create a new rate limiter with default configuration and the given component name
pub fn new_rate_limiter(component_id: &str) -> RateLimiter {
    let config = RateLimiterConfig {
        name: component_id.to_string(),
        ..RateLimiterConfig::default()
    };
    
    RateLimiter::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimiterConfig {
            limit_for_period: 3,
            limit_refresh_period: Duration::from_secs(1),
            wait_for_permits: false,
            ..RateLimiterConfig::default()
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // First 3 operations should succeed
        for i in 0..3 {
            let result = rate_limiter.execute(async move {
                Ok::<_, RateLimitError>(i)
            }).await;
            
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), i);
        }
        
        // 4th operation should be rejected
        let result = rate_limiter.execute(async {
            Ok::<_, RateLimitError>(4)
        }).await;
        
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                ResilienceError::RateLimit(msg) => {
                    assert!(msg.contains("exceeded"));
                },
                _ => panic!("Expected RateLimitError::LimitExceeded, got: {:?}", e),
            }
        }
        
        // Check metrics
        let metrics = rate_limiter.metrics().await;
        assert_eq!(metrics.available_permits, 0);
        assert_eq!(metrics.rejection_count, 1);
        assert_eq!(metrics.total_operations, 4);
        assert_eq!(metrics.successful_operations, 3);
    }
    
    #[tokio::test]
    async fn test_rate_limiter_refresh() {
        let config = RateLimiterConfig {
            limit_for_period: 2,
            limit_refresh_period: Duration::from_millis(100),
            wait_for_permits: false,
            ..RateLimiterConfig::default()
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // First 2 operations should succeed
        for i in 0..2 {
            let result = rate_limiter.execute(async move {
                Ok::<_, RateLimitError>(i)
            }).await;
            
            assert!(result.is_ok());
        }
        
        // 3rd operation should be rejected
        let result = rate_limiter.execute(async {
            Ok::<_, RateLimitError>(3)
        }).await;
        
        assert!(result.is_err());
        
        // Wait for tokens to refresh
        tokio::time::sleep(Duration::from_millis(110)).await;
        
        // Should be able to execute again
        let result = rate_limiter.execute(async {
            Ok::<_, RateLimitError>(4)
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
    }
    
    #[tokio::test]
    async fn test_rate_limiter_with_waiting() {
        let config = RateLimiterConfig {
            limit_for_period: 2,
            limit_refresh_period: Duration::from_millis(100),
            wait_for_permits: true,
            timeout_duration: Some(Duration::from_millis(50)),
            ..RateLimiterConfig::default()
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // Fill the bucket
        for i in 0..2 {
            let result = rate_limiter.execute(async move {
                Ok::<_, RateLimitError>(i)
            }).await;
            
            assert!(result.is_ok());
        }
        
        // This should time out while waiting
        let result = rate_limiter.execute(async {
            Ok::<_, RateLimitError>(3)
        }).await;
        
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                ResilienceError::Timeout(_) => {
                    // This is the expected error type
                    println!("Received expected timeout error: {}", e);
                },
                _ => panic!("Expected RateLimitError::Timeout, got: {:?}", e),
            }
        }
        
        // Now configure a long timeout and wait for refresh
        let config = RateLimiterConfig {
            limit_for_period: 2,
            limit_refresh_period: Duration::from_millis(100),
            wait_for_permits: true,
            timeout_duration: Some(Duration::from_millis(200)),
            ..RateLimiterConfig::default()
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // Fill the bucket
        for i in 0..2 {
            let result = rate_limiter.execute(async move {
                Ok::<_, RateLimitError>(i)
            }).await;
            
            assert!(result.is_ok());
        }
        
        // This should succeed after waiting for refresh
        let start = Instant::now();
        let result = rate_limiter.execute(async {
            Ok::<_, RateLimitError>(3)
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        
        // Should have waited at least the refresh period
        assert!(start.elapsed() >= Duration::from_millis(100));
    }
} 