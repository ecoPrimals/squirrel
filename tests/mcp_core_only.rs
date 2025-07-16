//! MCP Core Only Tests
//!
//! Tests ONLY the core MCP functionality that should remain in Squirrel.
//! Everything else was moved to other projects during the tearout:
//! - Web → Songbird
//! - Compute/Storage → ToadStool/NestGate  
//! - Security → BearDog
//! - Complex monitoring → Distributed

use squirrel::error::Result;
use squirrel::{PrimalError, VERSION};

#[test]
fn test_mcp_core_version() {
    // Test 1: Version information is available
    assert!(!VERSION.is_empty());
    println!("MCP Core Version: {VERSION}");
}

#[test]
fn test_mcp_error_creation() {
    // Test 2: Core error types work
    let validation_error = PrimalError::Configuration("test validation".to_string());
    assert!(validation_error.to_string().contains("test validation"));

    let operation_error = PrimalError::Protocol("test operation".to_string());
    assert!(operation_error.to_string().contains("test operation"));

    let internal_error = PrimalError::Internal("test internal".to_string());
    assert!(internal_error.to_string().contains("test internal"));
}

#[test]
fn test_mcp_result_handling() {
    // Test 3: Result type usage
    let success: Result<String> = Ok("success".to_string());
    assert!(success.is_ok());
    assert_eq!(success.unwrap(), "success");

    let failure: Result<String> = Err(PrimalError::Internal("failure".to_string()));
    assert!(failure.is_err());
}

#[test]
fn test_error_code_consistency() {
    // Test 4: Error codes are consistent for protocol compliance
    // Note: Using simple string checks since PrimalError doesn't have error_code method
    assert!(PrimalError::Configuration("".to_string())
        .to_string()
        .contains("Configuration"));
    assert!(PrimalError::Protocol("".to_string())
        .to_string()
        .contains("Protocol"));
    assert!(PrimalError::Internal("".to_string())
        .to_string()
        .contains("Internal"));
}

#[test]
fn test_error_debug_formatting() {
    // Test 5: Error debug formatting works
    let error = PrimalError::Configuration("debug test".to_string());
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("Configuration"));
    assert!(debug_str.contains("debug test"));
}

#[test]
fn test_error_display_formatting() {
    // Test 6: Error display formatting works
    let error = PrimalError::Network("connection failed".to_string());
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

    integration
        .initialize()
        .await
        .expect("Failed to initialize");
    assert!(integration.is_initialized());
}

// Note: All other tests removed because they test functionality moved to other projects:
// ❌ Web integration tests → Songbird
// ❌ Storage/compute tests → ToadStool/NestGate
// ❌ Security tests → BearDog
// ❌ Complex monitoring → Distributed
// ❌ Session/transport tests → May belong elsewhere
// ❌ Tool management tests → May belong elsewhere
