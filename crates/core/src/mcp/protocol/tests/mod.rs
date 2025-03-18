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

// Test fixtures and helpers

/// Test command handler for testing
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
            id: message.id.clone(),
            status: ResponseStatus::Success,
            message: "Command executed successfully".to_string(),
            data: json!({ "result": self.response_data }),
            metadata: None,
        })
    }
}

/// Helper function to create test messages
fn create_test_message(message_type: MessageType, payload: serde_json::Value) -> MCPMessage {
    MCPMessage {
        id: MessageId::new(),
        message_type,
        command: "test".to_string(),
        payload,
        metadata: Some(MessageMetadata::default()),
    }
}

/// Helper function to create a protocol base with DI pattern
fn create_test_protocol() -> MCPProtocolBase {
    // ARRANGE: Create a protocol base
    MCPProtocolBase::new()
}

/// Helper function to create a protocol adapter with DI pattern
async fn create_test_adapter() -> MCPProtocolAdapter {
    // ARRANGE: Create a protocol adapter
    let protocol = MCPProtocolBase::new();
    MCPProtocolAdapter::new(Arc::new(RwLock::new(protocol)))
}

// Tests with AAA pattern

#[test]
async fn test_protocol_base_creation() {
    // ARRANGE: Create protocol
    let protocol = create_test_protocol();
    
    // ACT & ASSERT: Verify initial state
    assert_eq!(protocol.handlers.read().await.len(), 0, "New protocol should have no handlers");
    
    // State should be empty
    let state = protocol.get_state().await;
    assert!(state.is_ok(), "Getting state should succeed");
    assert_eq!(state.unwrap(), json!({}), "Initial state should be empty");
}

#[test]
async fn test_protocol_with_custom_config() {
    // ARRANGE: Create custom config
    let config = ProtocolConfig {
        max_message_size: 10240,
        timeout_ms: 5000,
        enable_validation: true,
    };
    
    // ACT: Create protocol with config
    let protocol = MCPProtocolBase::with_config(config.clone());
    
    // ASSERT: Verify config was applied
    let retrieved_config = protocol.get_config().await;
    assert!(retrieved_config.is_ok(), "Getting config should succeed");
    
    let retrieved_config = retrieved_config.unwrap();
    assert_eq!(retrieved_config.max_message_size, config.max_message_size, "Max message size should match");
    assert_eq!(retrieved_config.timeout_ms, config.timeout_ms, "Timeout should match");
    assert_eq!(retrieved_config.enable_validation, config.enable_validation, "Validation flag should match");
}

#[test]
async fn test_protocol_state_management() {
    // ARRANGE: Create protocol
    let protocol = create_test_protocol();
    
    // ACT: Set state
    let test_state = json!({
        "key": "value",
        "number": 42
    });
    
    let set_result = protocol.set_state(test_state.clone()).await;
    
    // ASSERT: Verify state was set
    assert!(set_result.is_ok(), "Setting state should succeed");
    
    // Get state and verify
    let state = protocol.get_state().await.unwrap();
    assert_eq!(state, test_state, "Retrieved state should match set state");
}

#[test]
async fn test_handler_registration() {
    // ARRANGE: Create protocol and handler
    let protocol = create_test_protocol();
    let handler = TestCommandHandler::new("test-response", false);
    
    // ACT: Register handler
    let register_result = protocol.register_handler("test-command", Arc::new(handler)).await;
    
    // ASSERT: Verify handler was registered
    assert!(register_result.is_ok(), "Registering handler should succeed");
    
    // Verify handler count
    let handlers = protocol.handlers.read().await;
    assert_eq!(handlers.len(), 1, "Protocol should have one handler");
    assert!(handlers.contains_key("test-command"), "Handler should be registered under correct name");
}

