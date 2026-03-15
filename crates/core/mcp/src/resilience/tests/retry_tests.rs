// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the retry mechanism implementation

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use std::error::Error as StdError;

use tokio::test;
use tokio::time::sleep;
use std::io::{Error as IoError, ErrorKind};

use crate::resilience::{
    RetryMechanism,
    RetryConfig,
    BackoffStrategy,
    StandardRetryPolicy,
    RetryPolicy,
    ResilienceError
};

// Helper function to create a test function that fails a specified number of times
fn create_failing_function(
    failures: u32
) -> impl FnMut() -> Pin<Box<dyn Future<Output = std::result::Result<(), Box<dyn StdError + Send + Sync>>> + Send>> {
    let counter = Arc::new(AtomicU32::new(0));
    
    move || {
        let counter = counter.clone();
        
        Box::pin(async move {
            let attempt = counter.fetch_add(1, Ordering::SeqCst);
            
            if attempt < failures {
                Err(Box::new(IoError::new(
                    ErrorKind::Other, 
                    format!("Test failure {}", attempt)
                )) as Box<dyn StdError + Send + Sync>)
            } else {
                Ok(())
            }
        })
    }
}

#[test]
async fn test_retry_mechanism_success() {
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // Create a function that fails twice then succeeds
    let mut operation = create_failing_function(2);
    
    // Execute with retry
    let result = retry.execute(&mut operation).await;
    
    // Should succeed after 3 attempts (2 failures + 1 success)
    assert!(result.is_ok());
    
    // Check metrics
    let metrics = retry.get_metrics();
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.failure_count, 0);
    assert_eq!(metrics.retry_count, 2);
}

#[test]
async fn test_retry_mechanism_exhaustion() {
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // Create a function that always fails
    let mut operation = create_failing_function(10);
    
    // Execute with retry
    let result = retry.execute(&mut operation).await;
    
    // Should fail after 3 attempts
    assert!(result.is_err());
    match result {
        Err(err) => match err {
            crate::resilience::retry::RetryError::MaxAttemptsExceeded { attempts, .. } => {
                assert_eq!(attempts, 3);
            },
            _ => panic!("Expected MaxAttemptsExceeded error"),
        },
        _ => panic!("Expected error"),
    }
    
    // Check metrics
    let metrics = retry.get_metrics();
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.failure_count, 1);
    assert_eq!(metrics.retry_count, 2); // 3 attempts = 2 retries
}

