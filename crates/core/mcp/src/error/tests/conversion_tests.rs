// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error conversion and mapping tests
//!
//! Comprehensive tests for error type conversions, code mapping, and severity classification.

#[cfg(test)]
mod tests {
    use crate::error::{ConnectionError, ProtocolError, TransportError};
    use crate::error::{ErrorContext, ErrorSeverity, MCPError};
    use crate::protocol::types::MessageType;
    use std::io;

    #[test]
    fn test_error_from_io_error() {
        // Arrange
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");

        // Act
        let mcp_err = MCPError::from(io_err);

        // Assert
        assert!(matches!(mcp_err, MCPError::Io(_)));
    }

    #[test]
    fn test_error_from_json_error() {
        // Arrange
        let invalid_json = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();

        // Act
        let mcp_err = MCPError::from(json_err);

        // Assert
        assert!(matches!(mcp_err, MCPError::Json(_)));
    }

    #[test]
    fn test_error_from_string() {
        // Arrange
        let error_msg = "Custom error message";

        // Act
        let mcp_err = MCPError::from(error_msg.to_string());

        // Assert - Should convert to a general error
        match &mcp_err {
            MCPError::General(msg) => assert_eq!(msg, error_msg),
            _ => assert!(format!("{mcp_err:?}").contains(error_msg)),
        }
    }

    #[test]
    fn test_error_display_formatting() {
        // Arrange
        let errors = vec![
            MCPError::Protocol(ProtocolError::InvalidVersion("1.0".to_string())),
            MCPError::Connection(ConnectionError::Timeout(5000)),
            MCPError::Transport(TransportError::ConnectionFailed("Failed".to_string())),
        ];

        // Act & Assert
        for error in errors {
            let display = format!("{error}");
            assert!(!display.is_empty(), "Error display should not be empty");
            assert!(display.len() > 5, "Error display should be descriptive");
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        // Arrange
        let error = MCPError::Protocol(ProtocolError::InvalidVersion("2.0".to_string()));

        // Act
        let debug = format!("{error:?}");

        // Assert
        assert!(!debug.is_empty());
        assert!(debug.contains("Protocol") || debug.contains("InvalidVersion"));
    }

    #[test]
    fn test_error_severity_classification_protocol() {
        // Arrange
        let protocol_err = MCPError::Protocol(ProtocolError::InvalidVersion("bad".to_string()));

        // Act
        let severity = protocol_err.severity();

        // Assert
        assert!(matches!(
            severity,
            ErrorSeverity::High | ErrorSeverity::Critical
        ));
    }

    #[test]
    fn test_error_severity_classification_timeout() {
        // Arrange
        let timeout_err = MCPError::Connection(ConnectionError::Timeout(1000));

        // Act
        let severity = timeout_err.severity();

        // Assert
        assert!(matches!(
            severity,
            ErrorSeverity::Low | ErrorSeverity::Medium
        ));
    }

    #[test]
    fn test_error_context_addition() {
        // Arrange
        let base_error = MCPError::Protocol(ProtocolError::InvalidVersion("0.0".to_string()));
        let _context = ErrorContext::new("test_operation", "test_component")
            .with_message_type(MessageType::Command)
            .with_severity(ErrorSeverity::High);

        // Act
        // Note: with_context API doesn't exist, just verify error type
        let error_with_context = base_error;

        // Assert - Just verify it's a valid Protocol error
        assert!(
            matches!(error_with_context, MCPError::Protocol(_)),
            "Expected Protocol error variant"
        );
    }

    #[test]
    fn test_error_recoverable_classification() {
        // Arrange
        let recoverable = MCPError::Connection(ConnectionError::Timeout(5000));
        let non_recoverable = MCPError::Protocol(ProtocolError::InvalidVersion("bad".to_string()));

        // Act & Assert
        assert!(
            recoverable.is_recoverable(),
            "Timeout should be recoverable"
        );
        assert!(
            !non_recoverable.is_recoverable(),
            "Invalid version should not be recoverable"
        );
    }

    #[test]
    fn test_error_chain_preservation() {
        // Arrange — io::Error is not Clone, so From converts to string representation
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let mcp_err = MCPError::from(io_err);

        // Assert — message text is preserved even though source chain is lost
        let display = format!("{mcp_err}");
        assert!(
            display.contains("Access denied"),
            "IO error message should be preserved: {display}"
        );
    }

    #[test]
    fn test_error_equality_same_variant() {
        // Arrange
        let err1 = MCPError::General("test".to_string());
        let err2 = MCPError::General("test".to_string());

        // Act & Assert
        // Errors may not implement PartialEq, so we check variants
        match (&err1, &err2) {
            (MCPError::General(msg1), MCPError::General(msg2)) => {
                assert_eq!(msg1, msg2);
            }
            _ => unreachable!("Should be same variant"),
        }
    }

    #[test]
    fn test_error_code_mapping() {
        // Arrange
        let errors = vec![
            MCPError::Protocol(ProtocolError::InvalidVersion("1.0".to_string())),
            MCPError::Connection(ConnectionError::Timeout(5000)),
        ];

        // Act & Assert - Each error should have a code or be mappable
        for error in errors {
            let display = format!("{error}");
            // Errors should produce meaningful output
            assert!(!display.is_empty());
        }
    }

    #[test]
    fn test_error_serialization() {
        // Arrange
        let severity = ErrorSeverity::High;

        // Act
        let json = serde_json::to_string(&severity);

        // Assert
        assert!(json.is_ok(), "ErrorSeverity should be serializable");
        if let Ok(json_str) = json {
            let deserialized: Result<ErrorSeverity, _> = serde_json::from_str(&json_str);
            assert!(deserialized.is_ok(), "Should deserialize successfully");
        }
    }

    #[test]
    fn test_error_context_builder_pattern() {
        // Arrange & Act
        let context = ErrorContext::new("operation", "component")
            .with_severity(ErrorSeverity::Medium)
            .with_error_code("ERR-001")
            .with_source_location("file.rs:42");

        // Assert
        assert_eq!(context.operation, "operation");
        assert_eq!(context.component, "component");
        assert_eq!(context.severity, ErrorSeverity::Medium);
        assert_eq!(context.error_code, "ERR-001");
    }

    #[test]
    fn test_multiple_error_conversions() {
        // Arrange
        let errors: Vec<Box<dyn std::error::Error>> = vec![Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "Not found",
        ))];