#[test]
async fn test_message_handling() {
    // ARRANGE: Create protocol with handler
    let protocol = create_test_protocol();
    let handler = TestCommandHandler::new("success-response", false);
    
    protocol.register_handler("test-command", Arc::new(handler)).await
        .expect("Failed to register handler");
    
    // Create test message
    let message = create_test_message(
        MessageType::Command,
        json!({ "param": "value" })
    );
    
    // Set message command to match handler
    let mut message = message;
    message.command = "test-command".to_string();
    
    // ACT: Handle message
    let response = protocol.handle_message(&message).await;
    
    // ASSERT: Verify message was handled correctly
    assert!(response.is_ok(), "Handling message should succeed");
    
    let response = response.unwrap();
    assert_eq!(response.status, ResponseStatus::Success, "Response should have success status");
    assert_eq!(response.data.get("result").unwrap().as_str().unwrap(), "success-response", 
        "Response should contain expected data");
}

#[test]
async fn test_error_handling() {
    // ARRANGE: Create protocol with failing handler
    let protocol = create_test_protocol();
    let handler = TestCommandHandler::new("", true); // Set to fail
    
    protocol.register_handler("fail-command", Arc::new(handler)).await
        .expect("Failed to register handler");
    
    // Create test message
    let mut message = create_test_message(
        MessageType::Command,
        json!({ "param": "value" })
    );
    message.command = "fail-command".to_string();
    
    // ACT: Handle message
    let response = protocol.handle_message(&message).await;
    
    // ASSERT: Verify error is propagated
    assert!(response.is_err(), "Handling should fail for failing handler");
    
    // Try with unregistered command
    let mut message = create_test_message(
        MessageType::Command,
        json!({ "param": "value" })
    );
    message.command = "nonexistent-command".to_string();
    
    let response = protocol.handle_message(&message).await;
    assert!(response.is_err(), "Handling should fail for unregistered command");
}

#[test]
async fn test_protocol_factory() {
    // ARRANGE: Create factory
    let factory = MCPProtocolFactory::new();
    
    // ACT: Create protocol with factory
    let protocol_result = factory.create_protocol().await;
    
    // ASSERT: Verify protocol was created
    assert!(protocol_result.is_ok(), "Creating protocol should succeed");
    let protocol = protocol_result.unwrap();
    
    // ACT: Create protocol with custom config
    let config = ProtocolConfig {
        max_message_size: 20480,
        timeout_ms: 3000,
        enable_validation: false,
    };
    
    let protocol_with_config = factory.create_protocol_with_config(config.clone()).await;
    
    // ASSERT: Verify protocol with config was created
    assert!(protocol_with_config.is_ok(), "Creating protocol with config should succeed");
    let protocol_with_config = protocol_with_config.unwrap();
    
    // Verify config was applied
    let retrieved_config = protocol_with_config.protocol.get_config().await
        .expect("Failed to get config");
    
    assert_eq!(retrieved_config.max_message_size, config.max_message_size, 
        "Max message size should match");
}

#[test]
async fn test_protocol_adapter() {
    // ARRANGE: Create adapter
    let adapter = create_test_adapter().await;
    
    // ACT & ASSERT: Test initialization
    assert!(adapter.is_initialized().await, "Adapter should be initialized");
    
    // Test state management
    let test_state = json!({
        "adapter_key": "adapter_value",
        "nested": {
            "field": 123
        }
    });
    
    // ACT: Set state
    let set_result = adapter.set_state(test_state.clone()).await;
    
    // ASSERT: Verify state was set
    assert!(set_result.is_ok(), "Setting state should succeed");
    
    // Get state and verify
    let state = adapter.get_state().await.unwrap();
    assert_eq!(state, test_state, "Retrieved state should match set state");
}

