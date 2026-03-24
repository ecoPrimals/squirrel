// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::protocol::types::{MessageId, SecurityMetadata};
use crate::types::MCPResponse;
use crate::protocol::MessageType;
use crate::types::ResponseStatus;
use crate::types::MessageMetadata;
use chrono::Utc;
use crate::protocol::types::ProtocolVersion;
// BearDog handles security: // use crate::security::types::SecurityMetadata;
use serde_json::json;

/// Test handler implementation
#[derive(Debug)]
struct TestCommandHandler;

#[async_trait::async_trait]
impl crate::protocol::CommandHandler for TestCommandHandler {
    async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
        // Simple test implementation
        Ok(MCPResponse {
            protocol_version: "1.0".to_string(),
            message_id: message.id.clone(),
            status: ResponseStatus::Success,
            metadata: MessageMetadata::default(),
            payload: vec![json!({"response": "success"})],
            error_message: None,
        })
    }
}

#[tokio::test]
async fn test_adapter_initialization() {
    // Create a new adapter
    let adapter = MCPProtocolAdapter::new();

    // Should not be initialized yet
    assert!(!adapter.is_initialized().await);

    // Initialize it
    adapter.initialize().await.expect("should succeed");

    // Now it should be initialized
    assert!(adapter.is_initialized().await);

    // Second initialization should be ok (idempotent)
    let result = adapter.initialize().await;
    assert!(result.is_ok(), "Second initialization should be idempotent");
}

#[tokio::test]
async fn test_adapter_with_config() {
    // Create a custom config
    let config = ProtocolConfig {
        version: "2.0".to_string(),
        max_message_size: 2048,
        timeout_ms: 10000,
    };

    // Create and initialize with config
    let adapter = MCPProtocolAdapter::new();
    adapter
        .initialize_with_config(config.clone())
        .await
        .expect("should succeed");

    // Check the config was set
    let adapter_config = adapter.get_config().await;
    assert_eq!(adapter_config.version, "2.0");
    assert_eq!(adapter_config.max_message_size, 2048);
    assert_eq!(adapter_config.timeout_ms, 10000);
}

#[tokio::test]
async fn test_uninitialized_operations() {
    // Create a new adapter without initializing
    let adapter = MCPProtocolAdapter::new();

    // Trying to handle a message should fail
    let message = MCPMessage {
        id: MessageId("test-1".to_string()),
        type_: MessageType::Command,
        payload: json!({"command": "test"}),
        metadata: Some(json!({})),
        security: SecurityMetadata::default(),
        timestamp: Utc::now(),
        version: ProtocolVersion::default(),
        trace_id: None,
    };

    let err = adapter.handle_message(message).await.unwrap_err();
    assert!(err.to_string().contains("not initialized"));

    // Register handler should also fail
    let handler = Box::new(TestCommandHandler);
    let err = adapter
        .register_handler(MessageType::Command, handler)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not initialized"));
}

#[tokio::test]
async fn test_handler_registration() {
    // Create and initialize adapter
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.expect("should succeed");

    // Register a handler
    let handler = Box::new(TestCommandHandler);
    adapter
        .register_handler(MessageType::Command, handler)
        .await
        .expect("should succeed");

    // Create a test message for the registered handler
    let message = MCPMessage {
        id: MessageId("test-1".to_string()),
        type_: MessageType::Command,
        payload: json!({"command": "test"}),
        metadata: Some(json!({})),
        security: SecurityMetadata::default(),
        timestamp: Utc::now(),
        version: ProtocolVersion::default(),
        trace_id: None,
    };

    // Handle the message
    let response = adapter.handle_message(message).await.expect("should succeed");

    // Verify response
    assert_eq!(response.status, crate::types::ResponseStatus::Success);
    assert_eq!(response.message_id.0, "test-1");
}

#[tokio::test]
async fn test_factory_functions() {
    // Test simple creation
    let adapter1 = create_protocol_adapter();
    assert!(!adapter1.is_initialized().await);

    // Test creation with an existing protocol
    let config = ProtocolConfig::default();
    let protocol = MCPProtocolBase::new(config);
    let adapter2 = create_protocol_adapter_with_protocol(protocol);
    assert!(adapter2.is_initialized().await);

    // Test initialized creation
    let adapter3 = create_initialized_protocol_adapter().await.expect("should succeed");
    assert!(adapter3.is_initialized().await);

    // Test creation with config
    let custom_config = ProtocolConfig {
        version: "3.0".to_string(),
        max_message_size: 4096,
        timeout_ms: 15000,
    };
    let adapter4 = create_protocol_adapter_with_config(custom_config)
        .await
        .expect("should succeed");
    assert!(adapter4.is_initialized().await);

    let config4 = adapter4.get_config().await;
    assert_eq!(config4.version, "3.0");
}

#[tokio::test]
async fn test_state_management() {
    // Create and initialize adapter
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.expect("should succeed");

    // Default state should be null
    let state = adapter.get_state().await;
    assert!(state.is_null());

    // Set a new state
    let new_state = json!({"status": "connected", "client_id": "test-client"});
    adapter
        .set_state(new_state.clone())
        .await
        .expect("Failed to set state");

    // Get the state again
    let updated_state = adapter.get_state().await;
    assert_eq!(updated_state, new_state);
}

#[tokio::test]
async fn test_adapter_cloning() {
    // Create and initialize adapter
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await.expect("should succeed");

    // Register a handler
    let handler = Box::new(TestCommandHandler);
    adapter
        .register_handler(MessageType::Command, handler)
        .await
        .expect("should succeed");

    // Clone the adapter
    let adapter_clone = adapter.clone();

    // Both should be initialized
    assert!(adapter.is_initialized().await);
    assert!(adapter_clone.is_initialized().await);

    // Both should have the handler
    // BearDog handles security: // use crate::security::types::SecurityMetadata;
    use chrono::Utc;
    use crate::protocol::types::ProtocolVersion;
    
    let message = MCPMessage {
        id: MessageId("test-1".to_string()),
        type_: MessageType::Command,
        payload: json!({"command": "test"}),
        metadata: Some(json!({})),
        security: SecurityMetadata::default(),
        timestamp: Utc::now(),
        version: ProtocolVersion::default(),
        trace_id: None,
    };

    let response1 = adapter.handle_message(message.clone()).await.expect("should succeed");
    let response2 = adapter_clone.handle_message(message).await.expect("should succeed");

    assert_eq!(response1.status, response2.status);
}
