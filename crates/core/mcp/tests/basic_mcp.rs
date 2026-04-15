// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Basic MCP integration tests.
//!
//! WebSocket transport was removed per Tower Atomic policy — mesh provides
//! WebSocket when needed.  These tests exercise the core MCP API (init,
//! messages, errors, frame transport, utilities) over the retained transport
//! abstractions (UDS / frame codec).

#![expect(
    clippy::expect_used,
    reason = "Integration tests use expect after is_ok checks"
)]

use serde_json::json;
use squirrel_mcp::error::connection::ConnectionError;
use squirrel_mcp::transport::frame::{DefaultFrameCodec, Frame, FrameTransport, MessageCodec};
use squirrel_mcp::utils::{JsonUtils, StringUtils, ValidationUtils};
use squirrel_mcp::{MCPError, MCPMessage, VERSION, init};

#[tokio::test]
async fn test_core_mcp_init() {
    let result = init();
    assert!(result.is_ok());
    assert!(!VERSION.is_empty());
}

#[tokio::test]
async fn test_error_handling() {
    let error = MCPError::UnsupportedOperation("Test operation".to_string());
    assert!(error.is_recoverable());
    assert!(!error.error_code().is_empty());
    assert!(!error.category_str().is_empty());

    let non_recoverable_error = MCPError::General("Test error".to_string());
    assert!(!non_recoverable_error.is_recoverable());
}

#[tokio::test]
async fn test_protocol_message_creation() {
    let message = MCPMessage::default();
    assert!(!message.id.0.is_empty());
    assert!(!message.id.is_empty());

    let json_result = serde_json::to_string(&message);
    assert!(json_result.is_ok());

    let json_str = json_result.expect("serialization should succeed");
    let deserialized: Result<MCPMessage, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok());

    let deserialized_message = deserialized.expect("deserialization should succeed");
    assert_eq!(message.id.0, deserialized_message.id.0);
}

#[tokio::test]
async fn test_utils_functions() {
    let test_data = json!({"test": "value"});
    let json_str = JsonUtils::to_string(&test_data);
    assert!(json_str.is_ok());

    let parsed: Result<serde_json::Value, _> =
        JsonUtils::from_string(&json_str.expect("should succeed"));
    assert!(parsed.is_ok());

    let test_str = "hello world";
    let title_case = StringUtils::to_title_case(test_str);
    assert_eq!(title_case, "Hello World");

    assert!(ValidationUtils::is_valid_email("test@example.com"));
    assert!(!ValidationUtils::is_valid_email("invalid-email"));
    assert!(ValidationUtils::is_valid_url("https://example.com"));
    assert!(!ValidationUtils::is_valid_url("not-a-url"));
}

#[tokio::test]
async fn test_frame_transport() {
    let stream = tokio::io::empty();
    let sink = tokio::io::sink();
    let codec = DefaultFrameCodec;
    let _transport = FrameTransport::new(stream, sink, codec);
}

#[tokio::test]
async fn test_message_codec() {
    let _codec = MessageCodec::new();

    let test_data = b"test message";
    let frame = Frame::from_vec(test_data.to_vec());
    assert_eq!(frame.payload.len(), test_data.len());
}

#[tokio::test]
async fn test_comprehensive_mcp_workflow() {
    let init_result = init();
    assert!(init_result.is_ok());

    let message = MCPMessage::default();
    let json_result = serde_json::to_string(&message);
    assert!(json_result.is_ok());

    let connection_error = ConnectionError::Timeout(5000);
    let error = MCPError::Connection(connection_error);
    assert!(error.is_recoverable());

    let stream = tokio::io::empty();
    let sink = tokio::io::sink();
    let _transport = FrameTransport::new(stream, sink, DefaultFrameCodec);
}

#[tokio::test]
async fn test_connection_error_variants() {
    let timeout = ConnectionError::Timeout(3000);
    let mcp_err = MCPError::Connection(timeout);
    assert!(mcp_err.is_recoverable());

    let refused = ConnectionError::Refused;
    let mcp_err = MCPError::Connection(refused);
    assert!(!mcp_err.is_recoverable());
}

#[tokio::test]
async fn test_frame_roundtrip() {
    let payload = b"JSON-RPC 2.0 request payload";
    let frame = Frame::from_vec(payload.to_vec());
    assert_eq!(&*frame.payload, payload);
}
