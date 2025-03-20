use std::sync::Arc;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::context::{ContextManager, ContextTracker};
use crate::context_adapter::ContextAdapter;
use crate::mcp::protocol::{
    MCPProtocolAdapter,
    ProtocolConfig,
    create_initialized_protocol_adapter,
    create_protocol_adapter_with_config,
};
use crate::mcp::types::{MCPMessage, MessageId, MessageType, MessageMetadata};
use crate::test_utils::{TestData, TestFactory, MockContextAdapter};

#[test]
async fn test_adapter_with_context_integration() {
    // Create a test environment using DI
    let env = TestFactory::create_test_environment().unwrap();
    
    // Initialize the protocol adapter
    let protocol_adapter = Arc::new(MCPProtocolAdapter::new());
    protocol_adapter.initialize().await.unwrap();
    
    // Create test message for context update
    let message = MCPMessage {
        protocol_version: "1.0".to_string(),
        id: MessageId(format!("test-{}", uuid::Uuid::new_v4())),
        message_type: MessageType::ContextUpdate,
        payload: serde_json::to_vec(&json!({
            "context_id": "test-context",
            "state": TestData::create_test_state(),
        })).unwrap(),
        metadata: MessageMetadata::default(),
    };
    
    // Create context
    let context_id = "test-context";
    env.context_tracker.create_context(context_id).await.unwrap();
    
    // Set up context state via the adapter
    {
        let mock_adapter = env.context_adapter.clone();
        let mut adapter_guard = mock_adapter.write().await;
        adapter_guard.initialize().unwrap();
    }
    
    // Test synchronization between protocol and context
    let test_state = TestData::create_test_state();
    
    // Set state in protocol
    protocol_adapter.set_state(test_state.clone()).await;
    
    // Set state in context adapter
    {
        let context_adapter = env.context_adapter.clone();
        let adapter_guard = context_adapter.read().await;
        adapter_guard.set_state(test_state.clone()).await.unwrap();
    }
    
    // Verify states match
    let protocol_state = protocol_adapter.get_state().await;
    let context_state = {
        let context_adapter = env.context_adapter.clone();
        let adapter_guard = context_adapter.read().await;
        adapter_guard.get_state().await.unwrap()
    };
    
    assert_eq!(protocol_state, test_state);
    assert_eq!(context_state, test_state);
}

#[test]
async fn test_factory_integration() {
    // Test the factory methods with multiple adapters
    
    // Create first adapter with default config
    let adapter1 = create_initialized_protocol_adapter().await.unwrap();
    
    // Create second adapter with custom config
    let custom_config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048 * 1024,
        timeout_ms: 10000,
    };
    
    let adapter2 = create_protocol_adapter_with_config(custom_config).await.unwrap();
    
    // Verify they're both initialized but with different configs
    assert!(adapter1.is_initialized().await);
    assert!(adapter2.is_initialized().await);
    
    let config1 = adapter1.get_config().await;
    let config2 = adapter2.get_config().await;
    
    assert_eq!(config1.version, "1.0");
    assert_eq!(config2.version, "2.0");
    
    // Test they can operate independently
    let test_state1 = json!({"adapter": 1, "status": "active"});
    let test_state2 = json!({"adapter": 2, "status": "active"});
    
    adapter1.set_state(test_state1.clone()).await;
    adapter2.set_state(test_state2.clone()).await;
    
    let state1 = adapter1.get_state().await;
    let state2 = adapter2.get_state().await;
    
    assert_eq!(state1, test_state1);
    assert_eq!(state2, test_state2);
}

#[test]
async fn test_multi_adapter_isolation() {
    // Create multiple protocol adapters to test isolation
    let adapter1 = Arc::new(MCPProtocolAdapter::new());
    let adapter2 = Arc::new(MCPProtocolAdapter::new());
    
    // Initialize both
    adapter1.initialize().await.unwrap();
    adapter2.initialize().await.unwrap();
    
    // Set different states
    let state1 = json!({"id": 1, "name": "first"});
    let state2 = json!({"id": 2, "name": "second"});
    
    adapter1.set_state(state1.clone()).await;
    adapter2.set_state(state2.clone()).await;
    
    // Verify states are isolated
    assert_eq!(adapter1.get_state().await, state1);
    assert_eq!(adapter2.get_state().await, state2);
    
    // Change state in one
    let new_state1 = json!({"id": 1, "name": "updated"});
    adapter1.set_state(new_state1.clone()).await;
    
    // Verify other adapter is unaffected
    assert_eq!(adapter1.get_state().await, new_state1);
    assert_eq!(adapter2.get_state().await, state2);
}

#[test]
async fn test_adapter_cloning() {
    // Create an adapter
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.unwrap();
    
    // Set initial state
    let initial_state = json!({"status": "initial"});
    adapter.set_state(initial_state.clone()).await;
    
    // Clone the adapter
    let cloned_adapter = adapter.clone();
    
    // Verify states match
    assert_eq!(adapter.get_state().await, initial_state);
    assert_eq!(cloned_adapter.get_state().await, initial_state);
    
    // Change state in original
    let new_state = json!({"status": "updated"});
    adapter.set_state(new_state.clone()).await;
    
    // Verify both adapters see the change (they share internal state)
    assert_eq!(adapter.get_state().await, new_state);
    assert_eq!(cloned_adapter.get_state().await, new_state);
}

#[test]
async fn test_error_conditions() {
    // Test various error conditions
    
    // 1. Initialize adapter twice
    let adapter = MCPProtocolAdapter::new();
    assert!(adapter.initialize().await.is_ok());
    assert!(adapter.initialize().await.is_err()); // Should fail second time
    
    // 2. Use uninitialized adapter
    let uninit_adapter = MCPProtocolAdapter::new();
    
    // Message for testing
    let message = MCPMessage {
        protocol_version: "1.0".to_string(),
        id: MessageId(format!("test-{}", uuid::Uuid::new_v4())),
        message_type: MessageType::ContextUpdate,
        payload: vec![],
        metadata: MessageMetadata::default(),
    };
    
    // Try operations on uninitialized adapter - all should fail
    assert!(uninit_adapter.handle_message(&message).await.is_err());
    assert!(uninit_adapter.register_handler(
        MessageType::ContextUpdate, 
        Box::new(super::TestCommandHandler::new("test", false))
    ).await.is_err());
    assert!(uninit_adapter.unregister_handler(&MessageType::ContextUpdate).await.is_err());
    
    // State operations should not fail but have no effect or return defaults
    assert_eq!(uninit_adapter.get_state().await, serde_json::Value::Null);
    uninit_adapter.set_state(json!({"test": true})).await; // Should not crash
    
    // Config should return default
    let config = uninit_adapter.get_config().await;
    assert_eq!(config.version, "1.0");
} 