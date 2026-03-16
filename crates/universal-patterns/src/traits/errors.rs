// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Primal error types.

use thiserror::Error;

/// Primal error types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PrimalError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    /// Security error
    #[error("Security error: {0}")]
    Security(String),
    /// Orchestration error
    #[error("Orchestration error: {0}")]
    Orchestration(String),
    /// State error
    #[error("State error: {0}")]
    State(String),
    /// Health check error
    #[error("Health check error: {0}")]
    HealthCheck(String),
    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),
    /// Shutdown error
    #[error("Shutdown error: {0}")]
    Shutdown(String),
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    /// Permission error
    #[error("Permission error: {0}")]
    Permission(String),
    /// Resource error
    #[error("Resource error: {0}")]
    Resource(String),
    /// Communication error
    #[error("Communication error: {0}")]
    Communication(String),
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    /// Service unavailable error
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for primal operations
pub type PrimalResult<T> = std::result::Result<T, PrimalError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_error_display() {
        let cases = vec![
            (
                PrimalError::Configuration("bad config".into()),
                "Configuration error: bad config",
            ),
            (
                PrimalError::Network("timeout".into()),
                "Network error: timeout",
            ),
            (
                PrimalError::Security("denied".into()),
                "Security error: denied",
            ),
            (
                PrimalError::Orchestration("failed".into()),
                "Orchestration error: failed",
            ),
            (PrimalError::State("invalid".into()), "State error: invalid"),
            (
                PrimalError::HealthCheck("fail".into()),
                "Health check error: fail",
            ),
            (PrimalError::Metrics("bad".into()), "Metrics error: bad"),
            (PrimalError::Shutdown("err".into()), "Shutdown error: err"),
            (
                PrimalError::Internal("crash".into()),
                "Internal error: crash",
            ),
            (PrimalError::Timeout("5s".into()), "Timeout error: 5s"),
            (
                PrimalError::Permission("no access".into()),
                "Permission error: no access",
            ),
            (PrimalError::Resource("OOM".into()), "Resource error: OOM"),
            (
                PrimalError::Communication("lost".into()),
                "Communication error: lost",
            ),
            (
                PrimalError::Validation("bad input".into()),
                "Validation error: bad input",
            ),
            (
                PrimalError::NotImplemented("feature X".into()),
                "Not implemented: feature X",
            ),
            (
                PrimalError::ServiceUnavailable("down".into()),
                "Service unavailable: down",
            ),
            (
                PrimalError::AlreadyExists("dup".into()),
                "Already exists: dup",
            ),
            (
                PrimalError::NotFound("missing".into()),
                "Not found: missing",
            ),
            (
                PrimalError::InvalidConfiguration("key".into()),
                "Invalid configuration: key",
            ),
            (
                PrimalError::NetworkError("conn refused".into()),
                "Network error: conn refused",
            ),
            (
                PrimalError::AuthenticationError("bad creds".into()),
                "Authentication error: bad creds",
            ),
            (
                PrimalError::AuthorizationError("forbidden".into()),
                "Authorization error: forbidden",
            ),
            (
                PrimalError::InternalError("panic".into()),
                "Internal error: panic",
            ),
            (PrimalError::Other("misc".into()), "Other error: misc"),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_primal_error_is_error_trait() {
        let error: Box<dyn std::error::Error> = Box::new(PrimalError::Internal("test".into()));
        assert!(error.to_string().contains("Internal error"));
    }

    #[test]
    fn test_primal_error_debug() {
        let error = PrimalError::NotFound("resource".into());
        let debug = format!("{:?}", error);
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("resource"));
    }

    #[test]
    fn test_primal_result_ok() {
        let result: PrimalResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_primal_result_err() {
        let result: PrimalResult<i32> = Err(PrimalError::Timeout("expired".into()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Timeout"));
    }
}
