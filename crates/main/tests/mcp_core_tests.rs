//! MCP Core Tests
//!
//! Tests for core Machine Context Protocol functionality in Squirrel.
//! Tests the actual MCPError implementation and error codes.

use squirrel_mcp::MCPError;

#[cfg(test)]
mod core_functionality {
    use super::*;

    #[test]
    fn test_mcp_error_types() {
        // Test core error handling - fundamental MCP functionality
        let validation_error = MCPError::Validation("test validation".to_string());
        let _ = validation_error.to_string();
        assert_eq!(validation_error.error_code(), "MCP-023");

        let internal_error = MCPError::Internal("test internal".to_string());
        let _ = internal_error.to_string();
        assert_eq!(internal_error.error_code(), "MCP-057");
    }

    #[test]
    fn test_mcp_result_handling() {
        // Test Result type usage - core to MCP protocol
        let success: Result<String, MCPError> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        let failure: Result<String, MCPError> = Err(MCPError::Internal("failure".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_code_consistency() {
        // Verify error codes are consistent - important for protocol compliance
        assert_eq!(MCPError::Validation("".to_string()).error_code(), "MCP-023");
        assert_eq!(
            MCPError::OperationFailed("".to_string()).error_code(),
            "MCP-049"
        );
        assert_eq!(MCPError::Internal("".to_string()).error_code(), "MCP-057");
        assert_eq!(
            MCPError::InternalError("".to_string()).error_code(),
            "MCP-044"
        );
        assert_eq!(MCPError::Network("".to_string()).error_code(), "MCP-053");
        assert_eq!(
            MCPError::Configuration("".to_string()).error_code(),
            "MCP-051"
        );
    }

    #[test]
    fn test_error_category_str() {
        // Test category string representation
        assert_eq!(
            MCPError::Validation("".to_string()).category_str(),
            "VALIDATION"
        );
        assert_eq!(
            MCPError::Internal("".to_string()).category_str(),
            "INTERNAL"
        );
        assert_eq!(MCPError::Network("".to_string()).category_str(), "NETWORK");
    }

    #[test]
    fn test_transport_and_protocol_errors() {
        // Test that error codes for core MCP layers are correct
        assert_eq!(
            MCPError::Transport(squirrel_mcp::error::TransportError::ConnectionFailed(
                "test".to_string()
            ))
            .error_code(),
            "MCP-001"
        );
        assert_eq!(
            MCPError::Protocol(squirrel_mcp::error::ProtocolError::InvalidVersion(
                "test".to_string()
            ))
            .error_code(),
            "MCP-002"
        );
    }

    #[test]
    fn test_common_error_codes() {
        // Test common error variants and their codes
        let test_cases = vec![
            (MCPError::Timeout("".to_string()), "MCP-033"),
            (MCPError::Authentication("".to_string()), "MCP-058"),
            (MCPError::Authorization("".to_string()), "MCP-013"),
            (MCPError::NotFound("".to_string()), "MCP-039"),
            (MCPError::InvalidArgument("".to_string()), "MCP-038"),
            (MCPError::RateLimit("".to_string()), "MCP-059"),
        ];

        for (error, expected_code) in test_cases {
            assert_eq!(error.error_code(), expected_code);
        }
    }

    #[test]
    fn test_error_pattern_matching() {
        // Test pattern matching on errors
        let error = MCPError::Validation("test message".to_string());
        match error {
            MCPError::Validation(msg) => assert_eq!(msg, "test message"),
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_error_clone() {
        // MCPError should be Clone
        let error = MCPError::General("test".to_string());
        let cloned = error.clone();
        assert_eq!(error.error_code(), cloned.error_code());
    }
}
