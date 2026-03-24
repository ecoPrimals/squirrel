// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Retry mechanism for the MCP resilience framework
//!
//! This module provides functionality to retry operations that might fail transiently.

use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use squirrel_mcp_config::unified::LoadedConfig;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// Pin-boxed future type used by retry operation closures.
pub(crate) type RetryFuture<T> =
    Pin<Box<dyn Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send>>;

/// Serde helpers for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Default value for use_jitter field
const fn default_use_jitter() -> bool {
    true
}

/// Struct representing a constant backoff strategy
#[derive(Debug, Clone, Copy)]
pub struct ConstantBackoff {
    /// Constant delay to use between retry attempts
    pub delay: Duration,
}

impl ConstantBackoff {
    /// Create a new constant backoff with the specified delay
    #[must_use]
    pub const fn new(delay: Duration) -> Self {
        Self { delay }
    }

    /// Get the delay for a specific attempt (always returns the same value)
    #[must_use]
    pub const fn delay_for_attempt(&self, _attempt: u32) -> Duration {
        self.delay
    }
}

/// Defines different backoff strategies for retry operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackoffStrategy {
    /// Constant delay between retries
    Constant,
    /// Linear increase in delay (`base_delay` * attempt)
    Linear,
    /// Exponential increase in delay (`base_delay` * 2^attempt)
    #[default]
    Exponential,
    /// Fibonacci sequence for delay
    Fibonacci,
}

/// Configuration for the retry mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries (will be used with the backoff strategy)
    #[serde(with = "duration_serde")]
    pub base_delay: Duration,
    /// Maximum delay between retries
    #[serde(with = "duration_serde")]
    pub max_delay: Duration,
    /// Whether to use jitter to avoid retry storms
    #[serde(default = "default_use_jitter")]
    pub use_jitter: bool,
    /// The backoff strategy to use for calculating delays
    #[serde(default)]
    pub backoff_strategy: BackoffStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeout values
        let config = squirrel_mcp_config::unified::ConfigLoader::load()
            .ok()
            .map(LoadedConfig::into_config);

        let (base_delay, max_delay) = if let Some(cfg) = config {
            let base = if cfg.timeouts.is_custom_timeout("retry_base") {
                cfg.timeouts.get_custom_timeout("retry_base")
            } else {
                Duration::from_millis(100)
            };
            let max = if cfg.timeouts.is_custom_timeout("retry_max") {
                cfg.timeouts.get_custom_timeout("retry_max")
            } else {
                Duration::from_secs(10)
            };
            (base, max)
        } else {
            (Duration::from_millis(100), Duration::from_secs(10))
        };

        Self {
            max_attempts: 3,
            base_delay,
            max_delay,
            use_jitter: true,
            backoff_strategy: BackoffStrategy::Exponential,
        }
    }
}

/// Metrics for retry operations
#[derive(Debug, Clone)]
pub struct RetryMetrics {
    /// Total number of successful operations
    pub success_count: usize,
    /// Total number of failed operations
    pub failure_count: usize,
    /// Total number of retries performed
    pub retry_count: usize,
    /// Maximum number of retries performed for a single operation
    pub max_retries_performed: usize,
}

/// Error types for retry operations
#[derive(Debug, thiserror::Error)]
pub enum RetryError {
    /// Maximum number of retry attempts exceeded
    #[error("Maximum retry attempts ({attempts}) exceeded: {error}")]
    MaxAttemptsExceeded {
        /// Number of attempts made
        attempts: u32,
        /// The last error encountered
        error: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Retry operation was cancelled
    #[error("Retry operation cancelled: {0}")]
    Cancelled(String),

    /// Internal error in the retry mechanism
    #[error("Retry internal error: {0}")]
    Internal(String),
}

/// Retry mechanism for handling transient failures
#[derive(Debug, Clone)]
pub struct RetryMechanism {
    /// Configuration for the retry mechanism
    config: RetryConfig,
    /// Count of successful operations
    success_count: Arc<AtomicU32>,
    /// Count of failed operations after all retries
    failure_count: Arc<AtomicU32>,
    /// Count of retry attempts
    retry_count: Arc<AtomicU32>,
    /// Maximum number of retries performed for a single operation
    max_retries_performed: Arc<AtomicU32>,
}

impl RetryMechanism {
    /// Create a new retry mechanism with the specified configuration
    #[must_use]
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            success_count: Arc::new(AtomicU32::new(0)),
            failure_count: Arc::new(AtomicU32::new(0)),
            retry_count: Arc::new(AtomicU32::new(0)),
            max_retries_performed: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Get retry metrics
    #[must_use]
    pub fn get_metrics(&self) -> RetryMetrics {
        RetryMetrics {
            success_count: self.success_count.load(Ordering::Relaxed) as usize,
            failure_count: self.failure_count.load(Ordering::Relaxed) as usize,
            retry_count: self.retry_count.load(Ordering::Relaxed) as usize,
            max_retries_performed: self.max_retries_performed.load(Ordering::Relaxed) as usize,
        }
    }

