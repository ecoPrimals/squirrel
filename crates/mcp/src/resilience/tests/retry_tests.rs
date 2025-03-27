use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

use super::TestError;
use crate::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};

#[tokio::test]
async fn test_retry_success_first_attempt() {
    let retry = RetryMechanism::default();
    
    let result = retry.execute(|| {
        Ok::<_, TestError>(42)
    });
    
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
        max_attempts: 5,
        base_delay: Duration::from_millis(1), // Use small delay for tests
        ..RetryConfig::default()
    };
    
    let retry = RetryMechanism::new(config);
    
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    
    let result = retry.execute(move || {
        let mut count = counter_clone.borrow_mut();
        *count += 1;
        
        if *count < 3 {
            Err(TestError::timeout("Temporary failure".to_string()))
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
}

#[tokio::test]
async fn test_retry_max_attempts_exceeded() {
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(1), // Use small delay for tests
        ..RetryConfig::default()
    };
    
    let retry = RetryMechanism::new(config);
    
    let counter = Rc::new(RefCell::new(0));
    let counter_clone = counter.clone();
    
    let result = retry.execute(move || {
        let mut count = counter_clone.borrow_mut();
        *count += 1;
        
        Err(TestError::connection("Persistent failure".to_string()))
    });
    
    assert!(result.is_err());
    assert_eq!(*counter.borrow(), 3); // Initial attempt + 2 retries
    
    let metrics = retry.get_metrics();
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.failure_count, 1);
    assert_eq!(metrics.retry_count, 2);
}

#[tokio::test]
async fn test_retry_backoff_strategies() {
    // Test constant backoff
    let config = RetryConfig {
        backoff_strategy: BackoffStrategy::Constant,
        base_delay: Duration::from_millis(10),
        use_jitter: false,
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
}

#[tokio::test]
async fn test_retry_metrics() {
    let retry = RetryMechanism::default();
    
    // Execute a successful operation
    let _ = retry.execute(|| {
        Ok::<_, TestError>(42)
    });
    
    // Execute a failing operation but we'll ignore the error
    let _ = retry.execute(|| {
        Err::<i32, _>(TestError::generic("Failure".to_string()))
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