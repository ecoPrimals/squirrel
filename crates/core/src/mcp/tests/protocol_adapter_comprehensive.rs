//! Comprehensive tests for the MCPProtocolAdapter
//!
//! This module contains comprehensive tests for the MCPProtocolAdapter
//! implementation, focusing on message handling, handler registration,
//! error cases, and integration with other components.

use std::sync::Arc;
use assert_matches::assert_matches;
use serde_json::json;

use crate::error::{Result, SquirrelError};
use crate::mcp::protocol::{
    MCPProtocolAdapter,
    ProtocolConfig,
    CommandHandler,
    create_protocol_adapter,
    create_protocol_adapter_with_config,
    create_initialized_protocol_adapter,
};
use crate::mcp::types::{
    MCPMessage,
    MCPResponse,
    MessageId,
    MessageType,
    ResponseStatus,
    ProtocolState,
};

// Mock handler that processes messages of a specific type
struct MockCommandHandler {
    success: bool,
    response_payload: Vec<u8>,
}

#[async_trait::async_trait]
impl CommandHandler for MockCommandHandler {
    async fn handle_command(&self, message: &MCPMessage) -> Result<MCPResponse> {
        if !self.success {
            return Err(SquirrelError::MCP("Mock handler error".to_string()));
        }
        
        Ok(MCPResponse {
            protocol_version: "1.0".to_string(),
            message_id: message.id.0.clone(),
            status: ResponseStatus::Success,
            payload: self.response_payload.clone(),
            error_message: None,
            metadata: Default::default(),
        })
    }
}

