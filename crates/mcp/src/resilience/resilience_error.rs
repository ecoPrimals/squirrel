use std::error::Error as StdError;
use std::fmt;

/// Errors that can occur during resilience operations
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
}

impl fmt::Display for ResilienceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircuitOpen(msg) => write!(f, "Circuit breaker open: {msg}"),
            Self::RetryExceeded(msg) => write!(f, "Maximum retry attempts exceeded: {msg}"),
            Self::RecoveryFailed(msg) => write!(f, "Recovery strategy failed: {msg}"),
            Self::SyncFailed(msg) => write!(f, "State synchronization failed: {msg}"),
            Self::Timeout(msg) => write!(f, "Operation timed out: {msg}"),
            Self::General(msg) => write!(f, "Resilience error: {msg}"),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {msg}"),
        }
    }
}

impl StdError for ResilienceError {}

/// Convenience type alias for Results from resilience operations
pub type Result<T> = std::result::Result<T, ResilienceError>;

impl From<crate::resilience::circuit_breaker::CircuitBreakerError> for ResilienceError {
    fn from(err: crate::resilience::circuit_breaker::CircuitBreakerError) -> Self {
        match err {
            crate::resilience::circuit_breaker::CircuitBreakerError::CircuitOpen => {
                Self::CircuitOpen("Circuit is open, failing fast".to_string())
            }
            crate::resilience::circuit_breaker::CircuitBreakerError::OperationFailed(msg) => {
                Self::General(format!("Operation failed: {msg}"))
            }
        }
    }
}

impl From<crate::resilience::recovery::RecoveryError> for ResilienceError {
    fn from(err: crate::resilience::recovery::RecoveryError) -> Self {
        match err {
            crate::resilience::recovery::RecoveryError::RecoveryActionFailed { message, .. } => {
                Self::RecoveryFailed(message)
            }
            crate::resilience::recovery::RecoveryError::Timeout { duration } => {
                Self::Timeout(format!("Recovery timed out after {duration:?}"))
            }
            crate::resilience::recovery::RecoveryError::MaxAttemptsExceeded { severity, attempts, max_attempts } => {
                Self::RecoveryFailed(
                    format!("Maximum recovery attempts ({max_attempts}) exceeded for {severity} failure: {attempts} attempts made")
                )
            }
            crate::resilience::recovery::RecoveryError::CriticalFailureNoRecovery => {
                Self::RecoveryFailed("Recovery not attempted for critical failure".to_string())
            }
        }
    }
}

impl From<crate::resilience::state_sync::StateSyncError> for ResilienceError {
    fn from(err: crate::resilience::state_sync::StateSyncError) -> Self {
        match err {
            crate::resilience::state_sync::StateSyncError::Timeout { .. } => {
                Self::Timeout("State synchronization timed out".to_string())
            }
            crate::resilience::state_sync::StateSyncError::SizeExceeded { size, max_size } => {
                Self::SyncFailed(format!(
                    "State size exceeds maximum: {size} > {max_size}"
                ))
            }
            crate::resilience::state_sync::StateSyncError::ValidationFailed { message } => {
                Self::SyncFailed(format!("State validation failed: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::TargetUnavailable { target } => {
                Self::SyncFailed(format!("Target unavailable: {target}"))
            }
            crate::resilience::state_sync::StateSyncError::SerializationError { message } => {
                Self::SyncFailed(format!("State serialization error: {message}"))
            }
            crate::resilience::state_sync::StateSyncError::SyncFailed {
                message,
                source: _,
            } => Self::SyncFailed(format!("Sync failed: {message}")),
            crate::resilience::state_sync::StateSyncError::NotFound(message) => {
                Self::SyncFailed(format!("State not found: {message}"))
            },
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