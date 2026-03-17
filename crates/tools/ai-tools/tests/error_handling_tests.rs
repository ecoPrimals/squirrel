// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Error handling tests for AI tools
//!
//! Tests error path handling and error type conversions.
//! Uses `universal_error::tools::AIToolsError` (capability-based error type).

use std::error::Error;
use universal_error::tools::AIToolsError;

#[test]
fn test_error_display() {
    let config_err = AIToolsError::Configuration("Missing API key".to_string());
    assert_eq!(
        config_err.to_string(),
        "Configuration error: Missing API key"
    );

    let network_err = AIToolsError::Network("Connection refused".to_string());
    assert_eq!(network_err.to_string(), "Network error: Connection refused");

    let rate_limit_err = AIToolsError::RateLimitExceeded("Too many requests".to_string());
    assert_eq!(
        rate_limit_err.to_string(),
        "Rate limit exceeded for Too many requests"
    );
}

#[test]
fn test_error_from_api() {
    let err_msg = "HTTP request failed";
    let ai_error = AIToolsError::Api(err_msg.to_string());
    assert!(ai_error.to_string().contains("API error"));
}

#[test]
fn test_error_from_json() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());

    if let Err(e) = json_error {
        let ai_error: AIToolsError = AIToolsError::Parse(e.to_string());
        assert!(ai_error.to_string().contains("Parse error"));
    }
}

#[test]
fn test_error_variants() {
    let errors: Vec<AIToolsError> = vec![
        AIToolsError::Configuration("config".to_string()),
        AIToolsError::Network("network".to_string()),
        AIToolsError::Provider("provider".to_string()),
        AIToolsError::ModelNotFound("model".to_string()),
        AIToolsError::Parse("parse".to_string()),
        AIToolsError::RateLimitExceeded("rate".to_string()),
        AIToolsError::InvalidResponse("invalid".to_string()),
        AIToolsError::Authentication("auth".to_string()),
        AIToolsError::InvalidRequest("validation".to_string()),
        AIToolsError::Api("api".to_string()),
        AIToolsError::UnsupportedProvider("unsupported".to_string()),
    ];

    for error in errors {
        assert!(!error.to_string().is_empty());
        assert!(Error::source(&error).is_none());
    }
}

#[test]
fn test_result_type() {
    fn returns_result() -> Result<String, AIToolsError> {
        Ok("success".to_string())
    }

    fn returns_error() -> Result<String, AIToolsError> {
        Err(AIToolsError::Configuration("test error".to_string()))
    }

    assert!(returns_result().is_ok());
    assert!(returns_error().is_err());
}

#[test]
fn test_error_propagation() {
    fn inner_function() -> Result<i32, AIToolsError> {
        Err(AIToolsError::InvalidRequest("invalid input".to_string()))
    }

    fn outer_function() -> Result<i32, AIToolsError> {
        inner_function()?;
        Ok(42)
    }

    match outer_function() {
        Err(AIToolsError::InvalidRequest(msg)) => {
            assert_eq!(msg, "invalid input");
        }
        _ => panic!("Expected invalid request error"),
    }
}

#[test]
fn test_authentication_error_scenarios() {
    let auth_errors = vec![
        AIToolsError::Authentication("Invalid API key".to_string()),
        AIToolsError::Authentication("Expired token".to_string()),
        AIToolsError::Authentication("Missing credentials".to_string()),
    ];

    for error in auth_errors {
        assert!(error.to_string().contains("Authentication error"));
    }
}

#[test]
fn test_rate_limit_error_handling() {
    let rate_limit = AIToolsError::RateLimitExceeded("Exceeded quota".to_string());

    let should_retry = matches!(rate_limit, AIToolsError::RateLimitExceeded(_));
    assert!(should_retry);
}

#[test]
fn test_network_error_variants() {
    let network_errors = vec![
        AIToolsError::Network("DNS resolution failed".to_string()),
        AIToolsError::Network("Connection timeout".to_string()),
        AIToolsError::Network("Request timeout after 30s".to_string()),
    ];

    for error in network_errors {
        let error_str = error.to_string();
        assert!(
            error_str.contains("error") || error_str.contains("timeout"),
            "Error message should indicate connectivity issue: {}",
            error_str
        );
    }
}

#[test]
fn test_provider_specific_errors() {
    let provider_errors = vec![
        AIToolsError::Provider("OpenAI API returned 500".to_string()),
        AIToolsError::Provider("Anthropic rate limit exceeded".to_string()),
        AIToolsError::UnsupportedProvider("Unknown provider: XYZ".to_string()),
    ];

    for error in provider_errors {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn test_parsing_errors() {
    let parse_errors = vec![
        AIToolsError::Parse("Invalid JSON structure".to_string()),
        AIToolsError::Parse("Could not parse response".to_string()),
        AIToolsError::InvalidResponse("Unexpected format".to_string()),
    ];

    for error in parse_errors {
        let msg = error.to_string();
        assert!(
            msg.contains("parse")
                || msg.contains("Parse")
                || msg.contains("Invalid")
                || msg.contains("format"),
            "Error should indicate parsing issue: {}",
            msg
        );
    }
}

#[test]
fn test_error_context_preservation() {
    let original_msg = "Original error context with details";
    let error = AIToolsError::Configuration(original_msg.to_string());

    let error_string = error.to_string();
    assert!(error_string.contains(original_msg));
}

#[test]
fn test_timeout_error_creation() {
    use std::time::Duration;

    let timeout_duration = Duration::from_secs(30);
    let error = AIToolsError::Network(format!("Operation timed out after {:?}", timeout_duration));

    let err_str = error.to_string();
    assert!(err_str.contains("timed out") || err_str.contains("timeout"));
    assert!(err_str.contains("30"));
}

#[test]
fn test_validation_error_with_details() {
    let validation_errors = vec![
        AIToolsError::InvalidRequest("Temperature must be between 0 and 1".to_string()),
        AIToolsError::InvalidRequest("Max tokens cannot be negative".to_string()),
        AIToolsError::InvalidRequest("Missing required field: model".to_string()),
    ];

    for error in validation_errors {
        assert!(error.to_string().len() > 10);
    }
}
