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
//! MCP Core Tests
//!
//! Tests for core Machine Context Protocol functionality in Squirrel.
//! Tests the actual `MCPError` implementation and error codes.

use squirrel_mcp::MCPError;

#[cfg(test)]
mod core_functionality {
    use super::*;

    #[test]
    fn test_mcp_error_types() {
        // Test core error handling - fundamental MCP functionality
        let validation_error = MCPError::Validation("test validation".to_string());
        let _ = validation_error.to_string();
        assert_eq!(validation_error.error_code(), "MCP-023");

        let internal_error = MCPError::Internal("test internal".to_string());
        let _ = internal_error.to_string();
        assert_eq!(internal_error.error_code(), "MCP-057");
    }

    #[test]
    fn test_mcp_result_handling() {
        // Test Result type usage - core to MCP protocol
        let success: Result<String, MCPError> = Ok("success".to_string());
        assert!(matches!(success.as_ref(), Ok(s) if s == "success"));

        let failure: Result<String, MCPError> = Err(MCPError::Internal("failure".to_string()));
        assert!(failure.is_err());
    }

    #[test]
    fn test_error_code_consistency() {
        // Verify error codes are consistent - important for protocol compliance
        assert_eq!(MCPError::Validation(String::new()).error_code(), "MCP-023");
        assert_eq!(
            MCPError::OperationFailed(String::new()).error_code(),
            "MCP-049"
        );
        assert_eq!(MCPError::Internal(String::new()).error_code(), "MCP-057");
        assert_eq!(
            MCPError::InternalError(String::new()).error_code(),
            "MCP-044"
        );
        assert_eq!(MCPError::Network(String::new()).error_code(), "MCP-053");
        assert_eq!(
            MCPError::Configuration(String::new()).error_code(),
            "MCP-051"
        );
    }

    #[test]
    fn test_error_category_str() {
        // Test category string representation
        assert_eq!(
            MCPError::Validation(String::new()).category_str(),
            "VALIDATION"
        );
        assert_eq!(MCPError::Internal(String::new()).category_str(), "INTERNAL");
        assert_eq!(MCPError::Network(String::new()).category_str(), "NETWORK");
    }

    #[test]
    fn test_transport_and_protocol_errors() {
        // Test that error codes for core MCP layers are correct
        assert_eq!(
            MCPError::Transport(squirrel_mcp::error::TransportError::ConnectionFailed(
                "test".to_string()
            ))
            .error_code(),
            "MCP-001"
        );
        assert_eq!(
            MCPError::Protocol(squirrel_mcp::error::ProtocolError::InvalidVersion(
                "test".to_string()
            ))
            .error_code(),
            "MCP-002"
        );
    }

    #[test]
    fn test_common_error_codes() {
        // Test common error variants and their codes
        let test_cases = vec![
            (MCPError::Timeout(String::new()), "MCP-033"),
            (MCPError::Authentication(String::new()), "MCP-058"),
            (MCPError::Authorization(String::new()), "MCP-013"),
            (MCPError::NotFound(String::new()), "MCP-039"),
            (MCPError::InvalidArgument(String::new()), "MCP-038"),
            (MCPError::RateLimit(String::new()), "MCP-059"),
        ];

        for (error, expected_code) in test_cases {
            assert_eq!(error.error_code(), expected_code);
        }
    }

    #[test]
    fn test_error_pattern_matching() {
        // Test pattern matching on errors
        let error = MCPError::Validation("test message".to_string());
        match error {
            MCPError::Validation(msg) => assert_eq!(msg, "test message"),
            _ => unreachable!("Wrong error variant"),
        }
    }

    #[test]
    fn test_error_clone() {
        // MCPError should be Clone
        let error = MCPError::General("test".to_string());
        let cloned = error.clone();
        assert_eq!(error.error_code(), cloned.error_code());
    }
}
