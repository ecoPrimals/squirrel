use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::context::{ContextManager, ContextTracker, ContextSubscriber, ContextConfig};
use crate::test_utils::{TestFactory, TestData};
use crate::context::{ContextState, ContextError};
use crate::context_adapter::{ContextAdapter, ContextAdapterConfig};

// Mock subscriber for testing
#[derive(Debug)]
struct TestSubscriber {
    received_update: RwLock<bool>,
    context_id: RwLock<Option<String>>,
}

impl TestSubscriber {
    fn new() -> Self {
        Self {
            received_update: RwLock::new(false),
            context_id: RwLock::new(None),
        }
    }
    
    async fn has_received_update(&self) -> bool {
        *self.received_update.read().await
    }
    
    async fn get_context_id(&self) -> Option<String> {
        self.context_id.read().await.clone()
    }
}

impl ContextSubscriber for TestSubscriber {
    async fn on_context_change(&self, context_id: &str) {
        *self.received_update.write().await = true;
        *self.context_id.write().await = Some(context_id.to_string());
    }
    
    async fn on_context_create(&self, context_id: &str) {
        *self.context_id.write().await = Some(context_id.to_string());
    }
    
    async fn on_context_delete(&self, _context_id: &str) {
        *self.context_id.write().await = None;
    }
}

#[test]
async fn test_context_manager_creation() {
    let manager = ContextManager::new();
    assert!(manager.active_context.read().await.is_none());
}

#[test]
async fn test_context_tracker_creation() {
    let manager = Arc::new(ContextManager::new());
    let tracker = ContextTracker::new(manager);
    
    // Tracker should start with no active context
    assert!(tracker.active_context.read().await.is_none());
}

#[test]
async fn test_context_activation() {
    let manager = Arc::new(ContextManager::new());
    let tracker = ContextTracker::new(manager.clone());
    
    // Create and activate context
    let context_id = "test-context-1";
    assert!(tracker.create_context(context_id).await.is_ok());
    assert!(tracker.activate_context(context_id).await.is_ok());
    
    // Check active context
    let active = tracker.active_context.read().await;
    assert!(active.is_some());
    assert_eq!(active.as_ref().unwrap(), context_id);
}

#[test]
async fn test_context_subscriber() {
    let manager = Arc::new(ContextManager::new());
    let tracker = ContextTracker::new(manager.clone());
    
    // Create subscriber
    let subscriber = Arc::new(TestSubscriber::new());
    tracker.add_subscriber(subscriber.clone()).await.unwrap();
    
    // Create context and verify notification
    let context_id = "test-context-2";
    tracker.create_context(context_id).await.unwrap();
    
    // Verify subscriber was notified
    assert_eq!(subscriber.get_context_id().await.unwrap(), context_id);
}

#[test]
async fn test_context_state_management() {
    let manager = Arc::new(ContextManager::new());
    let tracker = ContextTracker::new(manager.clone());
    
    // Create and activate context
    let context_id = "test-state-context";
    tracker.create_context(context_id).await.unwrap();
    tracker.activate_context(context_id).await.unwrap();
    
    // Update state via context adapter
    let state = TestData::create_test_state();
    let adapter = tracker.get_adapter(context_id).await.unwrap();
    
    {
        let adapter_guard = adapter.write().await;
        adapter_guard.set_state(state.clone()).await.unwrap();
    }
    
    // Retrieve state
    let retrieved_state = {
        let adapter_guard = adapter.read().await;
        adapter_guard.get_state().await.unwrap()
    };
    
    assert_eq!(retrieved_state, state);
}

#[test]
async fn test_context_config() {
    let config = ContextConfig {
        persistence_enabled: true,
        auto_save: true,
        history_size: 10,
    };
    
    let factory = Arc::new(ContextManager::factory());
    let tracker = factory.create_with_config(config.clone()).await.unwrap();
    
    // Create context
    let context_id = "config-test-context";
    tracker.create_context(context_id).await.unwrap();
    
    // Check config was applied
    let adapter = tracker.get_adapter(context_id).await.unwrap();
    let adapter_config = {
        let adapter_guard = adapter.read().await;
        adapter_guard.get_config().await.unwrap()
    };
    
    // Verify config settings were applied
    assert!(adapter_config.auto_save_interval.is_some());
}

