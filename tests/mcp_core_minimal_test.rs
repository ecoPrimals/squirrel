//! Minimal MCP Core Tests
//!
//! Tests for the most basic core Machine Context Protocol functionality.
//! This is designed to compile and test only the essential parts that remain in Squirrel.

#[cfg(test)]
mod minimal_core_tests {
    use squirrel::error::types::{MCPError, Result};

    #[test]
    fn test_basic_error_creation() {
        // Test basic error types that should always work
        let validation_error = MCPError::ValidationFailed("test validation".to_string());
        assert!(validation_error.to_string().contains("test validation"));

        let operation_error = MCPError::OperationFailed("test operation".to_string());
        assert!(operation_error.to_string().contains("test operation"));
    }

    #[test]
    fn test_result_handling() {
        // Test Result type usage - core to MCP protocol
        let success: Result<String> = Ok("success".to_string());
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), "success");

        let failure: Result<String> = Err(MCPError::InternalError("failure".to_string()));
        assert!(failure.is_err());
    }
}
