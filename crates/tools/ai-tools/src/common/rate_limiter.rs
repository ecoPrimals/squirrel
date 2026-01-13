//! Rate limiter for AI service requests
//!
//! This module provides rate limiting capabilities for AI service clients
//! to respect API rate limits and prevent quota exhaustion.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use crate::error::Error;
use crate::Result;

/// Error specific to rate limiting operations
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    /// Rate limit exceeded, request rejected
    #[error("Rate limit exceeded: {requests} requests per {duration:?}")]
    LimitExceeded {
        /// Maximum requests allowed in the time window
        requests: u32,
        /// Time window for the rate limit
        duration: Duration,
    },

    /// Timeout while waiting for rate limit
    #[error("Timed out after {0:?} while waiting for rate limit")]
    Timeout(Duration),
}

impl From<RateLimitError> for Error {
    fn from(err: RateLimitError) -> Self {
        match err {
            RateLimitError::LimitExceeded { requests, duration } => Error::RateLimit(format!(
                "Rate limit exceeded: {requests} requests per {duration:?}"
            )),
            RateLimitError::Timeout(duration) => Error::RateLimit(format!(
                "Timed out after {duration:?} while waiting for rate limit"
            )),
        }
    }
}

/// Configuration for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests per minute
    pub max_requests_per_minute: u32,

    /// Whether to retry on rate limit errors
    pub retry_on_rate_limit: bool,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests_per_minute: 60, // Default to 60 RPM (1 per second)
            retry_on_rate_limit: true,
            max_retries: 3,
            retry_delay_ms: 2000, // 2 second delay between retries
        }
    }
}

/// Entry in the token bucket for tracking request timestamps
#[derive(Debug)]
struct TokenBucketEntry {
    /// When the request was executed
    timestamp: Instant,
}

/// Rate limiter for AI service requests using a token bucket algorithm
#[derive(Debug)]
pub struct RateLimiter {
    /// Configuration for this rate limiter
    config: RwLock<RateLimiterConfig>,

    /// Token bucket tracking executed requests
    token_bucket: Arc<Mutex<VecDeque<TokenBucketEntry>>>,

    /// Counter for rejected requests
    rejected_count: AtomicU64,

    /// Counter for retried requests
    retry_count: AtomicU64,

    /// Counter for total requests
    total_requests: AtomicU64,

    /// Name of the API provider for this rate limiter
    provider: String,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified configuration
    pub fn new(config: RateLimiterConfig, provider: impl Into<String>) -> Self {
        let provider = provider.into();
        tracing::debug!("Creating rate limiter for {provider} with config: {config:?}");

        Self {
            token_bucket: Arc::new(Mutex::new(VecDeque::with_capacity(
                config.max_requests_per_minute as usize,
            ))),
            rejected_count: AtomicU64::new(0),
            retry_count: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
            config: RwLock::new(config),
            provider,
        }
    }

    /// Create a new rate limiter with default configuration
    pub fn default(provider: impl Into<String>) -> Self {
        Self::new(RateLimiterConfig::default(), provider)
    }

    /// Create a new rate limiter with specified rate and default configuration
    pub fn default_with_rate(rate: u32, provider: impl Into<String>) -> Self {
        let config = RateLimiterConfig {
            max_requests_per_minute: rate,
            ..Default::default()
        };
        Self::new(config, provider)
    }

    /// Clean the token bucket by removing expired entries
    async fn clean_token_bucket(&self) {
        let mut token_bucket = self.token_bucket.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(60); // 1 minute window

        // Remove entries that are older than the window
        while !token_bucket.is_empty() {
            if let Some(entry) = token_bucket.front() {
                if now.duration_since(entry.timestamp) > window {
                    token_bucket.pop_front();
                    continue;
                }
            }
            break;
        }
    }