// Implementation of test cases
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    
    // Test adapter creation with factory functions
    #[tokio::test]
    async fn test_adapter_factory_functions() {
        // Test uninitialized adapter creation
        let adapter = create_protocol_adapter();
        assert!(!adapter.is_initialized().await);
        
        // Test adapter creation with config
        let config = ProtocolConfig {
            version: "2.0".to_string(),
            max_message_size: 4096,
            timeout_ms: 10000,
        };
        let adapter = create_protocol_adapter_with_config(config.clone()).await.unwrap();
        assert!(adapter.is_initialized().await);
        
        let retrieved_config = adapter.get_config().await;
        assert_eq!(retrieved_config.version, "2.0");
        assert_eq!(retrieved_config.max_message_size, 4096);
        
        // Test initialized adapter creation
        let initialized_adapter = create_initialized_protocol_adapter().await.unwrap();
        assert!(initialized_adapter.is_initialized().await);
    }
    
    // Test handler registration and message processing
    #[tokio::test]
    async fn test_handler_registration_and_message_handling() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Create a test message
        let message = MCPMessage {
            id: MessageId("test-message-id".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test_command", "params": {"key": "value"}}),
        };
        
        // Try to handle message without handler (should fail)
        let result = adapter.handle_message(&message).await;
        assert!(result.is_err());
        
        // Register a successful handler
        let success_handler = MockCommandHandler {
            success: true,
            response_payload: vec![1, 2, 3, 4],
        };
        adapter.register_handler(MessageType::Command, Box::new(success_handler)).await.unwrap();
        
        // Handle message with registered handler
        let response = adapter.handle_message(&message).await.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.message_id, "test-message-id");
        assert_eq!(response.payload, vec![1, 2, 3, 4]);
        
        // Unregister the handler
        adapter.unregister_handler(&MessageType::Command).await.unwrap();
        
        // Try to handle message after unregistering handler (should fail)
        let result = adapter.handle_message(&message).await;
        assert!(result.is_err());
    }
    
    // Test error handling for various scenarios
    #[tokio::test]
    async fn test_error_handling() {
        // Create adapter but don't initialize
        let adapter = MCPProtocolAdapter::new();
        
        // Create a test message
        let message = MCPMessage {
            id: MessageId("test-message-id".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test_command"}),
        };
        
        // Try operations on uninitialized adapter
        let handle_result = adapter.handle_message(&message).await;
        assert!(handle_result.is_err());
        
        let register_result = adapter.register_handler(
            MessageType::Command, 
            Box::new(MockCommandHandler { success: true, response_payload: vec![] })
        ).await;
        assert!(register_result.is_err());
        
        let unregister_result = adapter.unregister_handler(&MessageType::Command).await;
        assert!(unregister_result.is_err());
        
        // Initialize adapter
        adapter.initialize().await.unwrap();
        
        // Register a failing handler
        let failing_handler = MockCommandHandler {
            success: false,
            response_payload: vec![],
        };
        adapter.register_handler(MessageType::Command, Box::new(failing_handler)).await.unwrap();
        
        // Handle message with failing handler (should return error)
        let handle_result = adapter.handle_message(&message).await;
        assert!(handle_result.is_err());
        
        // Try to initialize adapter again (should fail)
        let reinit_result = adapter.initialize().await;
        assert!(reinit_result.is_err());
    }
    
    // Test protocol state management
    #[tokio::test]
    async fn test_protocol_state_management() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Set protocol state
        let state_value = json!({
            "state": "Initialized",
            "details": {
                "connected_clients": 5,
                "active_sessions": 3
            }
        });
        adapter.set_state(state_value.clone()).await;
        
        // Get protocol state
        let retrieved_state = adapter.get_state().await;
        
        // Verify state matches what we set
        assert_eq!(retrieved_state, state_value);
        
        // Set new state
        let new_state = json!({
            "state": "Error",
            "details": {
                "error_code": 500,
                "error_message": "Test error"
            }
        });
        adapter.set_state(new_state.clone()).await;
        
        // Verify state updated
        let updated_state = adapter.get_state().await;
        assert_eq!(updated_state, new_state);
    }
    
    // Test adapter clone functionality
    #[tokio::test]
    async fn test_adapter_clone() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Register a handler
        let success_handler = MockCommandHandler {
            success: true,
            response_payload: vec![1, 2, 3, 4],
        };
        adapter.register_handler(MessageType::Command, Box::new(success_handler)).await.unwrap();
        
        // Clone the adapter
        let cloned_adapter = adapter.clone();
        
        // Verify cloned adapter is initialized
        assert!(cloned_adapter.is_initialized().await);
        
        // Create a test message
        let message = MCPMessage {
            id: MessageId("test-clone-message".to_string()),
            message_type: MessageType::Command,
            payload: json!({"command": "test_command"}),
        };
        
        // Handle message with cloned adapter
        let response = cloned_adapter.handle_message(&message).await.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        
        // Modify state in one adapter and verify it affects the clone
        let state = json!({"modified": true});
        adapter.set_state(state.clone()).await;
        
        let cloned_state = cloned_adapter.get_state().await;
        assert_eq!(cloned_state, state);
    }
    
    // Test with different message types
    #[tokio::test]
    async fn test_different_message_types() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();
        
        // Register handlers for different message types
        for message_type in [MessageType::Command, MessageType::Event, MessageType::Error] {
            let handler = MockCommandHandler {
                success: true,
                response_payload: vec![message_type as u8],
            };
            adapter.register_handler(message_type, Box::new(handler)).await.unwrap();
        }
        
        // Test each message type
        for message_type in [MessageType::Command, MessageType::Event, MessageType::Error] {
            let message = MCPMessage {
                id: MessageId(format!("test-{:?}-message", message_type)),
                message_type,
                payload: json!({}),
            };
            
            let response = adapter.handle_message(&message).await.unwrap();
            assert_eq!(response.status, ResponseStatus::Success);
            assert_eq!(response.payload, vec![message_type as u8]);
        }
    }
    
    // Test concurrent message handling
    #[tokio::test]
    async fn test_concurrent_message_handling() {
        // Create and initialize adapter
        let adapter = Arc::new(MCPProtocolAdapter::new());
        adapter.initialize().await.unwrap();
        
        // Register handler
        let handler = MockCommandHandler {
            success: true,
            response_payload: vec![42],
        };
        adapter.register_handler(MessageType::Command, Box::new(handler)).await.unwrap();
        
        // Create multiple tasks to handle messages concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let adapter_clone = adapter.clone();
            let message = MCPMessage {
                id: MessageId(format!("concurrent-message-{}", i)),
                message_type: MessageType::Command,
                payload: json!({"index": i}),
            };
            
            let handle = tokio::spawn(async move {
                adapter_clone.handle_message(&message).await
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;
        
        // Verify all messages were processed successfully
        for result in results {
            let response = result.unwrap().unwrap();
            assert_eq!(response.status, ResponseStatus::Success);
            assert_eq!(response.payload, vec![42]);
        }
    }
} 