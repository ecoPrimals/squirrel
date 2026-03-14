// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Helper Functions for Integration Tests

use std::time::Duration;
use tokio::time::timeout;

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);
                if attempt < max_retries - 1 {
                    // LEGITIMATE SLEEP: Exponential backoff for retry logic
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}

/// Wait for a condition to become true
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout_duration: Duration,
    check_interval: Duration,
) -> Result<(), String>
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + timeout_duration;
    
    while tokio::time::Instant::now() < deadline {
        if condition() {
            return Ok(());
        }
        // LEGITIMATE SLEEP: Polling interval for condition checking
        tokio::time::sleep(check_interval).await;
    }
    
    Err("Condition not met within timeout".to_string())
}

/// Create a temporary test directory
pub fn create_test_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("squirrel_integration_tests")
        .join(test_name)
        .join(uuid::Uuid::new_v4().to_string());
    
    std::fs::create_dir_all(&dir).expect("Failed to create test directory");
    dir
}

/// Clean up test directory
pub fn cleanup_test_dir(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

