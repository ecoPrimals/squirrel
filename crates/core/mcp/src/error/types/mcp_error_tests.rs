// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for MCPError type
//!
//! This module provides thorough testing of the MCPError enum,
//! ensuring proper error construction, conversion, and formatting.

#[cfg(test)]
mod tests {
    use crate::error::types::mcp_error::MCPError;
    use crate::error::{
        alert::AlertError, client::ClientError, connection::ConnectionError,
        protocol_err::ProtocolError, session::SessionError, task::TaskError, tool::ToolError,
        transport::TransportError,
    };

    #[test]
    fn test_mcp_error_general() {
        let error = MCPError::General("test error".to_string());
        let error_str = error.to_string();
        assert!(error_str.len() > 0);
        use crate::error::ErrorSeverity;
        assert!(matches!(error.severity(), ErrorSeverity::Low));
    }

    #[test]
    fn test_mcp_error_transport_conversion() {
        let transport_err = TransportError::ConnectionClosed("test".to_string());
        let mcp_err: MCPError = transport_err.into();

        assert!(matches!(mcp_err, MCPError::Transport(_)));
        assert!(mcp_err.to_string().contains("Connection closed"));
    }

    #[test]
    fn test_mcp_error_protocol_conversion() {
        let protocol_err = ProtocolError::InvalidVersion("1.0".to_string());
        let mcp_err: MCPError = protocol_err.into();

        assert!(matches!(mcp_err, MCPError::Protocol(_)));
        assert!(mcp_err.to_string().contains("1.0"));
    }

    #[test]
    fn test_mcp_error_connection_conversion() {
        let conn_err = ConnectionError::ConnectionFailed("timeout".to_string());
        let mcp_err: MCPError = conn_err.into();

        assert!(matches!(mcp_err, MCPError::Connection(_)));
        assert!(mcp_err.to_string().contains("timeout"));
    }

    #[test]
    fn test_mcp_error_session_conversion() {
        let session_err = SessionError::NotFound("session-123".to_string());
        let mcp_err: MCPError = session_err.into();

        assert!(matches!(mcp_err, MCPError::Session(_)));
        assert!(mcp_err.to_string().contains("session-123"));
    }

    #[test]
    fn test_mcp_error_client_conversion() {
        let client_err = ClientError::NotConnected("disconnected".to_string());
        let mcp_err: MCPError = client_err.into();

        assert!(matches!(mcp_err, MCPError::Client(_)));
        assert!(mcp_err.to_string().contains("Client error"));
    }

    #[test]
    fn test_mcp_error_tool_conversion() {
        let tool_err = ToolError::ExecutionFailed("test-tool failed".to_string());
        let mcp_err: MCPError = tool_err.into();

        assert!(matches!(mcp_err, MCPError::Tool(_)));
        // Tool errors display as category string
        let err_str = mcp_err.to_string();
        assert!(err_str.len() > 0);
    }

    #[test]
    fn test_mcp_error_task_conversion() {
        let task_err = TaskError::NotFound("task-456".to_string());
        let mcp_err: MCPError = task_err.into();

        assert!(matches!(mcp_err, MCPError::Task(_)));
        // Task errors display as category string
        let err_str = mcp_err.to_string();
        assert!(err_str.len() > 0);
    }

    #[test]
    fn test_mcp_error_alert_conversion() {
        let alert_err = AlertError::NotificationFailed("channel closed".to_string());
        let mcp_err: MCPError = alert_err.into();

        assert!(matches!(mcp_err, MCPError::Alert(_)));
        assert!(mcp_err.to_string().contains("Alert error"));
    }

    // Old incompatible tests removed - use new comprehensive tests below

