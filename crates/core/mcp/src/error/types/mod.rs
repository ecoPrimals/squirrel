// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Error type definitions for the MCP system.
//!
//! This module provides a comprehensive error type hierarchy for the Machine Context Protocol (MCP)
//! system. It defines various error types for different categories of errors that can occur during
//! MCP operations, including context errors, protocol errors, security errors, connection errors,
//! and more.
//!
//! # Error Types
//!
//! The central error type is `MCPError`, which is a comprehensive enum that can represent any
//! error that may occur in the MCP system. Specialized error types like `ContextError`,
//! `ProtocolError`, `SecurityError`, and `ConnectionError` provide more detailed error information
//! for specific categories of errors.
//!
//! # Error Context
//!
//! The `ErrorContext` struct provides additional metadata about errors, including:
//! - Timestamp of when the error occurred
//! - Operation that was being performed
//! - Component where the error occurred
//! - Severity of the error
//! - Whether the error is recoverable
//! - Additional details about the error

// Module declarations
mod auth_error;
mod base_types;
mod context;
mod conversions;
mod mcp_error;
mod severity;

#[cfg(test)]
mod mcp_error_tests;

// Public re-exports
pub use auth_error::AuthError;
pub use base_types::{SecurityLevel, WireFormatError};
pub use context::ErrorContext;
pub use mcp_error::MCPError;
pub use severity::ErrorSeverity;

/// Error type alias for backward compatibility
///
/// This type alias is provided for backward compatibility with code
/// that refers to `crate::error::Error` instead of `MCPError`.
pub type Error = MCPError;

/// Result type alias for backward compatibility
///
/// This type alias is provided for backward compatibility with code
/// that refers to `crate::error::MCPResult` instead of `Result`.
pub type MCPResult<T> = std::result::Result<T, MCPError>;

/// Canonical Result type for MCP operations
///
/// This is the primary Result type used throughout the MCP system.
/// It provides a convenient alias for Result<T, MCPError>.
///
/// # Examples
///
/// ```ignore
/// use crate::error::{Result, MCPError};
///
/// fn do_something() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, MCPError>;

// NOTE: Enhanced module exists but is not yet exposed in lib.rs due to module structure issues
// The enhanced/config_validation.rs file exists and is complete, but the enhanced module
// has duplicate .rs and /mod.rs files for some submodules (coordinator, multi_agent, service_composition)
// which causes compilation errors when exposed publicly.
// NOTE(module-structure): Enhanced module conversion disabled until module structure ambiguities are resolved
// Tracked: Module structure cleanup needed before public exposure
// impl From<crate::enhanced::config_validation::ConfigValidationError> for MCPError {
//     fn from(error: crate::enhanced::config_validation::ConfigValidationError) -> Self {
//         MCPError::Validation(error.to_string())
//     }
// }

// Comment out the From<MCPError> for DomainError implementation until DomainError is defined
/*
impl From<MCPError> for DomainError {
    fn from(err: MCPError) -> Self {
        match err {
            MCPError::Transport(e) => Self::MCP(format!("Transport error: {e}")),
            MCPError::Protocol(e) => Self::MCP(format!("Protocol error: {e}")),
            MCPError::Security(e) => Self::MCP(format!("Security error: {e}")),
            MCPError::Connection(e) => Self::MCP(format!("Connection error: {e}")),
            MCPError::Session(e) => Self::MCP(format!("Session error: {e}")),
            MCPError::Context(e) => Self::MCP(format!("Context error: {e}")),
            MCPError::Client(e) => Self::MCP(format!("Client error: {e}")),
            MCPError::MessageRouter(e) => Self::MCP(format!("Message router error: {e}")),
            MCPError::Plugin(e) => Self::MCP(format!("Plugin error: {e}")),
            MCPError::Serialization(e) => Self::MCP(format!("Serialization error: {e}")),
            MCPError::Deserialization(e) => Self::MCP(format!("Deserialization error: {e}")),
            MCPError::InvalidMessage(e) => Self::MCP(format!("Invalid message: {e}")),
            MCPError::State(e) => Self::MCP(format!("State error: {e}")),
            MCPError::Authorization(e) => Self::MCP(format!("Authorization error: {e}")),
            MCPError::UnsupportedOperation(e) => Self::MCP(format!("Unsupported operation: {e}")),
            MCPError::CircuitBreaker(e) => Self::MCP(format!("Circuit breaker error: {e}")),
            MCPError::Io(e) => Self::MCP(format!("IO error: {e}")),
            MCPError::Json(e) => Self::MCP(format!("JSON error: {e}")),
            MCPError::SquirrelDetail(e) => Self::MCP(format!("Squirrel core error: {e}")),
            MCPError::PersistenceDetail(e) => Self::MCP(format!("Persistence error: {e}")),
            MCPError::Alert(e) => Self::MCP(format!("Alert error: {e}")),
            MCPError::Storage(e) => Self::MCP(format!("Storage error: {e}")),
            MCPError::NotInitialized(e) => Self::MCP(format!("Not initialized: {e}")),
            MCPError::General(e) => Self::MCP(format!("General error: {e}")),
            MCPError::Network(e) => Self::MCP(format!("Network error: {e}")),
            MCPError::AlreadyInProgress(e) => Self::MCP(format!("Already in progress: {e}")),
            MCPError::Monitoring(e) => Self::MCP(format!("Monitoring error: {e}")),
            MCPError::NotConnected(e) => Self::MCP(format!("Not connected: {e}")),
            MCPError::Timeout(e) => Self::MCP(format!("Timeout: {e}")),
            MCPError::Remote(e) => Self::MCP(format!("Remote error: {e}")),
            MCPError::Configuration(e) => Self::MCP(format!("Configuration error: {e}")),
            MCPError::Unexpected(e) => Self::MCP(format!("Unexpected error: {e}")),
            MCPError::VersionMismatch(e) => Self::MCP(format!("Version mismatch: {e}")),
            MCPError::Unsupported(e) => Self::MCP(format!("Unsupported: {e}")),
            MCPError::General(e) => Self::MCP(format!("Invalid argument: {e}")),
            MCPError::General(e) => Self::MCP(format!("Not found: {e}")),
            MCPError::NotImplemented(e) => Self::MCP(format!("Not implemented: {e}")),
            MCPError::NotAuthorized(e) => Self::MCP(format!("Not authorized: {e}")),
            MCPError::InvalidState(e) => Self::MCP(format!("Invalid state: {e}")),
            MCPError::InvalidOperation(e) => Self::MCP(format!("Invalid operation: {e}")),
            MCPError::GeneralError(e) => Self::MCP(format!("Internal error: {e}")),
        }
    }
}
*/

