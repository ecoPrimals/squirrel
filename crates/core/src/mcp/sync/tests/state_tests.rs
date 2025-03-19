use super::*;

#[tokio::test]
async fn test_state_manager_creation() {
    // ARRANGE: Create state manager
    let state_manager = StateSyncManager::new();
    
    // ACT & ASSERT: Verify initial state
    let version = state_manager.current_version();
    assert_eq!(version, 0, "Initial version should be 0");
    
    // Verify subscriber count
    let sub_count = state_manager.subscriber_count();
    assert_eq!(sub_count, 0, "Initial subscriber count should be 0");
}

#[tokio::test]
async fn test_state_operation_recording() {
    // ARRANGE: Create state manager
    let state_manager = StateSyncManager::new();
    let context = create_test_context();
    
    // ACT: Record create operation
    let change = state_manager.record_operation(
        context.id, 
        StateOperation::Create,
        &context.data
    );
    
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
    );
    
    // ASSERT: Verify updated change
    assert_eq!(update_change.context_id, context.id, "Change should have correct context ID");
    assert_eq!(update_change.operation, StateOperation::Update, "Operation should be Update");
    assert_eq!(update_change.version, 2, "Version should be incremented to 2");
    
    // Verify current version
    let version = state_manager.current_version();
    assert_eq!(version, 2, "Current version should be 2");
}

#[tokio::test]
async fn test_state_change_subscription() {
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
    );
    
    // ASSERT: Verify both receivers got the change
    let received1 = rx1.try_recv().expect("Receiver 1 should get change");
    let received2 = rx2.try_recv().expect("Receiver 2 should get change");
    
    assert_eq!(received1.context_id, context.id, "Should receive correct context ID");
    assert_eq!(received1.version, 1, "Should receive correct version");
    assert_eq!(received2.version, received1.version, "Both receivers should get same change");
}

#[tokio::test]
async fn test_state_missing_subscriber() {
    // ARRANGE: Create state manager and context
    let state_manager = StateSyncManager::new();
    let context = create_test_context();
    
    // ACT: Create and drop a subscriber
    {
        let _rx = state_manager.subscribe();
        // Let rx go out of scope
    }
    
    // Record changes
    state_manager.record_operation(
        context.id, 
        StateOperation::Create,
        &context.data
    );
    
    // ASSERT: Verify subscriber count returns to 0
    // Wait a bit longer to ensure cleanup has a chance to happen
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Increased wait time
    let sub_count = state_manager.subscriber_count();
    assert_eq!(sub_count, 0, "Subscriber count should be 0 after receiver dropped");
}

#[tokio::test]
async fn test_state_operation_enum() {
    // ARRANGE: Create context
    let context = create_test_context();
    
    // ACT & ASSERT: Test operation variants and their string representations
    assert_eq!(
        format!("{:?}", StateOperation::Create), 
        "Create", 
        "Create operation should have correct debug representation"
    );
    
    assert_eq!(
        format!("{:?}", StateOperation::Update), 
        "Update", 
        "Update operation should have correct debug representation"
    );
    
    assert_eq!(
        format!("{:?}", StateOperation::Delete), 
        "Delete", 
        "Delete operation should have correct debug representation"
    );
} 