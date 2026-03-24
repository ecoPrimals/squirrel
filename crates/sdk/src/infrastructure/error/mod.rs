// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive error handling system for the Squirrel Plugin SDK
//!
//! **DEPRECATED**: This error system is being replaced by `universal-error`.
//! Please migrate to the unified error system for all new code.
//!
//! Migration guide:
//! ```text
//! // Old:
//! use crate::infrastructure::error::*;
//! // New:
//! use universal_error::{Result, sdk::*};
//! ```
//!
//! See: `crates/universal-error/README.md` for complete migration guide.
//!
//! This module provides a complete error handling framework with:
//! - Core error types and result types
//! - Error context and enhanced error chaining
//! - Error severity and categorization
//! - Validation errors and helper functions
//! - WASM compatibility and error conversions
//! - Helper macros for easy error creation
//! - Comprehensive test coverage
//!
//! # Architecture
//!
//! The error system is organized into focused modules:
//!
//! - [`core`] - Main PluginError enum and result types
//! - [`context`] - Error context, enhanced errors, and chaining
//! - [`severity`] - Error severity levels and categorization
//! - [`validation`] - Validation errors and helper functions
//! - [`conversions`] - WASM compatibility and From implementations
//! - [`macros`] - Helper macros for error creation
//! - `tests` - Comprehensive test suite
//!
//! # Usage
//!
//! ```ignore
//! use crate::infrastructure::error::*;
//! use crate::{param_error, missing_param, network_error};
//!
//! // Create basic errors
//! let error = param_error!("name", "cannot be empty");
//! let missing = missing_param!("required_field");
//! let network = network_error!("fetch", "connection timeout");
//!
//! // Create enhanced errors with context
//! let context = ErrorContext::new("user_registration")
//!     .with_module("auth")
//!     .with_function("register_user");
//!
//! let enhanced = error.with_context(context);
//!
//! // Handle results
//! fn validate_user(data: &str) -> PluginResult<User> {
//!     if data.is_empty() {
//!         return Err(param_error!("user_data", "cannot be empty"));
//!     }
//!     Ok(User::new(data))
//! }
//! ```

pub mod context;
pub mod conversions;
pub mod core;
pub mod macros;
pub mod severity;
pub mod validation;

#[cfg(test)]
pub mod tests;

// Re-export all public APIs for easy access
pub use self::context::{EnhancedError, EnhancedResult, ErrorContext, PluginErrorExt, ResultExt};
pub use self::core::{PluginError, PluginResult};
pub use self::severity::{ErrorCategory, ErrorSeverity, PluginErrorClassification};
pub use self::validation::{
    ValidationError, validate_array, validate_array_length, validate_boolean, validate_email,
    validate_enum_value, validate_integer_range, validate_non_empty_string, validate_numeric_range,
    validate_object, validate_optional_string, validate_required_fields, validate_required_number,
    validate_required_string, validate_string_length, validate_url,
};

// Re-export commonly used items for convenience
pub use self::core::PluginError as Error;
pub use self::core::PluginResult as Result;

/// Utility module for retry logic with exponential backoff
pub mod retry {
    use std::fmt::Debug;

    /// Configuration for retry logic (SDK-specific, lightweight)
    #[derive(Debug, Clone)]
    pub struct RetryConfig {
        /// Maximum number of retry attempts
        pub max_retries: u32,
        /// Initial delay in milliseconds before first retry
        pub initial_delay: u64,
        /// Multiplier for exponential backoff
        pub backoff_multiplier: f64,
        /// Maximum delay in milliseconds between retries
        pub max_delay: u64,
    }

    impl Default for RetryConfig {
        fn default() -> Self {
            Self {
                max_retries: 3,
                initial_delay: 1000,
                backoff_multiplier: 2.0,
                max_delay: 30000,
            }
        }
    }

    /// Retry a function with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(config: RetryConfig, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: Debug,
    {
        let mut delay = config.initial_delay;
        let mut attempts = 0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    attempts += 1;
                    if attempts > config.max_retries {
                        return Err(error);
                    }

                    // Wait before retrying
                    crate::infrastructure::utils::sleep_ms(delay).await;

                    // Exponential backoff
                    delay =
                        ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
                }
            }
        }
    }
}

// Common error creation patterns
impl PluginError {
    /// Create a user input error
    pub fn user_error(message: &str) -> Self {
        PluginError::InvalidParameter {
            name: "user_input".to_string(),
            reason: message.to_string(),
        }
    }

    /// Create a system error
    pub fn system_error(message: &str) -> Self {
        PluginError::InternalError {
            message: message.to_string(),
        }
    }

    /// Create a network error
    pub fn network_error(operation: &str, message: &str) -> Self {
        PluginError::NetworkError {
            operation: operation.to_string(),
            message: message.to_string(),
        }
    }

