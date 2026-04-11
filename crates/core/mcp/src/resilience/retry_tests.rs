// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
