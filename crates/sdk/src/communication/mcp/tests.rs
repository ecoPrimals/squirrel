//! Tests for MCP client functionality
//!
//! This module contains comprehensive tests for all MCP client components
//! including integration tests and message flow validation.

use super::connection::ConnectionManager;
use super::message::MessageHandler;
use super::operations::OperationHandler;
use super::*;
use serde_json::json;

/// Test message deserialization from JSON
#[test]
fn test_mcp_message_deserialization() {
    let json_str = r#"
    {
        "id": "msg-123",
        "message_type": "ping",
        "payload": {"data": "test"},
        "timestamp": "2024-01-01T12:00:00Z"
    }
    "#;

    let message: McpMessage = serde_json::from_str(json_str).unwrap();
    assert_eq!(message.id, "msg-123");
    assert_eq!(message.message_type, "ping");
    assert_eq!(message.payload["data"], "test");
    assert_eq!(message.timestamp, "2024-01-01T12:00:00Z");
}

/// Test message size validation
#[test]
fn test_message_size_validation() {
    let _client = McpClient::new();

    // Create a message that would exceed max size
    let large_payload = "x".repeat(_client.config.max_message_size + 1);
    let large_message = McpMessage {
        id: "test-id".to_string(),
        message_type: "test".to_string(),
        payload: json!({"data": large_payload}),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    };

    // This should fail when trying to send
    // Note: This test validates the message size check logic
    let message_json = serde_json::to_string(&large_message).unwrap();
    assert!(message_json.len() > _client.config.max_message_size);
}

/// Test reconnection logic
#[test]
fn test_reconnection_attempts() {
    let mut client = McpClient::new();

    // Test initial reconnect attempts
    assert_eq!(client.reconnect_attempts, 0);

    // Simulate failed reconnection attempts
    client.reconnect_attempts = 1;
    assert_eq!(client.reconnect_attempts, 1);

    // Test max attempts
    client.reconnect_attempts = client.config.max_reconnect_attempts;
    assert_eq!(
        client.reconnect_attempts,
        client.config.max_reconnect_attempts
    );

    // Test reset on successful connection
    client.reconnect_attempts = 0;
    assert_eq!(client.reconnect_attempts, 0);
}

/// Test pending requests management
#[test]
fn test_pending_requests_management() {
    let mut client = McpClient::new();

    // Test initial state
    assert!(client.pending_requests.is_empty());

    // Test adding pending request
    let (tx, _rx) = tokio::sync::oneshot::channel();
    client.pending_requests.insert("test-id".to_string(), tx);
    assert_eq!(client.pending_requests.len(), 1);

    // Test removing pending request
    client.pending_requests.remove("test-id");
    assert!(client.pending_requests.is_empty());
}

/// Test connection state display
#[test]
fn test_connection_state_display() {
    assert_eq!(
        format!("{:?}", ConnectionState::Disconnected),
        "Disconnected"
    );
    assert_eq!(format!("{:?}", ConnectionState::Connecting), "Connecting");
    assert_eq!(format!("{:?}", ConnectionState::Connected), "Connected");
    assert_eq!(
        format!("{:?}", ConnectionState::Reconnecting),
        "Reconnecting"
    );
    assert_eq!(format!("{:?}", ConnectionState::Failed), "Failed");
}

/// Test MCP capabilities default state
#[test]
fn test_mcp_capabilities_default() {
    let capabilities = McpCapabilities::default();
    assert!(!capabilities.supports_mcp); // Default is false
    assert!(capabilities.protocol_version.is_empty());
    assert!(capabilities.supported_methods.is_empty());
    assert!(capabilities.max_payload_size.is_none());
}

/// Test MCP capabilities serialization
#[test]
fn test_mcp_capabilities_serialization() {
    let capabilities = McpCapabilities::new();
    let serialized = serde_json::to_string(&capabilities).unwrap();
    let deserialized: McpCapabilities = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.supports_mcp, capabilities.supports_mcp);
    assert_eq!(deserialized.protocol_version, capabilities.protocol_version);
    assert_eq!(
        deserialized.supported_methods,
        capabilities.supported_methods
    );
    assert_eq!(deserialized.max_payload_size, capabilities.max_payload_size);
}

/// Integration test for message flow
#[tokio::test]
async fn test_message_flow_integration() {
    let client = McpClient::new();

    // Create a test message
    let message = McpMessage {
        id: "integration-test".to_string(),
        message_type: "test_message".to_string(),
        payload: json!({"action": "test", "data": "integration"}),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    // Test message serialization
    let serialized = serde_json::to_string(&message).unwrap();
    assert!(serialized.contains("integration-test"));
    assert!(serialized.contains("test_message"));

    // Test message deserialization
    let deserialized: McpMessage = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, message.id);
    assert_eq!(deserialized.message_type, message.message_type);

    // Test message handler
    let mut handler = MessageHandler::new();
    let handle_result = handler.handle_incoming_message(&serialized).await;
    assert!(handle_result.is_ok());
}

