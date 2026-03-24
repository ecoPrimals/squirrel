// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Test Utilities - Modern Concurrent Patterns
//!
//! Reusable test helpers with proper timeout and concurrency handling

use std::future::Future;
use tokio::time::{timeout, Duration};

/// Wraps an async test with a timeout
///
/// # Examples
///
/// ```no_run
/// use test_utils::with_test_timeout;
///
/// #[tokio::test]
/// async fn my_test() {
///     let result = with_test_timeout(5, async {
///         expensive_operation().await
///     }).await;
///     
///     assert!(result.is_ok());
/// }
/// ```
pub async fn with_test_timeout<F, T>(duration_secs: u64, future: F) -> Result<T, String>
where
    F: Future<Output = T>,
{
    timeout(Duration::from_secs(duration_secs), future)
        .await
        .map_err(|_| format!("Test timed out after {} seconds", duration_secs))
}

/// Test duration multiplier for faster test execution
///
/// Set environment variable TEST_DURATION_MULTIPLIER to adjust:
/// - 1.0 = normal duration (default)
/// - 0.1 = 10x faster (for CI/development)
/// - 0.01 = 100x faster (for quick smoke tests)
///
/// # Examples
///
/// ```no_run
/// // Normal: TEST_DURATION_MULTIPLIER=1.0 cargo test
/// // Fast: TEST_DURATION_MULTIPLIER=0.1 cargo test
/// let duration = test_duration(5); // 5 seconds or 500ms if multiplier is 0.1
/// ```
pub fn test_duration(full_seconds: u64) -> Duration {
    test_duration_with_multiplier(full_seconds, None)
}

/// Internal: Test duration with optional override (for testing)
pub fn test_duration_with_multiplier(
    full_seconds: u64,
    override_multiplier: Option<f64>,
) -> Duration {
    let multiplier = override_multiplier.unwrap_or_else(|| {
        std::env::var("TEST_DURATION_MULTIPLIER")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0)
    });

    let millis = (full_seconds as f64 * 1000.0 * multiplier).max(10.0) as u64;
    Duration::from_millis(millis)
}

/// Quick test duration (optimized for test suites)
///
/// Always returns a short duration suitable for unit tests.
/// Respects TEST_DURATION_MULTIPLIER if set.
pub fn quick_test_duration(millis: u64) -> Duration {
    quick_test_duration_with_multiplier(millis, None)
}

/// Internal: Quick test duration with optional override (for testing)
pub fn quick_test_duration_with_multiplier(
    millis: u64,
    override_multiplier: Option<f64>,
) -> Duration {
    let multiplier = override_multiplier.unwrap_or_else(|| {
        std::env::var("TEST_DURATION_MULTIPLIER")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0)
    });

    let adjusted = (millis as f64 * multiplier).max(1.0) as u64;
    Duration::from_millis(adjusted)
}

/// Retry a test operation with exponential backoff
///
/// Useful for flaky integration tests that need network retries.
pub async fn retry_with_backoff<F, T, E>(
    max_attempts: u32,
    initial_delay_ms: u64,
    operation: impl Fn() -> F,
) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(initial_delay_ms);

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

