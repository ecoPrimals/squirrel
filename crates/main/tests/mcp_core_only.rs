// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! MCP Core Only Tests
//!
//! Tests ONLY the core MCP functionality that should remain in Squirrel.
//! Everything else was moved to other projects during the tearout:
//! - Web → Songbird
//! - Compute/Storage → ToadStool/NestGate  
//! - Security → BearDog
//! - Complex monitoring → Distributed

type Result<T> = std::result::Result<T, squirrel::error::PrimalError>;
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

    let operation_error = PrimalError::OperationFailed("test operation".to_string());
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
    assert!(
        PrimalError::Configuration("".to_string())
            .to_string()
            .contains("Configuration")
    );
    assert!(
        PrimalError::OperationFailed("".to_string())
            .to_string()
            .contains("Operation failed")
    );
    assert!(
        PrimalError::Internal("".to_string())
            .to_string()
            .contains("Internal")
    );
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

// NOTE: Integration module test removed - functionality moved to capability-based discovery
// The SimpleMCPIntegration was part of deprecated hardcoded primal modules
// Use CapabilityRegistry for service discovery instead

// Note: All other tests removed because they test functionality moved to other projects:
// ❌ Web integration tests → Songbird
// ❌ Storage/compute tests → ToadStool/NestGate
// ❌ Security tests → BearDog
// ❌ Complex monitoring → Distributed
// ❌ Session/transport tests → May belong elsewhere
// ❌ Tool management tests → May belong elsewhere
