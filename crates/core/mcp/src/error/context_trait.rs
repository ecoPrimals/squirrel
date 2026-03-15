// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Error Context Trait - Standardized error context access across all error types
//!
//! This module provides a standardized trait for accessing contextual information
//! from errors, enabling consistent error handling and debugging across the codebase.

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

use super::types::{ErrorContext, ErrorSeverity};

/// Trait for accessing contextual information from errors.
///
/// This trait provides a standardized interface for retrieving error context,
/// enabling consistent error handling, logging, and debugging across all error types.
///
/// # Design Principles
///
/// 1. **Consistent Interface**: All errors provide the same context access methods
/// 2. **Optional Context**: Not all errors need full context (default implementations provided)
/// 3. **Zero Cost**: Trait methods are inlined where possible
/// 4. **Backward Compatible**: Existing error types can opt-in gradually
///
/// # Examples
///
/// ```rust
/// use squirrel_mcp::error::{MCPError, ErrorContextTrait, ErrorSeverity};
///
/// fn log_error(error: &dyn ErrorContextTrait) {
///     println!("Error in {}: {}",
///         error.component().unwrap_or("unknown"),
///         error.operation().unwrap_or("unknown operation")
///     );
///     
///     if error.severity() == ErrorSeverity::Critical {
///         // Trigger alert
///     }
/// }
/// ```
pub trait ErrorContextTrait {
    /// Get the timestamp when the error occurred
    ///
    /// Returns None if timestamp information is not available.
    #[inline]
    fn timestamp(&self) -> Option<DateTime<Utc>> {
        None
    }

    /// Get the operation being performed when the error occurred
    ///
    /// Returns None if operation information is not available.
    #[inline]
    fn operation(&self) -> Option<&str> {
        None
    }

    /// Get the component where the error occurred
    ///
    /// Returns None if component information is not available.
    #[inline]
    fn component(&self) -> Option<&str> {
        None
    }

    /// Get the severity level of the error
    ///
    /// Defaults to Medium severity if not specified.
    #[inline]
    fn severity(&self) -> ErrorSeverity {
        ErrorSeverity::Medium
    }

    /// Check if the error is recoverable
    ///
    /// Defaults to true (optimistic recovery assumption).
    #[inline]
    fn is_recoverable(&self) -> bool {
        true
    }

    /// Get additional structured details about the error
    ///
    /// Returns an empty map if no details are available.
    #[inline]
    fn details(&self) -> HashMap<String, Value> {
        HashMap::new()
    }

    /// Get the full error context if available
    ///
    /// Returns None if the error doesn't provide a full ErrorContext struct.
    #[inline]
    fn get_context(&self) -> Option<&ErrorContext> {
        None
    }

    /// Create a structured log entry for this error
    ///
    /// This method provides a consistent format for logging errors across the system.
    fn to_log_entry(&self) -> HashMap<String, Value> {
        let mut log = HashMap::new();

        if let Some(timestamp) = self.timestamp() {
            log.insert(
                "timestamp".to_string(),
                Value::String(timestamp.to_rfc3339()),
            );
        }

        if let Some(operation) = self.operation() {
            log.insert(
                "operation".to_string(),
                Value::String(operation.to_string()),
            );
        }

        if let Some(component) = self.component() {
            log.insert(
                "component".to_string(),
                Value::String(component.to_string()),
            );
        }

        log.insert(
            "severity".to_string(),
            Value::String(format!("{:?}", self.severity())),
        );
        log.insert(
            "recoverable".to_string(),
            Value::Bool(self.is_recoverable()),
        );

        // Merge in any additional details
        for (key, value) in self.details() {
            log.insert(key, value);
        }

        log
    }

    /// Check if this error should trigger an alert
    ///
    /// Based on severity level.
    #[inline]
    fn should_alert(&self) -> bool {
        matches!(
            self.severity(),
            ErrorSeverity::High | ErrorSeverity::Critical
        )
    }

    /// Check if this error requires immediate attention
    ///
    /// Based on severity level and recoverability.
    #[inline]
    fn requires_immediate_attention(&self) -> bool {
        self.severity() == ErrorSeverity::Critical
            || (self.severity() == ErrorSeverity::High && !self.is_recoverable())
    }
}

/// Helper trait for errors that can be enriched with context
///
/// This trait allows errors to have context added to them after creation,
/// useful for adding context as errors propagate up the call stack.
pub trait WithContext: Sized {
    /// Add context information to this error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel_mcp::error::{MCPError, WithContext};
    ///
    /// fn process() -> Result<(), MCPError> {
    ///     some_operation()
    ///         .with_context("processing user request", "request_handler")?;
    ///     Ok(())
    /// }
    /// ```
    fn with_context(self, operation: &str, component: &str) -> Self;

    /// Add severity information to this error
    fn with_severity(self, severity: ErrorSeverity) -> Self;

    /// Mark this error as unrecoverable
    fn as_unrecoverable(self) -> Self;
}

/// Extension methods for Result types to add context
pub trait ResultContextExt<T, E> {
    /// Add context to an error if the Result is Err
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel_mcp::error::{Result, ResultContextExt};
    ///
    /// fn load_config() -> Result<Config> {
    ///     parse_file("config.toml")
    ///         .context("loading configuration", "config_loader")
    /// }
    /// ```
    fn context(self, operation: &str, component: &str) -> Result<T, E>
    where
        E: WithContext;
}

impl<T, E> ResultContextExt<T, E> for Result<T, E>
where
    E: WithContext,
{
    fn context(self, operation: &str, component: &str) -> Result<T, E> {
        self.map_err(|e| e.with_context(operation, component))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestError {
        message: String,
        severity: ErrorSeverity,
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    impl ErrorContextTrait for TestError {
        fn severity(&self) -> ErrorSeverity {
            self.severity
        }

        fn operation(&self) -> Option<&str> {
            Some("test_operation")
        }

        fn component(&self) -> Option<&str> {
            Some("test_component")
        }
    }

    #[test]
    fn test_error_context_trait() {
        let error = TestError {
            message: "test error".to_string(),
            severity: ErrorSeverity::High,
        };

        assert_eq!(error.severity(), ErrorSeverity::High);
        assert_eq!(error.operation(), Some("test_operation"));
        assert_eq!(error.component(), Some("test_component"));
        assert!(error.should_alert());
        assert!(error.requires_immediate_attention());
    }

    #[test]
    fn test_log_entry_creation() {
        let error = TestError {
            message: "test error".to_string(),
            severity: ErrorSeverity::Critical,
        };

        let log = error.to_log_entry();

        assert!(log.contains_key("severity"));
        assert!(log.contains_key("operation"));
        assert!(log.contains_key("component"));
    }

    #[test]
    fn test_default_implementations() {
        #[derive(Debug)]
        struct MinimalError;

        impl std::fmt::Display for MinimalError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "minimal error")
            }
        }

        impl std::error::Error for MinimalError {}
        impl ErrorContextTrait for MinimalError {}

        let error = MinimalError;

        // Should use default implementations
        assert_eq!(error.severity(), ErrorSeverity::Medium);
        assert!(error.is_recoverable());
        assert_eq!(error.operation(), None);
        assert_eq!(error.component(), None);
    }
}
