//! Simple test for the squirrel crate
//! This test verifies that the basic API works correctly.

use squirrel::error::Result;
use squirrel::{PrimalError, VERSION};

#[test]
fn test_version() {
    // Test that VERSION is defined and not empty
    assert!(!VERSION.is_empty());
    println!("Version: {VERSION}");
}

#[test]
fn test_basic_error_creation() {
    // Test that we can create basic errors
    let error = PrimalError::Internal("test error".to_string());
    assert!(error.to_string().contains("test error"));
}

#[test]
fn test_result_handling() {
    // Test that Result<T> works correctly
    let success: Result<String> = Ok("success".to_string());
    assert!(success.is_ok());
    assert_eq!(success.unwrap(), "success");

    let failure: Result<String> = Err(PrimalError::Internal("failure".to_string()));
    assert!(failure.is_err());
}

#[tokio::test]
async fn test_integration_module() {
    // Test that the integration module works
    use squirrel::integration::SimpleMCPIntegration;

    let mut integration = SimpleMCPIntegration::new();
    assert!(!integration.is_initialized());

    integration
        .initialize()
        .await
        .expect("Failed to initialize");
    assert!(integration.is_initialized());
}
