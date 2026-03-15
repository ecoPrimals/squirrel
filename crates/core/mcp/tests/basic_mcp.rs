// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Basic MCP Core Integration Tests
//!
//! This test file verifies that the core MCP functionality is working correctly
//! after the rebuild and simplification process.

use serde_json::json;
use squirrel_mcp::error::connection::ConnectionError;
use squirrel_mcp::protocol::websocket::{WebSocketClient, WebSocketServer, WebSocketTransport};
use squirrel_mcp::transport::frame::{DefaultFrameCodec, Frame, FrameTransport, MessageCodec};
use squirrel_mcp::utils::{JsonUtils, StringUtils, ValidationUtils};
use squirrel_mcp::{init, MCPError, MCPMessage, WebSocketConfig, VERSION};

#[tokio::test]
async fn test_core_mcp_init() {
    // Test that the core MCP system can be initialized
    let result = init();
    assert!(result.is_ok());

    // Test that the version is available
    assert!(!VERSION.is_empty());
    println!("MCP Core Version: {}", VERSION);
}

#[tokio::test]
async fn test_error_handling() {
    // Test error handling and result types
    let error = MCPError::UnsupportedOperation("Test operation".to_string());
    assert!(error.is_recoverable());

    let error_code = error.error_code();
    assert!(!error_code.is_empty());

    let error_category = error.category_str();
    assert!(!error_category.is_empty());

    // Test a non-recoverable error too
    let non_recoverable_error = MCPError::General("Test error".to_string());
    assert!(!non_recoverable_error.is_recoverable());
}

#[tokio::test]
async fn test_protocol_message_creation() {
    // Test MCP message creation
    let message = MCPMessage::default();
    // MessageId::new() generates a UUID, so it should not be empty
    assert!(!message.id.0.is_empty());
    assert!(!message.id.is_empty());

    // Test message serialization
    let json_result = serde_json::to_string(&message);
    assert!(json_result.is_ok());

    // Test message deserialization
    let json_str = json_result.unwrap();
    let deserialized: Result<MCPMessage, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    // Test that the deserialized message has the same ID
    let deserialized_message = deserialized.unwrap();
    assert_eq!(message.id.0, deserialized_message.id.0);
}

#[tokio::test]
async fn test_websocket_config() {
    // Test WebSocket configuration
    let config = WebSocketConfig::default();
    assert_eq!(config.bind_address, "127.0.0.1");
    assert_eq!(config.port, 8080);
    assert_eq!(config.max_connections, 100);
    assert!(config.enable_compression);
}

#[tokio::test]
async fn test_websocket_server_creation() {
    // Test WebSocket server creation
    let config = WebSocketConfig::default();
    let server = WebSocketServer::new(config);

    // Test server events subscription
    let _receiver = server.subscribe_events();

    // Test connection listing (should be empty initially)
    let connections = server.get_connections().await;
    assert!(connections.is_empty());
}

#[tokio::test]
async fn test_websocket_client_creation() {
    // Test WebSocket client creation
    let config = WebSocketConfig::default();
    let client = WebSocketClient::new(config);

    // Test connection info (should be None initially)
    let connection_info = client.get_connection_info().await;
    assert!(connection_info.is_none());
}

#[tokio::test]
async fn test_websocket_transport_creation() {
    // Test WebSocket transport creation
    let config = WebSocketConfig::default();
    let _transport = WebSocketTransport::new(config);

    // Test transport creation in different modes
    let config2 = WebSocketConfig::default();
    let _server_transport = WebSocketTransport::server(config2);

    let config3 = WebSocketConfig::default();
    let _client_transport = WebSocketTransport::client(config3);
}

#[tokio::test]
async fn test_utils_functions() {
    // Test JSON utilities
    let test_data = json!({"test": "value"});
    let json_str = JsonUtils::to_string(&test_data);
    assert!(json_str.is_ok());

    let parsed: Result<serde_json::Value, _> = JsonUtils::from_string(&json_str.unwrap());
    assert!(parsed.is_ok());

    // Test string utilities
    let test_str = "hello world";
    let title_case = StringUtils::to_title_case(test_str);
    assert_eq!(title_case, "Hello World");

    // Test validation utilities
    let valid_email = "test@example.com";
    assert!(ValidationUtils::is_valid_email(valid_email));

    let invalid_email = "invalid-email";
    assert!(!ValidationUtils::is_valid_email(invalid_email));

    let valid_url = "https://example.com";
    assert!(ValidationUtils::is_valid_url(valid_url));

    let invalid_url = "not-a-url";
    assert!(!ValidationUtils::is_valid_url(invalid_url));
}

#[tokio::test]
async fn test_frame_transport() {
    // Test frame transport creation
    let stream = tokio::io::empty();
    let sink = tokio::io::sink();
    let codec = DefaultFrameCodec;

    let _transport = FrameTransport::new(stream, sink, codec);
}

#[tokio::test]
async fn test_message_codec() {
    // Test message codec creation
    let codec = MessageCodec::new();

    // Test frame creation
    let test_data = b"test message";
    let frame = Frame::from_vec(test_data.to_vec());

    // Test that the frame contains the data
    assert_eq!(frame.payload.len(), test_data.len());
}

#[tokio::test]
async fn test_comprehensive_mcp_workflow() {
    // Test a complete MCP workflow with available functionality
    println!("Testing comprehensive MCP workflow...");

    // 1. Initialize MCP Core
    let init_result = init();
    assert!(init_result.is_ok());

    // 2. Create WebSocket configuration
    let config = WebSocketConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8081,
        max_message_size: 1024 * 1024,
        connection_timeout: std::time::Duration::from_secs(30),
        ping_interval: std::time::Duration::from_secs(30),
        pong_timeout: std::time::Duration::from_secs(5),
        max_connections: 50,
        enable_compression: true,
        subprotocol: Some("mcp".to_string()),
    };

    // 3. Create WebSocket server
    let _server = WebSocketServer::new(config.clone());

    // 4. Create WebSocket client
    let _client = WebSocketClient::new(config.clone());

    // 5. Create transport layer
    let _transport = WebSocketTransport::new(config);

    // 6. Test message creation and serialization
    let message = MCPMessage::default();
    let json_result = serde_json::to_string(&message);
    assert!(json_result.is_ok());

    // 7. Test error handling with recoverable error
    let connection_error = ConnectionError::Timeout(5000); // 5 second timeout
    let error = MCPError::Connection(connection_error);
    assert!(error.is_recoverable());

    println!("MCP workflow test completed successfully!");
}
