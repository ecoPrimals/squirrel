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
//! Basic MCP Core tests
//!
//! Minimal tests to verify core error handling works with the actual `MCPError` implementation.

use squirrel_mcp::MCPError;

#[test]
fn test_mcp_error_general() {
    let error = MCPError::General("test error".to_string());
    // Just verify the error can be created and displayed
    let _ = error.to_string();
    let _ = format!("{error:?}");
}

#[test]
fn test_mcp_error_validation() {
    let error = MCPError::Validation("validation failed".to_string());
    let _ = error.to_string();
    let _ = format!("{error:?}");
}

#[test]
fn test_mcp_error_internal() {
    let error = MCPError::Internal("internal error".to_string());
    let _ = error.to_string();
    let _ = format!("{error:?}");
}

#[test]
fn test_mcp_result_ok() {
    let result: Result<String, MCPError> = Ok("success".to_string());
    assert!(matches!(result.as_ref(), Ok(s) if s == "success"));
}

#[test]
fn test_mcp_result_error() {
    let result: Result<String, MCPError> = Err(MCPError::Internal("test".to_string()));
    assert!(result.is_err());
    match result {
        Err(MCPError::Internal(msg)) => assert_eq!(msg, "test"),
        _ => unreachable!("Wrong error type"),
    }
}

#[test]
fn test_error_variants_can_be_created() {
    // Test that various error variants can be created
    let errors = vec![
        MCPError::Network("network error".to_string()),
        MCPError::Configuration("config error".to_string()),
        MCPError::InvalidArgument("invalid arg".to_string()),
        MCPError::NotFound("not found".to_string()),
        MCPError::Authentication("auth failed".to_string()),
        MCPError::Authorization("authz failed".to_string()),
        MCPError::Timeout("timeout".to_string()),
        MCPError::RateLimit("rate limit".to_string()),
        MCPError::InvalidState("invalid state".to_string()),
        MCPError::General("general".to_string()),
    ];

    // Verify all can be displayed
    for error in errors {
        let _ = error.to_string();
        let _ = format!("{error:?}");
    }
}

#[test]
fn test_error_pattern_matching() {
    let error = MCPError::Validation("test validation".to_string());
    match error {
        MCPError::Validation(msg) => assert_eq!(msg, "test validation"),
        _ => unreachable!("Wrong variant"),
    }
}

#[test]
fn test_error_is_send_sync() {
    // Verify MCPError implements Send + Sync (required for async)
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MCPError>();
}

#[test]
fn test_error_clone() {
    let error = MCPError::General("test".to_string());
    let cloned = error.clone();
    match (error, cloned) {
        (MCPError::General(m1), MCPError::General(m2)) => assert_eq!(m1, m2),
        _ => unreachable!("Clone failed"),
    }
}
