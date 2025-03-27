//! Retry mechanism for the MCP resilience framework
//! 
//! This module provides functionality to retry operations that might fail transiently.

use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use rand::{Rng, thread_rng};

use crate::resilience::{ResilienceError, Result};

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

/// Retry strategy for handling transient failures
#[derive(Debug)]
pub struct RetryMechanism {
    /// Configuration for the retry mechanism
    config: RetryConfig,
    /// Metrics for retry operations
    success_count: AtomicUsize,
    /// Number of failed operations
    failure_count: AtomicUsize,
    /// Total number of retries performed
    retry_count: AtomicUsize,
    /// Maximum number of retries for a single operation
    max_retries_performed: AtomicUsize,
}

impl RetryMechanism {
    /// Create a new retry mechanism with the specified configuration
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            success_count: AtomicUsize::new(0),
            failure_count: AtomicUsize::new(0),
            retry_count: AtomicUsize::new(0),
            max_retries_performed: AtomicUsize::new(0),
        }
    }
    
    /// Create a new retry mechanism with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }
    
    /// Get retry metrics
    pub fn get_metrics(&self) -> RetryMetrics {
        RetryMetrics {
            success_count: self.success_count.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            retry_count: self.retry_count.load(Ordering::Relaxed),
            max_retries_performed: self.max_retries_performed.load(Ordering::Relaxed),
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
    pub fn execute<F, T, E>(&self, operation: F) -> Result<T, RetryError>
    where
        F: Fn() -> std::result::Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut attempts = 0;
        let mut last_error = None;
        let mut retries = 0;
        
        while attempts < self.config.max_attempts {
            match operation() {
                Ok(value) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);
                    
                    // Update max retries metrics if needed
                    let current_max = self.max_retries_performed.load(Ordering::Relaxed);
                    if retries > current_max {
                        self.max_retries_performed.store(retries, Ordering::Relaxed);
                    }
                    
                    return Ok(value);
                }
                Err(err) => {
                    attempts += 1;
                    
                    if attempts < self.config.max_attempts {
                        retries += 1;
                        self.retry_count.fetch_add(1, Ordering::Relaxed);
                        
                        let delay = self.calculate_delay(attempts);
                        std::thread::sleep(delay);
                    }
                    
                    last_error = Some(Box::new(err) as Box<dyn std::error::Error + Send + Sync>);
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
    fn calculate_delay(&self, attempt: u32) -> Duration {
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
    use std::cell::RefCell;
    use std::rc::Rc;
    
    #[test]
    fn test_retry_success_first_attempt() {
        let retry = RetryMechanism::default();
        
        let result = retry.execute(|| {
            Ok::<_, &'static str>(42)
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.retry_count, 0);
        assert_eq!(metrics.max_retries_performed, 0);
    }
    
    #[test]
    fn test_retry_success_after_failure() {
        let retry = RetryMechanism::default();
        
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let result = retry.execute(|| {
            let mut count = counter_clone.borrow_mut();
            *count += 1;
            
            if *count < 3 {
                Err("Temporary failure")
            } else {
                Ok(42)
            }
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*counter.borrow(), 3);
        
        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.retry_count, 2);
        assert_eq!(metrics.max_retries_performed, 2);
    }
    
    #[test]
    fn test_retry_max_attempts_exceeded() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1), // Make test run faster
            ..RetryConfig::default()
        };
        
        let retry = RetryMechanism::new(config);
        
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();
        
        let result = retry.execute(|| {
            let mut count = counter_clone.borrow_mut();
            *count += 1;
            
            Err::<i32, _>("Persistent failure")
        });
        
        assert!(result.is_err());
        assert_eq!(*counter.borrow(), 3); // Initial attempt + 2 retries
        
        let metrics = retry.get_metrics();
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.retry_count, 2);
    }
    
    #[test]
    fn test_backoff_strategies() {
        // Test constant backoff
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Constant,
            base_delay: Duration::from_millis(10),
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        assert_eq!(retry.calculate_delay(1).as_millis(), 10);
        assert_eq!(retry.calculate_delay(2).as_millis(), 10);
        assert_eq!(retry.calculate_delay(3).as_millis(), 10);
        
        // Test linear backoff
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Linear,
            base_delay: Duration::from_millis(10),
            use_jitter: false,
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        assert_eq!(retry.calculate_delay(1).as_millis(), 10);
        assert_eq!(retry.calculate_delay(2).as_millis(), 20);
        assert_eq!(retry.calculate_delay(3).as_millis(), 30);
        
        // Test exponential backoff
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Exponential,
            base_delay: Duration::from_millis(10),
            use_jitter: false,
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        assert_eq!(retry.calculate_delay(1).as_millis(), 10);
        assert_eq!(retry.calculate_delay(2).as_millis(), 20);
        assert_eq!(retry.calculate_delay(3).as_millis(), 40);
        
        // Test fibonacci backoff
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Fibonacci,
            base_delay: Duration::from_millis(10),
            use_jitter: false,
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        assert_eq!(retry.calculate_delay(1).as_millis(), 10);
        assert_eq!(retry.calculate_delay(2).as_millis(), 10);
        assert_eq!(retry.calculate_delay(3).as_millis(), 20);
        assert_eq!(retry.calculate_delay(4).as_millis(), 30);
        assert_eq!(retry.calculate_delay(5).as_millis(), 50);
    }
    
    #[test]
    fn test_jitter() {
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Constant,
            base_delay: Duration::from_millis(100),
            use_jitter: true,
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        // With jitter, the delays shouldn't be exactly the base_delay
        // and should vary between calls
        let mut all_same = true;
        let first = retry.calculate_delay(1);
        
        for _ in 0..10 {
            let delay = retry.calculate_delay(1);
            if delay != first {
                all_same = false;
                break;
            }
        }
        
        assert!(!all_same, "Jitter should cause varying delays");
    }
    
    #[test]
    fn test_max_delay() {
        let config = RetryConfig {
            backoff_strategy: BackoffStrategy::Exponential,
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            use_jitter: false,
            ..RetryConfig::default()
        };
        let retry = RetryMechanism::new(config);
        
        // At attempt 5, the exponential delay would be 10 * 2^4 = 160ms
        // But max_delay is 50ms, so it should be capped
        assert_eq!(retry.calculate_delay(5).as_millis(), 50);
    }
    
    #[test]
    fn test_reset_metrics() {
        let retry = RetryMechanism::default();
        
        // Execute a successful operation
        let _ = retry.execute(|| {
            Ok::<_, &'static str>(42)
        });
        
        // Execute a failing operation
        let _ = retry.execute(|| {
            Err::<i32, _>("Failure")
        });
        
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