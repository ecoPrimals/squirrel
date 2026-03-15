// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication error handling utilities
//!
//! This module provides safer alternatives to unwrap/expect patterns in production code

use crate::AuthError;
use tracing::{error, warn};

/// Safe extraction of authentication results with proper error logging
pub fn extract_auth_result<T>(
    result: Result<T, AuthError>, 
    operation: &str
) -> Result<T, AuthError> {
    match result {
        Ok(value) => {
            tracing::debug!("✅ Auth operation '{}' succeeded", operation);
            Ok(value)
        }
        Err(e) => {
            error!("🚨 Auth operation '{}' failed: {}", operation, e);
            Err(e)
        }
    }
}

/// Safe extraction with fallback value for non-critical operations
pub fn extract_with_fallback<T>(
    result: Result<T, AuthError>, 
    fallback: T,
    operation: &str
) -> T {
    match result {
        Ok(value) => value,
        Err(e) => {
            warn!("⚠️ Auth operation '{}' failed, using fallback: {}", operation, e);
            fallback
        }
    }
}

/// Safe extraction for Option types with meaningful error messages
pub fn extract_option<T>(
    option: Option<T>,
    operation: &str
) -> Result<T, AuthError> {
    option.ok_or_else(|| {
        let msg = format!("Required value missing for operation: {}", operation);
        error!("🚨 {}", msg);
        AuthError::InvalidInput(msg)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_auth_result_success() {
        let result = Ok("success");
        let extracted = extract_auth_result(result, "test_operation");
        assert!(extracted.is_ok());
        assert_eq!(extracted.unwrap(), "success");
    }

    #[test]
    fn test_extract_auth_result_failure() {
        let result: Result<&str, AuthError> = Err(AuthError::TokenExpired);
        let extracted = extract_auth_result(result, "test_operation");
        assert!(extracted.is_err());
    }

    #[test]
    fn test_extract_with_fallback() {
        let result: Result<i32, AuthError> = Err(AuthError::TokenExpired);
        let value = extract_with_fallback(result, 42, "test_operation");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_extract_option_some() {
        let option = Some("value");
        let extracted = extract_option(option, "test_operation");
        assert!(extracted.is_ok());
        assert_eq!(extracted.unwrap(), "value");
    }

    #[test]
    fn test_extract_option_none() {
        let option: Option<&str> = None;
        let extracted = extract_option(option, "test_operation");
        assert!(extracted.is_err());
    }
} 