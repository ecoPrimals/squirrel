//! Tests for the refactored MCP module with dependency injection

use crate::mcp::protocol::{
    MCPProtocolAdapter, 
    ProtocolConfig,
    CommandHandler
};
use crate::mcp::types::{
    MCPMessage,
    MessageId,
    MessageType,
    ResponseStatus,
    MCPResponse,
};
use serde_json::{json, Value};
use crate::error::Result;
use std::sync::Arc;
use async_trait::async_trait;

#[tokio::test]
async fn test_adapter_initialization() {
    // Create a new protocol adapter
    let adapter = MCPProtocolAdapter::new();
    
    // Check that the adapter is not initialized
    assert!(!adapter.is_initialized().await);
    
    // Initialize the adapter
    adapter.initialize().await.unwrap();
    
    // Check that the adapter is initialized
    assert!(adapter.is_initialized().await);
    
    // Get and set state
    let state_value = json!({"state": "ready"});
    adapter.set_state(state_value.clone()).await.unwrap();
    
    let retrieved_state = adapter.get_state().await.unwrap();
    assert_eq!(retrieved_state, state_value);
    
    // Check the config
    let config = adapter.get_config().await.unwrap();
    assert_eq!(config.version, "1.0");
}

#[tokio::test]
async fn test_message_handling() {
    // Create a new protocol adapter
    let adapter = MCPProtocolAdapter::new();
    
    // Initialize the adapter
    adapter.initialize().await.unwrap();
    
    // Create a test message handler
    #[derive(Debug)]
    struct TestHandler;
    
    #[async_trait]
    impl CommandHandler for TestHandler {
        async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
            // Create a response message
            Ok(MCPResponse {
                message_id: message.id.0.clone(),
                status: ResponseStatus::Success,
                payload: json!({"status": "success"}),
            })
        }
    }
    
    // Register the handler
    adapter.register_handler(MessageType::Command, Arc::new(TestHandler)).await.unwrap();
    
    // Create a test message
    let message = MCPMessage {
        id: MessageId("test-1".to_string()),
        message_type: MessageType::Command,
        payload: json!({
            "command": "test",
            "args": {
                "value": 42
            }
        }),
    };
    
    // Handle the message
    let response = adapter.handle_message(&message).await.unwrap();
    
    // Check the response
    assert_eq!(response.message_id, "test-1");
    assert_eq!(response.status, ResponseStatus::Success);
}

#[tokio::test]
async fn test_uninitialized_adapter() {
    // Create a new adapter without initialization
    let adapter = MCPProtocolAdapter::new();
    
    // Check that the adapter is not initialized
    assert!(!adapter.is_initialized().await);
    
    // Create a test message
    let message = MCPMessage {
        id: MessageId("test-uninitialized".to_string()),
        message_type: MessageType::Command,
        payload: json!({}),
    };
    
    // Try to handle the message - should fail
    let result = adapter.handle_message(&message).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_protocol_config() {
    // Create a custom config
    let config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048,
        timeout_ms: 10000,
    };
    
    // Create and initialize adapter with config
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize_with_config(config.clone()).await.unwrap();
    
    // Check the config was applied
    let retrieved_config = adapter.get_config().await.unwrap();
    assert_eq!(retrieved_config.version, "2.0");
    assert_eq!(retrieved_config.max_message_size, 2048);
    assert_eq!(retrieved_config.timeout_ms, 10000);
} 