    /// Reset retry metrics
    pub fn reset_metrics(&self) {
        self.success_count.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.retry_count.store(0, Ordering::Relaxed);
        self.max_retries_performed.store(0, Ordering::Relaxed);
    }

    /// Execute an operation with retry logic
    ///
    /// Executes the provided operation and automatically retries it if it fails,
    /// according to the configured retry policy. The operation can be retried multiple
    /// times with increasing delays based on the backoff strategy.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute, provided as a mutable closure that
    ///   returns a future
    ///
    /// # Returns
    ///
    /// The result of the operation if successful within the allowed attempts
    ///
    /// # Errors
    ///
    /// Returns a `RetryError` if:
    /// * The maximum number of retry attempts is exceeded (`RetryError::MaxAttemptsExceeded`)
    /// * The retry operation is cancelled for any reason (`RetryError::Cancelled`)
    /// * An internal error occurs in the retry mechanism (`RetryError::Internal`)
    pub async fn execute<F, T>(&self, mut operation: F) -> std::result::Result<T, RetryError>
    where
        F: FnMut() -> RetryFuture<T>,
        T: Send + 'static,
    {
        let mut attempts: u32 = 0;
        let mut last_error = None;
        let mut retries = 0;

        while attempts < self.config.max_attempts {
            let future = operation();
            match future.await {
                Ok(value) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);

                    // Update max retries metrics if needed
                    let current_max = self.max_retries_performed.load(Ordering::Relaxed) as usize;
                    if retries > current_max {
                        self.max_retries_performed
                            .store(retries as u32, Ordering::Relaxed);
                    }

                    return Ok(value);
                }
                Err(err) => {
                    attempts += 1;

                    if attempts < self.config.max_attempts {
                        retries += 1;
                        self.retry_count.fetch_add(1, Ordering::Relaxed);

                        let delay = self.calculate_delay(attempts);
                        tokio::time::sleep(delay).await;
                    }

                    last_error = Some(err);
                }
            }
        }

        self.failure_count.fetch_add(1, Ordering::Relaxed);