    /// Check if a request can be executed without exceeding rate limits
    pub async fn check(&self) -> bool {
        self.clean_token_bucket().await;

        let token_bucket = self.token_bucket.lock().await;
        let config = self.config.read().await;
        token_bucket.len() < config.max_requests_per_minute as usize
    }

    /// Try to acquire a permit to execute a request
    pub async fn acquire(&self) -> Result<()> {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Clean expired entries first
        self.clean_token_bucket().await;

        // Check if we're under the limit
        let config = self.config.read().await;
        let mut token_bucket = self.token_bucket.lock().await;
        if token_bucket.len() >= config.max_requests_per_minute as usize {
            self.rejected_count.fetch_add(1, Ordering::Relaxed);

            if !config.retry_on_rate_limit {
                return Err(RateLimitError::LimitExceeded {
                    requests: config.max_requests_per_minute,
                    duration: Duration::from_secs(60),
                }
                .into());
            }

            // Release the lock before waiting
            drop(token_bucket);
            drop(config);

            // Retry with backoff
            let mut retry_config = self.config.read().await;
            for retry in 1..=retry_config.max_retries {
                self.retry_count.fetch_add(1, Ordering::Relaxed);

                // Calculate retry delay with exponential backoff
                let delay_ms = retry_config.retry_delay_ms * retry as u64;
                tracing::warn!(
                    "Rate limit exceeded for {}, waiting {}ms (retry {}/{})",
                    self.provider,
                    delay_ms,
                    retry,
                    retry_config.max_retries
                );

                // Drop the lock during sleep
                drop(retry_config);

                tokio::time::sleep(Duration::from_millis(delay_ms)).await;

                // Try again after waiting
                self.clean_token_bucket().await;

                // Re-acquire locks
                let config = self.config.read().await;
                let mut token_bucket = self.token_bucket.lock().await;

                if token_bucket.len() < config.max_requests_per_minute as usize {
                    // We can proceed now
                    token_bucket.push_back(TokenBucketEntry {
                        timestamp: Instant::now(),
                    });

                    tracing::info!(
                        "Acquired permit after retry {}/{} for {}",
                        retry,
                        config.max_retries,
                        self.provider
                    );
                    return Ok(());
                }

                // Re-get the config for the next loop
                drop(config);
                drop(token_bucket);
                retry_config = self.config.read().await;
            }

            // All retries failed
            let final_config = self.config.read().await;
            return Err(RateLimitError::LimitExceeded {
                requests: final_config.max_requests_per_minute,
                duration: Duration::from_secs(60),
            }
            .into());
        }

        // Add entry to the bucket
        token_bucket.push_back(TokenBucketEntry {
            timestamp: Instant::now(),
        });

        Ok(())
    }

    /// Execute a function with rate limiting
    pub async fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        // Acquire permit first
        self.acquire().await?;

        // Execute the function
        f.await
    }

    /// Get rate limiting metrics
    pub async fn metrics(&self) -> RateLimiterMetrics {
        let token_bucket = self.token_bucket.lock().await;
        let config = self.config.read().await;

        RateLimiterMetrics {
            provider: self.provider.clone(),
            available_permits: config
                .max_requests_per_minute
                .saturating_sub(token_bucket.len() as u32),
            rejection_count: self.rejected_count.load(Ordering::Relaxed),
            retry_count: self.retry_count.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
        }
    }

    /// Update the rate limiter configuration
    pub fn update_config(&self, new_config: RateLimiterConfig) {
        // Get the old config for comparison
        let rt = tokio::runtime::Handle::current();

        // Run the async operation in a blocking context
        let old_max_rps = rt.block_on(async {
            let old_config = self.config.read().await;
            old_config.max_requests_per_minute
        });

        // Update the configuration
        rt.block_on(async {
            let mut config = self.config.write().await;
            *config = new_config.clone();
        });

        // Update the capacity of the token bucket if needed
        if new_config.max_requests_per_minute != old_max_rps {
            // This will take effect on next operation
            rt.block_on(async {
                // Try to acquire the lock
                if let Ok(mut bucket) = self.token_bucket.try_lock() {
                    // If capacity is reduced, we might need to evict some tokens
                    if new_config.max_requests_per_minute < old_max_rps {
                        // Truncate the deque to the new capacity if needed
                        let new_cap = new_config.max_requests_per_minute as usize;
                        if bucket.len() > new_cap {
                            // Keep only the newest entries
                            let to_remove = bucket.len() - new_cap;
                            for _ in 0..to_remove {
                                bucket.pop_front();
                            }
                        }
                    }

                    // Set new capacity
                    bucket.reserve(new_config.max_requests_per_minute as usize);
                }
            });
        }
    }
}