/// Creates a test context adapter with default configuration
async fn create_test_adapter() -> Arc<ContextAdapter> {
    Arc::new(ContextAdapter::default())
}

#[test]
async fn test_context_adapter_basic_operations() {
    // Create a new adapter
    let adapter = create_test_adapter().await;
    
    // Create a new context
    let context_id = "test-context-1";
    let test_data = json!({
        "key": "value",
        "number": 42
    });
    
    let create_result = adapter.create_context(context_id.to_string(), test_data.clone()).await;
    assert!(create_result.is_ok());
    
    // Get the context and verify data
    let context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(context.id, context_id);
    assert_eq!(context.data, test_data);
    
    // Update the context
    let updated_data = json!({
        "key": "new-value",
        "number": 100,
        "additional": true
    });
    
    let update_result = adapter.update_context(context_id, updated_data.clone()).await;
    assert!(update_result.is_ok());
    
    // Verify the update
    let updated_context = adapter.get_context(context_id).await.unwrap();
    assert_eq!(updated_context.data, updated_data);
    
    // Delete the context
    let delete_result = adapter.delete_context(context_id).await;
    assert!(delete_result.is_ok());
    
    // Verify deletion
    let get_after_delete = adapter.get_context(context_id).await;
    assert!(get_after_delete.is_err());
}

#[test]
async fn test_context_adapter_multiple_contexts() {
    // Create a new adapter
    let adapter = create_test_adapter().await;
    
    // Create multiple contexts
    for i in 0..5 {
        let context_id = format!("test-context-{}", i);
        let data = json!({ "index": i });
        adapter.create_context(context_id, data).await.unwrap();
    }
    
    // List all contexts
    let contexts = adapter.list_contexts().await.unwrap();
    assert_eq!(contexts.len(), 5);
    
    // Verify context data
    for i in 0..5 {
        let context_id = format!("test-context-{}", i);
        let context = adapter.get_context(&context_id).await.unwrap();
        assert_eq!(context.data["index"], i);
    }
    
    // Delete contexts
    for i in 0..5 {
        let context_id = format!("test-context-{}", i);
        adapter.delete_context(&context_id).await.unwrap();
    }
    
    // Verify all deleted
    let remaining = adapter.list_contexts().await.unwrap();
    assert_eq!(remaining.len(), 0);
}

#[test]
async fn test_context_adapter_error_handling() {
    // Create a new adapter
    let adapter = create_test_adapter().await;
    
    // Try to get non-existent context
    let not_found = adapter.get_context("non-existent").await;
    assert!(not_found.is_err());
    
    // Try to update non-existent context
    let update_error = adapter.update_context("non-existent", json!({"test": true})).await;
    assert!(update_error.is_err());
    
    // Try to delete non-existent context
    let delete_error = adapter.delete_context("non-existent").await;
    assert!(delete_error.is_err());
    
    // Test max contexts limit
    let config = ContextAdapterConfig {
        max_contexts: 3,
        ttl_seconds: 3600,
        enable_auto_cleanup: true,
    };
    
    let limited_adapter = Arc::new(ContextAdapter::new(config));
    
    // Create 3 contexts (max limit)
    for i in 0..3 {
        let context_id = format!("limited-{}", i);
        limited_adapter.create_context(context_id, json!({"data": i})).await.unwrap();
    }
    
    // Try to create one more - should fail
    let overflow_result = limited_adapter.create_context("overflow".to_string(), json!({"test": true})).await;
    assert!(overflow_result.is_err());
}

#[test]
async fn test_context_adapter_config_update() {
    // Create a new adapter
    let adapter = create_test_adapter().await;
    
    // Check initial config
    let initial_config = adapter.get_config().await.unwrap();
    assert_eq!(initial_config.max_contexts, 1000);
    
    // Update config
    let new_config = ContextAdapterConfig {
        max_contexts: 2000,
        ttl_seconds: 7200,
        enable_auto_cleanup: false,
    };
    
    adapter.update_config(new_config.clone()).await.unwrap();
    
    // Verify updated config
    let updated_config = adapter.get_config().await.unwrap();
    assert_eq!(updated_config.max_contexts, 2000);
    assert_eq!(updated_config.ttl_seconds, 7200);
    assert_eq!(updated_config.enable_auto_cleanup, false);
} 