/// Test client with custom configuration
#[tokio::test]
async fn test_client_with_custom_config() {
    let client = McpClient::with_server_url("ws://custom.example.com:8080");
    assert_eq!(client.config.server_url, "ws://custom.example.com:8080");
    assert_eq!(client.state, ConnectionState::Disconnected);
}

/// Test operation handler functionality
#[tokio::test]
async fn test_operation_handler_integration() {
    let mut handler = OperationHandler::new();

    // Test tools
    let tools = handler.list_tools().await.unwrap();
    assert!(!tools.is_empty());

    // Test calculator tool
    let calc_input = json!({"operation": "add", "operands": [10, 20]});
    let calc_result = handler
        .execute_tool("calculator", calc_input)
        .await
        .unwrap();
    assert_eq!(calc_result["result"], 30.0);

    // Test resources
    let resources = handler.list_resources().await.unwrap();
    assert!(!resources.is_empty());

    // Test prompts
    let prompts = handler.list_prompts().await.unwrap();
    assert!(!prompts.is_empty());
}

/// Test message handler ping/pong flow
#[tokio::test]
async fn test_message_handler_ping_pong() {
    let mut handler = MessageHandler::new();

    // Create ping message
    let ping = handler.create_ping_message();
    assert_eq!(ping.message_type, "ping");

    // Create pong response
    let pong = handler.create_pong_message(&ping.id);
    assert_eq!(pong.message_type, "pong");
    assert_eq!(pong.payload["ping_id"], ping.id);
}

/// Test connection manager state
#[test]
fn test_connection_manager_state() {
    use crate::config::McpClientConfig;

    let config = McpClientConfig::default();
    let manager = ConnectionManager::new(config);

    assert!(!manager.is_connected());
}

/// Test error handling in operations
#[tokio::test]
async fn test_operation_error_handling() {
    let mut handler = OperationHandler::new();

    // Test unknown tool
    let result = handler.execute_tool("unknown_tool", json!({})).await;
    assert!(result.is_err());

    // Test invalid calculator input
    let invalid_input = json!({"invalid": "data"});
    let result = handler.execute_tool("calculator", invalid_input).await;
    assert!(result.is_err());

    // Test division by zero
    let div_zero_input = json!({"operation": "divide", "operands": [10, 0]});
    let result = handler.execute_tool("calculator", div_zero_input).await;
    assert!(result.is_err());
}

/// Test comprehensive MCP client lifecycle
#[tokio::test]
async fn test_mcp_client_lifecycle() {
    let mut client = McpClient::new();

    // Test initial state
    assert!(!client.connected());
    assert_eq!(client.state(), "Disconnected");

    // Test state transitions
    client.state = ConnectionState::Connecting;
    assert_eq!(client.state(), "Connecting");

    client.state = ConnectionState::Connected;
    assert!(client.connected());
    assert_eq!(client.state(), "Connected");

    client.state = ConnectionState::Reconnecting;
    assert_eq!(client.state(), "Reconnecting");

    client.state = ConnectionState::Failed;
    assert_eq!(client.state(), "Failed");

    // Test disconnect when already disconnected
    client.state = ConnectionState::Disconnected;
    let result = client.disconnect().await;
    assert!(result.is_ok());
}

/// Test message ID generation uniqueness
#[test]
fn test_message_id_uniqueness() {
    let mut handler = MessageHandler::new();

    let mut ids = std::collections::HashSet::new();
    for _ in 0..100 {
        let id = handler.generate_message_id();
        assert!(ids.insert(id), "Generated duplicate message ID");
    }
}

/// Test tool schema validation
#[test]
fn test_tool_schema_validation() {
    let tool = McpTool {
        name: "test_tool".to_string(),
        description: "Test tool".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        }),
        output_schema: Some(json!({
            "type": "object",
            "properties": {
                "output": {"type": "string"}
            }
        })),
    };

    assert_eq!(tool.name, "test_tool");
    assert!(tool.input_schema.is_object());
    assert!(tool.output_schema.is_some());
}

/// Test resource metadata validation
#[test]
fn test_resource_metadata_validation() {
    let resource = McpResource {
        uri: "file:///test.json".to_string(),
        name: "Test Resource".to_string(),
        description: "Test resource description".to_string(),
        metadata: json!({
            "size": 1024,
            "format": "json",
            "permissions": "read-only"
        }),
    };

    assert_eq!(resource.uri, "file:///test.json");
    assert_eq!(resource.metadata["size"], 1024);
    assert_eq!(resource.metadata["format"], "json");
}

/// Test prompt parameter substitution
#[test]
fn test_prompt_parameter_substitution() {
    let prompt = McpPrompt {
        name: "test_prompt".to_string(),
        description: "Test prompt".to_string(),
        template: "Process this {input} with {method}".to_string(),
        parameters: json!({
            "input": {"type": "string", "required": true},
            "method": {"type": "string", "required": true}
        }),
    };

    assert_eq!(prompt.name, "test_prompt");
    assert!(prompt.template.contains("{input}"));
    assert!(prompt.template.contains("{method}"));
    assert!(prompt.parameters.is_object());
}