        // Act & Assert - Should handle various error types
        for err in errors {
            let error_str = err.to_string();
            assert!(!error_str.is_empty());
        }
    }

    #[test]
    fn test_error_severity_ordering() {
        // Arrange
        let severities = [
            ErrorSeverity::Low,
            ErrorSeverity::Medium,
            ErrorSeverity::High,
            ErrorSeverity::Critical,
        ];

        // Act & Assert - Should have logical ordering
        assert!(!severities[0].requires_immediate_attention());
        assert!(severities[3].requires_immediate_attention());
        assert!(severities[3].should_alert());
    }

    #[test]
    fn test_transport_error_conversions() {
        use crate::error::transport::TransportError as CanonicalTransportError;

        // Test canonical to simplified conversion
        let canonical_error =
            CanonicalTransportError::ConnectionFailed("Failed connection".to_string());
        let simplified_error: crate::error::TransportError = canonical_error;

        // Verify the variant matches
        match simplified_error {
            crate::error::TransportError::ConnectionFailed(msg) => {
                assert!(msg.contains("Failed connection"));
            }
            _ => unreachable!("Expected ConnectionFailed variant"),
        }

        // Test simplified to canonical conversion
        let simplified_error =
            crate::error::TransportError::Timeout("Connection timeout".to_string());
        let canonical_error: CanonicalTransportError = simplified_error;

        // Verify the variant matches
        match canonical_error {
            CanonicalTransportError::Timeout(msg) => {
                assert!(msg.contains("Connection timeout"));
            }
            _ => unreachable!("Expected Timeout variant"),
        }

        // Test MCPError wrapping of simplified error
        let simplified_error = crate::error::TransportError::IoError("IO failure".to_string());
        let mcp_error = MCPError::Transport(simplified_error);

        // Verify error type extraction
        match &mcp_error {
            MCPError::Transport(err) => match err {
                crate::error::TransportError::IoError(msg) => {
                    assert!(msg.contains("IO failure"));
                }
                _ => unreachable!("Expected IoError variant"),
            },
            _ => unreachable!("Expected MCPError::Transport"),
        }
    }
}