// Tonic gRPC integration removed - using WebSocket + tarpc instead

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::MessageType;

    // ========== ErrorSeverity Tests ==========

    #[test]
    fn test_error_severity_variants() {
        let _ = ErrorSeverity::Low;
        let _ = ErrorSeverity::Medium;
        let _ = ErrorSeverity::High;
        let _ = ErrorSeverity::Critical;
    }

    #[test]
    fn test_error_severity_requires_immediate_attention() {
        assert!(!ErrorSeverity::Low.requires_immediate_attention());
        assert!(!ErrorSeverity::Medium.requires_immediate_attention());
        assert!(ErrorSeverity::High.requires_immediate_attention());
        assert!(ErrorSeverity::Critical.requires_immediate_attention());
    }

    #[test]
    fn test_error_severity_should_alert() {
        assert!(!ErrorSeverity::Low.should_alert());
        assert!(!ErrorSeverity::Medium.should_alert());
        assert!(ErrorSeverity::High.should_alert());
        assert!(ErrorSeverity::Critical.should_alert());
    }

    #[test]
    fn test_error_severity_equality() {
        assert_eq!(ErrorSeverity::Low, ErrorSeverity::Low);
        assert_ne!(ErrorSeverity::Low, ErrorSeverity::High);
    }

    #[test]
    fn test_error_severity_serialization() {
        let severity = ErrorSeverity::Critical;
        let json = serde_json::to_string(&severity).expect("test: should succeed");
        let deserialized: ErrorSeverity =
            serde_json::from_str(&json).expect("test: should succeed");
        assert_eq!(severity, deserialized);
    }

    #[test]
    fn test_error_severity_clone() {
        let severity = ErrorSeverity::High;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }

    // ========== SecurityLevel Tests ==========

    #[test]
    fn test_security_level_default() {
        let level = SecurityLevel::default();
        assert!(matches!(level, SecurityLevel::Medium));
    }

    #[test]
    fn test_security_level_variants() {
        let _ = SecurityLevel::Low;
        let _ = SecurityLevel::Medium;
        let _ = SecurityLevel::High;
        let _ = SecurityLevel::Critical;
    }

    #[test]
    fn test_security_level_serialization() {
        let level = SecurityLevel::High;
        let json = serde_json::to_string(&level).expect("test: should succeed");
        let deserialized: SecurityLevel =
            serde_json::from_str(&json).expect("test: should succeed");
        // Just verify it round-trips
        let _ = deserialized;
    }

    #[test]
    fn test_security_level_clone() {
        let level = SecurityLevel::Critical;
        let cloned = level.clone();
        // Just verify clone works
        let _ = cloned;
    }

    // ========== WireFormatError Tests ==========

    #[test]
    fn test_wire_format_error_creation() {
        let error = WireFormatError {
            message: "Invalid format".to_string(),
        };
        assert_eq!(error.message, "Invalid format");
    }

    #[test]
    fn test_wire_format_error_display() {
        let error = WireFormatError {
            message: "Test error".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Test error"));
        assert!(display.contains("Wire format error"));
    }

    #[test]
    fn test_wire_format_error_serialization() {
        let error = WireFormatError {
            message: "Serialization test".to_string(),
        };
        let json = serde_json::to_string(&error).expect("test: should succeed");
        let deserialized: WireFormatError =
            serde_json::from_str(&json).expect("test: should succeed");
        assert_eq!(error.message, deserialized.message);
    }

    #[test]
    fn test_wire_format_error_clone() {
        let error = WireFormatError {
            message: "Clone test".to_string(),
        };
        let cloned = error.clone();
        assert_eq!(error.message, cloned.message);
    }

    // ========== ErrorContext Tests ==========

    #[test]
    fn test_error_context_new() {
        let context = ErrorContext::new("test_operation", "test_component");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.component, "test_component");
        assert_eq!(context.severity, ErrorSeverity::Low);
        assert!(context.is_recoverable);
    }

    #[test]
    fn test_error_context_with_severity() {
        let context =
            ErrorContext::new("operation", "component").with_severity(ErrorSeverity::High);

        assert_eq!(context.severity, ErrorSeverity::High);
    }

    #[test]
    fn test_error_context_with_message_type() {
        let context =
            ErrorContext::new("operation", "component").with_message_type(MessageType::Command);

        assert_eq!(context.message_type, Some(MessageType::Command));
    }

    #[test]
    fn test_error_context_serialization() {
        let context = ErrorContext::new("test_op", "test_comp");

        let json = serde_json::to_string(&context).expect("test: should succeed");
        let deserialized: ErrorContext = serde_json::from_str(&json).expect("test: should succeed");

        assert_eq!(context.operation, deserialized.operation);
        assert_eq!(context.component, deserialized.component);
    }

    #[test]
    fn test_error_context_clone() {
        let context = ErrorContext::new("operation", "component");

        let cloned = context.clone();
        assert_eq!(context.operation, cloned.operation);
        assert_eq!(context.severity, cloned.severity);
    }

    // ========== MCPError Tests ==========

    #[test]
    fn test_mcp_error_general() {
        let error = MCPError::General("Test error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Test"));
    }

    #[test]
    fn test_mcp_error_resource_exhausted() {
        let error = MCPError::ResourceExhausted("Out of memory".to_string());
        assert!(matches!(error, MCPError::ResourceExhausted(_)));
    }

    #[test]
    fn test_mcp_error_invalid_argument() {
        let error = MCPError::General("Bad input".to_string());
        assert!(matches!(error, MCPError::General(_)));
    }

    #[test]
    fn test_mcp_error_not_found() {
        let error = MCPError::General("Resource missing".to_string());
        assert!(matches!(error, MCPError::General(_)));
    }

    #[test]
    fn test_mcp_error_already_exists() {
        let error = MCPError::General("Duplicate".to_string());
        assert!(matches!(error, MCPError::General(_)));
    }

    #[test]
    fn test_mcp_error_permission_denied() {
        let error = MCPError::General("Access denied".to_string());
        assert!(matches!(error, MCPError::General(_)));
    }

    #[test]
    fn test_mcp_error_internal() {
        let error = MCPError::General("Internal error".to_string());
        assert!(matches!(error, MCPError::General(_)));
    }

    #[tokio::test]
    async fn test_error_context_timestamp() {
        let context1 = ErrorContext::new("op", "comp");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let context2 = ErrorContext::new("op", "comp");

        // Timestamps should be different
        assert!(context2.timestamp >= context1.timestamp);
    }

    #[test]
    fn test_error_severity_all_levels() {
        let severities = vec![
            ErrorSeverity::Low,
            ErrorSeverity::Medium,
            ErrorSeverity::High,
            ErrorSeverity::Critical,
        ];

        for severity in severities {
            let context = ErrorContext::new("op", "comp").with_severity(severity);
            assert_eq!(context.severity, severity);
        }
    }

    #[test]
    fn test_error_context_empty_details() {
        let context = ErrorContext::new("op", "comp");
        assert!(context.details.is_empty());
    }

    #[test]
    fn test_mcp_error_display() {
        let error = MCPError::General("Test message".to_string());
        let display = format!("{}", error);
        assert!(!display.is_empty());
    }

    #[test]
    fn test_error_context_builder_chain() {
        let context = ErrorContext::new("op", "comp")
            .with_severity(ErrorSeverity::High)
            .with_message_type(MessageType::Error);

        assert_eq!(context.severity, ErrorSeverity::High);
        assert_eq!(context.message_type, Some(MessageType::Error));
    }
}
