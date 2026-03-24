// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types and handling for resilience operations
//!
//! This module defines the various error types used in the resilience framework
//! and their handling mechanisms.

use std::error::Error as StdError;
use std::fmt;

/// Error type for resilience operations
#[derive(Debug)]
pub enum ResilienceError {
    /// Circuit breaker prevented an operation from executing
    CircuitOpen(String),

    /// Maximum retry attempts were exceeded
    RetryExceeded(String),

    /// Recovery strategy failed
    RecoveryFailed(String),

    /// State synchronization failed
    SyncFailed(String),

    /// Operation timed out
    Timeout(String),

    /// Generic error with message
    General(String),

    /// Operation failed after recovery attempts
    OperationFailed(String),

    /// Bulkhead isolation error
    Bulkhead(String),

    /// Rate limiting error
    RateLimit(String),

    /// Health check failed
    HealthCheck(String),
}

impl fmt::Display for ResilienceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircuitOpen(msg) => write!(f, "Circuit open: {msg}"),
            Self::RetryExceeded(msg) => write!(f, "Retry exceeded: {msg}"),
            Self::RecoveryFailed(msg) => write!(f, "Recovery failed: {msg}"),
            Self::SyncFailed(msg) => write!(f, "State synchronization failed: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::General(msg) => write!(f, "Resilience error: {msg}"),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {msg}"),
            Self::Bulkhead(msg) => write!(f, "Bulkhead isolation error: {msg}"),
            Self::RateLimit(msg) => write!(f, "Rate limit exceeded: {msg}"),
            Self::HealthCheck(msg) => write!(f, "Health check failed: {msg}"),
        }
    }
}

impl StdError for ResilienceError {}

/// Convenience type alias for Results from resilience operations
pub type Result<T> = std::result::Result<T, ResilienceError>;

impl From<crate::resilience::retry::RetryError> for ResilienceError {
    fn from(err: crate::resilience::retry::RetryError) -> Self {
        match err {
            crate::resilience::retry::RetryError::MaxAttemptsExceeded { attempts, error } => {
                Self::RetryExceeded(format!(
                    "Maximum retry attempts ({attempts}) exceeded: {error}"
                ))
            }
            crate::resilience::retry::RetryError::Cancelled(msg) => {
                Self::RetryExceeded(format!("Retry cancelled: {msg}"))
            }
            crate::resilience::retry::RetryError::Internal(msg) => {
                Self::RetryExceeded(format!("Retry internal error: {msg}"))
            }
        }
    }
}
