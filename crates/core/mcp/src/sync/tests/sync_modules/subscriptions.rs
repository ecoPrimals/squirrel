// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Change Subscription Tests
//!
//! Tests for change subscription functionality including event handling,
//! notification delivery, and continuous subscription verification.

use super::*;

/// Test change subscription with event notifications
#[tokio::test]
async fn test_change_subscription() {
    // Create a temporary directory for this test
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    
    // Create sync instance with all dependencies
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    // Initialize sync with version reset
    initialize_sync_with_reset(&mut sync, &state_manager).await;

    // Create a test context
    let context = create_test_context();

    // Subscribe to changes
    let mut rx = state_manager.subscribe_changes();

    // Record a context change
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record context change");

    // Verify change was received with a timeout
    let received = wait_for_change_with_timeout(&mut rx, 5, "first change").await;
    
    verify_change_properties(&received, context.id, StateOperation::Create, "First received change");

    // Verify state manager version increased
    let version = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    assert!(version > 0, "Version should be incremented after change");

    // Record another change to verify continuous subscription
    sync.record_context_change(&context, StateOperation::Update)
        .await
        .expect("Failed to record update");

    // Verify second change was received with a timeout
    let received2 = wait_for_change_with_timeout(&mut rx, 5, "second change").await;
    
    verify_change_properties(&received2, context.id, StateOperation::Update, "Second received change");

    // Verify version increased again
    let version2 = state_manager
        .get_current_version()
        .await
        .expect("Failed to get current version");
    assert!(
        version2 > version,
        "Version should increase after second change"
    );
}

/// Test multiple subscribers receiving the same changes
#[tokio::test]
async fn test_multiple_subscribers() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    // Create multiple subscribers
    let mut rx1 = state_manager.subscribe_changes();
    let mut rx2 = state_manager.subscribe_changes();
    let mut rx3 = state_manager.subscribe_changes();
    
    let context = create_test_context();
    
    // Record a change
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record change");
    
    // All subscribers should receive the change
    let change1 = wait_for_change_with_timeout(&mut rx1, 5, "subscriber 1").await;
    let change2 = wait_for_change_with_timeout(&mut rx2, 5, "subscriber 2").await;
    let change3 = wait_for_change_with_timeout(&mut rx3, 5, "subscriber 3").await;
    
    // Verify all received the same change
    assert_eq!(change1.context_id, context.id, "Subscriber 1 should receive correct change");
    assert_eq!(change2.context_id, context.id, "Subscriber 2 should receive correct change");
    assert_eq!(change3.context_id, context.id, "Subscriber 3 should receive correct change");
    
    assert_eq!(change1.operation, StateOperation::Create);
    assert_eq!(change2.operation, StateOperation::Create);
    assert_eq!(change3.operation, StateOperation::Create);
}

/// Test subscription with multiple different changes
#[tokio::test]
async fn test_subscription_multiple_changes() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    let mut rx = state_manager.subscribe_changes();
    
    // Create multiple contexts
    let context1 = create_test_context();
    let context2 = create_test_context();
    let context3 = create_test_context();
    
    // Record multiple changes
    sync.record_context_change(&context1, StateOperation::Create)
        .await
        .expect("Failed to record context1 create");
    
    sync.record_context_change(&context2, StateOperation::Create)
        .await
        .expect("Failed to record context2 create");
    
    sync.record_context_change(&context1, StateOperation::Update)
        .await
        .expect("Failed to record context1 update");
    
    sync.record_context_change(&context3, StateOperation::Create)
        .await
        .expect("Failed to record context3 create");
    
    sync.record_context_change(&context2, StateOperation::Delete)
        .await
        .expect("Failed to record context2 delete");
    
    // Collect all changes
    let mut received_changes = Vec::new();
    for i in 0..5 {
        let change = wait_for_change_with_timeout(&mut rx, 5, &format!("change {}", i + 1)).await;
        received_changes.push(change);
    }
    
    // Verify we got all the expected changes
    assert_eq!(received_changes.len(), 5, "Should receive all 5 changes");
    
    // Verify specific changes exist
    let context1_create = received_changes.iter().find(|c| 
        c.context_id == context1.id && c.operation == StateOperation::Create
    );
    let context2_create = received_changes.iter().find(|c| 
        c.context_id == context2.id && c.operation == StateOperation::Create
    );
    let context1_update = received_changes.iter().find(|c| 
        c.context_id == context1.id && c.operation == StateOperation::Update
    );
    let context3_create = received_changes.iter().find(|c| 
        c.context_id == context3.id && c.operation == StateOperation::Create
    );
    let context2_delete = received_changes.iter().find(|c| 
        c.context_id == context2.id && c.operation == StateOperation::Delete
    );
    
    assert!(context1_create.is_some(), "Should receive context1 create");
    assert!(context2_create.is_some(), "Should receive context2 create");
    assert!(context1_update.is_some(), "Should receive context1 update");
    assert!(context3_create.is_some(), "Should receive context3 create");
    assert!(context2_delete.is_some(), "Should receive context2 delete");
}

/// Test subscription behavior after sync reinitialization
#[tokio::test]
async fn test_subscription_after_reinit() {
    let temp_dir = tempdir().expect("Failed to create temp dir").into_path();
    let (mut sync, state_manager, _monitor) = create_test_sync_instance(&temp_dir).await;
    
    initialize_sync_with_reset(&mut sync, &state_manager).await;
    
    // Subscribe before recording changes
    let mut rx = state_manager.subscribe_changes();
    
    let context = create_test_context();
    
    // Record initial change
    sync.record_context_change(&context, StateOperation::Create)
        .await
        .expect("Failed to record initial change");
    
    let first_change = wait_for_change_with_timeout(&mut rx, 5, "first change").await;
    verify_change_properties(&first_change, context.id, StateOperation::Create, "First change");
    
    // Reinitialize sync (this might affect subscriptions)
    sync.init().await.expect("Failed to reinitialize sync");
    
    // Record another change after reinitialization
    sync.record_context_change(&context, StateOperation::Update)
        .await
        .expect("Failed to record change after reinit");
    
    // Should still receive the change on existing subscription
    let second_change = wait_for_change_with_timeout(&mut rx, 5, "change after reinit").await;
    verify_change_properties(&second_change, context.id, StateOperation::Update, "Change after reinit");
} 