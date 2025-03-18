use std::sync::Arc;
use std::collections::HashMap;

use serde_json::json;
use tokio::sync::RwLock;
use tokio::test;

use crate::mcp::protocol::{
    MCPProtocolBase,
    ProtocolConfig,
    CommandHandler,
    MCPProtocolAdapter,
    MCPProtocolFactory,
    create_protocol_adapter,
    create_protocol_adapter_with_protocol,
};
use crate::mcp::types::{
    MCPMessage,
    MCPResponse,
    MessageId,
    MessageType,
    ResponseStatus,
    MessageMetadata,
};
use crate::error::{SquirrelError, Result};
use crate::test_utils::{TestData, TestFactory};

// Test command handler for testing
#[derive(Debug, Clone)]
struct TestCommandHandler {
    response_data: String,
    should_fail: bool,
}

impl TestCommandHandler {
    fn new(response_data: &str, should_fail: bool) -> Self {
        Self {
            response_data: response_data.to_string(),
            should_fail,
        }
    }
}

#[async_trait::async_trait]
impl CommandHandler for TestCommandHandler {
    async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
        if self.should_fail {
            return Err(SquirrelError::MCP("Test failure".to_string()));
        }
        
        Ok(MCPResponse {
            protocol_version: "1.0".to_string(),
            message_id: message.id.0.clone(),
            status: ResponseStatus::Success,
            payload: serde_json::to_vec(&json!({
                "response": self.response_data,
                "original": message.payload,
            })).unwrap(),
            error_message: None,
            metadata: MessageMetadata::default(),
        })
    }
}

// Helper function to create a test message
fn create_test_message(message_type: MessageType, payload: serde_json::Value) -> MCPMessage {
    MCPMessage {
        protocol_version: "1.0".to_string(),
        id: MessageId(format!("test-{}", uuid::Uuid::new_v4())),
        message_type,
        payload: serde_json::to_vec(&payload).unwrap(),
        metadata: MessageMetadata::default(),
    }
}

#[test]
async fn test_protocol_base_creation() {
    // Test with default config
    let protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Verify config values
    let config = protocol.get_config();
    assert_eq!(config.version, "1.0");
    assert_eq!(config.max_message_size, 1024 * 1024);
    assert_eq!(config.timeout_ms, 5000);
    
    // Verify initial state
    assert_eq!(*protocol.get_state(), serde_json::Value::Null);
}

#[test]
async fn test_protocol_with_custom_config() {
    // Test with custom config
    let config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048 * 1024,
        timeout_ms: 10000,
    };
    
    let protocol = MCPProtocolBase::new(config.clone());
    
    // Verify config values
    let retrieved_config = protocol.get_config();
    assert_eq!(retrieved_config.version, "2.0");
    assert_eq!(retrieved_config.max_message_size, 2048 * 1024);
    assert_eq!(retrieved_config.timeout_ms, 10000);
}

#[test]
async fn test_protocol_state_management() {
    let mut protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Set state
    let test_state = json!({
        "status": "ready",
        "session_id": "test-session",
        "connected": true
    });
    
    protocol.set_state(test_state.clone());
    
    // Get state
    let retrieved_state = protocol.get_state();
    assert_eq!(*retrieved_state, test_state);
}

#[test]
async fn test_handler_registration() {
    let mut protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Register handler
    let handler = Box::new(TestCommandHandler::new("test response", false));
    let message_type = MessageType::ContextUpdate;
    
    assert!(protocol.register_handler(message_type.clone(), handler).is_ok());
    
    // Try to register duplicate handler - should fail
    let another_handler = Box::new(TestCommandHandler::new("another response", false));
    assert!(protocol.register_handler(message_type.clone(), another_handler).is_err());
    
    // Unregister handler
    assert!(protocol.unregister_handler(&message_type).is_ok());
    
    // Try to unregister non-existent handler - should fail
    assert!(protocol.unregister_handler(&message_type).is_err());
}

#[test]
async fn test_message_handling() {
    let mut protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Register handler
    let handler = Box::new(TestCommandHandler::new("test response", false));
    let message_type = MessageType::ContextUpdate;
    protocol.register_handler(message_type.clone(), handler).unwrap();
    
    // Create test message
    let test_payload = json!({"data": "test data"});
    let message = create_test_message(message_type, test_payload.clone());
    
    // Handle message
    let response = protocol.handle_message_with_handler(&message).await.unwrap();
    
    // Verify response
    assert_eq!(response.message_id, message.id.0);
    assert_eq!(response.status, ResponseStatus::Success);
    
    // Verify response payload
    let response_payload: serde_json::Value = serde_json::from_slice(&response.payload).unwrap();
    assert_eq!(response_payload["response"], "test response");
    assert_eq!(response_payload["original"], serde_json::to_vec(&test_payload).unwrap());
}