#[test]
async fn test_protocol_adapter_with_config() {
    // ARRANGE: Create protocol with custom config
    let config = ProtocolConfig {
        max_message_size: 30720,
        timeout_ms: 10000,
        enable_validation: true,
    };
    
    let protocol = MCPProtocolBase::with_config(config.clone());
    
    // ACT: Create adapter with protocol
    let adapter = MCPProtocolAdapter::new(Arc::new(RwLock::new(protocol)));
    
    // ASSERT: Verify adapter is initialized with correct config
    assert!(adapter.is_initialized().await, "Adapter should be initialized");
    
    // Get config and verify
    let retrieved_config = adapter.get_config().await.unwrap();
    assert_eq!(retrieved_config.max_message_size, config.max_message_size, 
        "Max message size should match");
    assert_eq!(retrieved_config.timeout_ms, config.timeout_ms, 
        "Timeout should match");
    assert_eq!(retrieved_config.enable_validation, config.enable_validation, 
        "Validation flag should match");
}

#[test]
async fn test_protocol_adapter_handler_operations() {
    // ARRANGE: Create adapter
    let adapter = create_test_adapter().await;
    
    // Create handler
    let handler = TestCommandHandler::new("adapter-response", false);
    
    // ACT: Register handler
    let register_result = adapter.register_handler("adapter-command", Arc::new(handler)).await;
    
    // ASSERT: Verify handler was registered
    assert!(register_result.is_ok(), "Registering handler should succeed");
    
    // Create message
    let mut message = create_test_message(
        MessageType::Command,
        json!({ "adapter_param": "adapter_value" })
    );
    message.command = "adapter-command".to_string();
    
    // ACT: Handle message
    let response = adapter.handle_message(&message).await;
    
    // ASSERT: Verify message was handled correctly
    assert!(response.is_ok(), "Handling message should succeed");
    
    let response = response.unwrap();
    assert_eq!(response.status, ResponseStatus::Success, "Response should have success status");
    assert_eq!(response.data.get("result").unwrap().as_str().unwrap(), "adapter-response", 
        "Response should contain expected data");
}

#[test]
async fn test_di_pattern_with_protocol() {
    // ARRANGE: Create test environment with shared protocol
    let protocol = Arc::new(RwLock::new(create_test_protocol()));
    
    // Create two adapters sharing the same protocol
    let adapter1 = MCPProtocolAdapter::new(protocol.clone());
    let adapter2 = MCPProtocolAdapter::new(protocol.clone());
    
    // Create and register handler for first adapter
    let handler = TestCommandHandler::new("shared-protocol-response", false);
    adapter1.register_handler("shared-command", Arc::new(handler)).await
        .expect("Failed to register handler");
    
    // ACT: Create and handle message with second adapter
    let mut message = create_test_message(
        MessageType::Command,
        json!({ "shared": true })
    );
    message.command = "shared-command".to_string();
    
    let response = adapter2.handle_message(&message).await;
    
    // ASSERT: Verify second adapter uses handler from first
    assert!(response.is_ok(), "Handling message should succeed");
    
    let response = response.unwrap();
    assert_eq!(response.status, ResponseStatus::Success, "Response should have success status");
    assert_eq!(response.data.get("result").unwrap().as_str().unwrap(), "shared-protocol-response", 
        "Response should contain expected data");
}

#[test]
async fn test_factory_functions() {
    // ARRANGE: Test the create_protocol_adapter function
    
    // ACT: Create adapter with factory function
    let adapter_result = create_protocol_adapter().await;
    
    // ASSERT: Verify adapter creation
    assert!(adapter_result.is_ok(), "Creating adapter with factory function should succeed");
    let adapter = adapter_result.unwrap();
    assert!(adapter.is_initialized().await, "Adapter should be initialized");
    
    // ACT: Create adapter with custom protocol using factory function
    let custom_protocol = MCPProtocolBase::with_config(ProtocolConfig {
        max_message_size: 40960,
        timeout_ms: 15000,
        enable_validation: false,
    });
    
    let adapter_with_protocol_result = create_protocol_adapter_with_protocol(
        Arc::new(RwLock::new(custom_protocol))
    ).await;
    
    // ASSERT: Verify custom adapter creation
    assert!(adapter_with_protocol_result.is_ok(), 
        "Creating adapter with custom protocol should succeed");
} 