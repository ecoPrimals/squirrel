// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Production-safe error types for MCP operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ProductionError {
    /// Configuration validation or loading failed
    #[error("Configuration error: {message}")]
    Configuration {
        /// Human-readable error message
        message: String,
        /// Configuration field that failed validation, if applicable
        field: Option<String>,
        /// Expected value or format, if applicable
        expected: Option<String>,
        /// Actual value received, if applicable
        actual: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        /// Human-readable error message
        message: String,
        /// Endpoint that failed, if applicable
        endpoint: Option<String>,
        /// HTTP status code if applicable
        status_code: Option<u16>,
        /// Suggested retry delay in seconds
        retry_after: Option<u64>,
    },

    /// Protocol-level errors
    #[error("Protocol error: {message}")]
    Protocol {
        /// Human-readable error message
        message: String,
        /// Protocol version received, if applicable
        protocol_version: Option<String>,
        /// Expected protocol version, if applicable
        expected_version: Option<String>,
        /// Whether retry may succeed
        retry_possible: bool,
    },

    /// Service unavailable or degraded
    #[error("Service unavailable: {service} - {message}")]
    ServiceUnavailable {
        /// Name of the unavailable service
        service: String,
        /// Human-readable error message
        message: String,
        /// Suggested retry delay in seconds
        retry_after: Option<u64>,
        /// Whether a fallback service is available
        fallback_available: bool,
    },

    /// Database operation errors
    #[error("Database error: {message}")]
    Database {
        /// Human-readable error message
        message: String,
        /// Query that failed, if applicable
        query: Option<String>,
        /// Whether database connection is still available
        connection_available: bool,
        /// Whether retry may succeed
        retry_possible: bool,
    },

    /// Authentication/Authorization errors
    #[error("Authentication error: {message}")]
    Authentication {
        /// Human-readable error message
        message: String,
        /// Whether retry with new credentials is allowed
        retry_allowed: bool,
        /// Permissions required for the operation
        required_permissions: Vec<String>,
    },

    /// Resource exhaustion or limits reached
    #[error("Resource exhausted: {resource} - {message}")]
    ResourceExhausted {
        /// Name of the exhausted resource
        resource: String,
        /// Human-readable error message
        message: String,
        /// Current usage level, if known
        current_usage: Option<u64>,
        /// Resource limit, if known
        limit: Option<u64>,
        /// Suggested retry delay in seconds
        retry_after: Option<u64>,
    },

    /// Lock acquisition or concurrency errors
    #[error("Concurrency error: {message}")]
    Concurrency {
        /// Human-readable error message
        message: String,
        /// Resource that could not be acquired
        resource: String,
        /// Whether retry may succeed
        retry_possible: bool,
    },

    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Human-readable error message
        message: String,
        /// Data type that failed serialization
        data_type: String,
        /// Whether recovery is possible
        recovery_possible: bool,
    },

    /// Timeout errors
    #[error("Timeout error: {message}")]
    Timeout {
        /// Human-readable error message
        message: String,
        /// Operation that timed out
        operation: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
        /// Whether retry may succeed
        retry_possible: bool,
    },

    /// Resource not found
    #[error("Not found: {message}")]
    NotFound {
        /// Human-readable error message
        message: String,
        /// Resource that was not found
        resource: String,
        /// Suggestion for alternative resource, if applicable
        suggestion: Option<String>,
    },

    /// Command execution failed
    #[error("Execution failed: {message}")]
    Execution {
        /// Human-readable error message
        message: String,
        /// Resource or command that failed
        resource: String,
        /// Whether retry may succeed
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
            expected: expected.map(std::convert::Into::into),
            actual: actual.map(std::convert::Into::into),
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
            suggestion: suggestion.map(std::convert::Into::into),
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
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::Configuration { .. } | Self::NotFound { .. } => false,
            Self::Network { retry_after, .. }
            | Self::ServiceUnavailable { retry_after, .. }
            | Self::ResourceExhausted { retry_after, .. } => retry_after.is_some(),
            Self::Protocol { retry_possible, .. }
            | Self::Database { retry_possible, .. }
            | Self::Concurrency { retry_possible, .. }
            | Self::Timeout { retry_possible, .. }
            | Self::Execution { retry_possible, .. } => *retry_possible,
            Self::Authentication { retry_allowed, .. } => *retry_allowed,
            Self::Serialization {
                recovery_possible, ..
            } => *recovery_possible,
        }
    }

    /// Get suggested retry delay in milliseconds
    #[must_use]
    pub const fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            Self::Network { retry_after, .. }
            | Self::ServiceUnavailable { retry_after, .. }
            | Self::ResourceExhausted { retry_after, .. } => *retry_after,
            Self::Concurrency { retry_possible, .. } => {
                if *retry_possible {
                    Some(100)
                } else {
                    None
                }
            }
            Self::Timeout { retry_possible, .. } => {
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
    #[must_use]
    pub const fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Configuration { .. } => ErrorSeverity::Critical,
            Self::Network { .. } | Self::Protocol { .. } | Self::Authentication { .. } => {
                ErrorSeverity::High
            }
            Self::ServiceUnavailable {
                fallback_available, ..
            } => {
                if *fallback_available {
                    ErrorSeverity::Medium
                } else {
                    ErrorSeverity::High
                }
            }
            Self::Database {
                connection_available,
                ..
            } => {
                if *connection_available {
                    ErrorSeverity::Medium
                } else {
                    ErrorSeverity::High
                }
            }
            Self::ResourceExhausted { .. } => ErrorSeverity::Medium,
            Self::Concurrency { .. } | Self::NotFound { .. } => ErrorSeverity::Low,
            Self::Serialization { .. } | Self::Timeout { .. } | Self::Execution { .. } => {
                ErrorSeverity::Medium
            }
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
    pub const fn new(result: Result<T, ProductionError>) -> Self {
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
    /// Converts the error into a production-safe `ProductionError`.
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

    #[test]
    fn production_error_display_all_constructors() {
        let cases: Vec<(ProductionError, &'static str)> = vec![
            (
                ProductionError::configuration("c"),
                "Configuration error: c",
            ),
            (
                ProductionError::configuration_field("bad", "field_x", Some("exp"), Some("act")),
                "Configuration error: bad",
            ),
            (ProductionError::network("n"), "Network error: n"),
            (
                ProductionError::network_endpoint("e", "http://x", Some(503), Some(5)),
                "Network error: e",
            ),
            (ProductionError::protocol("p", true), "Protocol error: p"),
            (
                ProductionError::ServiceUnavailable {
                    service: "svc".into(),
                    message: "m".into(),
                    retry_after: Some(10),
                    fallback_available: true,
                },
                "Service unavailable: svc - m",
            ),
            (ProductionError::database("d", false), "Database error: d"),
            (
                ProductionError::authentication("a", true, vec!["read".into()]),
                "Authentication error: a",
            ),
            (
                ProductionError::resource_exhausted("cpu", "hot", Some(9), Some(10)),
                "Resource exhausted: cpu - hot",
            ),
            (
                ProductionError::concurrency("c", "lock", false),
                "Concurrency error: c",
            ),
            (
                ProductionError::serialization("s", "json", false),
                "Serialization error: s",
            ),
            (
                ProductionError::timeout("t", "op", 1000, false),
                "Timeout error: t",
            ),
            (
                ProductionError::not_found("nf", "res", Some(String::from("try"))),
                "Not found: nf",
            ),
            (
                ProductionError::execution("ex", "cmd", true),
                "Execution failed: ex",
            ),
        ];
        for (err, prefix) in cases {
            let s = err.to_string();
            assert!(
                s.starts_with(prefix),
                "got {s:?}, expected prefix {prefix:?}"
            );
        }
    }

    #[test]
    fn production_error_serde_round_trip() {
        let samples = vec![
            ProductionError::configuration("cfg"),
            ProductionError::network_endpoint("n", "e", None, None),
            ProductionError::protocol("p", true),
            ProductionError::authentication("auth", false, vec![]),
        ];
        for err in samples {
            let json = serde_json::to_string(&err).expect("serialize");
            let back: ProductionError = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(format!("{err}"), format!("{back}"));
        }
    }

    #[test]
    fn is_recoverable_and_retry_delay_and_severity_branches() {
        let net = ProductionError::Network {
            message: "m".into(),
            endpoint: None,
            status_code: None,
            retry_after: Some(3),
        };
        assert!(net.is_recoverable());
        assert_eq!(net.retry_delay_ms(), Some(3));

        let net_no = ProductionError::network("x");
        assert!(!net_no.is_recoverable());
        assert_eq!(net_no.retry_delay_ms(), None);

        let su = ProductionError::service_unavailable("s", "m", true);
        assert_eq!(su.severity(), ErrorSeverity::Medium);
        assert!(!su.is_recoverable());

        let su_high = ProductionError::ServiceUnavailable {
            service: "s".into(),
            message: "m".into(),
            retry_after: Some(1),
            fallback_available: false,
        };
        assert_eq!(su_high.severity(), ErrorSeverity::High);
        assert!(su_high.is_recoverable());

        let db_ok = ProductionError::Database {
            message: "m".into(),
            query: None,
            connection_available: true,
            retry_possible: false,
        };
        assert_eq!(db_ok.severity(), ErrorSeverity::Medium);

        let db_bad = ProductionError::Database {
            message: "m".into(),
            query: None,
            connection_available: false,
            retry_possible: true,
        };
        assert_eq!(db_bad.severity(), ErrorSeverity::High);

        let conc = ProductionError::concurrency("c", "r", true);
        assert_eq!(conc.retry_delay_ms(), Some(100));

        let tout = ProductionError::timeout("t", "o", 1, true);
        assert_eq!(tout.retry_delay_ms(), Some(1000));

        let ser = ProductionError::serialization("e", "t", true);
        assert!(ser.is_recoverable());
    }

    #[test]
    fn safe_operation_execute_and_combinators() {
        let ok = SafeOperation::execute(|| Ok::<i32, ProductionError>(7));
        assert_eq!(ok.unwrap_or(0), 7);
        let ok2 = SafeOperation::execute(|| Ok::<i32, ProductionError>(7));
        assert!(matches!(ok2.result(), Ok(7)));

        let err_op: SafeOperation<i32> =
            SafeOperation::execute(|| Err(ProductionError::configuration("bad")));
        assert_eq!(err_op.unwrap_or(99), 99);

        let err_else: SafeOperation<i32> =
            SafeOperation::execute(|| Err(ProductionError::configuration("bad")));
        assert_eq!(
            err_else.unwrap_or_else(|e| e.to_string().len() as i32),
            "Configuration error: bad".len() as i32
        );

        let err_log: SafeOperation<i32> =
            SafeOperation::execute(|| Err(ProductionError::configuration("bad")));
        let def = err_log.log_error_and_default("ctx");
        assert_eq!(def, 0i32);

        let err_ret: SafeOperation<i32> =
            SafeOperation::execute(|| Err(ProductionError::configuration("bad")));
        let def2 = err_ret.log_error_and_return("ctx", 42);
        assert_eq!(def2, 42);
    }

    #[test]
    fn into_production_error_json_io() {
        let json_err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let p = json_err.into_production_error();
        assert!(matches!(p, ProductionError::Serialization { .. }));

        let io_err = std::io::Error::other("io");
        let p2 = io_err.into_production_error();
        assert!(matches!(p2, ProductionError::Network { .. }));
    }

    #[tokio::test]
    async fn into_production_error_elapsed() {
        let elapsed = tokio::time::timeout(std::time::Duration::ZERO, std::future::pending::<()>())
            .await
            .expect_err("should time out");
        let p = elapsed.into_production_error();
        assert!(matches!(p, ProductionError::Timeout { .. }));
    }

    #[test]
    fn safe_operation_macro_expands() {
        let op = crate::safe_operation!(Ok::<i32, ProductionError>(3));
        assert!(matches!(op.result(), Ok(3)));
        let v: i32 =
            crate::safe_operation_with_context!(Err(ProductionError::configuration("x")), "ctx");
        assert_eq!(v, 0);
    }
}
