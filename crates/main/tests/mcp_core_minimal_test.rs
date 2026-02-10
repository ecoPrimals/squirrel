// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Minimal MCP Core Tests
//!
//! Tests for the most basic core Machine Context Protocol functionality.
//! This tests the actual MCPError types that exist in the codebase.

#[cfg(test)]
mod minimal_core_tests {
    use squirrel_mcp::MCPError;

    #[test]
    fn test_basic_error_creation() {
        // Test basic error types that actually exist
        let validation_error = MCPError::Validation("test validation".to_string());
        let _ = validation_error.to_string();

        let internal_error = MCPError::Internal("test internal".to_string());
        let _ = internal_error.to_string();

        let general_error = MCPError::General("test general".to_string());
        let _ = general_error.to_string();
    }

    #[test]
    fn test_result_handling() {
        // Test Result type usage - core to MCP protocol
        let success: Result<String, MCPError> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.expect("Expected success value"), "success");

        let failure: Result<String, MCPError> = Err(MCPError::Internal("failure".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_matching() {
        // Test pattern matching on errors
        let error = MCPError::Validation("test".to_string());
        match error {
            MCPError::Validation(msg) => assert_eq!(msg, "test"),
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_error_types() {
        // Test all common error types can be created
        let _ = MCPError::Network("network".to_string());
        let _ = MCPError::Configuration("config".to_string());
        let _ = MCPError::Authentication("auth".to_string());
        let _ = MCPError::Authorization("authz".to_string());
        let _ = MCPError::Timeout("timeout".to_string());
        let _ = MCPError::InvalidArgument("invalid".to_string());
        let _ = MCPError::NotFound("not found".to_string());
    }
}
