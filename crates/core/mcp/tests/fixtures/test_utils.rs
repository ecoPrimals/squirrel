//! Test Utilities
//!
//! Common utilities and helpers for MCP testing.

use serde_json::json;
use squirrel_mcp::MCPMessage;
use chrono::Utc;

/// Create a test MCP message
pub fn create_test_message(method: &str) -> MCPMessage {
    let mut msg = MCPMessage::default();
    msg.method = method.to_string();
    msg.params = Some(json!({
        "test": true,
        "timestamp": Utc::now().to_rfc3339()
    }));
    msg
}

/// Create a test message with specific params
pub fn create_test_message_with_params(method: &str, params: serde_json::Value) -> MCPMessage {
    let mut msg = MCPMessage::default();
    msg.method = method.to_string();
    msg.params = Some(params);
    msg
}

/// Create a test response message
pub fn create_test_response(request_id: &str, result: serde_json::Value) -> MCPMessage {
    let mut msg = MCPMessage::default();
    msg.id = squirrel_mcp::protocol::MessageId(request_id.to_string());
    msg.result = Some(result);
    msg
}

/// Create a test error response
pub fn create_test_error_response(request_id: &str, error_msg: &str) -> MCPMessage {
    let mut msg = MCPMessage::default();
    msg.id = squirrel_mcp::protocol::MessageId(request_id.to_string());
    msg.error = Some(json!({
        "code": -32600,
        "message": error_msg
    }));
    msg
}

/// Assert that two messages are equivalent
pub fn assert_messages_equal(msg1: &MCPMessage, msg2: &MCPMessage) {
    assert_eq!(msg1.id.0, msg2.id.0);
    assert_eq!(msg1.method, msg2.method);
    assert_eq!(msg1.params, msg2.params);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_message() {
        let msg = create_test_message("test_method");
        assert_eq!(msg.method, "test_method");
        assert!(msg.params.is_some());
    }

    #[test]
    fn test_create_test_response() {
        let response = create_test_response("123", json!({"status": "ok"}));
        assert_eq!(response.id.0, "123");
        assert!(response.result.is_some());
    }

    #[test]
    fn test_create_test_error_response() {
        let response = create_test_error_response("456", "Test error");
        assert_eq!(response.id.0, "456");
        assert!(response.error.is_some());
    }
}

