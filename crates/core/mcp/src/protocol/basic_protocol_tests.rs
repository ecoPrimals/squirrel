// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Unit tests for MCP protocol handlers
//!
//! These tests verify the core MCP protocol message handling logic.

#[cfg(test)]
mod tests {
    use crate::error::{MCPError, Result};
    use crate::types::{MCPMessage, MCPResponse, MessageType, ResponseStatus};
    use crate::protocol::{MCPProtocolBase, ProtocolConfig};
    use serde_json::json;
    use std::collections::HashMap;

    /// Helper to create a test protocol instance
    fn create_test_protocol() -> MCPProtocolBase {
        let config = ProtocolConfig::default();
        MCPProtocolBase::new(config)
    }

    /// Helper to create a test message
    fn create_test_message(msg_type: MessageType, payload: serde_json::Value) -> MCPMessage {
        MCPMessage {
            id: crate::types::MessageId(uuid::Uuid::new_v4().to_string()),
            type_: msg_type,
            command: "test_command".to_string(),
            payload: payload.as_object().expect("test: should succeed").clone(),
            source: "test_source".to_string(),
            target: "test_target".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            version: "1.0.0".to_string(),
            session_id: Some("test_session".to_string()),
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_protocol_initialization() {
        let protocol = create_test_protocol();
        assert_eq!(protocol.get_config().version, "1.0.0");
    }

    #[tokio::test]
    async fn test_handle_setup_message() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Setup, json!({}));

        let result = protocol.handle_protocol_message(&message).await;
        assert!(result.is_ok());

        let response = result.expect("test: should succeed");
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.protocol_version, "1.0.0");
    }

    #[tokio::test]
    async fn test_handle_message_without_handler() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Command, json!({}));

        let result = protocol.handle_protocol_message(&message).await;
        assert!(result.is_err());
        
        match result {
            Err(MCPError::Protocol(_)) => {}, // Expected
            _ => unreachable!("Expected Protocol error"),
        }
    }

    #[tokio::test]
    async fn test_has_handler_for_returns_false_initially() {
        let protocol = create_test_protocol();
        assert!(!protocol.has_handler_for(&MessageType::Command));
        assert!(!protocol.has_handler_for(&MessageType::Event));
        assert!(!protocol.has_handler_for(&MessageType::Response));
    }

    #[tokio::test]
    async fn test_protocol_config_default_values() {
        let config = ProtocolConfig::default();
        
        assert_eq!(config.version, "1.0.0");
        assert!(config.max_message_size > 0);
        assert!(config.timeout_ms > 0);
    }

    #[tokio::test]
    async fn test_protocol_config_custom_values() {
        let config = ProtocolConfig {
            version: "2.0.0".to_string(),
            max_message_size: 2048,
            timeout_ms: 5000,
            ..Default::default()
        };
        
        assert_eq!(config.version, "2.0.0");
        assert_eq!(config.max_message_size, 2048);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[tokio::test]
    async fn test_message_type_display() {
        assert_eq!(MessageType::Command.to_string(), "Command");
        assert_eq!(MessageType::Event.to_string(), "Event");
        assert_eq!(MessageType::Response.to_string(), "Response");
        assert_eq!(MessageType::Setup.to_string(), "Setup");
    }

    #[tokio::test]
    async fn test_response_status_variants() {
        let success = ResponseStatus::Success;
        let error = ResponseStatus::Error;
        
        assert!(matches!(success, ResponseStatus::Success));
        assert!(matches!(error, ResponseStatus::Error));
    }

    #[tokio::test]
    async fn test_message_id_generation() {
        let msg1 = create_test_message(MessageType::Command, json!({}));
        let msg2 = create_test_message(MessageType::Command, json!({}));
        
        assert_ne!(msg1.id.0, msg2.id.0, "Message IDs should be unique");
    }

    #[tokio::test]
    async fn test_message_with_session_id() {
        let message = create_test_message(MessageType::Command, json!({}));
        
        assert!(message.session_id.is_some());
        assert_eq!(message.session_id.expect("test: should succeed"), "test_session");
    }

    #[tokio::test]
    async fn test_message_with_correlation_id() {
        let mut message = create_test_message(MessageType::Response, json!({}));
        message.correlation_id = Some("parent_message_id".to_string());
        
        assert!(message.correlation_id.is_some());
        assert_eq!(message.correlation_id.expect("test: should succeed"), "parent_message_id");
    }

    #[tokio::test]
    async fn test_message_metadata() {
        let mut message = create_test_message(MessageType::Event, json!({}));
        message.metadata.insert("key1".to_string(), "value1".to_string());
        message.metadata.insert("key2".to_string(), "value2".to_string());
        
        assert_eq!(message.metadata.len(), 2);
        assert_eq!(message.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(message.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[tokio::test]
    async fn test_protocol_version_in_response() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Setup, json!({}));

        let response = protocol.handle_protocol_message(&message).await.expect("test: should succeed");
        
        assert_eq!(response.protocol_version, "1.0.0");
        assert_eq!(response.message_id, message.id.0);
    }

    #[tokio::test]
    async fn test_setup_response_includes_config() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Setup, json!({}));

        let response = protocol.handle_protocol_message(&message).await.expect("test: should succeed");
        
        assert_eq!(response.status, ResponseStatus::Success);
        assert!(!response.payload.is_empty());
        
        // Verify payload contains protocol configuration
        let payload = &response.payload[0];
        assert!(payload.get("protocol_version").is_some());
        assert!(payload.get("max_message_size").is_some());
        assert!(payload.get("timeout_ms").is_some());
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Command, json!({}));

        let result = protocol.handle_protocol_message(&message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_message_timestamp_is_recent() {
        let message = create_test_message(MessageType::Command, json!({}));
        let now = chrono::Utc::now().timestamp();
        
        // Message timestamp should be within 1 second of now
        let diff = (now - message.timestamp).abs();
        assert!(diff <= 1, "Message timestamp should be recent");
    }

    #[tokio::test]
    async fn test_protocol_handles_empty_payload() {
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Setup, json!({}));

        let result = protocol.handle_protocol_message(&message).await;
        assert!(result.is_ok(), "Protocol should handle empty payload");
    }

    #[tokio::test]
    async fn test_protocol_handles_complex_payload() {
        let protocol = create_test_protocol();
        let complex_payload = json!({
            "nested": {
                "data": "value",
                "array": [1, 2, 3],
                "boolean": true
            }
        });
        let message = create_test_message(MessageType::Setup, complex_payload);

        let result = protocol.handle_protocol_message(&message).await;
        assert!(result.is_ok(), "Protocol should handle complex payload");
    }
}

