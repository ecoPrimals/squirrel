// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Production-safe error types for MCP operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ProductionError {
    /// Configuration validation or loading failed
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        field: Option<String>,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        endpoint: Option<String>,
        status_code: Option<u16>,
        retry_after: Option<u64>,
    },

    /// Protocol-level errors
    #[error("Protocol error: {message}")]
    Protocol {
        message: String,
        protocol_version: Option<String>,
        expected_version: Option<String>,
        retry_possible: bool,
    },

    /// Service unavailable or degraded
    #[error("Service unavailable: {service} - {message}")]
    ServiceUnavailable {
        service: String,
        message: String,
        retry_after: Option<u64>,
        fallback_available: bool,
    },

    /// Database operation errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        query: Option<String>,
        connection_available: bool,
        retry_possible: bool,
    },

    /// Authentication/Authorization errors
    #[error("Authentication error: {message}")]
    Authentication {
        message: String,
        retry_allowed: bool,
        required_permissions: Vec<String>,
    },

    /// Resource exhaustion or limits reached
    #[error("Resource exhausted: {resource} - {message}")]
    ResourceExhausted {
        resource: String,
        message: String,
        current_usage: Option<u64>,
        limit: Option<u64>,
        retry_after: Option<u64>,
    },

    /// Lock acquisition or concurrency errors
    #[error("Concurrency error: {message}")]
    Concurrency {
        message: String,
        resource: String,
        retry_possible: bool,
    },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        data_type: String,
        recovery_possible: bool,
    },

    /// Timeout errors
    #[error("Timeout error: {message}")]
    Timeout {
        message: String,
        operation: String,
        timeout_ms: u64,
        retry_possible: bool,
    },

    /// Resource not found
    #[error("Not found: {message}")]
    NotFound {
        message: String,
        resource: String,
        suggestion: Option<String>,
    },

    /// Command execution failed
    #[error("Execution failed: {message}")]
    Execution {
        message: String,
        resource: String,
        retry_possible: bool,
    },
}

impl ProductionError {
    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field: None,
            expected: None,
            actual: None,
        }
    }

    /// Create a configuration error with field details
    pub fn configuration_field(
        message: impl Into<String>,
        field: impl Into<String>,
        expected: Option<impl Into<String>>,
        actual: Option<impl Into<String>>,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            field: Some(field.into()),
            expected: expected.map(|e| e.into()),
            actual: actual.map(|a| a.into()),
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: None,
            status_code: None,
            retry_after: None,
        }
    }

    /// Create a network error with endpoint details
    pub fn network_endpoint(
        message: impl Into<String>,
        endpoint: impl Into<String>,
        status_code: Option<u16>,
        retry_after: Option<u64>,
    ) -> Self {
        Self::Network {
            message: message.into(),
            endpoint: Some(endpoint.into()),
            status_code,
            retry_after,
        }
    }

    /// Create a protocol error
    pub fn protocol(message: impl Into<String>, retry_possible: bool) -> Self {
        Self::Protocol {
            message: message.into(),
            protocol_version: None,
            expected_version: None,
            retry_possible,
        }
    }

    /// Create a service unavailable error
    pub fn service_unavailable(
        service: impl Into<String>,
        message: impl Into<String>,
        fallback_available: bool,
    ) -> Self {
        Self::ServiceUnavailable {
            service: service.into(),
            message: message.into(),
            retry_after: None,
            fallback_available,
        }
    }

    /// Create a database error
    pub fn database(message: impl Into<String>, retry_possible: bool) -> Self {
        Self::Database {
            message: message.into(),
            query: None,
            connection_available: true,
            retry_possible,
        }
    }

    /// Create an authentication error
    pub fn authentication(
        message: impl Into<String>,
        retry_allowed: bool,
        required_permissions: Vec<String>,
    ) -> Self {
        Self::Authentication {
            message: message.into(),
            retry_allowed,
            required_permissions,
        }
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted(
        resource: impl Into<String>,
        message: impl Into<String>,
        current_usage: Option<u64>,
        limit: Option<u64>,
    ) -> Self {
        Self::ResourceExhausted {
            resource: resource.into(),
            message: message.into(),
            current_usage,
            limit,
            retry_after: None,
        }
    }

    /// Create a concurrency error
    pub fn concurrency(
        message: impl Into<String>,
        resource: impl Into<String>,
        retry_possible: bool,
    ) -> Self {
        Self::Concurrency {
            message: message.into(),
            resource: resource.into(),
            retry_possible,
        }
    }

    /// Create a serialization error
    pub fn serialization(
        message: impl Into<String>,
        data_type: impl Into<String>,
        recovery_possible: bool,
    ) -> Self {
        Self::Serialization {
            message: message.into(),
            data_type: data_type.into(),
            recovery_possible,
        }
    }

    /// Create a timeout error
    pub fn timeout(
        message: impl Into<String>,
        operation: impl Into<String>,
        timeout_ms: u64,
        retry_possible: bool,
    ) -> Self {
        Self::Timeout {
            message: message.into(),
            operation: operation.into(),
            timeout_ms,
            retry_possible,
        }
    }

    /// Create a not found error
    pub fn not_found(
        message: impl Into<String>,
        resource: impl Into<String>,
        suggestion: Option<impl Into<String>>,
    ) -> Self {
        Self::NotFound {
            message: message.into(),
            resource: resource.into(),
            suggestion: suggestion.map(|s| s.into()),
        }
    }

    /// Create an execution error
    pub fn execution(
        message: impl Into<String>,
        resource: impl Into<String>,
        retry_possible: bool,
    ) -> Self {
        Self::Execution {
            message: message.into(),
            resource: resource.into(),
            retry_possible,
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ProductionError::Configuration { .. } => false,
            ProductionError::Network { retry_after, .. } => retry_after.is_some(),
            ProductionError::Protocol { retry_possible, .. } => *retry_possible,
            ProductionError::ServiceUnavailable { retry_after, .. } => retry_after.is_some(),
            ProductionError::Database { retry_possible, .. } => *retry_possible,
            ProductionError::Authentication { retry_allowed, .. } => *retry_allowed,
            ProductionError::ResourceExhausted { retry_after, .. } => retry_after.is_some(),
            ProductionError::Concurrency { retry_possible, .. } => *retry_possible,
            ProductionError::Serialization {
                recovery_possible, ..
            } => *recovery_possible,
            ProductionError::Timeout { retry_possible, .. } => *retry_possible,
            ProductionError::NotFound { .. } => false,
            ProductionError::Execution { retry_possible, .. } => *retry_possible,
        }
    }

    /// Get suggested retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            ProductionError::Network { retry_after, .. } => *retry_after,
            ProductionError::ServiceUnavailable { retry_after, .. } => *retry_after,
            ProductionError::ResourceExhausted { retry_after, .. } => *retry_after,
            ProductionError::Concurrency { retry_possible, .. } => {
                if *retry_possible {
                    Some(100)
                } else {
                    None
                }
            }
            ProductionError::Timeout { retry_possible, .. } => {
                if *retry_possible {
                    Some(1000)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ProductionError::Configuration { .. } => ErrorSeverity::Critical,
            ProductionError::Network { .. } => ErrorSeverity::High,
            ProductionError::Protocol { .. } => ErrorSeverity::High,
            ProductionError::ServiceUnavailable {
                fallback_available, ..
            } => {
                if *fallback_available {
                    ErrorSeverity::Medium
                } else {
                    ErrorSeverity::High
                }
            }
            ProductionError::Database {
                connection_available,
                ..
            } => {
                if *connection_available {
                    ErrorSeverity::Medium
                } else {
                    ErrorSeverity::High
                }
            }
            ProductionError::Authentication { .. } => ErrorSeverity::High,
            ProductionError::ResourceExhausted { .. } => ErrorSeverity::Medium,
            ProductionError::Concurrency { .. } => ErrorSeverity::Low,
            ProductionError::Serialization { .. } => ErrorSeverity::Medium,
            ProductionError::Timeout { .. } => ErrorSeverity::Medium,
            ProductionError::NotFound { .. } => ErrorSeverity::Low,
            ProductionError::Execution { .. } => ErrorSeverity::Medium,
        }
    }
}

