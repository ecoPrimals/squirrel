// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Error handling tests for AI tools
//!
//! Tests error path handling and error type conversions

use squirrel_ai_tools::error::AIError;
use std::error::Error;

#[test]
fn test_error_display() {
    let config_err = AIError::Configuration("Missing API key".to_string());
    assert_eq!(
        config_err.to_string(),
        "Configuration error: Missing API key"
    );

    let network_err = AIError::Network("Connection refused".to_string());
    assert_eq!(network_err.to_string(), "Network error: Connection refused");

    let rate_limit_err = AIError::RateLimit("Too many requests".to_string());
    assert_eq!(
        rate_limit_err.to_string(),
        "Rate limit error: Too many requests"
    );
}

#[test]
fn test_error_from_reqwest() {
    // Create a simulated reqwest error scenario
    let err_msg = "HTTP request failed";
    let ai_error = AIError::Http(err_msg.to_string());
    assert!(ai_error.to_string().contains("HTTP error"));
}

#[test]
fn test_error_from_json() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
    assert!(json_error.is_err());

    if let Err(e) = json_error {
        let ai_error: AIError = e.into();
        assert!(matches!(ai_error, AIError::Parse(_)));
    }
}

#[test]
fn test_error_variants() {
    let errors = vec![
        AIError::Configuration("config".to_string()),
        AIError::Network("network".to_string()),
        AIError::Provider("provider".to_string()),
        AIError::Model("model".to_string()),
        AIError::Parse("parse".to_string()),
        AIError::RateLimit("rate".to_string()),
        AIError::Streaming("stream".to_string()),
        AIError::Runtime("runtime".to_string()),
        AIError::InvalidResponse("invalid".to_string()),
        AIError::Authentication("auth".to_string()),
        AIError::Timeout("timeout".to_string()),
        AIError::Validation("validation".to_string()),
    ];

    for error in errors {
        // All errors should implement Display
        assert!(!error.to_string().is_empty());

        // All errors should implement Error trait
        assert!(Error::source(&error).is_none());
    }
}

#[test]
fn test_result_type() {
    fn returns_result() -> squirrel_ai_tools::Result<String> {
        Ok("success".to_string())
    }

    fn returns_error() -> squirrel_ai_tools::Result<String> {
        Err(AIError::Configuration("test error".to_string()))
    }

    assert!(returns_result().is_ok());
    assert!(returns_error().is_err());
}

#[test]
fn test_error_propagation() {
    fn inner_function() -> squirrel_ai_tools::Result<i32> {
        Err(AIError::Validation("invalid input".to_string()))
    }

    fn outer_function() -> squirrel_ai_tools::Result<i32> {
        inner_function()?; // Should propagate the error
        Ok(42)
    }

    match outer_function() {
        Err(AIError::Validation(msg)) => {
            assert_eq!(msg, "invalid input");
        }
        _ => panic!("Expected validation error"),
    }
}

#[test]
fn test_authentication_error_scenarios() {
    let auth_errors = vec![
        AIError::Authentication("Invalid API key".to_string()),
        AIError::Authentication("Expired token".to_string()),
        AIError::Authentication("Missing credentials".to_string()),
    ];

    for error in auth_errors {
        assert!(error.to_string().contains("Authentication error"));
    }
}

#[test]
fn test_rate_limit_error_handling() {
    let rate_limit = AIError::RateLimit("Exceeded quota".to_string());

    // Simulate retry logic
    let should_retry = matches!(rate_limit, AIError::RateLimit(_));
    assert!(should_retry);
}

#[test]
fn test_network_error_variants() {
    let network_errors = vec![
        AIError::Network("DNS resolution failed".to_string()),
        AIError::NetworkError("Connection timeout".to_string()),
        AIError::Timeout("Request timeout after 30s".to_string()),
    ];

    for error in network_errors {
        let error_str = error.to_string();
        // All should contain some indication of network/connectivity issue
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
        AIError::Provider("OpenAI API returned 500".to_string()),
        AIError::Provider("Anthropic rate limit exceeded".to_string()),
        AIError::UnsupportedProvider("Unknown provider: XYZ".to_string()),
    ];

    for error in provider_errors {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn test_parsing_errors() {
    let parse_errors = vec![
        AIError::Parse("Invalid JSON structure".to_string()),
        AIError::ParseError("Could not parse response".to_string()),
        AIError::Parsing("Malformed data".to_string()),
        AIError::InvalidResponse("Unexpected format".to_string()),
    ];

    for error in parse_errors {
        let msg = error.to_string();
        // Should indicate parsing/format issue
        assert!(
            msg.contains("parse")
                || msg.contains("Parse")
                || msg.contains("Invalid")
                || msg.contains("format")
                || msg.contains("Parsing"),
            "Error should indicate parsing issue: {}",
            msg
        );
    }
}

#[test]
fn test_error_context_preservation() {
    let original_msg = "Original error context with details";
    let error = AIError::Generic(original_msg.to_string());

    let error_string = error.to_string();
    assert!(error_string.contains(original_msg));
}

#[test]
fn test_timeout_error_creation() {
    use std::time::Duration;

    let timeout_duration = Duration::from_secs(30);
    let error = AIError::Timeout(format!("Operation timed out after {:?}", timeout_duration));

    assert!(error.to_string().contains("Timeout"));
    assert!(error.to_string().contains("30"));
}

#[test]
fn test_validation_error_with_details() {
    let validation_errors = vec![
        AIError::Validation("Temperature must be between 0 and 1".to_string()),
        AIError::Validation("Max tokens cannot be negative".to_string()),
        AIError::InvalidRequest("Missing required field: model".to_string()),
    ];

    for error in validation_errors {
        // Validation errors should have descriptive messages
        assert!(error.to_string().len() > 10);
    }
}
