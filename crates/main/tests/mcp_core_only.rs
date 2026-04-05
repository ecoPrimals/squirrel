// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! MCP Core Only Tests
//!
//! Tests ONLY the core MCP functionality that remains in Squirrel.
//! Other concerns were moved to sibling primals during the tearout:
//! - Web/orchestration → service mesh (discovered at runtime)
//! - Compute/Storage → ToadStool/NestGate
//! - Security → BearDog
//! - Complex monitoring → distributed observability

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
    assert!(matches!(success.as_ref(), Ok(s) if s == "success"));

    let failure: Result<String> = Err(PrimalError::Internal("failure".to_string()));
    assert!(failure.is_err());
}

#[test]
fn test_error_code_consistency() {
    // Test 4: Error codes are consistent for protocol compliance
    // Note: Using simple string checks since PrimalError doesn't have error_code method
    assert!(
        PrimalError::Configuration(String::new())
            .to_string()
            .contains("Configuration")
    );
    assert!(
        PrimalError::OperationFailed(String::new())
            .to_string()
            .contains("Operation failed")
    );
    assert!(
        PrimalError::Internal(String::new())
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

// Remaining tests were moved to their respective primals during the tearout:
// Web/orchestration → service mesh, Storage/compute → ToadStool/NestGate,
// Security → BearDog, Monitoring → distributed observability.
