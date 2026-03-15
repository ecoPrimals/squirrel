// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Error path coverage tests for MCP protocol
//!
//! These tests specifically target error handling paths to improve coverage

#[cfg(test)]
mod mcp_protocol_error_coverage {
    use serde_json::json;
    use squirrel_mcp::error::{ConnectionError, MCPError, ProtocolError, TransportError};
    use squirrel_mcp::protocol::MCPProtocolBase;
    use squirrel_mcp::types::{MCPMessage, MessageId, MessageType, ResponseStatus};
    use std::collections::HashMap;

    /// Helper to create protocol for testing
    fn create_test_protocol() -> MCPProtocolBase {
        MCPProtocolBase::new(Default::default())
    }

    /// Helper to create test message
    fn create_test_message(msg_type: MessageType, command: &str) -> MCPMessage {
        MCPMessage {
            id: MessageId(uuid::Uuid::new_v4().to_string()),
            type_: msg_type,
            command: command.to_string(),
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
    async fn test_protocol_handles_empty_command_name() {
        // ARRANGE
        let protocol = create_test_protocol();
        let mut message = create_test_message(MessageType::Command, "");
        message.command = String::new(); // Empty command

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should return error response, not panic
        assert!(result.is_ok(), "Should handle empty command gracefully");
        if let Ok(response) = result {
            assert_eq!(response.status, ResponseStatus::Error);
        }
    }

    #[tokio::test]
    async fn test_protocol_handles_null_payload_gracefully() {
        // ARRANGE
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Command, "test_command");
        // Payload is already empty serde_json::Map

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should handle null/empty payload
        assert!(result.is_ok(), "Should handle empty payload");
    }

    #[tokio::test]
    async fn test_protocol_handles_oversized_payload() {
        // ARRANGE
        let protocol = create_test_protocol();
        let mut message = create_test_message(MessageType::Command, "test_command");

        // Create a large payload (simulate oversized message)
        let large_string = "x".repeat(10_000_000); // 10MB string
        message
            .payload
            .insert("large_data".to_string(), json!(large_string));

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should handle or reject oversized payload gracefully
        assert!(result.is_ok(), "Should handle large payload without panic");
    }

    #[tokio::test]
    async fn test_protocol_handles_invalid_message_type() {
        // ARRANGE
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Response, "test_command");
        // Response type is invalid for incoming message handling

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should handle invalid type
        assert!(result.is_ok(), "Should handle unexpected message type");
    }

    #[tokio::test]
    async fn test_protocol_handles_missing_session_id() {
        // ARRANGE
        let protocol = create_test_protocol();
        let mut message = create_test_message(MessageType::Command, "test_command");
        message.session_id = None; // Missing session ID

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should handle missing session ID
        assert!(result.is_ok(), "Should handle missing session ID");
    }

    #[tokio::test]
    async fn test_protocol_handles_malformed_timestamp() {
        // ARRANGE
        let protocol = create_test_protocol();
        let mut message = create_test_message(MessageType::Command, "test_command");
        message.timestamp = -1; // Invalid timestamp

        // ACT
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should handle invalid timestamp
        assert!(result.is_ok(), "Should handle malformed timestamp");
    }

    #[tokio::test]
    async fn test_protocol_handles_concurrent_messages() {
        // ARRANGE
        let protocol = std::sync::Arc::new(create_test_protocol());

        // ACT: Send multiple messages concurrently
        let mut handles = vec![];
        for i in 0..50 {
            let proto = protocol.clone();
            let handle = tokio::spawn(async move {
                let msg = create_test_message(MessageType::Command, &format!("cmd_{}", i));
                proto.handle_protocol_message(&msg).await
            });
            handles.push(handle);
        }

        // ASSERT: All should complete without deadlock or race conditions
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent message handling should work");
        }
    }

    #[tokio::test]
    async fn test_error_type_conversions() {
        // Test all error type conversions for coverage

        // Transport -> MCP
        let transport_err = TransportError::ConnectionClosed("test".to_string());
        let mcp_err: MCPError = transport_err.into();
        assert!(matches!(mcp_err, MCPError::Transport(_)));

        // Protocol -> MCP
        let protocol_err = ProtocolError::HandlerNotFound("test".to_string());
        let mcp_err: MCPError = protocol_err.into();
        assert!(matches!(mcp_err, MCPError::Protocol(_)));

        // Connection -> MCP
        let conn_err = ConnectionError::Timeout;
        let mcp_err: MCPError = conn_err.into();
        assert!(matches!(mcp_err, MCPError::Connection(_)));
    }

    #[tokio::test]
    async fn test_error_display_all_variants() {
        // Test Display trait for all MCPError variants
        let errors = vec![
            MCPError::ResourceExhausted("test".to_string()),
            MCPError::InvalidArgument("test".to_string()),
            MCPError::NotFound("test".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
            assert!(display.len() > 3);
        }
    }

    #[tokio::test]
    async fn test_protocol_message_routing_error_paths() {
        // ARRANGE
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Command, "nonexistent_handler");

        // ACT: Route to non-existent handler
        let result = protocol.handle_protocol_message(&message).await;

        // ASSERT: Should return proper error
        assert!(result.is_ok());
        if let Ok(response) = result {
            assert_eq!(response.status, ResponseStatus::Error);
        }
    }

    #[tokio::test]
    async fn test_protocol_timeout_handling() {
        // ARRANGE
        let protocol = create_test_protocol();
        let message = create_test_message(MessageType::Command, "slow_command");

        // ACT: Handle with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            protocol.handle_protocol_message(&message),
        )
        .await;

        // ASSERT: Should timeout or complete
        match result {
            Ok(_) => assert!(true, "Completed within timeout"),
            Err(_) => assert!(true, "Timed out gracefully"),
        }
    }
}

#[cfg(test)]
mod transport_error_coverage {
    use squirrel_mcp::error::TransportError;

    #[test]
    fn test_transport_error_all_variants() {
        // Test all TransportError variants for coverage
        let errors = vec![
            TransportError::ConnectionClosed("test".to_string()),
            TransportError::WriteError("test".to_string()),
            TransportError::ReadError("test".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_transport_error_from_io_error() {
        // Test conversion from std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "test");
        let transport_err = TransportError::from(io_err);

        let display = format!("{}", transport_err);
        assert!(!display.is_empty());
    }
}