        Err(RetryError::MaxAttemptsExceeded {
            attempts: self.config.max_attempts,
            error: last_error.unwrap_or_else(|| {
                Box::new(std::io::Error::other(
                    "Unknown error during retry operation",
                ))
            }),
        })
    }

    /// Calculate the delay for the next retry based on the backoff strategy
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = match self.config.backoff_strategy {
            BackoffStrategy::Constant => self.config.base_delay,
            BackoffStrategy::Linear => self.config.base_delay.mul_f32(attempt as f32),
            BackoffStrategy::Exponential => {
                // 2^attempt scaling for exponential backoff
                let scale = 2u32.pow(attempt - 1) as f32;
                self.config.base_delay.mul_f32(scale)
            }
            BackoffStrategy::Fibonacci => {
                // Calculate Fibonacci number (simplified approach)
                let mut a = 1;
                let mut b = 1;
                for _ in 0..attempt {
                    let temp = a;
                    a = b;
                    b += temp;
                }
                self.config.base_delay.mul_f32(a as f32)
            }
        };

        // Apply jitter if configured
        let mut final_delay = if self.config.use_jitter {
            Self::apply_jitter(base_delay)
        } else {
            base_delay
        };

        // Ensure we don't exceed max delay
        if final_delay > self.config.max_delay {
            final_delay = self.config.max_delay;
        }

        final_delay
    }

    /// Apply jitter to avoid retry storms when many instances retry simultaneously
    fn apply_jitter(delay: Duration) -> Duration {
        let mut rng = thread_rng();

        // Apply full jitter: random value between 0.1 and calculated delay
        // This helps prevent retry storm synchronization and ensures we never return zero delay
        let millis = delay.as_millis();
        let numer = rng.gen_range(10u128..=100u128);
        let jitter_scaled = millis.saturating_mul(numer).saturating_div(100);
        let jitter_millis =
            u64::try_from(jitter_scaled.min(u128::from(u64::MAX))).unwrap_or(u64::MAX);

        Duration::from_millis(jitter_millis)
    }

    /// Execute with a predicate that determines if an error should be retried
    ///
    /// Similar to `execute`, but allows specifying a predicate to decide which
    /// errors should be retried. This is useful for only retrying specific error types.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute, provided as a mutable closure that
    ///   returns a future
    /// * `should_retry` - Predicate that returns true if the error should be retried
    ///
    /// # Returns
    ///
    /// The result of the operation if successful within the allowed attempts
    ///
    /// # Errors
    ///
    /// Returns a `RetryError` if:
    /// * The maximum number of retry attempts is exceeded (`RetryError::MaxAttemptsExceeded`)
    /// * The retry operation is cancelled for any reason (`RetryError::Cancelled`)
    /// * An internal error occurs in the retry mechanism (`RetryError::Internal`)
    pub async fn execute_with_predicate<F, T, P>(
        &self,
        mut operation: F,
        should_retry: P,
    ) -> std::result::Result<T, RetryError>
    where
        F: FnMut() -> RetryFuture<T>,
        P: Fn(&Box<dyn StdError + Send + Sync>) -> bool + Send + Sync + 'static,
        T: Send + 'static,
    {
        let mut attempts: u32 = 0;
        let mut last_error = None;
        let mut retries = 0;

        while attempts < self.config.max_attempts {
            let future = operation();
            match future.await {
                Ok(value) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);

                    // Update max retries metrics if needed
                    let current_max = self.max_retries_performed.load(Ordering::Relaxed) as usize;
                    if retries > current_max {
                        self.max_retries_performed
                            .store(retries as u32, Ordering::Relaxed);
                    }

                    return Ok(value);
                }
                Err(err) => {
                    attempts += 1;

                    // Check if we should retry this error using the predicate
                    if attempts < self.config.max_attempts && should_retry(&err) {
                        retries += 1;
                        self.retry_count.fetch_add(1, Ordering::Relaxed);

                        let delay = self.calculate_delay(attempts);
                        tokio::time::sleep(delay).await;
                    } else if attempts < self.config.max_attempts {
                        // Error doesn't match retry criteria, exit early
                        self.failure_count.fetch_add(1, Ordering::Relaxed);
                        return Err(RetryError::MaxAttemptsExceeded {
                            attempts,
                            error: err,
                        });
                    }

                    last_error = Some(err);
                }
            }
        }

        self.failure_count.fetch_add(1, Ordering::Relaxed);

        Err(RetryError::MaxAttemptsExceeded {
            attempts: self.config.max_attempts,
            error: last_error.unwrap_or_else(|| {
                Box::new(std::io::Error::other(
                    "Unknown error during retry operation",
                ))
            }),
        })
    }

    /// Execute an operation with timeout for each retry attempt
    ///
    /// Executes the provided operation with a timeout for each retry attempt.
    /// The operation is retried according to the configured policy, and each attempt
    /// is limited by the specified timeout.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute, provided as a mutable closure that
    ///   returns a future
    /// * `timeout` - Maximum duration to wait for each attempt
    ///
    /// # Returns
    ///
    /// The result of the operation if successful within the allowed attempts and timeout
    ///
    /// # Errors
    ///
    /// Returns a `RetryError` if:
    /// * The maximum number of retry attempts is exceeded (`RetryError::MaxAttemptsExceeded`)
    /// * Any attempt times out (`RetryError::Cancelled` with timeout message)
    /// * The retry operation is cancelled for any reason (`RetryError::Cancelled`)
    /// * An internal error occurs in the retry mechanism (`RetryError::Internal`)
    pub async fn execute_with_timeout<F, T>(
        &self,
        mut operation: F,
        timeout: Duration,
    ) -> std::result::Result<T, RetryError>
    where
        F: FnMut() -> RetryFuture<T>,
        T: Send + 'static,
    {
        let mut attempts: u32 = 0;
        let mut last_error = None;
        let mut retries = 0;

        while attempts < self.config.max_attempts {
            // Create a timeout future for the operation
            let future = operation();
            let timeout_future = tokio::time::timeout(timeout, future);

            if let Ok(result) = timeout_future.await {
                match result {
                    Ok(value) => {
                        self.success_count.fetch_add(1, Ordering::Relaxed);

                        // Update max retries metrics if needed
                        let current_max =
                            self.max_retries_performed.load(Ordering::Relaxed) as usize;
                        if retries > current_max {
                            self.max_retries_performed
                                .store(retries as u32, Ordering::Relaxed);
                        }

                        return Ok(value);
                    }
                    Err(err) => {
                        attempts += 1;

                        if attempts < self.config.max_attempts {
                            retries += 1;
                            self.retry_count.fetch_add(1, Ordering::Relaxed);

                            let delay = self.calculate_delay(attempts);
                            tokio::time::sleep(delay).await;
                        }

                        last_error = Some(err);
                    }
                }
            } else {
                // Timeout occurred
                attempts += 1;

                if attempts < self.config.max_attempts {
                    retries += 1;
                    self.retry_count.fetch_add(1, Ordering::Relaxed);

                    let delay = self.calculate_delay(attempts);
                    tokio::time::sleep(delay).await;
                }

                // Create a timeout error
                let timeout_error = Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("Operation timed out after {}ms", timeout.as_millis()),
                ));

                last_error = Some(timeout_error);
            }
        }

        self.failure_count.fetch_add(1, Ordering::Relaxed);

        Err(RetryError::MaxAttemptsExceeded {
            attempts: self.config.max_attempts,
            error: last_error.unwrap_or_else(|| {
                Box::new(std::io::Error::other(
                    "Unknown error during retry operation",
                ))
            }),
        })
    }
}

