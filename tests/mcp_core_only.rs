//! MCP Core Only Tests
//! 
//! Tests ONLY the core MCP functionality that should remain in Squirrel.
//! Everything else was moved to other projects during the tearout:
//! - Web → Songbird
//! - Compute/Storage → ToadStool/NestGate  
//! - Security → BearDog
//! - Complex monitoring → Distributed

use squirrel::{MCPError, Result, VERSION};

#[test]
fn test_mcp_core_version() {
    // Test 1: Version information is available
    assert!(!VERSION.is_empty());
    println!("MCP Core Version: {}", VERSION);
}

#[test]
fn test_mcp_error_creation() {
    // Test 2: Core error types work
    let validation_error = MCPError::ValidationFailed("test validation".to_string());
    assert!(validation_error.to_string().contains("test validation"));
    
    let operation_error = MCPError::OperationFailed("test operation".to_string());
    assert!(operation_error.to_string().contains("test operation"));
    
    let internal_error = MCPError::InternalError("test internal".to_string());
    assert!(internal_error.to_string().contains("test internal"));
}

#[test]
fn test_mcp_result_handling() {
    // Test 3: Result type usage
    let success: Result<String> = Ok("success".to_string());
    assert!(success.is_ok());
    assert_eq!(success.unwrap(), "success");

    let failure: Result<String> = Err(MCPError::InternalError("failure".to_string()));
    assert!(failure.is_err());
}

#[test]
fn test_error_code_consistency() {
    // Test 4: Error codes are consistent for protocol compliance
    assert_eq!(MCPError::ValidationFailed("".to_string()).error_code(), "MCP-001");
    assert_eq!(MCPError::OperationFailed("".to_string()).error_code(), "MCP-002");
    assert_eq!(MCPError::InternalError("".to_string()).error_code(), "MCP-003");
}

#[test]
fn test_error_debug_formatting() {
    // Test 5: Error debug formatting works
    let error = MCPError::ValidationFailed("debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ValidationFailed"));
    assert!(debug_str.contains("debug test"));
}

#[test]
fn test_error_display_formatting() {
    // Test 6: Error display formatting works  
    let error = MCPError::Network("connection failed".to_string());
    let display_str = error.to_string();
    assert!(display_str.contains("Network error"));
    assert!(display_str.contains("connection failed"));
}

#[tokio::test]
async fn test_integration_module() {
    // Test 7: Basic integration module works
    use squirrel::integration::SimpleMCPIntegration;
    
    let mut integration = SimpleMCPIntegration::new();
    assert!(!integration.is_initialized());
    
    integration.initialize().await.expect("Failed to initialize");
    assert!(integration.is_initialized());
}

// Note: All other tests removed because they test functionality moved to other projects:
// ❌ Web integration tests → Songbird  
// ❌ Storage/compute tests → ToadStool/NestGate
// ❌ Security tests → BearDog
// ❌ Complex monitoring → Distributed
// ❌ Session/transport tests → May belong elsewhere
// ❌ Tool management tests → May belong elsewhere 