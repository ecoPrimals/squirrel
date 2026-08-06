// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error handling system for the Squirrel Plugin SDK
//!
//! Uses the unified `universal_error::sdk::SDKError` hierarchy:
//! - `SDKError::Infrastructure(InfrastructureError)` — config, validation, logging
//! - `SDKError::Communication(CommunicationError)` — MCP, events, commands, serde
//! - `SDKError::Client(ClientError)` — HTTP, connections, timeouts
//! - `SDKError::General(String)` — catch-all
//!
//! ```text
//! use crate::infrastructure::error::{Error, Result};
//! // Error = universal_error::sdk::SDKError
//! // Result<T> = std::result::Result<T, SDKError>
//! ```

pub mod conversions;
pub mod core;
pub mod validation;

#[cfg(test)]
pub mod tests;

// Primary error type: SDKError from universal-error
pub use universal_error::sdk::{
    ClientError, CommunicationError, InfrastructureError, SDKError,
};

// Convenience aliases — these are the canonical types for SDK code
pub use universal_error::sdk::SDKError as Error;
/// SDK result type using `SDKError`
pub type Result<T> = std::result::Result<T, SDKError>;
/// Backward-compatible alias for `Result<T>`
pub type PluginResult<T> = std::result::Result<T, SDKError>;

pub use self::validation::{
    ValidationError, validate_array, validate_array_length, validate_boolean, validate_email,
    validate_enum_value, validate_integer_range, validate_non_empty_string, validate_numeric_range,
    validate_object, validate_optional_string, validate_required_fields, validate_required_number,
    validate_required_string, validate_string_length, validate_url,
};

/// Result type for validation operations
pub type ValidationResult<T> = std::result::Result<T, ValidationError>;
/// Result type for network operations
pub type NetworkResult<T> = std::result::Result<T, SDKError>;
/// Result type for configuration operations
pub type ConfigResult<T> = std::result::Result<T, SDKError>;

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
    pub async fn retry_with_backoff<F, T, E>(
        config: RetryConfig,
        mut operation: F,
    ) -> std::result::Result<T, E>
    where
        F: FnMut() -> std::result::Result<T, E>,
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

                    crate::infrastructure::utils::sleep_ms(delay).await;

                    delay =
                        ((delay as f64 * config.backoff_multiplier) as u64).min(config.max_delay);
                }
            }
        }
    }
}

#[cfg(test)]
mod mod_tests {
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
    fn test_sdk_error_general() {
        let err = SDKError::General("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_sdk_error_infrastructure() {
        let err: SDKError = InfrastructureError::Configuration("bad config".to_string()).into();
        assert!(err.to_string().contains("bad config"));
    }

    #[test]
    fn test_sdk_error_communication() {
        let err: SDKError = CommunicationError::MCP("protocol error".to_string()).into();
        assert!(err.to_string().contains("protocol error"));
    }

    #[test]
    fn test_sdk_error_client() {
        let err: SDKError = ClientError::Timeout(30).into();
        assert!(err.to_string().contains("30"));
    }
}