/// Run multiple async operations concurrently with timeout
///
/// Returns all results or fails if any operation times out or fails.
pub async fn run_concurrent<F, T>(operations: Vec<F>, timeout_secs: u64) -> Result<Vec<T>, String>
where
    F: Future<Output = Result<T, String>> + Send + 'static,
    T: Send + 'static,
{
    let handles: Vec<_> = operations.into_iter().map(|op| tokio::spawn(op)).collect();

    let result = timeout(
        Duration::from_secs(timeout_secs),
        futures::future::try_join_all(handles),
    )
    .await;

    match result {
        Ok(Ok(results)) => results
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Task panicked: {:?}", e)),
        Ok(Err(e)) => Err(format!("Task failed: {:?}", e)),
        Err(_) => Err(format!(
            "Operations timed out after {} seconds",
            timeout_secs
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_with_timeout_success() {
        let result = with_test_timeout(1, async {
            // Fast actual work instead of sleep
            tokio::task::yield_now().await;
            42
        })
        .await;

        assert_eq!(result.expect("should succeed"), 42);
    }

    #[tokio::test]
    async fn test_with_timeout_failure() {
        let result = with_test_timeout(1, async {
            // pending() never completes -- tests timeout without wasting time sleeping
            std::future::pending::<()>().await;
            42
        })
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }

    #[test]
    fn test_duration_multiplier() {
        // Use explicit multiplier instead of env var mutation for concurrent safety
        let duration = test_duration_with_multiplier(5, Some(0.1));
        assert_eq!(duration, Duration::from_millis(500));
    }

    #[test]
    fn test_duration_default() {
        // Test default behavior (1.0 multiplier)
        let duration = test_duration_with_multiplier(5, Some(1.0));
        assert_eq!(duration, Duration::from_millis(5000));
    }

    #[test]
    fn test_quick_duration_multiplier() {
        // Test quick duration with explicit multiplier
        let duration = quick_test_duration_with_multiplier(100, Some(0.1));
        assert_eq!(duration, Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let attempt = Arc::new(AtomicU32::new(0));
        let attempt_clone = Arc::clone(&attempt);

        let result = retry_with_backoff(3, 10, move || {
            let attempt = Arc::clone(&attempt_clone);
            async move {
                let current = attempt.fetch_add(1, Ordering::SeqCst) + 1;
                if current < 3 {
                    Err("Temporary failure")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.expect("should succeed"), 42);
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use futures::future::BoxFuture;

        let operations: Vec<BoxFuture<'static, Result<i32, String>>> = vec![
            Box::pin(async { Ok::<_, String>(1) }),
            Box::pin(async { Ok::<_, String>(2) }),
            Box::pin(async { Ok::<_, String>(3) }),
        ];

        let results = run_concurrent(operations, 5).await;
        assert_eq!(results.expect("should succeed"), vec![1, 2, 3]);
    }
}

// ============================================================================
// Ecosystem Test Helpers
// ============================================================================

/// Creates a test ecosystem manager with default configuration
///
/// This is the primary helper for creating ecosystem managers in tests.
/// Uses sensible defaults suitable for testing.
///
/// # Examples
///
/// ```no_run
/// # use crate::tests::common::create_test_ecosystem_manager;
/// #[tokio::test]
/// async fn test_ecosystem_discovery() {
///     let manager = create_test_ecosystem_manager().await;
///     let services = manager.discover_capability("storage").await.expect("should succeed");
///     assert!(services.is_empty()); // No services registered yet
/// }
/// ```
pub async fn create_test_ecosystem_manager() -> squirrel::ecosystem::EcosystemManager {
    use squirrel::ecosystem::EcosystemConfig;
    use squirrel::monitoring::metrics::MetricsCollector;
    use std::sync::Arc;

    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    squirrel::ecosystem::EcosystemManager::new(config, metrics)
}

/// Creates a test ecosystem manager with custom configuration
///
/// Use this when you need specific configuration for your test.
///
/// # Examples
///
/// ```no_run
/// # use crate::tests::common::create_test_ecosystem_manager_with_config;
/// # use crate::ecosystem::EcosystemConfig;
/// #[tokio::test]
/// async fn test_with_custom_config() {
///     let mut config = EcosystemConfig::default();
///     config.discovery_timeout = std::time::Duration::from_secs(1);
///     
///     let manager = create_test_ecosystem_manager_with_config(config).await;
///     // Test with custom config...
/// }
/// ```
pub async fn create_test_ecosystem_manager_with_config(
    config: squirrel::ecosystem::EcosystemConfig,
) -> squirrel::ecosystem::EcosystemManager {
    use squirrel::monitoring::metrics::MetricsCollector;
    use std::sync::Arc;

    let metrics = Arc::new(MetricsCollector::new());
    squirrel::ecosystem::EcosystemManager::new(config, metrics)
}

/// Creates a minimal test ecosystem manager (no metrics, minimal config)
///
/// Use this for tests that don't need full ecosystem functionality
/// and want the fastest possible initialization.
pub async fn create_minimal_ecosystem_manager() -> squirrel::ecosystem::EcosystemManager {
    create_test_ecosystem_manager().await
}