// Re-export canonical ErrorSeverity from types module
pub use crate::error::types::ErrorSeverity;

/// Safe wrapper for potentially fallible operations
pub struct SafeOperation<T> {
    result: Result<T, ProductionError>,
}

impl<T> SafeOperation<T> {
    /// Create a new safe operation
    pub fn new(result: Result<T, ProductionError>) -> Self {
        Self { result }
    }

    /// Execute operation with error handling
    pub fn execute<F>(operation: F) -> Self
    where
        F: FnOnce() -> Result<T, ProductionError>,
    {
        Self::new(operation())
    }

    /// Get the result
    pub fn result(self) -> Result<T, ProductionError> {
        self.result
    }

    /// Get the result or default value
    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        self.result.unwrap_or_default()
    }

    /// Get the result or provided default value
    pub fn unwrap_or(self, default: T) -> T {
        self.result.unwrap_or(default)
    }

    /// Get the result or compute default value
    pub fn unwrap_or_else<F>(self, default: F) -> T
    where
        F: FnOnce(ProductionError) -> T,
    {
        self.result.unwrap_or_else(default)
    }

    /// Log error and return default value
    pub fn log_error_and_default(self, context: &str) -> T
    where
        T: Default,
    {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("Error in {}: {}", context, error);
                T::default()
            }
        }
    }

    /// Log error and return provided value
    pub fn log_error_and_return(self, context: &str, default: T) -> T {
        match self.result {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("Error in {}: {}", context, error);
                default
            }
        }
    }
}

/// Trait for converting standard errors to production errors
pub trait IntoProductionError {
    fn into_production_error(self) -> ProductionError;
}

impl IntoProductionError for serde_json::Error {
    fn into_production_error(self) -> ProductionError {
        ProductionError::serialization(self.to_string(), "json", true)
    }
}

impl IntoProductionError for std::io::Error {
    fn into_production_error(self) -> ProductionError {
        ProductionError::network(self.to_string())
    }
}

impl IntoProductionError for tokio::time::error::Elapsed {
    fn into_production_error(self) -> ProductionError {
        ProductionError::timeout("Operation timed out", "unknown", 30000, true)
    }
}

/// Convenience macro for safe operations
#[macro_export]
macro_rules! safe_operation {
    ($operation:expr) => {
        SafeOperation::execute(|| $operation)
    };
}

/// Convenience macro for safe operations with context
#[macro_export]
macro_rules! safe_operation_with_context {
    ($operation:expr, $context:expr) => {
        SafeOperation::execute(|| $operation).log_error_and_default($context)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_error_creation() {
        let error = ProductionError::configuration("Invalid config");
        assert!(matches!(error, ProductionError::Configuration { .. }));
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_safe_operation() {
        let result = SafeOperation::new(Ok(42));
        assert_eq!(result.unwrap_or_default(), 42);

        let error_result: SafeOperation<i32> =
            SafeOperation::new(Err(ProductionError::configuration("test")));
        assert_eq!(error_result.unwrap_or_default(), 0);
    }

    #[test]
    fn test_error_severity() {
        assert!(ErrorSeverity::Critical.requires_immediate_attention());
        assert!(ErrorSeverity::High.should_alert());
        assert!(!ErrorSeverity::Low.requires_immediate_attention());
    }
}
