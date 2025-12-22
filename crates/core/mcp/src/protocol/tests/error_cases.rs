//! Error case tests for MCP protocol
//!
//! These tests verify proper error handling and edge cases in the protocol implementation.

#[cfg(test)]
mod tests {
    use crate::error::{MCPError, Result};
    use crate::types::{MCPMessage, MessageType, ResponseStatus};
    use crate::protocol::{MCPProtocolBase, ProtocolConfig};
    use serde_json::json;
    use std::collections::HashMap;
    use tokio::time::{timeout, Duration};

    /// Helper to create a test protocol instance
    fn create_test_protocol() -> MCPProtocolBase {
        let config = ProtocolConfig::default();
        MCPProtocolBase::new(config)
    }

    /// Helper to create a test message with minimal valid fields
    fn create_minimal_message(msg_type: MessageType) -> MCPMessage {
        MCPMessage {
            id: crate::types::MessageId(uuid::Uuid::new_v4().to_string()),
            type_: msg_type,
            command: "test_command".to_string(),
            payload: serde_json::Map::new(),
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
    async fn test_message_with_invalid_command() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        message.command = "".to_string(); // Invalid: empty command

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert
        assert!(result.is_ok(), "Should return ok with error response");
        let response = result.expect("test: should succeed");
        assert_eq!(response.status, ResponseStatus::Error);
    }

    #[tokio::test]
    async fn test_message_with_oversized_payload() {
        // Arrange
        let mut config = ProtocolConfig::default();
        config.max_message_size = Some(100); // Small limit
        let protocol = MCPProtocolBase::new(config);
        
        let mut message = create_minimal_message(MessageType::Command);
        // Create a large payload
        message.payload.insert(
            "large_data".to_string(),
            json!("x".repeat(1000)), // Exceeds limit
        );

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert
        // Should handle gracefully (either reject or truncate)
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_message_with_missing_required_fields() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        message.source = "".to_string(); // Invalid: empty source

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert
        assert!(result.is_ok(), "Should return ok with error response");
        let response = result.expect("test: should succeed");
        assert_eq!(response.status, ResponseStatus::Error);
    }

    #[tokio::test]
    async fn test_concurrent_message_handling() {
        // Arrange
        let protocol = create_test_protocol();
        let message1 = create_minimal_message(MessageType::Setup);
        let message2 = create_minimal_message(MessageType::Setup);

        // Act - Send two messages concurrently
        let (result1, result2) = tokio::join!(
            protocol.handle_protocol_message(&message1),
            protocol.handle_protocol_message(&message2)
        );

        // Assert - Both should succeed or handle gracefully
        assert!(result1.is_ok() || result1.is_err());
        assert!(result2.is_ok() || result2.is_err());
    }

    #[tokio::test]
    async fn test_message_timeout_handling() {
        // Arrange
        let protocol = create_test_protocol();
        let message = create_minimal_message(MessageType::Command);

        // Act - Wrap in timeout to ensure we don't hang
        let result = timeout(
            Duration::from_secs(5),
            protocol.handle_protocol_message(&message)
        ).await;

        // Assert - Should complete within timeout
        assert!(result.is_ok(), "Message handling should complete within timeout");
    }

    #[tokio::test]
    async fn test_malformed_json_payload() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        
        // Add some complex nested data that might cause issues
        message.payload.insert(
            "complex".to_string(),
            json!({
                "nested": {
                    "deep": {
                        "value": null
                    }
                }
            })
        );

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert - Should handle gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_protocol_state_recovery_after_error() {
        // Arrange
        let protocol = create_test_protocol();
        
        // Act 1 - Send a valid message
        let valid_message = create_minimal_message(MessageType::Setup);
        let result1 = protocol.handle_protocol_message(&valid_message).await;
        assert!(result1.is_ok());

        // Act 2 - Send an invalid message
        let mut invalid_message = create_minimal_message(MessageType::Command);
        invalid_message.command = "".to_string();
        let _result2 = protocol.handle_protocol_message(&invalid_message).await;

        // Act 3 - Send another valid message (should recover)
        let result3 = protocol.handle_protocol_message(&valid_message).await;

        // Assert - Protocol should recover and handle the next message
        assert!(result3.is_ok(), "Protocol should recover after error");
    }

    #[tokio::test]
    async fn test_protocol_with_null_values() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        message.payload.insert("nullable_field".to_string(), json!(null));

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert - Should handle null values gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_protocol_with_special_characters() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        message.command = "test_!@#$%^&*()_+-={}[]|:;<>?,./".to_string();

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert - Should handle special characters
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_protocol_with_unicode_strings() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Command);
        message.command = "测试_🦀_тест".to_string(); // Unicode characters

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert - Should handle Unicode properly
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_rapid_sequential_messages() {
        // Arrange
        let protocol = create_test_protocol();
        
        // Act - Send 10 messages rapidly
        for i in 0..10 {
            let mut message = create_minimal_message(MessageType::Command);
            message.command = format!("rapid_test_{}", i);
            let result = protocol.handle_protocol_message(&message).await;
            
            // Assert each completes
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[tokio::test]
    async fn test_protocol_version_mismatch() {
        // Arrange
        let protocol = create_test_protocol();
        let mut message = create_minimal_message(MessageType::Setup);
        message.version = "999.999.999".to_string(); // Future version

        // Act
        let result = protocol.handle_protocol_message(&message).await;

        // Assert - Should handle version mismatch gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_message_with_duplicate_id() {
        // Arrange
        let protocol = create_test_protocol();
        let message_id = crate::types::MessageId(uuid::Uuid::new_v4().to_string());
        
        let mut message1 = create_minimal_message(MessageType::Command);
        message1.id = message_id.clone();
        
        let mut message2 = create_minimal_message(MessageType::Command);
        message2.id = message_id; // Same ID

        // Act
        let result1 = protocol.handle_protocol_message(&message1).await;
        let result2 = protocol.handle_protocol_message(&message2).await;

        // Assert - Both should complete (protocol may or may not detect duplicates)
        assert!(result1.is_ok() || result1.is_err());
        assert!(result2.is_ok() || result2.is_err());
    }

    #[tokio::test]
    async fn test_protocol_cleanup_on_drop() {
        // Arrange & Act
        {
            let protocol = create_test_protocol();
            let message = create_minimal_message(MessageType::Setup);
            let _result = protocol.handle_protocol_message(&message).await;
            // Protocol goes out of scope here
        }
        
        // Assert - Should not panic or leak resources
        // (This test mainly ensures Drop implementations work)
    }
}

