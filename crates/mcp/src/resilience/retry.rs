//! Retry mechanism for the MCP resilience framework
//! 
//! This module provides functionality to retry operations that might fail transiently.

use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use rand::{Rng, thread_rng};
use std::error::Error as StdError;

use crate::resilience::ResilienceError;

/// Defines different backoff strategies for retry operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackoffStrategy {
    /// Constant delay between retries
    Constant,
    /// Linear increase in delay (base_delay * attempt)
    Linear,
    /// Exponential increase in delay (base_delay * 2^attempt)
    Exponential,
    /// Fibonacci sequence for delay
    Fibonacci,
}

/// Configuration for the retry mechanism
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries (will be used with the backoff strategy)
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to use jitter to avoid retry storms
    pub use_jitter: bool,
    /// The backoff strategy to use for calculating delays
    pub backoff_strategy: BackoffStrategy,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
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

impl From<RetryError> for ResilienceError {
    fn from(value: RetryError) -> Self {
        match value {
            RetryError::MaxAttemptsExceeded { attempts, error } => {
                ResilienceError::RetryExceeded(format!(
                    "Maximum retry attempts ({}) exceeded: {}", 
                    attempts, error
                ))
            },
            RetryError::Cancelled(msg) => {
                ResilienceError::General(format!("Retry operation cancelled: {}", msg))
            },
            RetryError::Internal(msg) => {
                ResilienceError::General(format!("Retry internal error: {}", msg))
            },
        }
    }
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
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            success_count: Arc::new(AtomicU32::new(0)),
            failure_count: Arc::new(AtomicU32::new(0)),
            retry_count: Arc::new(AtomicU32::new(0)),
            max_retries_performed: Arc::new(AtomicU32::new(0)),
        }
    }
    
    /// Create a new retry mechanism with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }
    
    /// Get retry metrics
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
    pub async fn execute<F, T>(&self, mut operation: F) -> std::result::Result<T, RetryError>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send>>,
        T: Send + 'static,
    {
        let mut attempts = 0;
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
                        self.max_retries_performed.store(retries as u32, Ordering::Relaxed);
                    }
                    
                    return Ok(value);
                }
                Err(err) => {
                    attempts += 1;
                    
                    if attempts < self.config.max_attempts {
                        retries += 1;
                        self.retry_count.fetch_add(1, Ordering::Relaxed);
                        
                        let delay = self.calculate_delay(attempts as u32);
                        tokio::time::sleep(delay).await;
                    }
                    
                    last_error = Some(err);
                }
            }
        }
        
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        
        Err(RetryError::MaxAttemptsExceeded { 
            attempts: self.config.max_attempts, 
            error: last_error.unwrap() 
        })
    }
    
    /// Calculate the delay for the next retry based on the backoff strategy
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_ms = self.config.base_delay.as_millis() as u64;
        
        // Calculate delay based on strategy
        let delay_ms = match self.config.backoff_strategy {
            BackoffStrategy::Constant => base_ms,
            
            BackoffStrategy::Linear => {
                base_ms * attempt as u64
            },
            
            BackoffStrategy::Exponential => {
                // Use saturating_mul to prevent overflow
                let factor = 2u64.saturating_pow(attempt - 1);
                base_ms.saturating_mul(factor)
            },
            
            BackoffStrategy::Fibonacci => {
                if attempt <= 1 {
                    base_ms
                } else if attempt == 2 {
                    base_ms
                } else {
                    let mut prev = base_ms;
                    let mut curr = base_ms;
                    
                    for _ in 2..attempt {
                        let next = prev.saturating_add(curr);
                        prev = curr;
                        curr = next;
                    }
                    
                    curr
                }
            },
        };
        
        // Apply jitter if configured
        let final_delay = if self.config.use_jitter {
            // Add random jitter between 0% and 25%
            let jitter_factor = thread_rng().gen_range(0.0..0.25);
            let jitter = (delay_ms as f64 * jitter_factor) as u64;
            delay_ms.saturating_add(jitter)
        } else {
            delay_ms
        };
        
        // Cap at max delay
        let max_ms = self.config.max_delay.as_millis() as u64;
        
        Duration::from_millis(final_delay.min(max_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let retry = RetryMechanism::default();
        
        let result = retry.execute(|| {
            Box::pin(async {
                Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
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
        
        let result: std::result::Result<i32, RetryError> = retry.execute(|| {
            let attempts_clone = attempts.clone();
            Box::pin(async move {
                attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                if attempts_clone.load(std::sync::atomic::Ordering::SeqCst) < 2 {
                    Err(Box::<dyn StdError + Send + Sync>::from("Temporary failure".to_string()))
                } else {
                    Ok(42)
                }
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
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
        
        let result: std::result::Result<i32, RetryError> = retry.execute(|| {
            let attempts_clone = attempts.clone();
            Box::pin(async move {
                attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                // Always fail
                Err(Box::<dyn StdError + Send + Sync>::from("Persistent failure".to_string()))
            })
        }).await;
        
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
        let _: std::result::Result<i32, RetryError> = retry.execute(|| {
            Box::pin(async {
                Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
            })
        }).await;
        
        // Execute a failing operation
        let _: std::result::Result<i32, RetryError> = retry.execute(|| {
            Box::pin(async {
                Err::<i32, Box<dyn StdError + Send + Sync>>(Box::from("Failure".to_string()))
            })
        }).await;
        
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
} 