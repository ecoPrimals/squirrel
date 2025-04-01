use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::error::Error as StdError;

use super::TestError;
use crate::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy, RetryError};

#[tokio::test]
async fn test_retry_mechanism_basic_success() {
    let retry = RetryMechanism::new(RetryConfig::default());
    
    let result: std::result::Result<i32, RetryError> = retry.execute(|| {
        Box::pin(async {
            Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[tokio::test]
async fn test_retry_mechanism_with_retries() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // Create a counter to track attempts
    let attempt_count = Arc::new(Mutex::new(0));
    
    let attempt_count_clone = attempt_count.clone();
    let result: std::result::Result<i32, RetryError> = retry.execute(move || {
        let count_clone = attempt_count_clone.clone();
        Box::pin(async move {
            let mut count = count_clone.lock().unwrap();
            *count += 1;
            
            if *count < 2 {
                // Fail on first attempt
                Err(Box::<dyn StdError + Send + Sync>::from(TestError::timeout("Temporary failure".to_string())))
            } else {
                // Succeed on second attempt
                Ok(42)
            }
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(*attempt_count.lock().unwrap(), 2); // Should have made 2 attempts
}

#[tokio::test]
async fn test_retry_mechanism_max_attempts_exceeded() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // This will always fail
    let result: std::result::Result<i32, RetryError> = retry.execute(|| {
        Box::pin(async {
            Err(Box::<dyn StdError + Send + Sync>::from(TestError::connection("Persistent failure".to_string())))
        })
    }).await;
    
    assert!(result.is_err());
    match result {
        Err(RetryError::MaxAttemptsExceeded { attempts, .. }) => {
            assert_eq!(attempts, 3); // Should have tried 3 times
        },
        _ => panic!("Expected MaxAttemptsExceeded error"),
    }
}

#[tokio::test]
async fn test_retry_different_backoff_strategies() {
    // Test Constant backoff
    let retry1 = RetryMechanism::new(RetryConfig {
        backoff_strategy: BackoffStrategy::Constant,
        base_delay: Duration::from_millis(10),
        use_jitter: false,
        ..RetryConfig::default()
    });
    
    // For Constant backoff: delays should be approximately the same
    let delay1 = retry1.calculate_delay(1);
    let delay2 = retry1.calculate_delay(2);
    let delay3 = retry1.calculate_delay(3);
    
    // All delays should be approximately equal for constant backoff
    assert!((delay1.as_millis() as i64 - delay2.as_millis() as i64).abs() < 5,
           "Constant backoff delays should be approximately the same");
    assert!((delay1.as_millis() as i64 - delay3.as_millis() as i64).abs() < 5,
           "Constant backoff delays should be approximately the same");
    
    // Test Exponential backoff
    let retry2 = RetryMechanism::new(RetryConfig {
        base_delay: Duration::from_millis(10),
        backoff_strategy: BackoffStrategy::Exponential,
        use_jitter: false,
        ..RetryConfig::default()
    });
    
    // For Exponential backoff: should increase exponentially (approximately)
    let delay1 = retry2.calculate_delay(1);
    let delay2 = retry2.calculate_delay(2);
    let delay3 = retry2.calculate_delay(3);
    
    // Verify exponential growth pattern - delays should increase
    assert!(delay2 > delay1, "Second delay should be greater than first");
    assert!(delay3 > delay2, "Third delay should be greater than second");
    
    // Check that the ratio between consecutive delays is approximately 2
    // (with reasonable tolerance for implementation details)
    let ratio1_2 = delay2.as_millis() as f64 / delay1.as_millis() as f64;
    let ratio2_3 = delay3.as_millis() as f64 / delay2.as_millis() as f64;
    
    assert!((ratio1_2 - 2.0).abs() < 0.5, "Expected ratio around 2, got {}", ratio1_2);
    assert!((ratio2_3 - 2.0).abs() < 0.5, "Expected ratio around 2, got {}", ratio2_3);
}

#[tokio::test]
async fn test_retry_with_jitter() {
    let retry = RetryMechanism::new(RetryConfig {
        base_delay: Duration::from_millis(100),
        use_jitter: true,
        ..RetryConfig::default()
    });
    
    // Get multiple delay calculations for the same attempt
    let delay1 = retry.calculate_delay(1);
    let delay2 = retry.calculate_delay(1);
    let delay3 = retry.calculate_delay(1);
    
    // They should all be different due to jitter
    assert!(delay1 != delay2 || delay2 != delay3 || delay1 != delay3);
}

#[tokio::test]
async fn test_retry_respects_max_delay() {
    let retry = RetryMechanism::new(RetryConfig {
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        backoff_strategy: BackoffStrategy::Exponential,
        ..RetryConfig::default()
    });
    
    // Calculate delay for a high attempt number
    let delay = retry.calculate_delay(10);
    
    // Should be capped at max_delay
    assert!(delay <= Duration::from_millis(50));
}

#[tokio::test]
async fn test_retry_success_first_attempt() {
    let retry = RetryMechanism::default();
    
    let result: std::result::Result<i32, RetryError> = retry.execute(|| {
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
        max_attempts: 5,
        base_delay: Duration::from_millis(1), // Use small delay for tests
        ..RetryConfig::default()
    };
    
    let retry = RetryMechanism::new(config);
    
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    
    let result: std::result::Result<i32, RetryError> = retry.execute(move || {
        let counter = counter_clone.clone();
        Box::pin(async move {
            let mut count = counter.lock().unwrap();
            *count += 1;
            
            if *count < 3 {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError::timeout("Temporary failure".to_string())))
            } else {
                Ok(42)
            }
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(*counter.lock().unwrap(), 3);
    
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
    
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    
    let result: std::result::Result<i32, RetryError> = retry.execute(move || {
        let counter = counter_clone.clone();
        Box::pin(async move {
            let mut count = counter.lock().unwrap();
            *count += 1;
            
            Err(Box::<dyn StdError + Send + Sync>::from(TestError::connection("Persistent failure".to_string())))
        })
    }).await;
    
    assert!(result.is_err());
    assert_eq!(*counter.lock().unwrap(), 3); // Initial attempt + 2 retries
    
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
    
    // Use approximate checks to avoid timing issues
    let delay1 = retry.calculate_delay(1).as_millis();
    let delay2 = retry.calculate_delay(2).as_millis();
    let delay3 = retry.calculate_delay(3).as_millis();
    
    assert!((delay1 as i64 - 10).abs() <= 5, "Expected around 10ms, got {}ms", delay1);
    assert!((delay2 as i64 - 10).abs() <= 5, "Expected around 10ms, got {}ms", delay2);
    assert!((delay3 as i64 - 10).abs() <= 5, "Expected around 10ms, got {}ms", delay3);
    
    // Test linear backoff
    let config = RetryConfig {
        backoff_strategy: BackoffStrategy::Linear,
        base_delay: Duration::from_millis(10),
        use_jitter: false,
        ..RetryConfig::default()
    };
    let retry = RetryMechanism::new(config);
    
    // Use approximate checks to avoid timing issues
    let delay1 = retry.calculate_delay(1).as_millis();
    let delay2 = retry.calculate_delay(2).as_millis();
    let delay3 = retry.calculate_delay(3).as_millis();
    
    assert!((delay1 as i64 - 10).abs() <= 5, "Expected around 10ms, got {}ms", delay1);
    assert!((delay2 as i64 - 20).abs() <= 5, "Expected around 20ms, got {}ms", delay2);
    assert!((delay3 as i64 - 30).abs() <= 5, "Expected around 30ms, got {}ms", delay3);
    
    // Test exponential backoff
    let config = RetryConfig {
        backoff_strategy: BackoffStrategy::Exponential,
        base_delay: Duration::from_millis(10),
        use_jitter: false,
        ..RetryConfig::default()
    };
    let retry = RetryMechanism::new(config);
    
    // Use approximate checks to avoid timing issues
    let delay1 = retry.calculate_delay(1).as_millis();
    let delay2 = retry.calculate_delay(2).as_millis();
    let delay3 = retry.calculate_delay(3).as_millis();
    
    assert!((delay1 as i64 - 10).abs() <= 5, "Expected around 10ms, got {}ms", delay1);
    assert!((delay2 as i64 - 20).abs() <= 5, "Expected around 20ms, got {}ms", delay2);
    assert!((delay3 as i64 - 40).abs() <= 5, "Expected around 40ms, got {}ms", delay3);
}

#[tokio::test]
async fn test_retry_metrics() {
    let retry = RetryMechanism::default();
    
    // Execute a successful operation
    let _: std::result::Result<i32, RetryError> = retry.execute(|| {
        Box::pin(async {
            Ok::<i32, Box<dyn StdError + Send + Sync>>(42)
        })
    }).await;
    
    // Execute a failing operation but we'll ignore the error
    let _: std::result::Result<i32, RetryError> = retry.execute(|| {
        Box::pin(async {
            Err::<i32, Box<dyn StdError + Send + Sync>>(Box::from(TestError::generic("Failure".to_string())))
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