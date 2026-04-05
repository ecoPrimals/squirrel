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
//! Minimal MCP Core Tests
//!
//! Tests for the most basic core Machine Context Protocol functionality.
//! This tests the actual `MCPError` types that exist in the codebase.

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
        assert!(matches!(success.as_ref(), Ok(s) if s == "success"));

        let failure: Result<String, MCPError> = Err(MCPError::Internal("failure".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_matching() {
        // Test pattern matching on errors
        let error = MCPError::Validation("test".to_string());
        match error {
            MCPError::Validation(msg) => assert_eq!(msg, "test"),
            _ => unreachable!("Wrong error type"),
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
