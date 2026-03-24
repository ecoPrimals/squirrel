// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Async test utilities for concurrent, robust testing
//!
//! This module provides utilities to eliminate sleeps and make tests deterministic.
//! Philosophy: "Test issues WILL BE production issues" - we must be robust and concurrent.

use std::future::Future;
use std::time::Duration;
use tokio::time::{sleep, timeout};

/// Error type for timeout operations
#[derive(Debug)]
pub struct TimeoutError {
    pub message: String,
    pub duration: Duration,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timeout after {:?}: {}", self.duration, self.message)
    }
}

impl std::error::Error for TimeoutError {}

/// Wait for a condition to become true with timeout
///
/// This is the CORRECT way to wait for async operations instead of using sleep().
///
/// # Example
/// ```rust
/// wait_for(|| service.is_ready(), Duration::from_secs(5)).await?;
/// ```
pub async fn wait_for<F>(mut condition: F, timeout_duration: Duration) -> Result<(), TimeoutError>
where
    F: FnMut() -> bool,
{
    let result = timeout(timeout_duration, async {
        while !condition() {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(_) => Err(TimeoutError {
            message: "Condition never became true".to_string(),
            duration: timeout_duration,
        }),
    }
}

/// Wait for an async condition to become true with timeout
///
/// Like `wait_for` but for async conditions.
///
/// # Example
/// ```rust
/// wait_for_async(|| async { service.check_ready().await }, Duration::from_secs(5)).await?;
/// ```
pub async fn wait_for_async<F, Fut>(
    mut condition: F,
    timeout_duration: Duration,
) -> Result<(), TimeoutError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let result = timeout(timeout_duration, async {
        while !condition().await {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(_) => Err(TimeoutError {
            message: "Async condition never became true".to_string(),
            duration: timeout_duration,
        }),
    }
}

/// Retry an operation until it succeeds or max attempts reached
///
/// Use this instead of sleeps when dealing with eventually-consistent operations.
///
/// # Example
/// ```rust
/// let result = retry_until_success(
///     || perform_operation(),
///     10,  // max attempts
///     Duration::from_millis(50),  // delay between attempts
/// ).await?;
/// ```
pub async fn retry_until_success<F, T, E>(
    mut operation: F,
    max_attempts: usize,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for attempt in 1..=max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(_) => sleep(delay).await,
        }
    }
    unreachable!("Loop should have returned by now")
}

/// Retry an async operation until it succeeds or max attempts reached
///
/// Async version of `retry_until_success`.
///
/// # Example
/// ```rust
/// let result = retry_until_success_async(
///     || async { fetch_data().await },
///     10,
///     Duration::from_millis(50),
/// ).await?;
/// ```
pub async fn retry_until_success_async<F, Fut, T, E>(
    mut operation: F,
    max_attempts: usize,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_attempts => return Err(e),
            Err(_) => sleep(delay).await,
        }
    }
    unreachable!("Loop should have returned by now")
}

