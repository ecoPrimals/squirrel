// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::time::Duration;
use serde::{Serialize, Deserialize};

use crate::resilience::state_sync::{StateSynchronizer, StateSyncConfig, StateType};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestState {
    id: String,
    count: u32,
    data: Vec<String>,
}

#[tokio::test]
async fn test_state_sync_basic() {
    // Create a state synchronizer with default config
    let syncer = StateSynchronizer::default();
    
    // Create a test state
    let state = TestState {
        id: "test-123".to_string(),
        count: 42,
        data: vec!["item1".to_string(), "item2".to_string()],
    };
    
    // Synchronize state
    let result = syncer.sync_state(
        StateType::Configuration,
        "test-config",
        "backup-service",
        state
    ).await;
    
    // Should succeed
    assert!(result.is_ok());
    
    // Check metrics
    let metrics = syncer.get_metrics().expect("Failed to get metrics");
    assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
    assert!(metrics.failed_syncs.is_empty());
    assert!(metrics.total_bytes_synced > 0);
}

#[tokio::test]
async fn test_state_sync_size_limit() {
    // Create a state synchronizer with a small size limit
    let syncer = StateSynchronizer::new(StateSyncConfig {
        max_state_size: 50, // Small size limit
        ..StateSyncConfig::default()
    });
    
    // Create a state that is likely to exceed the limit
    let state = TestState {
        id: "test-123".to_string(),
        count: 42,
        data: vec![
            "this is a very long string that should exceed our size limit".to_string(),
            "another long string to make sure we go over the limit".to_string(),
        ],
    };
    
    // Attempt to synchronize the state
    let result = syncer.sync_state(
        StateType::Runtime,
        "test-runtime",
        "backup-service",
        state
    ).await;
    
    // Should fail due to size
    assert!(result.is_err());
    
    // Check metrics
    let metrics = syncer.get_metrics().expect("Failed to get metrics");
    assert_eq!(*metrics.failed_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
    assert!(metrics.successful_syncs.is_empty());
}

#[tokio::test]
async fn test_state_sync_multiple_operations() {
    let syncer = StateSynchronizer::default();
    
    // Configuration state
    let config_state = TestState {
        id: "config-123".to_string(),
        count: 42,
        data: vec!["config-item1".to_string()],
    };
    
    // Runtime state
    let runtime_state = TestState {
        id: "runtime-456".to_string(),
        count: 100,
        data: vec!["runtime-item1".to_string(), "runtime-item2".to_string()],
    };
    
    // Recovery state
    let recovery_state = TestState {
        id: "recovery-789".to_string(),
        count: 200,
        data: vec!["recovery-item1".to_string(), "recovery-item2".to_string(), "recovery-item3".to_string()],
    };
    
    // Synchronize configuration state
    let result1 = syncer.sync_state(
        StateType::Configuration,
        "config",
        "backup-service",
        config_state
    ).await;
    assert!(result1.is_ok());
    
    // Synchronize runtime state
    let result2 = syncer.sync_state(
        StateType::Runtime,
        "runtime",
        "backup-service",
        runtime_state
    ).await;
    assert!(result2.is_ok());
    
    // Synchronize recovery state
    let result3 = syncer.sync_state(
        StateType::Recovery,
        "recovery",
        "backup-service",
        recovery_state
    ).await;
    assert!(result3.is_ok());
    
    // Check metrics
    let metrics = syncer.get_metrics().expect("Failed to get metrics");
    assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
    assert_eq!(*metrics.successful_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
    assert_eq!(*metrics.successful_syncs.get(&StateType::Recovery).unwrap_or(&0), 1);
    assert!(metrics.failed_syncs.is_empty());
}

#[tokio::test]
async fn test_state_sync_custom_timeout() {
    let syncer = StateSynchronizer::default();
    
    // Create a test state
    let state = TestState {
        id: "test-123".to_string(),
        count: 42,
        data: vec!["item1".to_string()],
    };
    
    // Synchronize state with custom timeout
    let result = syncer.sync_state_with_timeout(
        StateType::Configuration,
        "test-config",
        "backup-service",
        state,
        Duration::from_millis(500)
    ).await;
    
    // Should succeed
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_state_sync_reset_metrics() {
    let syncer = StateSynchronizer::default();
    
    // Create a test state
    let state = TestState {
        id: "test-123".to_string(),
        count: 42,
        data: vec!["item1".to_string()],
    };
    
    // Synchronize state a few times
    for i in 0..3 {
        let _ = syncer.sync_state(
            StateType::Configuration,
            &format!("test-config-{}", i),
            "backup-service",
            state.clone()
        ).await;
    }
    
    // Check metrics before reset
    let metrics_before = syncer.get_metrics().expect("Failed to get metrics before reset");
    assert_eq!(*metrics_before.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 3);
    assert!(metrics_before.total_bytes_synced > 0);
    
    // Reset metrics
    syncer.reset_metrics().expect("Failed to reset metrics");
    
    // Check metrics after reset
    let metrics_after = syncer.get_metrics().expect("Failed to get metrics after reset");
    assert!(metrics_after.successful_syncs.is_empty());
    assert_eq!(metrics_after.total_bytes_synced, 0);
}

#[tokio::test]
async fn test_state_sync_update_config() {
    // Create with default config
    let mut syncer = StateSynchronizer::default();
    
    // Check initial config
    assert_eq!(syncer.get_config().max_state_size, 1024 * 1024); // 1MB
    
    // Update config
    let new_config = StateSyncConfig {
        max_state_size: 2 * 1024 * 1024, // 2MB
        sync_timeout: Duration::from_secs(20),
        validate_state: false,
    };
    
    syncer.update_config(new_config);
    
    // Check updated config
    assert_eq!(syncer.get_config().max_state_size, 2 * 1024 * 1024);
    assert_eq!(syncer.get_config().sync_timeout, Duration::from_secs(20));
    assert_eq!(syncer.get_config().validate_state, false);
} 