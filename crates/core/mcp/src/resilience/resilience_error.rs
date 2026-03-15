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

impl From<crate::resilience::circuit_breaker::BreakerError> for ResilienceError {
    fn from(err: crate::resilience::circuit_breaker::BreakerError) -> Self {
        match err {
            crate::resilience::circuit_breaker::BreakerError::CircuitOpen { name, reset_time_ms } => {
                Self::CircuitOpen(format!("Circuit '{}' is open, reset time: {}ms", name, reset_time_ms))
            }
            crate::resilience::circuit_breaker::BreakerError::Timeout { name, timeout_ms } => {
                Self::Timeout(format!("Operation on circuit '{}' timed out after {}ms", name, timeout_ms))
            }
            crate::resilience::circuit_breaker::BreakerError::OperationFailed { name, reason } => {
                Self::OperationFailed(format!("Operation on circuit '{}' failed: {}", name, reason))
            }
            crate::resilience::circuit_breaker::BreakerError::Internal { name, details } => {
                Self::General(format!("Internal error in circuit '{}': {}", name, details))
            }
        }
    }
}

impl From<crate::resilience::recovery::RecoveryError> for ResilienceError {
    fn from(err: crate::resilience::recovery::RecoveryError) -> Self {
        match err {
            crate::resilience::recovery::RecoveryError::MaxAttemptsExceeded { severity, attempts, max_attempts } => {
                Self::RecoveryFailed(format!(
                    "Maximum recovery attempts ({max_attempts}) exceeded for {severity} failure: {attempts} attempts made"
                ))
            }
            crate::resilience::recovery::RecoveryError::CriticalFailureNoRecovery => {
                Self::RecoveryFailed("Recovery not attempted for critical failure".to_string())
            }
            crate::resilience::recovery::RecoveryError::RecoveryActionFailed { message, .. } => {
                Self::RecoveryFailed(format!("Recovery action failed: {message}"))
            }
            crate::resilience::recovery::RecoveryError::Timeout { duration } => {
                Self::Timeout(format!("Recovery timed out after {duration:?}"))
            }
        }
    }
}

impl From<crate::resilience::state_sync::StateSyncError> for ResilienceError {
    fn from(err: crate::resilience::state_sync::StateSyncError) -> Self {
        match err {
            crate::resilience::state_sync::StateSyncError::Timeout { duration } => {
                Self::Timeout(format!("State synchronization timed out after {duration:?}"))
            }
            crate::resilience::state_sync::StateSyncError::SizeExceeded { size, max_size } => {
                Self::SyncFailed(format!(
                    "State size ({size} bytes) exceeds maximum allowed size ({max_size} bytes)"
                ))
            }
            crate::resilience::state_sync::StateSyncError::ValidationFailed { message } => {
                Self::SyncFailed(format!("State validation failed: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::TargetUnavailable { target } => {
                Self::SyncFailed(format!("Target system unavailable: {target}"))
            }
            crate::resilience::state_sync::StateSyncError::SerializationError { message } => {
                Self::SyncFailed(format!("State serialization error: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::SyncFailed { message, .. } => {
                Self::SyncFailed(format!("Synchronization failed: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::NotFound(message) => {
                Self::SyncFailed(format!("State not found: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::DeserializationFailed(message) => {
                Self::SyncFailed(format!("State deserialization failed: {message}"))
            },
        }
    }
}

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

impl From<crate::resilience::health::HealthCheckError> for ResilienceError {
    fn from(err: crate::resilience::health::HealthCheckError) -> Self {
        match err {
            crate::resilience::health::HealthCheckError::Timeout { component_id, duration } => {
                Self::Timeout(format!(
                    "Health check for component '{component_id}' timed out after {duration:?}"
                ))
            }
            crate::resilience::health::HealthCheckError::CheckFailed { component_id, message } => {
                Self::General(format!(
                    "Health check for component '{component_id}' failed: {message}"
                ))
            }
            crate::resilience::health::HealthCheckError::ComponentUnavailable { component_id } => {
                Self::General(format!(
                    "Component '{component_id}' is unavailable for health check"
                ))
            }
        }
    }
} 