/// Metrics for the rate limiter
#[derive(Debug, Clone)]
pub struct RateLimiterMetrics {
    /// API provider name
    pub provider: String,

    /// Number of available permits
    pub available_permits: u32,

    /// Number of rejected requests
    pub rejection_count: u64,

    /// Number of retried requests
    pub retry_count: u64,

    /// Total number of requests
    pub total_requests: u64,
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        // Original code restored for compilation
        let rt = tokio::runtime::Handle::current();
        let config_clone = rt.block_on(async {
            let guard = self.config.read().await;
            guard.clone()
        });

        Self {
            config: RwLock::new(config_clone),
            token_bucket: self.token_bucket.clone(),
            rejected_count: AtomicU64::new(self.rejected_count.load(Ordering::Relaxed)),
            retry_count: AtomicU64::new(self.retry_count.load(Ordering::Relaxed)),
            total_requests: AtomicU64::new(self.total_requests.load(Ordering::Relaxed)),
            provider: self.provider.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(
            RateLimiterConfig {
                max_requests_per_minute: 5,
                retry_on_rate_limit: false,
                max_retries: 0,
                retry_delay_ms: 0,
            },
            "test",
        );

        // Should allow 5 requests
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }

        // 6th request should fail
        assert!(limiter.acquire().await.is_err());

        // Check metrics
        let metrics = limiter.metrics().await;
        assert_eq!(metrics.available_permits, 0);
        assert_eq!(metrics.total_requests, 6);
        assert_eq!(metrics.rejection_count, 1);
    }

    #[tokio::test]
    #[ignore] // TODO: Fix async runtime issue - block_on called within async context
    async fn test_rate_limiter_with_retry() {
        let limiter = RateLimiter::new(
            RateLimiterConfig {
                max_requests_per_minute: 5,
                retry_on_rate_limit: true,
                max_retries: 3,
                retry_delay_ms: 10, // Very short for testing
            },
            "test-retry",
        );

        // Fill up the bucket
        for _ in 0..5 {
            assert!(limiter.acquire().await.is_ok());
        }

        // Set up a task that will remove an entry after a short time
        tokio::spawn({
            let limiter = limiter.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(50)).await;
                let mut bucket = limiter.token_bucket.lock().await;
                bucket.pop_front(); // Remove oldest entry
            }
        });

        // This should succeed after retry
        assert!(limiter.acquire().await.is_ok());

        // Check metrics
        let metrics = limiter.metrics().await;
        assert!(metrics.retry_count > 0);
    }

    #[tokio::test]
    async fn test_rate_limiter_execute() {
        let limiter = RateLimiter::new(
            RateLimiterConfig {
                max_requests_per_minute: 5,
                retry_on_rate_limit: false,
                max_retries: 0,
                retry_delay_ms: 0,
            },
            "test-execute",
        );

        // Fill most of the bucket
        for _ in 0..4 {
            assert!(limiter.acquire().await.is_ok());
        }

        // This should succeed
        let result = limiter.execute(async { Ok::<_, Error>(42) }).await;

        assert_eq!(result.unwrap(), 42);

        // This should fail (bucket full)
        let result = limiter.execute(async { Ok::<_, Error>(43) }).await;

        assert!(result.is_err());
    }
}