#[test]
async fn test_error_handling() {
    let mut protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Register handler that will fail
    let handler = Box::new(TestCommandHandler::new("", true));
    let message_type = MessageType::ContextUpdate;
    protocol.register_handler(message_type.clone(), handler).unwrap();
    
    // Create test message
    let test_payload = json!({"data": "test data"});
    let message = create_test_message(message_type, test_payload);
    
    // Handle message - should fail
    let result = protocol.handle_message_with_handler(&message).await;
    assert!(result.is_err());
}

#[test]
async fn test_protocol_factory() {
    // Create factory with default config
    let factory = MCPProtocolFactory::default();
    
    // Create protocol
    let protocol = factory.create_protocol();
    
    // Verify config
    let config = protocol.get_config();
    assert_eq!(config.version, "1.0");
    
    // Create factory with custom config
    let custom_config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048 * 1024,
        timeout_ms: 10000,
    };
    
    let factory = MCPProtocolFactory::with_config(custom_config.clone());
    
    // Create protocol
    let protocol = factory.create_protocol();
    
    // Verify config
    let config = protocol.get_config();
    assert_eq!(config.version, "2.0");
    assert_eq!(config.max_message_size, 2048 * 1024);
    assert_eq!(config.timeout_ms, 10000);
}

#[test]
async fn test_protocol_adapter() {
    // Create adapter
    let adapter = MCPProtocolAdapter::new();
    
    // Should not be initialized
    assert!(!adapter.is_initialized().await);
    
    // Initialize
    adapter.initialize().await.unwrap();
    
    // Should be initialized
    assert!(adapter.is_initialized().await);
    
    // Get config
    let config = adapter.get_config().await;
    assert_eq!(config.version, "1.0");
    
    // Set state
    let test_state = json!({"status": "active"});
    adapter.set_state(test_state.clone()).await;
    
    // Get state
    let state = adapter.get_state().await;
    assert_eq!(state, test_state);
}

#[test]
async fn test_protocol_adapter_with_config() {
    // Create adapter
    let adapter = MCPProtocolAdapter::new();
    
    // Custom config
    let config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048 * 1024,
        timeout_ms: 10000,
    };
    
    // Initialize with config
    adapter.initialize_with_config(config.clone()).await.unwrap();
    
    // Should be initialized
    assert!(adapter.is_initialized().await);
    
    // Get config
    let retrieved_config = adapter.get_config().await;
    assert_eq!(retrieved_config.version, "2.0");
    assert_eq!(retrieved_config.max_message_size, 2048 * 1024);
    assert_eq!(retrieved_config.timeout_ms, 10000);
}

#[test]
async fn test_protocol_adapter_handler_operations() {
    // Create adapter
    let adapter = Arc::new(MCPProtocolAdapter::new());
    
    // Initialize
    adapter.initialize().await.unwrap();
    
    // Register handler
    let handler = Box::new(TestCommandHandler::new("test response", false));
    let message_type = MessageType::ContextUpdate;
    adapter.register_handler(message_type.clone(), handler).await.unwrap();
    
    // Create test message
    let test_payload = json!({"data": "test data"});
    let message = create_test_message(message_type.clone(), test_payload);
    
    // Handle message
    let response = adapter.handle_message(&message).await.unwrap();
    
    // Verify response
    assert_eq!(response.message_id, message.id.0);
    assert_eq!(response.status, ResponseStatus::Success);
    
    // Unregister handler
    adapter.unregister_handler(&message_type).await.unwrap();
}

#[test]
async fn test_di_pattern_with_protocol() {
    // Create protocol
    let protocol = MCPProtocolBase::new(ProtocolConfig::default());
    
    // Create adapter with protocol using DI
    let adapter = Arc::new(MCPProtocolAdapter::with_protocol(protocol));
    
    // Should be initialized already
    assert!(adapter.is_initialized().await);
    
    // Register handler
    let handler = Box::new(TestCommandHandler::new("test response", false));
    let message_type = MessageType::ContextUpdate;
    adapter.register_handler(message_type.clone(), handler).await.unwrap();
    
    // Create test message
    let test_payload = json!({"data": "test data"});
    let message = create_test_message(message_type, test_payload);
    
    // Handle message
    let response = adapter.handle_message(&message).await.unwrap();
    
    // Verify response
    assert_eq!(response.message_id, message.id.0);
    assert_eq!(response.status, ResponseStatus::Success);
}

#[test]
async fn test_factory_functions() {
    // Test create_protocol_adapter
    let adapter = create_protocol_adapter();
    
    // Should not be initialized
    assert!(!adapter.is_initialized().await);
    
    // Initialize
    adapter.initialize().await.unwrap();
    
    // Should be initialized
    assert!(adapter.is_initialized().await);
    
    // Test create_protocol_adapter_with_protocol
    let protocol = MCPProtocolBase::new(ProtocolConfig::default());
    let adapter = create_protocol_adapter_with_protocol(protocol);
    
    // Should be initialized
    assert!(adapter.is_initialized().await);
} 