#[test]
async fn test_retry_with_predicate() {
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 5,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    
    // Create an operation that returns different error types
    let operation = move || {
        let counter = counter_clone.clone();
        
        Box::pin(async move {
            let attempt = counter.fetch_add(1, Ordering::SeqCst);
            
            match attempt {
                0 => Err(Box::new(IoError::new(
                    ErrorKind::ConnectionReset, 
                    "Connection reset"
                )) as Box<dyn StdError + Send + Sync>),
                1 => Err(Box::new(IoError::new(
                    ErrorKind::PermissionDenied, 
                    "Permission denied"
                )) as Box<dyn StdError + Send + Sync>),
                2 => Err(Box::new(IoError::new(
                    ErrorKind::ConnectionReset, 
                    "Another connection reset"
                )) as Box<dyn StdError + Send + Sync>),
                _ => Ok(()),
            }
        }) as Pin<Box<dyn Future<Output = std::result::Result<(), Box<dyn StdError + Send + Sync>>> + Send>>
    };
    
    // Execute with predicate that only retries ConnectionReset errors
    let result = retry.execute_with_predicate(
        operation,
        |err| {
            if let Some(io_err) = err.downcast_ref::<IoError>() {
                io_err.kind() == ErrorKind::ConnectionReset
            } else {
                false
            }
        }
    ).await;
    
    // Should fail because we don't retry PermissionDenied errors
    assert!(result.is_err());
    
    // Should have made exactly 2 attempts (stops at PermissionDenied)
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
async fn test_retry_with_timeout() {
    // Create a retry mechanism
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();
    
    // Create an operation that takes time
    let operation = move || {
        let counter = counter_clone.clone();
        
        Box::pin(async move {
            let attempt = counter.fetch_add(1, Ordering::SeqCst);
            
            match attempt {
                0 | 1 => {
                    // These will timeout
                    sleep(Duration::from_millis(50)).await;
                    Ok(())
                },
                _ => {
                    // This will complete quickly
                    Ok(())
                }
            }
        }) as Pin<Box<dyn Future<Output = std::result::Result<(), Box<dyn StdError + Send + Sync>>> + Send>>
    };
    
    // Execute with timeout
    let result = retry.execute_with_timeout(
        operation,
        Duration::from_millis(20) // 20ms timeout
    ).await;
    
    // Should succeed on third attempt
    assert!(result.is_ok());
    
    // Should have made 3 attempts (2 timeouts + 1 success)
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

/// Test retry policy configuration and behavior
#[tokio::test]
async fn test_retry_policy() {
    // Create a StandardRetryPolicy with deterministic settings (no jitter)
    let policy = StandardRetryPolicy::new(
        3, // max_retries
        Duration::from_millis(10), // base_delay
        Duration::from_millis(1000), // max_delay
        BackoffStrategy::Exponential, // backoff_strategy 
        false, // use_jitter (disabled for deterministic testing)
    );
    
    // Test different error types
    assert!(!policy.should_retry(0, &ResilienceError::CircuitOpen("Circuit open".to_string())));
    assert!(policy.should_retry(0, &ResilienceError::General("General error".to_string())));
    assert!(!policy.should_retry(0, &ResilienceError::RecoveryFailed("Recovery failed".to_string())));
    
    // Test backoff_duration behavior with strictly increasing attempt numbers
    let d1 = policy.backoff_duration(1);
    let d2 = policy.backoff_duration(2);
    let d3 = policy.backoff_duration(3);
    
    println!("d1: {:?}, d2: {:?}, d3: {:?}", d1, d2, d3);
    
    // With exponential backoff and no jitter, d2 should be greater than d1
    assert!(d2 > d1, "d2 ({:?}) should be greater than d1 ({:?})", d2, d1);
    
    // d3 should be greater than d2 with exponential backoff and no jitter
    assert!(d3 > d2, "d3 ({:?}) should be greater than d2 ({:?})", d3, d2);
    
    // Ensure max delay is respected
    let large_attempt = policy.backoff_duration(10);
    assert!(large_attempt <= Duration::from_millis(1000));
}

#[test]
async fn test_exponential_backoff() {
    // Create a retry mechanism with exponential backoff
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(1000),
        use_jitter: false, // Disable jitter for deterministic testing
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    // Test delay calculations
    let d1 = retry.calculate_delay(1);
    let d2 = retry.calculate_delay(2);
    let d3 = retry.calculate_delay(3);
    
    // For exponential backoff, should roughly double each time
    assert!(d2 >= d1.mul_f32(1.8)); // Allow some floating point imprecision
    assert!(d3 >= d2.mul_f32(1.8));
    
    // Test with a function that fails all attempts
    let mut operation = create_failing_function(10);
    
    // Execute with retry and measure time
    let start = tokio::time::Instant::now();
    let _result = retry.execute(&mut operation).await;
    let elapsed = start.elapsed();
    
    // Verify the execution time is at least the sum of the delays
    // d1 + d2 (we don't wait after the last attempt)
    let min_expected = d1 + d2;
    assert!(elapsed >= min_expected);
}

#[test]
async fn test_jitter() {
    // Create a retry mechanism with jitter
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(1000),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // Test jitter behavior by collecting multiple samples
    let mut samples = Vec::new();
    for _ in 0..10 {
        samples.push(retry.calculate_delay(1));
    }
    
    // Check that jitter produces different values
    let unique_samples = samples.iter().collect::<std::collections::HashSet<_>>().len();
    assert!(unique_samples > 1, "Jitter should produce different delays");
    
    // Ensure all samples are between 0 and the base delay
    let base_delay = Duration::from_millis(100);
    for sample in &samples {
        assert!(*sample <= base_delay);
        assert!(*sample > Duration::from_millis(0));
    }
} 