/// Wait for multiple conditions to all become true
///
/// # Example
/// ```rust
/// wait_for_all(
///     &[
///         || service_a.is_ready(),
///         || service_b.is_ready(),
///     ],
///     Duration::from_secs(5)
/// ).await?;
/// ```
pub async fn wait_for_all<F>(
    conditions: &[F],
    timeout_duration: Duration,
) -> Result<(), TimeoutError>
where
    F: Fn() -> bool,
{
    let result = timeout(timeout_duration, async {
        loop {
            if conditions.iter().all(|cond| cond()) {
                break;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    match result {
        Ok(()) => Ok(()),
        Err(_) => Err(TimeoutError {
            message: "Not all conditions became true".to_string(),
            duration: timeout_duration,
        }),
    }
}

/// Wait for any condition to become true
///
/// # Example
/// ```rust
/// wait_for_any(
///     &[
///         || service_a.is_ready(),
///         || service_b.is_ready(),
///     ],
///     Duration::from_secs(5)
/// ).await?;
/// ```
pub async fn wait_for_any<F>(
    conditions: &[F],
    timeout_duration: Duration,
) -> Result<usize, TimeoutError>
where
    F: Fn() -> bool,
{
    let result = timeout(timeout_duration, async {
        loop {
            for (index, cond) in conditions.iter().enumerate() {
                if cond() {
                    return index;
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    match result {
        Ok(index) => Ok(index),
        Err(_) => Err(TimeoutError {
            message: "No condition became true".to_string(),
            duration: timeout_duration,
        }),
    }
}

/// Macro to assert a condition eventually becomes true
///
/// This replaces patterns like:
/// ```rust
/// // BAD
/// operation().await;
/// tokio::time::sleep(Duration::from_millis(100)).await;
/// assert!(condition());
/// ```
///
/// With:
/// ```rust
/// // GOOD
/// operation().await;
/// assert_eventually!(condition(), Duration::from_secs(5));
/// ```
#[macro_export]
macro_rules! assert_eventually {
    ($condition:expr, $timeout:expr) => {
        $crate::common::async_test_utils::wait_for(|| $condition, $timeout)
            .await
            .expect(&format!(
                "Condition `{}` never became true within {:?}",
                stringify!($condition),
                $timeout
            ));
    };
    ($condition:expr, $timeout:expr, $($arg:tt)+) => {
        $crate::common::async_test_utils::wait_for(|| $condition, $timeout)
            .await
            .expect(&format!($($arg)+));
    };
}

/// Macro to assert an async condition eventually becomes true
#[macro_export]
macro_rules! assert_eventually_async {
    ($condition:expr, $timeout:expr) => {
        $crate::common::async_test_utils::wait_for_async(|| async { $condition.await }, $timeout)
            .await
            .expect(&format!(
                "Async condition `{}` never became true within {:?}",
                stringify!($condition),
                $timeout
            ));
    };
    ($condition:expr, $timeout:expr, $($arg:tt)+) => {
        $crate::common::async_test_utils::wait_for_async(|| async { $condition.await }, $timeout)
            .await
            .expect(&format!($($arg)+));
    };
}

/// Helper to create a channel-based notification system
///
/// Use this when you need to be notified when an async operation completes.
///
/// # Example
/// ```rust
/// let (notifier, waiter) = create_notification();
/// tokio::spawn(async move {
///     perform_operation().await;
///     notifier.notify();
/// });
/// waiter.wait().await; // Wait for notification
/// ```
pub fn create_notification() -> (Notifier, Waiter) {
    use std::sync::Arc;
    use tokio::sync::Notify;

    let notify = Arc::new(Notify::new());
    (
        Notifier {
            notify: notify.clone(),
        },
        Waiter { notify },
    )
}

/// Notifier side of a notification channel
pub struct Notifier {
    notify: std::sync::Arc<tokio::sync::Notify>,
}

impl Notifier {
    /// Notify waiting tasks
    pub fn notify(&self) {
        self.notify.notify_one();
    }

    /// Notify all waiting tasks
    pub fn notify_all(&self) {
        self.notify.notify_waiters();
    }
}

/// Waiter side of a notification channel
pub struct Waiter {
    notify: std::sync::Arc<tokio::sync::Notify>,
}

impl Waiter {
    /// Wait for notification
    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    /// Wait for notification with timeout
    pub async fn wait_timeout(&self, duration: Duration) -> Result<(), TimeoutError> {
        timeout(duration, self.notify.notified())
            .await
            .map_err(|_| TimeoutError {
                message: "Notification timeout".to_string(),
                duration,
            })
    }
}

/// Helper to run async code with a timeout
///
/// # Example
/// ```rust
/// with_timeout(Duration::from_secs(5), async {
///     // Your async code here
///     perform_operation().await
/// }).await?;
/// ```
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    timeout(duration, future).await.map_err(|_| TimeoutError {
        message: "Operation timed out".to_string(),
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_wait_for_success() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        // Spawn task that sets flag after 50ms
        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Wait for flag (should succeed)
        wait_for(|| flag.load(Ordering::SeqCst), Duration::from_secs(1))
            .await
            .expect("Flag should be set");
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let flag = Arc::new(AtomicBool::new(false));

        // Wait for flag that never gets set (should timeout)
        let result = wait_for(|| flag.load(Ordering::SeqCst), Duration::from_millis(100)).await;

        assert!(result.is_err(), "Should timeout");
    }

    #[tokio::test]
    async fn test_retry_until_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // Operation that succeeds on 3rd attempt
        let result = retry_until_success(
            || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count >= 2 {
                    Ok(count)
                } else {
                    Err("Not yet")
                }
            },
            5,
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), 2);
    }

    #[tokio::test]
    async fn test_wait_for_all() {
        let flag1 = Arc::new(AtomicBool::new(false));
        let flag2 = Arc::new(AtomicBool::new(false));
        let flag1_clone = flag1.clone();
        let flag2_clone = flag2.clone();

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            flag1_clone.store(true, Ordering::SeqCst);
            sleep(Duration::from_millis(50)).await;
            flag2_clone.store(true, Ordering::SeqCst);
        });

        // Clone flags for the closures (they need to be the same type)
        let flag1_check = flag1.clone();
        let flag2_check = flag2.clone();

        // Use array with same closure type
        wait_for_all(
            &[
                move || flag1_check.load(Ordering::SeqCst),
                move || flag2_check.load(Ordering::SeqCst),
            ],
            Duration::from_secs(1),
        )
        .await
        .expect("Both flags should be set");
    }

    #[tokio::test]
    async fn test_notification() {
        let (notifier, waiter) = create_notification();

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            notifier.notify();
        });

        waiter
            .wait_timeout(Duration::from_secs(1))
            .await
            .expect("Should be notified");
    }

    #[tokio::test]
    async fn test_assert_eventually_macro() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        assert_eventually!(flag.load(Ordering::SeqCst), Duration::from_secs(1));
    }
}
