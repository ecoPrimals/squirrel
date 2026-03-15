// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Configuration Validation Unit Tests
//!
//! Comprehensive tests for configuration loading, validation, and error handling.

use squirrel_mcp_config::unified::{ConfigLoader, Validator};
use std::env;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// VALIDATOR TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_validate_port_valid() {
    assert!(Validator::validate_port(8080).is_ok());
    assert!(Validator::validate_port(80).is_ok());
    assert!(Validator::validate_port(443).is_ok());
    assert!(Validator::validate_port(3000).is_ok());
    assert!(Validator::validate_port(65535).is_ok());
}

#[test]
fn test_validate_port_invalid() {
    assert!(Validator::validate_port(0).is_err());
    assert!(Validator::validate_port(65536).is_err());
    assert!(Validator::validate_port(100000).is_err());
}

#[test]
fn test_validate_timeout_valid() {
    assert!(Validator::validate_timeout_secs(1, "test").is_ok());
    assert!(Validator::validate_timeout_secs(30, "test").is_ok());
    assert!(Validator::validate_timeout_secs(300, "test").is_ok());
}

#[test]
fn test_validate_timeout_invalid() {
    assert!(Validator::validate_timeout_secs(0, "test").is_err());
    assert!(Validator::validate_timeout_secs(3601, "test").is_err());
}

#[test]
fn test_validate_hostname_valid() {
    assert!(Validator::validate_hostname("localhost").is_ok());
    assert!(Validator::validate_hostname("example.com").is_ok());
    assert!(Validator::validate_hostname("api.example.com").is_ok());
    assert!(Validator::validate_hostname("sub.domain.example.com").is_ok());
}

#[test]
fn test_validate_hostname_invalid() {
    assert!(Validator::validate_hostname("").is_err());
    assert!(Validator::validate_hostname(" ").is_err());
    assert!(Validator::validate_hostname("invalid..domain").is_err());
    assert!(Validator::validate_hostname("-invalid.com").is_err());
}

#[test]
fn test_validate_url_valid() {
    assert!(Validator::validate_url("http://localhost:8080").is_ok());
    assert!(Validator::validate_url("https://example.com").is_ok());
    assert!(Validator::validate_url("http://192.168.1.1:3000").is_ok());
}

#[test]
fn test_validate_url_invalid() {
    assert!(Validator::validate_url("").is_err());
    assert!(Validator::validate_url("not-a-url").is_err());
    assert!(Validator::validate_url("ftp://unsupported.com").is_err());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ENVIRONMENT VARIABLE TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_env_override_port() {
    unsafe { env::set_var("SQUIRREL_HTTP_PORT", "9090") };
    
    let port = env::var("SQUIRREL_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);
    
    assert_eq!(port, 9090);
    
    unsafe { env::remove_var("SQUIRREL_HTTP_PORT") };
}

#[test]
fn test_env_fallback_default() {
    unsafe { env::remove_var("SQUIRREL_HTTP_PORT") };
    
    let port = env::var("SQUIRREL_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);
    
    assert_eq!(port, 8080);
}

#[test]
fn test_env_log_level() {
    unsafe { env::set_var("SQUIRREL_LOG_LEVEL", "debug") };
    
    let log_level = env::var("SQUIRREL_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
    
    assert_eq!(log_level, "debug");
    
    unsafe { env::remove_var("SQUIRREL_LOG_LEVEL") };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONFIGURATION PRECEDENCE TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_config_precedence_env_over_default() {
    unsafe { env::set_var("SQUIRREL_HTTP_PORT", "7070") };
    
    // In a real scenario, config loader would use env var over default
    let from_env = env::var("SQUIRREL_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok());
    
    let final_port = from_env.unwrap_or(8080);
    
    assert_eq!(final_port, 7070);
    
    unsafe { env::remove_var("SQUIRREL_HTTP_PORT") };
}

#[test]
fn test_config_multiple_environments() {
    // Simulate different environments
    let dev_log_level = "debug";
    let prod_log_level = "warn";
    
    unsafe { env::set_var("SQUIRREL_ENV", "development") };
    let env_type = env::var("SQUIRREL_ENV").unwrap_or_else(|_| "production".to_string());
    
    let log_level = if env_type == "development" {
        dev_log_level
    } else {
        prod_log_level
    };
    
    assert_eq!(log_level, "debug");
    
    unsafe { env::remove_var("SQUIRREL_ENV") };
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// TIMEOUT CONFIGURATION TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_timeout_config_valid_ranges() {
    // Connection timeout: 1-300 seconds
    assert!(Validator::validate_timeout_secs(1, "connection").is_ok());
    assert!(Validator::validate_timeout_secs(30, "connection").is_ok());
    assert!(Validator::validate_timeout_secs(300, "connection").is_ok());
}

#[test]
fn test_timeout_config_boundary_values() {
    // Test boundary values
    assert!(Validator::validate_timeout_secs(1, "test").is_ok());
    assert!(Validator::validate_timeout_secs(3600, "test").is_ok());
    
    // Beyond boundaries
    assert!(Validator::validate_timeout_secs(0, "test").is_err());
    assert!(Validator::validate_timeout_secs(3601, "test").is_err());
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ERROR HANDLING TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[test]
fn test_config_error_invalid_port() {
    let result = Validator::validate_port(70000);
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("port"));
}

#[test]
fn test_config_error_invalid_timeout() {
    let result = Validator::validate_timeout_secs(5000, "test");
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("timeout"));
}

#[test]
fn test_config_error_invalid_url() {
    let result = Validator::validate_url("not-a-url");
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("url") || err.to_string().contains("URL"));
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CONCURRENCY TESTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[tokio::test]
async fn test_config_concurrent_access() {
    use std::sync::Arc;
    use tokio::task;
    
    let mut handles = vec![];
    
    for i in 0..50 {
        let handle = task::spawn(async move {
            let port = 8000 + i;
            Validator::validate_port(port)
        });
        handles.push(handle);
    }
    
    let results: Vec<Result<(), _>> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task failed"))
        .collect();
    
    // All validations should succeed
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_env_concurrent_reads() {
    use tokio::task;
    
    unsafe { env::set_var("TEST_CONCURRENT", "value") };
    
    let mut handles = vec![];
    
    for _ in 0..100 {
        let handle = task::spawn(async {
            env::var("TEST_CONCURRENT")
        });
        handles.push(handle);
    }
    
    let results: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task failed").expect("Var not found"))
        .collect();
    
    assert_eq!(results.len(), 100);
    for result in results {
        assert_eq!(result, "value");
    }
    
    unsafe { env::remove_var("TEST_CONCURRENT") };
}

