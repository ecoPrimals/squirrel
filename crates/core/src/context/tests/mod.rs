use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::context::{ContextManager, ContextTracker, ContextSubscriber, ContextConfig};
use crate::test_utils::{TestFactory, TestData};

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