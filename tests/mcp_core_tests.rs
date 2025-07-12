//! MCP Core Tests
//! 
//! Tests for core Machine Context Protocol functionality that belongs in Squirrel.
//! This excludes functionality moved to other projects:
//! - Web integration (moved to Songbird)
//! - Compute/storage (moved to ToadStool/NestGate) 
//! - Complex monitoring (distributed across ecosystem)

use squirrel::error::types::{MCPError, Result};

#[cfg(test)]
mod core_functionality {
    use super::*;

    #[test]
    fn test_mcp_error_types() {
        // Test core error handling - this is fundamental MCP functionality
        let validation_error = MCPError::ValidationFailed("test validation".to_string());
        assert!(validation_error.to_string().contains("test validation"));
        assert_eq!(validation_error.error_code(), "MCP-001");

        let operation_error = MCPError::OperationFailed("test operation".to_string());
        assert!(operation_error.to_string().contains("test operation"));
        assert_eq!(operation_error.error_code(), "MCP-002");
    }

    #[test]
    fn test_mcp_result_handling() {
        // Test Result type usage - core to MCP protocol
        let success: Result<String> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        let failure: Result<String> = Err(MCPError::InternalError("failure".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_code_consistency() {
        // Verify error codes are consistent - important for protocol compliance
        assert_eq!(MCPError::ValidationFailed("".to_string()).error_code(), "MCP-001");
        assert_eq!(MCPError::OperationFailed("".to_string()).error_code(), "MCP-002");
        assert_eq!(MCPError::InternalError("".to_string()).error_code(), "MCP-003");
        assert_eq!(MCPError::Network("".to_string()).error_code(), "MCP-024");
        assert_eq!(MCPError::Configuration("".to_string()).error_code(), "MCP-030");
    }

    #[test]
    fn test_authentication_errors() {
        // Test authentication error types - core MCP security
        assert_eq!(MCPError::InvalidCredentials.error_code(), "MCP-040");
        assert_eq!(MCPError::InvalidToken.error_code(), "MCP-041");
        assert_eq!(MCPError::AccountLocked.error_code(), "MCP-042");
        assert_eq!(MCPError::MissingContext.error_code(), "MCP-043");
    }
}

#[cfg(test)]
mod protocol_basics {
    use super::*;

    #[test]
    fn test_mcp_core_constants() {
        // Test that core MCP constants are available
        // Note: These should be the minimal constants needed for MCP protocol
        assert!(!squirrel::VERSION.is_empty());
    }

    #[test]
    fn test_error_debug_formatting() {
        // Test error formatting - important for debugging MCP issues
        let error = MCPError::ValidationFailed("debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ValidationFailed"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_error_display_formatting() {
        // Test error display - important for user-facing error messages
        let error = MCPError::Network("connection failed".to_string());
        let display_str = error.to_string();
        assert!(display_str.contains("Network error"));
        assert!(display_str.contains("connection failed"));
    }
}

// Note: Tests for the following have been removed as they belong to other projects:
// - Web integration tests (moved to Songbird)
// - Storage/compute tests (moved to ToadStool/NestGate)
// - Complex monitoring tests (distributed)
// - Task management tests (removed from MCP Core)
// - Plugin system tests (if moved to other projects)

// TODO: Add tests for core MCP functionality as it's implemented:
// - Basic protocol message handling
// - Core transport mechanisms (if they remain in MCP Core)
// - Essential session management (if it remains in MCP Core)
// - Basic tool integration (if it remains in MCP Core) 