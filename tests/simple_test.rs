//! Simple MCP Core tests
//! 
//! Basic tests to verify core functionality works.

use squirrel_mcp::{MCPError, Result, VERSION};

#[tokio::test]
async fn test_mcp_core_version() {
    // Test 1: Version information
    assert!(!VERSION.is_empty());
    println!("MCP Core Version: {}", VERSION);
}

#[tokio::test]
async fn test_mcp_error_creation() {
    // Test 2: Error handling
    let error = MCPError::ValidationFailed("test error".to_string());
    assert!(error.to_string().contains("test error"));
}

#[tokio::test]
async fn test_mcp_result_ok() {
    // Test 3: Result type usage
    let result: Result<String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_mcp_result_error() {
    // Test 4: Result error handling
    let result: Result<String> = Err(MCPError::InternalError("test".to_string()));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_integration_module() {
    // Test 5: Integration module
    let mut integration = squirrel_mcp::integration::SimpleMCPIntegration::new();
    assert!(integration.initialize().await.is_ok());
    assert!(integration.is_initialized());
} 