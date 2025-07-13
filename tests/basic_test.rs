//! Basic MCP Core tests
//!
//! Minimal tests to verify core error handling works.

use squirrel::error::types::{MCPError, Result};

#[test]
fn test_mcp_error_validation_failed() {
    let error = MCPError::ValidationFailed("test error".to_string());
    assert!(error.to_string().contains("test error"));
    assert_eq!(error.error_code(), "MCP-001");
}

#[test]
fn test_mcp_error_operation_failed() {
    let error = MCPError::OperationFailed("operation failed".to_string());
    assert!(error.to_string().contains("operation failed"));
    assert_eq!(error.error_code(), "MCP-002");
}

#[test]
fn test_mcp_error_internal_error() {
    let error = MCPError::InternalError("internal error".to_string());
    assert!(error.to_string().contains("internal error"));
    assert_eq!(error.error_code(), "MCP-003");
}

#[test]
fn test_mcp_result_ok() {
    let result: Result<String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_mcp_result_error() {
    let result: Result<String> = Err(MCPError::InternalError("test".to_string()));
    assert!(result.is_err());
    match result {
        Err(MCPError::InternalError(msg)) => assert_eq!(msg, "test"),
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_error_code_mapping() {
    assert_eq!(
        MCPError::Network("test".to_string()).error_code(),
        "MCP-024"
    );
    assert_eq!(
        MCPError::Configuration("test".to_string()).error_code(),
        "MCP-030"
    );
    assert_eq!(
        MCPError::InvalidArgument("test".to_string()).error_code(),
        "MCP-035"
    );
    assert_eq!(
        MCPError::NotFound("test".to_string()).error_code(),
        "MCP-036"
    );
    assert_eq!(
        MCPError::PermissionDenied("test".to_string()).error_code(),
        "MCP-037"
    );
}

#[test]
fn test_authentication_errors() {
    assert_eq!(MCPError::InvalidCredentials.error_code(), "MCP-040");
    assert_eq!(MCPError::InvalidToken.error_code(), "MCP-041");
    assert_eq!(MCPError::AccountLocked.error_code(), "MCP-042");
    assert_eq!(MCPError::MissingContext.error_code(), "MCP-043");
    assert_eq!(
        MCPError::ProviderError("test".to_string()).error_code(),
        "MCP-044"
    );
}

#[test]
fn test_error_debug_format() {
    let error = MCPError::ValidationFailed("debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ValidationFailed"));
    assert!(debug_str.contains("debug test"));
}
