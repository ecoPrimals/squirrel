use super::*;

#[tokio::test]
async fn test_state_manager_creation() {
    // Reset the version counter for consistent test results
    StateSyncManager::reset_version_counter();
    
    // ARRANGE: Create state manager
    let state_manager = StateSyncManager::new();
    
    // ACT & ASSERT: Verify initial state
    let version = state_manager.current_version().await;
    assert_eq!(version, 0, "Initial version should be 0");
    
    // Verify subscriber count
    let sub_count = state_manager.subscriber_count();
    assert_eq!(sub_count, 0, "Initial subscriber count should be 0");
}

#[tokio::test]
async fn test_state_operation_recording() {
    StateSyncManager::reset_version_counter();
    
    // ARRANGE: Create state manager and test context
    let state_manager = StateSyncManager::new();
    // Reset the internal RwLock version counter to 0
    state_manager.reset_version().await.expect("Failed to reset version");
    
    let context = create_test_context();
    
    // ACT: Record creation operation
    let change = state_manager.record_operation(
        context.id, 
        StateOperation::Create,
        &context.data
    ).await;
    
    // ASSERT: Verify change details
    assert_eq!(change.context_id, context.id, "Change should have correct context ID");
    assert_eq!(change.operation, StateOperation::Create, "Operation should be Create");
    assert_eq!(change.version, 1, "Version should be incremented to 1");
    assert!(!change.data.is_null(), "Data should be included");
    
    // ACT: Record update operation and verify version increments
    let update_change = state_manager.record_operation(
        context.id, 
        StateOperation::Update,
        &serde_json::json!({"updated": true})
    ).await;
    
    // ASSERT: Verify updated change
    assert_eq!(update_change.context_id, context.id, "Change should have correct context ID");
    assert_eq!(update_change.operation, StateOperation::Update, "Operation should be Update");
    assert_eq!(update_change.version, 2, "Version should be incremented to 2");
    
    // Verify current version
    let version = state_manager.current_version().await;
    assert_eq!(version, 2, "Current version should be 2");
}

#[tokio::test]
async fn test_state_change_subscription() {
    // Reset the version counter for consistent test results
    StateSyncManager::reset_version_counter();
    
    // ARRANGE: Create state manager
    let state_manager = StateSyncManager::new();
    let context = create_test_context();
    
    // ACT: Subscribe to changes
    let mut rx1 = state_manager.subscribe();
    let mut rx2 = state_manager.subscribe();
    
    // ASSERT: Verify subscriber count
    let sub_count = state_manager.subscriber_count();
    assert_eq!(sub_count, 2, "Should have 2 subscribers");
    
    // ACT: Record change
    let change = state_manager.record_operation(
        context.id, 
        StateOperation::Create,
        &context.data
    ).await;
    
    // ASSERT: Verify changes received by subscribers
    let received1 = rx1.recv().await.expect("Subscriber 1 failed to receive change");
    let received2 = rx2.recv().await.expect("Subscriber 2 failed to receive change");
    
    assert_eq!(received1.id, change.id, "Subscriber 1 should receive exact change");
    assert_eq!(received2.id, change.id, "Subscriber 2 should receive exact change");
}

#[tokio::test]
async fn test_state_missing_subscriber() {
    // Reset the version counter for consistent test results
    StateSyncManager::reset_version_counter();
    
    // ARRANGE: Create state manager
    let state_manager = StateSyncManager::new();
    let context = create_test_context();
    
    // ACT: Subscribe to changes and then drop the receiver
    let rx = state_manager.subscribe();
    drop(rx);
    
    // ASSERT: Verify subscriber count
    let sub_count = state_manager.subscriber_count();
    assert_eq!(sub_count, 0, "Should have 0 subscribers after dropping");
    
    // ACT: Record change with no active subscribers
    let change = state_manager.record_operation(
        context.id, 
        StateOperation::Create,
        &context.data
    ).await;
    
    // ASSERT: Verify the change was created normally
    assert_eq!(change.context_id, context.id, "Change should have correct context ID");
    assert_eq!(change.version, 1, "Version should be 1");
}

#[tokio::test]
async fn test_state_operation_enum() {
    // Simple test for enum variants
    let create = StateOperation::Create;
    let update = StateOperation::Update;
    let delete = StateOperation::Delete;
    let sync = StateOperation::Sync;
    
    assert_ne!(create, update, "Create should not equal Update");
    assert_ne!(create, delete, "Create should not equal Delete");
    assert_ne!(create, sync, "Create should not equal Sync");
    assert_ne!(update, delete, "Update should not equal Delete");
    assert_ne!(update, sync, "Update should not equal Sync");
    assert_ne!(delete, sync, "Delete should not equal Sync");
} 