    /// Create a configuration error
    pub fn config_error(message: &str) -> Self {
        PluginError::ConfigurationError {
            message: message.to_string(),
        }
    }

    /// Create a permission error
    pub fn permission_error(operation: &str, reason: &str) -> Self {
        PluginError::PermissionDenied {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a resource error
    pub fn resource_error(resource: &str, _message: &str) -> Self {
        PluginError::ResourceNotFound {
            resource: resource.to_string(),
        }
    }

    /// Create a temporary failure error
    pub fn temporary_error(operation: &str, message: &str) -> Self {
        PluginError::TemporaryFailure {
            operation: operation.to_string(),
            message: message.to_string(),
        }
    }

    /// Create a validation error
    pub fn validation_error(field: &str, message: &str) -> Self {
        PluginError::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
}

/// Common result patterns
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;
/// Result type for network operations that may fail with plugin errors
pub type NetworkResult<T> = std::result::Result<T, PluginError>;
/// Result type for configuration operations that may fail with plugin errors  
pub type ConfigResult<T> = std::result::Result<T, PluginError>;

/// Error handling utilities
pub mod utils {
    #![allow(clippy::wildcard_imports)] // Aligned with parent module re-exports

    use super::*;

    /// Check if an error is recoverable
    pub fn is_recoverable_error(error: &PluginError) -> bool {
        use severity::PluginErrorClassification;
        error.is_recoverable()
    }

    /// Get error category as string
    pub fn get_error_category(error: &PluginError) -> &'static str {
        use severity::PluginErrorClassification;
        error.category().as_str()
    }

    /// Get error severity as string
    pub fn get_error_severity(error: &PluginError) -> &'static str {
        use severity::PluginErrorClassification;
        error.severity().as_str()
    }

    /// Get recovery suggestions for an error
    pub fn get_recovery_suggestions(error: &PluginError) -> Vec<String> {
        use severity::PluginErrorClassification;
        error.recovery_suggestions()
    }

    /// Format error for logging
    pub fn format_error_for_logging(error: &PluginError) -> String {
        use severity::PluginErrorClassification;
        format!(
            "[{}] {} ({}): {}",
            error.severity().as_str(),
            error.category().as_str(),
            error.error_type(),
            error
        )
    }

    /// Format enhanced error for logging
    pub fn format_enhanced_error_for_logging(error: &EnhancedError) -> String {
        format!(
            "[{}] {} ({}): {} - Operation: {}",
            error.severity.as_str(),
            error.category.as_str(),
            error.error.error_type(),
            error.error,
            error.context.operation
        )
    }

    /// Check if error should be retried
    pub fn should_retry_error(error: &PluginError) -> bool {
        use severity::PluginErrorClassification;
        match error.category() {
            ErrorCategory::Network => true,
            ErrorCategory::External => true,
            ErrorCategory::System => {
                matches!(error, PluginError::TemporaryFailure { .. })
            }
            _ => false,
        }
    }

    /// Get suggested retry delay for an error
    pub fn get_retry_delay(error: &PluginError, attempt: u32) -> u64 {
        match error {
            PluginError::RateLimitError { retry_after, .. } => *retry_after * 1000,
            PluginError::NetworkError { .. } => 1000 * (2u64.pow(attempt.min(5))),
            PluginError::TemporaryFailure { .. } => 2000 * (2u64.pow(attempt.min(4))),
            _ => 1000,
        }
    }
}

#[cfg(test)]
mod mod_tests {
    #![allow(deprecated)]
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = retry::RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, 1000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert_eq!(config.max_delay, 30000);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = retry::RetryConfig {
            max_retries: 5,
            initial_delay: 500,
            backoff_multiplier: 1.5,
            max_delay: 10000,
        };
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, 500);
    }

    #[test]
    fn test_plugin_error_user_error() {
        let err = PluginError::user_error("bad input");
        match err {
            PluginError::InvalidParameter { name, reason } => {
                assert_eq!(name, "user_input");
                assert_eq!(reason, "bad input");
            }
            _ => panic!("Expected InvalidParameter"),
        }
    }

    #[test]
    fn test_plugin_error_system_error() {
        let err = PluginError::system_error("crash");
        match err {
            PluginError::InternalError { message } => assert_eq!(message, "crash"),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_plugin_error_network_error() {
        let err = PluginError::network_error("fetch", "timeout");
        match err {
            PluginError::NetworkError { operation, message } => {
                assert_eq!(operation, "fetch");
                assert_eq!(message, "timeout");
            }
            _ => panic!("Expected NetworkError"),
        }
    }

    #[test]
    fn test_plugin_error_config_error() {
        let err = PluginError::config_error("bad config");
        match err {
            PluginError::ConfigurationError { message } => assert_eq!(message, "bad config"),
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_plugin_error_permission_error() {
        let err = PluginError::permission_error("write", "read only");
        match err {
            PluginError::PermissionDenied { operation, reason } => {
                assert_eq!(operation, "write");
                assert_eq!(reason, "read only");
            }
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn test_plugin_error_resource_error() {
        let err = PluginError::resource_error("config.json", "not found");
        match err {
            PluginError::ResourceNotFound { resource } => assert_eq!(resource, "config.json"),
            _ => panic!("Expected ResourceNotFound"),
        }
    }

    #[test]
    fn test_plugin_error_temporary_error() {
        let err = PluginError::temporary_error("api_call", "service busy");
        match err {
            PluginError::TemporaryFailure { operation, message } => {
                assert_eq!(operation, "api_call");
                assert_eq!(message, "service busy");
            }
            _ => panic!("Expected TemporaryFailure"),
        }
    }

    #[test]
    fn test_plugin_error_validation_error() {
        let err = PluginError::validation_error("email", "invalid format");
        match err {
            PluginError::ValidationError { field, message } => {
                assert_eq!(field, "email");
                assert_eq!(message, "invalid format");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_utils_is_recoverable_error() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        assert!(utils::is_recoverable_error(&err));

        let err = PluginError::SecurityViolation {
            violation: "unauthorized".into(),
        };
        assert!(!utils::is_recoverable_error(&err));
    }

    #[test]
    fn test_utils_get_error_category() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        assert_eq!(utils::get_error_category(&err), "NETWORK");
    }

    #[test]
    fn test_utils_get_error_severity() {
        let err = PluginError::SecurityViolation {
            violation: "test".into(),
        };
        assert_eq!(utils::get_error_severity(&err), "CRITICAL");
    }

    #[test]
    fn test_utils_get_recovery_suggestions() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        let suggestions = utils::get_recovery_suggestions(&err);
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_utils_format_error_for_logging() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        let formatted = utils::format_error_for_logging(&err);
        assert!(formatted.contains("HIGH"));
        assert!(formatted.contains("NETWORK"));
        assert!(formatted.contains("NetworkError"));
    }

    #[test]
    fn test_utils_format_enhanced_error_for_logging() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        let enhanced = err.with_operation("test_op");
        let formatted = utils::format_enhanced_error_for_logging(&enhanced);
        assert!(formatted.contains("HIGH"));
        assert!(formatted.contains("NETWORK"));
        assert!(formatted.contains("test_op"));
    }

    #[test]
    fn test_utils_should_retry_network() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        assert!(utils::should_retry_error(&err));
    }

    #[test]
    fn test_utils_should_retry_external() {
        let err = PluginError::ExternalServiceError {
            service: "api".into(),
            message: "down".into(),
        };
        assert!(utils::should_retry_error(&err));
    }

    #[test]
    fn test_utils_should_not_retry_temporary() {
        // TemporaryFailure maps to ErrorCategory::Unknown, not System
        // so should_retry_error returns false (only System+TemporaryFailure would retry)
        let err = PluginError::TemporaryFailure {
            operation: "call".into(),
            message: "retry".into(),
        };
        assert!(!utils::should_retry_error(&err));
    }

    #[test]
    fn test_utils_should_not_retry_security() {
        let err = PluginError::SecurityViolation {
            violation: "test".into(),
        };
        assert!(!utils::should_retry_error(&err));
    }

    #[test]
    fn test_utils_should_not_retry_user() {
        let err = PluginError::MissingParameter {
            parameter: "name".into(),
        };
        assert!(!utils::should_retry_error(&err));
    }

    #[test]
    fn test_utils_get_retry_delay_rate_limit() {
        let err = PluginError::RateLimitError {
            resource: "api".into(),
            retry_after: 60,
        };
        assert_eq!(utils::get_retry_delay(&err, 0), 60000);
    }

    #[test]
    fn test_utils_get_retry_delay_network_exponential() {
        let err = PluginError::NetworkError {
            operation: "fetch".into(),
            message: "timeout".into(),
        };
        assert_eq!(utils::get_retry_delay(&err, 0), 1000);
        assert_eq!(utils::get_retry_delay(&err, 1), 2000);
        assert_eq!(utils::get_retry_delay(&err, 2), 4000);
        assert_eq!(utils::get_retry_delay(&err, 3), 8000);
    }

    #[test]
    fn test_utils_get_retry_delay_temporary_exponential() {
        let err = PluginError::TemporaryFailure {
            operation: "call".into(),
            message: "retry".into(),
        };
        assert_eq!(utils::get_retry_delay(&err, 0), 2000);
        assert_eq!(utils::get_retry_delay(&err, 1), 4000);
    }

    #[test]
    fn test_utils_get_retry_delay_default() {
        let err = PluginError::Unknown {
            message: "???".into(),
        };
        assert_eq!(utils::get_retry_delay(&err, 0), 1000);
        assert_eq!(utils::get_retry_delay(&err, 5), 1000);
    }
}
