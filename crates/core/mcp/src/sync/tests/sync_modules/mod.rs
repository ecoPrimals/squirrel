// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Sync Test Modules
//!
//! This module contains modularized sync tests including basic flow tests,
//! subscription tests, persistence tests, and error handling tests.

use super::*;
use crate::sync::state::StateOperation;
use crate::sync::state::StateSyncManager;
use crate::sync::MCPSync;
use crate::sync::{PersistenceConfig, MCPPersistence, MCPMonitor};
use crate::sync::tests::{create_test_config, create_test_context, tempdir};
use std::sync::Arc;

// Re-export all test modules
pub mod basic_flow;
pub mod subscriptions;
pub mod persistence;
pub mod helpers;
pub mod error_handling;

// ----- Shared Test Utilities -----

/// Create a standard persistence configuration for testing
pub fn create_test_persistence_config(temp_dir: &std::path::Path) -> PersistenceConfig {
    let data_dir = temp_dir.join("data_dir");
    let persistence_path = temp_dir.join("persistence_path");
    
    // Create the directory structure
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    std::fs::create_dir_all(data_dir.join("states")).expect("Failed to create states directory");
    std::fs::create_dir_all(data_dir.join("changes")).expect("Failed to create changes directory");
    std::fs::create_dir_all(&persistence_path).expect("Failed to create persistence directory");
    
    PersistenceConfig {
        data_dir,
        storage_path: persistence_path.to_string_lossy().to_string(),
        max_file_size: 1024 * 1024,         // 1MB
        auto_compact_threshold: 1024 * 512, // 512KB
        enable_compression: false,
        enable_encryption: false,
        storage_format: "json".to_string(),
        created_at: chrono::Utc::now(),
    }
}

/// Create and initialize a test persistence instance
pub fn create_and_init_persistence(temp_dir: &std::path::Path) -> MCPPersistence {
    let persistence_config = create_test_persistence_config(temp_dir);
    let mut persistence = MCPPersistence::new(persistence_config);
    
    // Initialize persistence manually
    persistence
        .init()
        .expect("Failed to initialize persistence");
    
    persistence
}

/// Create a complete test sync instance with all dependencies
pub async fn create_test_sync_instance(temp_dir: &std::path::Path) -> (MCPSync, Arc<StateSyncManager>, Arc<MCPMonitor>) {
    let persistence = create_and_init_persistence(temp_dir);
    let monitor = Arc::new(MCPMonitor::new().await.expect("Failed to create monitor"));
    let state_manager = Arc::new(StateSyncManager::new());
    
    let sync = MCPSync::new(
        create_test_config(),
        Arc::new(persistence),
        monitor.clone(),
        state_manager.clone(),
    );
    
    (sync, state_manager, monitor)
}

/// Initialize a sync instance and reset its version
pub async fn initialize_sync_with_reset(
    sync: &mut MCPSync,
    state_manager: &Arc<StateSyncManager>,
) {
    // Reset the internal RwLock version counter to 0
    state_manager
        .reset_version()
        .await
        .expect("Failed to reset version");

    // Initialize the sync instance
    sync.init().await.expect("Failed to initialize MCPSync");
    
    // Verify initial version
    let initial_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    assert_eq!(initial_version, 0, "Initial version should be 0 after reset");
}

/// Assert that version has increased after an operation
pub async fn assert_version_increased(
    state_manager: &Arc<StateSyncManager>,
    previous_version: u64,
    context: &str,
) {
    let current_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    
    assert!(
        current_version > previous_version,
        "{}: Version should increase from {} to {}", 
        context, previous_version, current_version
    );
}

/// Find a specific change in the changes list by context ID
pub fn find_change_by_context_id(
    changes: &[crate::sync::state::StateChange],
    context_id: uuid::Uuid,
) -> Option<&crate::sync::state::StateChange> {
    changes.iter().find(|change| change.context_id == context_id)
}

/// Verify that a change has the expected properties
pub fn verify_change_properties(
    change: &crate::sync::state::StateChange,
    expected_context_id: uuid::Uuid,
    expected_operation: StateOperation,
    context: &str,
) {
    assert_eq!(
        change.context_id, expected_context_id,
        "{}: Change should have correct context ID", context
    );
    assert_eq!(
        change.operation, expected_operation,
        "{}: Change should have correct operation", context
    );
    assert!(
        change.version > 0,
        "{}: Change should have positive version", context
    );
}

/// Wait for a change notification with timeout
pub async fn wait_for_change_with_timeout(
    rx: &mut tokio::sync::broadcast::Receiver<crate::sync::state::StateChange>,
    timeout_secs: u64,
    context: &str,
) -> crate::sync::state::StateChange {
    tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        rx.recv()
    )
    .await
    .unwrap_or_else(|_| panic!("Timeout waiting for change in {}", context))
    .unwrap_or_else(|e| panic!("Failed to receive change in {}: {:?}", context, e))
}

/// Create a dual sync setup for persistence testing
pub async fn create_dual_sync_setup() -> (
    (MCPSync, Arc<StateSyncManager>, Arc<MCPMonitor>),
    (MCPSync, Arc<StateSyncManager>, Arc<MCPMonitor>),
) {
    // Create first instance
    let temp_dir1 = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync1, state_manager1, monitor1) = create_test_sync_instance(&temp_dir1).await;
    sync1.init().await.expect("Failed to initialize first sync");
    
    // Create second instance
    let temp_dir2 = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync2, state_manager2, monitor2) = create_test_sync_instance(&temp_dir2).await;
    sync2.init().await.expect("Failed to initialize second sync");
    
    ((sync1, state_manager1, monitor1), (sync2, state_manager2, monitor2))
}

/// Debug print version information
pub async fn debug_print_version(state_manager: &Arc<StateSyncManager>, context: &str) {
    let version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    println!("{}: Version = {}", context, version);
}

/// Debug print changes information
pub fn debug_print_changes(changes: &[crate::sync::state::StateChange], context: &str) {
    println!("{}: Number of changes = {}", context, changes.len());
    for (i, change) in changes.iter().enumerate() {
        println!(
            "  Change {}: ID: {}, Context ID: {}, Operation: {:?}, Version: {}",
            i, change.id, change.context_id, change.operation, change.version
        );
    }
} 