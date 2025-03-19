use crate::mcp::{MCPError, error::types::MCPError as MCPErrorType};
use crate::mcp::sync::state::StateOperation;
use crate::mcp::sync::MCPSync;
use std::sync::Arc;
use super::*;

#[tokio::test]
async fn test_sync_flow() {
    // ARRANGE: Create test environment with DI
    let (mut sync, persistence, monitor, state_manager) = create_test_mcp_sync().await;
    
    // Initialize the sync instance
    let init_result = sync.init().await;
    assert!(init_result.is_ok(), "Failed to initialize sync");
    
    // Create test context
    let context = create_test_context();
    
    // ACT: Record a change and perform sync
    let record_result = sync.record_context_change(&context, StateOperation::Create).await;
    assert!(record_result.is_ok(), "Failed to record context change");
    
    let sync_result = sync.sync().await;
    assert!(sync_result.is_ok(), "Failed to perform sync");
    
    // ASSERT: Verify state after sync
    let state = sync.get_state().await.unwrap();
    assert_eq!(state.sync_count, 1, "Sync count should be 1");
    assert_eq!(state.error_count, 0, "Error count should be 0");
    assert_eq!(state.last_version, 1, "Version should be 1");
    
    // Verify metrics
    let metrics = monitor.get_metrics().await.unwrap();
    assert_eq!(metrics.context_operations, 1, "Should have 1 context operation");
    assert!(metrics.sync_operations >= 1, "Should have at least 1 sync operation");
    assert_eq!(metrics.total_errors, 0, "Should have 0 errors");
}

#[tokio::test]
async fn test_change_subscription() {
    // ARRANGE: Create test environment with DI
    let (mut sync, _, monitor, _) = create_test_mcp_sync().await;
    
    // Initialize the sync instance
    sync.init().await.expect("Failed to initialize sync");
    
    // Subscribe to changes
    let mut rx = sync.subscribe_changes().await.expect("Failed to subscribe to changes");
    
    // Create test context
    let context = create_test_context();
    
    // ACT: Record change in separate task to test async notification
    let sync_clone = sync.clone();
    let context_clone = context.clone();
    tokio::spawn(async move {
        sync_clone.record_context_change(&context_clone, StateOperation::Create).await
            .expect("Failed to record context change");
    });
    
    // ASSERT: Verify change notification is received
    let change = rx.recv().await.expect("Failed to receive change notification");
    assert_eq!(change.context_id, context.id, "Change should have correct context ID");
    assert_eq!(change.version, 1, "Change version should be 1");
    
    // Verify metrics
    let metrics = monitor.get_metrics().await.expect("Failed to get metrics");
    assert_eq!(metrics.context_operations, 1, "Should have 1 context operation");
    assert!(metrics.total_messages >= 2, "Should have at least 2 messages");
}

#[tokio::test]
async fn test_persistence() {
    // ARRANGE: Create test environments with same persistence
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let persistence_config = PersistenceConfig {
        data_dir: temp_dir.path().to_path_buf(),
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        max_file_size: 1024 * 1024,
        auto_compact_threshold: 1024 * 512,
        enable_compression: false,
        enable_encryption: false,
        storage_format: "json".to_string(),
    };
    
    let persistence = Arc::new(MCPPersistence::new(persistence_config));
    let monitor1 = Arc::new(MCPMonitor::new().await.expect("Failed to create monitor"));
    let state_manager1 = Arc::new(StateSyncManager::new());
    
    // Create first sync instance
    let mut sync1 = MCPSync::new(
        create_test_config(),
        persistence.clone(),
        monitor1,
        state_manager1
    );
    
    sync1.init().await.expect("Failed to initialize first sync instance");
    
    // Create test context
    let context = create_test_context();
    
    // ACT: Record change and sync with first instance
    sync1.record_context_change(&context, StateOperation::Create).await
        .expect("Failed to record context change");
    
    sync1.sync().await.expect("Failed to perform sync");
    
    // Create second sync instance with same persistence
    let monitor2 = Arc::new(MCPMonitor::new().await.expect("Failed to create monitor"));
    let state_manager2 = Arc::new(StateSyncManager::new());
    
    let mut sync2 = MCPSync::new(
        create_test_config(),
        persistence.clone(),
        monitor2.clone(),
        state_manager2
    );
    
    // Initialize second instance which should load from persistence
    sync2.init().await.expect("Failed to initialize second sync instance");
    
    // ASSERT: Verify state was loaded from persistence
    let state = sync2.get_state().await.expect("Failed to get state");
    assert_eq!(state.last_version, 1, "Version should be preserved across instances");
    
    // Verify metrics on second instance
    let metrics = monitor2.get_metrics().await.expect("Failed to get metrics");
    assert!(metrics.total_messages > 0, "Should have recorded messages");
    assert_eq!(metrics.total_errors, 0, "Should have 0 errors");
}

#[tokio::test]
async fn test_helper_functions() {
    // ARRANGE: Create dependencies for testing helpers
    let config = create_test_config();
    
    // ACT: Test create_mcp_sync helper
    let sync1 = create_mcp_sync(config.clone()).await.expect("Failed to create sync with helper");
    
    // ASSERT: Verify created instance is properly initialized
    let state1 = sync1.get_state().await;
    assert!(state1.is_ok(), "Should return valid state");
    
    // ARRANGE: Create explicit dependencies
    let persistence = Arc::new(MCPPersistence::new(PersistenceConfig::default()));
    let monitor = Arc::new(MCPMonitor::new().await.expect("Failed to create monitor"));
    let state_manager = Arc::new(StateSyncManager::new());
    
    // ACT: Test create_mcp_sync_with_deps helper
    let sync2 = create_mcp_sync_with_deps(
        config,
        persistence,
        monitor,
        state_manager
    ).await.expect("Failed to create sync with explicit deps");
    
    // ASSERT: Verify created instance is properly initialized
    let state2 = sync2.get_state().await;
    assert!(state2.is_ok(), "Should return valid state");
}

#[tokio::test]
async fn test_uninitialized_error() {
    // ARRANGE: Create sync instance but don't initialize it
    let (sync, _, _, _) = create_test_mcp_sync().await;
    // Deliberately not calling init()
    
    // Create test context
    let context = create_test_context();
    
    // ACT & ASSERT: Operations should fail with NotInitialized error
    let sync_result = sync.sync().await;
    assert!(sync_result.is_err(), "Sync should fail when not initialized");
    
    let record_result = sync.record_context_change(&context, StateOperation::Create).await;
    assert!(record_result.is_err(), "Record change should fail when not initialized");
} 