impl Default for RetryMechanism {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let retry = RetryMechanism::default();

        let result = retry
            .execute(|| Box::pin(async { Ok::<i32, Box<dyn StdError + Send + Sync>>(42) }))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.expect("retry should succeed"), 42);

        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.retry_count, 0);
    }

    #[tokio::test]
    async fn test_retry_success_after_failure() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            ..RetryConfig::default()
        };

        let retry = RetryMechanism::new(config);

        // Use an Arc<AtomicU32> to ensure it lives long enough
        let attempts = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let result: std::result::Result<i32, RetryError> = retry
            .execute(|| {
                let attempts_clone = attempts.clone();
                Box::pin(async move {
                    attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    if attempts_clone.load(std::sync::atomic::Ordering::SeqCst) < 2 {
                        Err(Box::<dyn StdError + Send + Sync>::from(
                            "Temporary failure".to_string(),
                        ))
                    } else {
                        Ok(42)
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.expect("retry should succeed"), 42);
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 2); // Should have made 2 attempts

        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.retry_count, 1);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(10),
            ..RetryConfig::default()
        };

        let retry = RetryMechanism::new(config);

        // Use an Arc<AtomicU32> to ensure it lives long enough
        let attempts = Arc::new(std::sync::atomic::AtomicU32::new(0));

        let result: std::result::Result<i32, RetryError> = retry
            .execute(|| {
                let attempts_clone = attempts.clone();
                Box::pin(async move {
                    attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    // Always fail
                    Err(Box::<dyn StdError + Send + Sync>::from(
                        "Persistent failure".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 2); // Should have made 2 attempts

        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.retry_count, 1);
    }

    #[tokio::test]
    async fn test_retry_with_jitter() {
        let config = RetryConfig {
            use_jitter: true,
            ..RetryConfig::default()
        };

        let retry = RetryMechanism::new(config);

        // Get multiple delay calculations for the same attempt
        let delay1 = retry.calculate_delay(1);
        let delay2 = retry.calculate_delay(1);
        let delay3 = retry.calculate_delay(1);

        // At least one of them should be different due to jitter
        assert!(delay1 != delay2 || delay2 != delay3 || delay1 != delay3);
    }

    #[tokio::test]
    async fn test_max_delay() {
        let config = RetryConfig {
            max_delay: Duration::from_millis(100),
            base_delay: Duration::from_millis(10),
            backoff_strategy: BackoffStrategy::Exponential,
            ..RetryConfig::default()
        };

        let retry = RetryMechanism::new(config);

        // Calculate delay for a high attempt number
        let delay = retry.calculate_delay(10);

        // Should be capped at max_delay
        assert!(delay <= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_reset_metrics() {
        let retry = RetryMechanism::default();

        // Execute a successful operation
        let _: std::result::Result<i32, RetryError> = retry
            .execute(|| Box::pin(async { Ok::<i32, Box<dyn StdError + Send + Sync>>(42) }))
            .await;

        // Execute a failing operation
        let _: std::result::Result<i32, RetryError> = retry
            .execute(|| {
                Box::pin(async {
                    Err::<i32, Box<dyn StdError + Send + Sync>>(Box::from("Failure".to_string()))
                })
            })
            .await;

        // Verify metrics were recorded
        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 1);

        // Reset metrics
        retry.reset_metrics();

        // Verify metrics were reset
        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.retry_count, 0);
        assert_eq!(metrics.max_retries_performed, 0);
    }

    #[tokio::test]
    async fn test_predicate_retry() {
        let retry = RetryMechanism::new(RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(100),
            use_jitter: false,
            backoff_strategy: BackoffStrategy::Constant,
        });

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // Test predicate that only retries I/O errors
        let result = retry
            .execute_with_predicate(
                move || {
                    let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                    let future: RetryFuture<()> = if count < 2 {
                        // Return an I/O error for the first two attempts
                        Box::pin(async {
                            Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::ConnectionReset,
                                "Connection reset",
                            ))
                                as Box<dyn StdError + Send + Sync>)
                        })
                    } else if count < 3 {
                        // Return a permission error which shouldn't be retried
                        Box::pin(async {
                            Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::PermissionDenied,
                                "Permission denied",
                            ))
                                as Box<dyn StdError + Send + Sync>)
                        })
                    } else {
                        Box::pin(async { Ok(()) })
                    };
                    future
                },
                |err| {
                    // Only retry connection errors, not permission errors
                    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                        io_err.kind() == std::io::ErrorKind::ConnectionReset
                    } else {
                        false
                    }
                },
            )
            .await;

        // Should fail because the third attempt is a permission error
        // which doesn't match our retry predicate
        assert!(result.is_err());

        // Should have made 3 attempts (original + 2 retries)
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let retry = RetryMechanism::new(RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(100),
            use_jitter: false,
            backoff_strategy: BackoffStrategy::Constant,
        });

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // Test with timeout
        let result = retry
            .execute_with_timeout(
                move || {
                    let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                    let future: RetryFuture<()> = if count < 2 {
                        // Simulate an operation that takes too long
                        Box::pin(async {
                            tokio::time::sleep(Duration::from_millis(50)).await;
                            Ok(())
                        })
                    } else {
                        // Third attempt succeeds quickly
                        Box::pin(async { Ok(()) })
                    };
                    future
                },
                Duration::from_millis(10),
            )
            .await;

        // Should succeed eventually
        assert!(result.is_ok());

        // Should have made 3 attempts (original + 2 retries)
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exponential_backoff_with_jitter() {
        let retry = RetryMechanism::new(RetryConfig {
            max_attempts: 5,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(1000),
            use_jitter: true,
            backoff_strategy: BackoffStrategy::Exponential,
        });

        // Calculate delays for multiple attempts and check they follow exponential pattern with jitter
        let mut last_max = Duration::from_millis(0);

        for attempt in 1..=5 {
            let mut delays = Vec::new();

            // Sample several delays to check jitter
            for _ in 0..10 {
                delays.push(retry.calculate_delay(attempt));
            }

            // With jitter, delays should all be different
            let unique_delays = delays
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert!(unique_delays > 1, "Jitter should produce different delays");

            // Max delay should increase exponentially
            let max_delay = *delays.iter().max().expect("delays should be non-empty");
            if attempt > 1 {
                // Each max should be approximately double the previous max (exponential)
                assert!(max_delay > last_max);
            }

            last_max = max_delay;

            // Check it respects the max_delay
            assert!(max_delay <= retry.config.max_delay);
        }
    }
}
