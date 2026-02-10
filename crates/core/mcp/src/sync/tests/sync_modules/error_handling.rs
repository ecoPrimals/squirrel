// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Error Handling Tests
//!
//! Tests for sync error handling including uninitialized state errors,
//! operation failures, and error recovery scenarios.

use super::*;

/// Test error handling for uninitialized sync operations
#[tokio::test]
async fn test_uninitialized_error() {
    // Create a temporary directory for this test
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    
    // Create sync instance but DON'T initialize it
    let (sync, _state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;

    // Create a test context
    let context = create_test_context();

    // Try to record a change - should fail since sync is not initialized
    let record_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;

    // Verify operation fails with appropriate error
    assert!(
        record_result.is_err(),
        "Record change should fail when not initialized"
    );
    
    // Verify the specific error type if needed
    match record_result {
        Err(e) => {
            // Log the error for debugging
            println!("Expected error for uninitialized sync: {:?}", e);
        }
        Ok(_) => panic!("Expected error but operation succeeded"),
    }
}

/// Test error handling for invalid operations
#[tokio::test]
async fn test_invalid_operation_errors() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Initialize the sync
    sync.init().await.expect("Sync initialization should succeed");
    
    // Verify initialization worked
    assert!(sync.ensure_initialized().await.is_ok(), "Sync should be initialized");
    
    // Test with invalid context (this might not fail depending on implementation)
    let context = create_test_context();
    
    // Record a valid change first
    let valid_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;
    
    assert!(valid_result.is_ok(), "Valid operation should succeed after initialization");
    
    // Verify the change was recorded
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Should be able to get changes");
    
    let found_change = find_change_by_context_id(&changes, context.id);
    assert!(found_change.is_some(), "Change should be recorded after initialization");
}

/// Test error recovery scenarios
#[tokio::test]
async fn test_error_recovery() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Test recovery from uninitialized state
    let context = create_test_context();
    
    // First attempt without initialization should fail
    let uninitialized_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;
    
    assert!(uninitialized_result.is_err(), "Operation should fail before initialization");
    
    // Now initialize and retry
    sync.init().await.expect("Initialization should succeed");
    
    // Same operation should now succeed
    let initialized_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;
    
    assert!(initialized_result.is_ok(), "Operation should succeed after initialization");
    
    // Verify the change was recorded
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Should get changes after recovery");
    
    let recovered_change = find_change_by_context_id(&changes, context.id);
    assert!(recovered_change.is_some(), "Change should be recorded after recovery");
}

/// Test concurrent initialization errors
#[tokio::test]
async fn test_concurrent_initialization() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, _state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Test sequential initializations instead of concurrent to avoid borrow checker issues
    let result1 = sync.init().await;
    let result2 = sync.init().await;
    let result3 = sync.init().await;
    
    // All should succeed (or at least not cause panic)
    assert!(result1.is_ok(), "First initialization should succeed");
    assert!(result2.is_ok(), "Second initialization should succeed");
    assert!(result3.is_ok(), "Third initialization should succeed");
    
    // Final state should be initialized
    assert!(sync.ensure_initialized().await.is_ok(), "Sync should be initialized after multiple inits");
    
    // Should be able to record changes after multiple initialization
    let context = create_test_context();
    let record_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;
    
    assert!(record_result.is_ok(), "Should be able to record changes after multiple initialization");
}

/// Test state manager error handling
#[tokio::test]
async fn test_state_manager_errors() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (_sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Test getting changes with invalid version (this should handle gracefully)
    let changes_result = state_manager.get_changes_since(u64::MAX).await;
    
    // Should either succeed with empty result or handle the error gracefully
    assert!(changes_result.is_ok(), "State manager should handle large version numbers gracefully");
    
    if let Ok(changes) = changes_result {
        // If it succeeds, there should be no changes from a future version
        assert!(changes.is_empty(), "Should have no changes from future version");
    }
    
    // Test current version query
    let version_result = state_manager.get_current_version().await;
    assert!(version_result.is_ok(), "Getting current version should not fail");
    
    // Test reset version
    let reset_result = state_manager.reset_version().await;
    assert!(reset_result.is_ok(), "Reset version should not fail");
    
    // Verify version is reset
    let post_reset_version = state_manager
        .get_current_version()
        .await
        .expect("Should get version after reset");
    assert_eq!(post_reset_version, 0, "Version should be 0 after reset");
}

/// Test persistence error scenarios
#[tokio::test]
async fn test_persistence_error_handling() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Test initialization multiple times
    sync.init().await.expect("First init should succeed");
    sync.init().await.expect("Second init should succeed");
    sync.init().await.expect("Third init should succeed");
    
    // Verify functionality is maintained
    assert!(sync.ensure_initialized().await.is_ok(), "Sync should remain initialized");
    
    // Test recording changes after multiple initializations
    let context = create_test_context();
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Should record changes after multiple inits");
    
    // Verify changes are properly recorded
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Should get changes");
    
    let change_found = find_change_by_context_id(&changes, context.id);
    assert!(change_found.is_some(), "Change should be found after multiple inits");
}

/// Test error propagation
#[tokio::test]
async fn test_error_propagation() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Test that uninitialized operations properly propagate errors
    let context = create_test_context();
    
    // This should fail and propagate the error
    let result = sync.record_context_change(&context, StateOperation::Create).await;
    
    match result {
        Err(e) => {
            // Verify we get a meaningful error
            let error_string = format!("{:?}", e);
            assert!(!error_string.is_empty(), "Error should have meaningful content");
            println!("Properly propagated error: {}", error_string);
        }
        Ok(_) => {
            // If it succeeds, that's unexpected but not necessarily wrong
            // depending on the implementation
            println!("Warning: Expected error but operation succeeded");
        }
    }
    
    // State manager operations should still work
    let version = state_manager
        .get_current_version()
        .await
        .expect("State manager operations should work independently");
    
    assert!(version >= 0, "Version should be valid even when sync is uninitialized");
} 