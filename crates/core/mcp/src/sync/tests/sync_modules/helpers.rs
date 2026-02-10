// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Helper Function Tests
//!
//! Tests for sync helper functions including instance creation,
//! initialization helpers, and utility function verification.

use super::*;
use futures::future::FutureExt;

/// Test helper functions for sync functionality
#[tokio::test]
async fn test_helper_functions() {
    // Create dual sync setup using helper function
    let ((mut sync1, state_manager1, monitor1), (mut sync2, state_manager2, monitor2)) = 
        create_dual_sync_setup().await;

    // Verify both instances are properly initialized
    assert!(sync1.ensure_initialized().await.is_ok(), "First sync should be initialized");
    assert!(sync2.ensure_initialized().await.is_ok(), "Second sync should be initialized");

    // Test that both instances can record changes independently
    let context1 = create_test_context();
    let context2 = create_test_context();

    sync1.record_context_change(&context1, StateOperation::Create)
        .await
        .expect("First sync should record changes");

    sync2.record_context_change(&context2, StateOperation::Create)
        .await
        .expect("Second sync should record changes");

    // Verify changes are recorded independently
    let changes1 = state_manager1
        .get_changes_since(0)
        .await
        .expect("Should get changes from first instance");
    
    let changes2 = state_manager2
        .get_changes_since(0)
        .await
        .expect("Should get changes from second instance");

    // Each should have their own change
    let change1_exists = find_change_by_context_id(&changes1, context1.id);
    let change2_exists = find_change_by_context_id(&changes2, context2.id);

    assert!(change1_exists.is_some(), "First instance should have its change");
    assert!(change2_exists.is_some(), "Second instance should have its change");

    // Verify independence - first shouldn't have second's change
    let cross_check1 = find_change_by_context_id(&changes1, context2.id);
    let cross_check2 = find_change_by_context_id(&changes2, context1.id);

    assert!(cross_check1.is_none(), "First instance should not have second's change");
    assert!(cross_check2.is_none(), "Second instance should not have first's change");

    // Verify monitors are working
    let metrics1 = monitor1.get_metrics().await.expect("Should get metrics from first monitor");
    let metrics2 = monitor2.get_metrics().await.expect("Should get metrics from second monitor");

    assert_eq!(metrics1.total_errors, 0, "First instance should have no errors");
    assert_eq!(metrics2.total_errors, 0, "Second instance should have no errors");
}

/// Test sync-specific helper functions
#[tokio::test]
async fn test_sync_helper_functions() {
    // Test individual helper functions
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    
    // Test persistence config creation
    let persistence_config = create_test_persistence_config(&temp_dir);
    assert!(persistence_config.data_dir.exists(), "Data directory should be created");
    assert_eq!(persistence_config.storage_format, "json", "Should use JSON format");
    assert_eq!(persistence_config.max_file_size, 1024 * 1024, "Should have correct file size");

    // Test persistence creation and initialization
    let persistence = create_and_init_persistence(&temp_dir);
    // Persistence should be created and initialized without errors

    // Test sync instance creation
    let (sync, state_manager, monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Verify components are properly created
    let initial_version = state_manager
        .get_current_version()
        .await
        .expect("Should get initial version");
    assert!(initial_version >= 0, "Initial version should be valid");

    let initial_metrics = monitor.get_metrics().await.expect("Should get initial metrics");
    assert_eq!(initial_metrics.total_errors, 0, "Should start with no errors");
}

/// Test helper functions with version management
#[tokio::test]
async fn test_helper_version_management() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;

    // Test initialization with reset
    initialize_sync_with_reset(&mut sync, &state_manager).await;

    let initial_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get initial version");

    // Test version assertion helper
    let context = create_test_context();
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record change");

    // This should pass - version should increase
    assert_version_increased(&state_manager, initial_version, "After recording change").await;

    // Test debug helpers
    debug_print_version(&state_manager, "Test version print").await;

    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Failed to get changes");
    debug_print_changes(&changes, "Test changes print");
}

/// Test change verification helpers
#[tokio::test]
async fn test_change_verification_helpers() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;

    initialize_sync_with_reset(&mut sync, &state_manager).await;

    let context = create_test_context();

    // Record a change
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record change");

    // Get changes and test helper functions
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Failed to get changes");

    // Test find change helper
    let found_change = find_change_by_context_id(&changes, context.id);
    assert!(found_change.is_some(), "Helper should find the change");

    // Test verification helper
    if let Some(change) = found_change {
        verify_change_properties(change, context.id, StateOperation::Create, "Helper verification");
    }

    // Test with non-existent change
    let fake_context = create_test_context();
    let not_found = find_change_by_context_id(&changes, fake_context.id);
    assert!(not_found.is_none(), "Helper should not find non-existent change");
}

/// Test subscription helpers
#[tokio::test]
async fn test_subscription_helpers() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;

    initialize_sync_with_reset(&mut sync, &state_manager).await;

    // Subscribe to changes
    let mut rx = state_manager.subscribe_changes();

    let context = create_test_context();

    // Record a change
    sync.record_context_change(&context, StateOperation::Update)
        .await
        .expect("Failed to record change");

    // Test wait helper
    let received_change = wait_for_change_with_timeout(&mut rx, 5, "test timeout").await;

    // Verify received change using helper
    verify_change_properties(&received_change, context.id, StateOperation::Update, "Subscription helper test");
}

/// Test error handling in helpers
#[tokio::test]
async fn test_helper_error_handling() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;

    // Test with uninitialized sync (no init called)
    let uninitialized_result = sync.ensure_initialized().await;
    assert!(uninitialized_result.is_err(), "Uninitialized sync should return error");

    // Test version queries work even without initialization
    let version_result = state_manager.get_current_version().await;
    assert!(version_result.is_ok(), "Version query should work");

    // Test changes query works
    let changes_result = state_manager.get_changes_since(0).await;
    assert!(changes_result.is_ok(), "Changes query should work");
} 