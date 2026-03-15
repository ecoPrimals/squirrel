// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Basic Sync Flow Tests
//!
//! Tests for core sync functionality including state management,
//! version tracking, and change recording.

use super::*;

/// Test basic sync flow with state management and change recording
#[tokio::test]
async fn test_sync_flow() {
    println!("Starting test_sync_flow");
    
    // Create a temporary directory for this test
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    
    println!("Created temp directory: {:?}", &temp_dir);
    
    // Create sync instance with all dependencies
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    println!("MCPSync instance created");
    
    // Initialize sync with version reset
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    let initial_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    
    debug_print_version(&state_manager, "After initialization").await;
    
    // Create a test context with known UUID
    let context = create_test_context();
    let context_id = context.id;
    println!("Test context ID: {}", context_id);

    // Record a change with the proper method signature
    let record_result = sync
        .record_context_change(&context, StateOperation::Create)
        .await;
    
    match &record_result {
        Ok(_) => println!("Context change recorded successfully"),
        Err(e) => println!("ERROR: Failed to record context change: {:?}", e),
    }
    record_result.expect("Failed to record context change");

    // Verify version increased
    assert_version_increased(&state_manager, initial_version, "After recording change").await;
    debug_print_version(&state_manager, "After change").await;

    // Verify change was recorded in state manager
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Failed to get changes");
    
    debug_print_changes(&changes, "All changes");

    // Look for our specific change
    let our_change = find_change_by_context_id(&changes, context_id);
    assert!(
        our_change.is_some(),
        "Our context change should be in the changes list"
    );

    // Verify change properties
    if let Some(change) = our_change {
        verify_change_properties(change, context_id, StateOperation::Create, "Recorded change");
    }
}

/// Test sync flow with multiple operations
#[tokio::test]
async fn test_sync_flow_multiple_operations() {
    println!("Starting test_sync_flow_multiple_operations");
    
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Initialize sync
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    let initial_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    
    // Create test contexts
    let context1 = create_test_context();
    let context2 = create_test_context();
    
    // Record multiple changes
    sync.record_context_change(&context1, StateOperation::Create)
        .await
        .expect("Failed to record first change");
    
    let version_after_first = state_manager
        .get_current_version()
        .await
        .expect("Failed to get version after first change");
    
    sync.record_context_change(&context2, StateOperation::Create)
        .await
        .expect("Failed to record second change");
    
    let version_after_second = state_manager
        .get_current_version()
        .await
        .expect("Failed to get version after second change");
    
    sync.record_context_change(&context1, StateOperation::Update)
        .await
        .expect("Failed to record update change");
    
    let final_version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get final version");
    
    // Verify version progression
    assert!(version_after_first > initial_version, "Version should increase after first change");
    assert!(version_after_second > version_after_first, "Version should increase after second change");
    assert!(final_version > version_after_second, "Version should increase after update");
    
    // Verify all changes are recorded
    let changes = state_manager
        .get_changes_since(0)
        .await
        .expect("Failed to get changes");
    
    assert!(changes.len() >= 3, "Should have at least 3 changes recorded");
    
    // Verify specific changes exist
    let context1_create = changes.iter().find(|c| 
        c.context_id == context1.id && c.operation == StateOperation::Create
    );
    let context2_create = changes.iter().find(|c| 
        c.context_id == context2.id && c.operation == StateOperation::Create
    );
    let context1_update = changes.iter().find(|c| 
        c.context_id == context1.id && c.operation == StateOperation::Update
    );
    
    assert!(context1_create.is_some(), "Context1 Create change should exist");
    assert!(context2_create.is_some(), "Context2 Create change should exist");
    assert!(context1_update.is_some(), "Context1 Update change should exist");
}

/// Test sync flow with version queries
#[tokio::test]
async fn test_sync_flow_version_queries() {
    println!("Starting test_sync_flow_version_queries");
    
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Initialize sync
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    let context = create_test_context();
    
    // Record some changes to build up version history
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record create");
    
    let version_after_create = state_manager
        .get_current_version()
        .await
        .expect("Failed to get version after create");
    
    sync.record_context_change(&context, StateOperation::Update)
        .await
        .expect("Failed to record update");
    
    let version_after_update = state_manager
        .get_current_version()
        .await
        .expect("Failed to get version after update");
    
    sync.record_context_change(&context, StateOperation::Delete)
        .await
        .expect("Failed to record delete");
    
    // Test getting changes since different versions
    let changes_since_0 = state_manager
        .get_changes_since(0)
        .await
        .expect("Failed to get changes since 0");
    
    let changes_since_create = state_manager
        .get_changes_since(version_after_create)
        .await
        .expect("Failed to get changes since create");
    
    let changes_since_update = state_manager
        .get_changes_since(version_after_update)
        .await
        .expect("Failed to get changes since update");
    
    // Verify change counts
    assert!(changes_since_0.len() >= 3, "Should have all changes since version 0");
    assert!(changes_since_create.len() >= 2, "Should have changes since create version");
    assert!(changes_since_update.len() >= 1, "Should have changes since update version");
    
    // Verify change order and content
    assert!(changes_since_create.len() < changes_since_0.len(), 
           "Changes since create should be fewer than all changes");
    assert!(changes_since_update.len() < changes_since_create.len(), 
           "Changes since update should be fewer than changes since create");
} 