    #[test]
    fn test_mcp_result_type() {
        use crate::error::Result;

        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_mcp_result_error() {
        use crate::error::Result;

        fn returns_error() -> Result<i32> {
            Err(MCPError::General("failed".to_string()))
        }

        assert!(returns_error().is_err());
    }

    // ========== New Comprehensive Tests ==========

    // Test all String-based error variants
    #[test]
    fn test_resource_exhausted_variant() {
        let error = MCPError::ResourceExhausted("memory limit reached".to_string());
        assert_eq!(error.code_str(), "MCP-056");
        assert_eq!(error.category_str(), "RESOURCE_EXHAUSTED");
        assert!(error.to_string().len() > 0);
    }

    #[test]
    fn test_invalid_argument_variant() {
        let error = MCPError::InvalidArgument("port must be positive".to_string());
        assert_eq!(error.code_str(), "MCP-038");
        assert_eq!(error.category_str(), "INVALID_ARGUMENT");
    }

    #[test]
    fn test_not_found_variant() {
        let error = MCPError::NotFound("session-123".to_string());
        assert_eq!(error.code_str(), "MCP-039");
        assert_eq!(error.category_str(), "NOT_FOUND");
    }

    #[test]
    fn test_internal_variant() {
        let error = MCPError::Internal("unexpected state".to_string());
        assert_eq!(error.code_str(), "MCP-057");
        assert_eq!(error.category_str(), "INTERNAL");
    }

    #[test]
    fn test_authentication_variant() {
        let error = MCPError::Authentication("invalid token".to_string());
        assert_eq!(error.code_str(), "MCP-058");
        assert_eq!(error.category_str(), "AUTHENTICATION");
    }

    #[test]
    fn test_authorization_variant() {
        let error = MCPError::Authorization("permission denied".to_string());
        assert_eq!(error.code_str(), "MCP-013");
        assert_eq!(error.category_str(), "AUTHORIZATION");
    }

    #[test]
    fn test_rate_limit_variant() {
        let error = MCPError::RateLimit("too many requests".to_string());
        assert_eq!(error.code_str(), "MCP-059");
        assert_eq!(error.category_str(), "RATE_LIMIT");
    }

    #[test]
    fn test_timeout_variant() {
        let error = MCPError::Timeout("operation timed out".to_string());
        assert_eq!(error.code_str(), "MCP-033");
        assert_eq!(error.category_str(), "TIMEOUT");
    }

    #[test]
    fn test_configuration_variant() {
        let error = MCPError::Configuration("invalid config".to_string());
        assert_eq!(error.code_str(), "MCP-051");
        assert_eq!(error.category_str(), "CONFIGURATION");
    }

    #[test]
    fn test_validation_variant() {
        let error = MCPError::Validation("field required".to_string());
        assert_eq!(error.code_str(), "MCP-023");
        assert_eq!(error.category_str(), "VALIDATION");
    }

    #[test]
    fn test_invalid_state_variant() {
        let error = MCPError::InvalidState("cannot connect in shutdown state".to_string());
        assert_eq!(error.code_str(), "MCP-042");
        assert_eq!(error.category_str(), "INVALID_STATE");
    }

    #[test]
    fn test_invalid_operation_variant() {
        let error = MCPError::InvalidOperation("operation not allowed".to_string());
        assert_eq!(error.code_str(), "MCP-043");
        assert_eq!(error.category_str(), "INVALID_OPERATION");
    }

    #[test]
    fn test_network_variant() {
        let error = MCPError::Network("connection refused".to_string());
        assert_eq!(error.code_str(), "MCP-053");
        assert_eq!(error.category_str(), "NETWORK");
    }

    #[test]
    fn test_io_variant() {
        let error = MCPError::Io("file not found".to_string());
        assert_eq!(error.code_str(), "MCP-016");
        assert_eq!(error.category_str(), "IO");
    }

    #[test]
    fn test_json_variant() {
        let error = MCPError::Json("invalid JSON".to_string());
        assert_eq!(error.code_str(), "MCP-017");
        assert_eq!(error.category_str(), "JSON");
    }

    #[test]
    fn test_generic_variant() {
        let error = MCPError::Generic("generic error".to_string());
        assert_eq!(error.code_str(), "MCP-050");
        assert_eq!(error.category_str(), "GENERIC");
    }

    #[test]
    fn test_message_router_variant() {
        let error = MCPError::MessageRouter("routing failed".to_string());
        assert_eq!(error.code_str(), "MCP-008");
        assert_eq!(error.category_str(), "MESSAGE_ROUTER");
    }

    #[test]
    fn test_serialization_variant() {
        let error = MCPError::Serialization("serialize failed".to_string());
        assert_eq!(error.code_str(), "MCP-009");
        assert_eq!(error.category_str(), "SERIALIZATION");
    }

    #[test]
    fn test_deserialization_variant() {
        let error = MCPError::Deserialization("deserialize failed".to_string());
        assert_eq!(error.code_str(), "MCP-010");
        assert_eq!(error.category_str(), "DESERIALIZATION");
    }

    #[test]
    fn test_invalid_message_variant() {
        let error = MCPError::InvalidMessage("malformed message".to_string());
        assert_eq!(error.code_str(), "MCP-011");
        assert_eq!(error.category_str(), "INVALID_MESSAGE");
    }

    #[test]
    fn test_state_variant() {
        let error = MCPError::State("invalid state transition".to_string());
        assert_eq!(error.code_str(), "MCP-012");
        assert_eq!(error.category_str(), "STATE");
    }

    #[test]
    fn test_unsupported_operation_variant() {
        let error = MCPError::UnsupportedOperation("feature not supported".to_string());
        assert_eq!(error.code_str(), "MCP-014");
        assert_eq!(error.category_str(), "UNSUPPORTED_OPERATION");
    }

    #[test]
    fn test_circuit_breaker_variant() {
        let error = MCPError::CircuitBreaker("circuit open".to_string());
        assert_eq!(error.code_str(), "MCP-015");
        assert_eq!(error.category_str(), "CIRCUIT_BREAKER");
    }

    #[test]
    fn test_security_variant() {
        let error = MCPError::Security("security violation".to_string());
        assert_eq!(error.code_str(), "MCP-021");
        assert_eq!(error.category_str(), "SECURITY");
    }

    #[test]
    fn test_resource_variant() {
        let error = MCPError::Resource("resource unavailable".to_string());
        assert_eq!(error.code_str(), "MCP-022");
        assert_eq!(error.category_str(), "RESOURCE");
    }

    #[test]
    fn test_lifecycle_variant() {
        let error = MCPError::Lifecycle("initialization failed".to_string());
        assert_eq!(error.code_str(), "MCP-024");
        assert_eq!(error.category_str(), "LIFECYCLE");
    }

    #[test]
    fn test_wire_format_variant() {
        let error = MCPError::WireFormat("invalid wire format".to_string());
        assert_eq!(error.code_str(), "MCP-027");
        assert_eq!(error.category_str(), "WIRE_FORMAT");
    }

    #[test]
    fn test_not_initialized_variant() {
        let error = MCPError::NotInitialized("service not initialized".to_string());
        assert_eq!(error.code_str(), "MCP-028");
        assert_eq!(error.category_str(), "NOT_INITIALIZED");
    }

    #[test]
    fn test_already_in_progress_variant() {
        let error = MCPError::AlreadyInProgress("operation pending".to_string());
        assert_eq!(error.code_str(), "MCP-030");
        assert_eq!(error.category_str(), "ALREADY_IN_PROGRESS");
    }

    #[test]
    fn test_monitoring_variant() {
        let error = MCPError::Monitoring("metrics error".to_string());
        assert_eq!(error.code_str(), "MCP-031");
        assert_eq!(error.category_str(), "MONITORING");
    }

    #[test]
    fn test_not_connected_variant() {
        let error = MCPError::NotConnected("client not connected".to_string());
        assert_eq!(error.code_str(), "MCP-032");
        assert_eq!(error.category_str(), "NOT_CONNECTED");
    }

    #[test]
    fn test_remote_variant() {
        let error = MCPError::Remote("remote error".to_string());
        assert_eq!(error.code_str(), "MCP-034");
        assert_eq!(error.category_str(), "REMOTE");
    }

    #[test]
    fn test_unexpected_variant() {
        let error = MCPError::Unexpected("unexpected error".to_string());
        assert_eq!(error.code_str(), "MCP-035");
        assert_eq!(error.category_str(), "UNEXPECTED");
    }

    #[test]
    fn test_version_mismatch_variant() {
        let error = MCPError::VersionMismatch("version incompatible".to_string());
        assert_eq!(error.code_str(), "MCP-036");
        assert_eq!(error.category_str(), "VERSION_MISMATCH");
    }

    #[test]
    fn test_unsupported_variant() {
        let error = MCPError::Unsupported("not supported".to_string());
        assert_eq!(error.code_str(), "MCP-037");
        assert_eq!(error.category_str(), "UNSUPPORTED");
    }

    #[test]
    fn test_not_implemented_variant() {
        let error = MCPError::NotImplemented("feature not implemented".to_string());
        assert_eq!(error.code_str(), "MCP-040");
        assert_eq!(error.category_str(), "NOT_IMPLEMENTED");
    }

    #[test]
    fn test_not_authorized_variant() {
        let error = MCPError::NotAuthorized("access denied".to_string());
        assert_eq!(error.code_str(), "MCP-041");
        assert_eq!(error.category_str(), "NOT_AUTHORIZED");
    }

    #[test]
    fn test_internal_error_variant() {
        let error = MCPError::InternalError("internal failure".to_string());
        assert_eq!(error.code_str(), "MCP-044");
        assert_eq!(error.category_str(), "INTERNAL_ERROR");
    }

    #[test]
    fn test_sync_variant() {
        let error = MCPError::Sync("sync failed".to_string());
        assert_eq!(error.code_str(), "MCP-045");
        assert_eq!(error.category_str(), "SYNC");
    }

    #[test]
    fn test_already_exists_variant() {
        let error = MCPError::AlreadyExists("resource exists".to_string());
        assert_eq!(error.code_str(), "MCP-046");
        assert_eq!(error.category_str(), "ALREADY_EXISTS");
    }

    #[test]
    fn test_invalid_request_variant() {
        let error = MCPError::InvalidRequest("malformed request".to_string());
        assert_eq!(error.code_str(), "MCP-047");
        assert_eq!(error.category_str(), "INVALID_REQUEST");
    }

    #[test]
    fn test_database_variant() {
        let error = MCPError::Database("database error".to_string());
        assert_eq!(error.code_str(), "MCP-048");
        assert_eq!(error.category_str(), "DATABASE");
    }

    #[test]
    fn test_operation_failed_variant() {
        let error = MCPError::OperationFailed("operation failed".to_string());
        assert_eq!(error.code_str(), "MCP-049");
        assert_eq!(error.category_str(), "OPERATION_FAILED");
    }

    // Test from_message method
    #[test]
    fn test_from_message() {
        let error = MCPError::from_message("test error");
        assert!(matches!(error, MCPError::Generic(_)));
        // Generic errors display as category string
        assert_eq!(error.to_string(), "GENERIC");
    }

    // Test error_code method (backwards compatibility)
    #[test]
    fn test_error_code_method() {
        let error = MCPError::General("test".to_string());
        assert_eq!(error.error_code(), "MCP-029");
    }

    // Test is_recoverable for various error types
    #[test]
    fn test_is_recoverable_timeout() {
        let error = MCPError::Connection(ConnectionError::Timeout(5000));
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_is_recoverable_reset() {
        let error = MCPError::Connection(ConnectionError::Reset);
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_is_recoverable_unsupported_operation() {
        let error = MCPError::UnsupportedOperation("test".to_string());
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_is_not_recoverable_general() {
        let error = MCPError::General("test".to_string());
        assert!(!error.is_recoverable());
    }

    // Test severity for various error types
    #[test]
    fn test_severity_connection_failed() {
        use crate::error::ErrorSeverity;
        let error = MCPError::Connection(ConnectionError::ConnectionFailed("test".to_string()));
        assert!(matches!(error.severity(), ErrorSeverity::High));
    }

    #[test]
    fn test_severity_connection_closed() {
        use crate::error::ErrorSeverity;
        let error = MCPError::Connection(ConnectionError::Closed("test".to_string()));
        assert!(matches!(error.severity(), ErrorSeverity::High));
    }

    #[test]
    fn test_severity_protocol_invalid_version() {
        use crate::error::ErrorSeverity;
        let error = MCPError::Protocol(ProtocolError::InvalidVersion("1.0".to_string()));
        assert!(matches!(error.severity(), ErrorSeverity::High));
    }

    #[test]
    fn test_severity_unsupported_operation_medium() {
        use crate::error::ErrorSeverity;
        let error = MCPError::UnsupportedOperation("test".to_string());
        assert!(matches!(error.severity(), ErrorSeverity::Medium));
    }

    #[test]
    fn test_severity_general_low() {
        use crate::error::ErrorSeverity;
        let error = MCPError::General("test".to_string());
        assert!(matches!(error.severity(), ErrorSeverity::Low));
    }

    // Test Display implementation
    #[test]
    fn test_display_transport() {
        let transport_err = TransportError::ConnectionClosed("test".to_string());
        let error = MCPError::Transport(transport_err);
        assert!(error.to_string().contains("Transport error"));
    }

    #[test]
    fn test_display_protocol() {
        let protocol_err = ProtocolError::InvalidVersion("test".to_string());
        let error = MCPError::Protocol(protocol_err);
        assert!(error.to_string().contains("Protocol error"));
    }

    #[test]
    fn test_display_connection() {
        let conn_err = ConnectionError::ConnectionFailed("test".to_string());
        let error = MCPError::Connection(conn_err);
        assert!(error.to_string().contains("Connection error"));
    }

    #[test]
    fn test_display_session() {
        let session_err = SessionError::NotFound("test".to_string());
        let error = MCPError::Session(session_err);
        assert!(error.to_string().contains("Session error"));
    }

    #[test]
    fn test_display_client() {
        let client_err = ClientError::NotConnected("test".to_string());
        let error = MCPError::Client(client_err);
        assert!(error.to_string().contains("Client error"));
    }

    #[test]
    fn test_display_alert() {
        let alert_err = AlertError::NotificationFailed("test".to_string());
        let error = MCPError::Alert(alert_err);
        assert!(error.to_string().contains("Alert error"));
    }

    #[test]
    fn test_display_fallback() {
        let error = MCPError::General("test".to_string());
        let display = error.to_string();
        assert_eq!(display, "GENERAL");
    }
}
