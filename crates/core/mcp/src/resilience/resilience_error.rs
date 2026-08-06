// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types and handling for resilience operations
//!
//! This module defines the various error types used in the resilience framework
//! and their handling mechanisms.

use thiserror::Error;

/// Error type for resilience operations
#[derive(Debug, Error)]
pub enum ResilienceError {
    /// Circuit breaker prevented an operation from executing
    #[error("Circuit open: {0}")]
    CircuitOpen(String),

    /// Maximum retry attempts were exceeded
    #[error("Retry exceeded: {0}")]
    RetryExceeded(String),

    /// Recovery strategy failed
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    /// State synchronization failed
    #[error("State synchronization failed: {0}")]
    SyncFailed(String),

    /// Operation timed out
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Generic error with message
    #[error("Resilience error: {0}")]
    General(String),

    /// Operation failed after recovery attempts
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Bulkhead isolation error
    #[error("Bulkhead isolation error: {0}")]
    Bulkhead(String),

    /// Rate limiting error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheck(String),
}

/// Convenience type alias for Results from resilience operations
pub type Result<T> = std::result::Result<T, ResilienceError>;

impl From<crate::resilience::retry::RetryError> for ResilienceError {
    fn from(err: crate::resilience::retry::RetryError) -> Self {
        match err {
            crate::resilience::retry::RetryError::MaxAttemptsExceeded {
                attempts,
                last_error,
            } => Self::RetryExceeded(format!(
                "Maximum retry attempts ({attempts}) exceeded: {last_error}"
            )),
            crate::resilience::retry::RetryError::Cancelled(msg) => {
                Self::RetryExceeded(format!("Retry cancelled: {msg}"))
            }
            crate::resilience::retry::RetryError::Internal(msg) => {
                Self::RetryExceeded(format!("Retry internal error: {msg}"))
            }
        }
    }
}
