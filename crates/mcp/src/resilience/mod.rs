//! Resilience framework for the MCP protocol
//! 
//! This module provides mechanisms for enhancing fault tolerance and reliability
//! in MCP systems. It includes circuit breakers, retry mechanisms, recovery strategies,
//! state synchronization, and health monitoring.

use std::fmt;
use std::error::Error;

pub mod circuit_breaker;
pub mod retry;
pub mod recovery;
pub mod state_sync;
pub mod health;

#[cfg(test)]
pub mod tests;

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
}

impl fmt::Display for ResilienceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResilienceError::CircuitOpen(msg) => write!(f, "Circuit open: {}", msg),
            ResilienceError::RetryExceeded(msg) => write!(f, "Retry exceeded: {}", msg),
            ResilienceError::RecoveryFailed(msg) => write!(f, "Recovery failed: {}", msg),
            ResilienceError::SyncFailed(msg) => write!(f, "State synchronization failed: {}", msg),
            ResilienceError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ResilienceError::General(msg) => write!(f, "Resilience error: {}", msg),
        }
    }
}

impl Error for ResilienceError {}

/// Convenience type alias for Results from resilience operations
pub type Result<T> = std::result::Result<T, ResilienceError>;

// Implement From for the various component errors
impl From<circuit_breaker::CircuitBreakerError> for ResilienceError {
    fn from(err: circuit_breaker::CircuitBreakerError) -> Self {
        match err {
            circuit_breaker::CircuitBreakerError::CircuitOpen => {
                ResilienceError::CircuitOpen("Circuit is open".to_string())
            }
            circuit_breaker::CircuitBreakerError::OperationFailed(msg) => {
                ResilienceError::General(format!("Circuit breaker operation failed: {}", msg))
            }
        }
    }
}

impl From<retry::RetryError> for ResilienceError {
    fn from(err: retry::RetryError) -> Self {
        match err {
            retry::RetryError::MaxAttemptsExceeded { attempts } => {
                ResilienceError::RetryExceeded(format!("Maximum retry attempts ({}) exceeded", attempts))
            }
            retry::RetryError::OperationCancelled => {
                ResilienceError::General("Retry operation was cancelled".to_string())
            }
            retry::RetryError::InternalError(msg) => {
                ResilienceError::General(format!("Retry internal error: {}", msg))
            }
        }
    }
}

impl From<recovery::RecoveryError> for ResilienceError {
    fn from(err: recovery::RecoveryError) -> Self {
        match err {
            recovery::RecoveryError::MaxAttemptsExceeded { severity, attempts, max_attempts } => {
                ResilienceError::RecoveryFailed(format!(
                    "Maximum recovery attempts ({}) exceeded for {} failure: {} attempts made",
                    max_attempts, severity, attempts
                ))
            }
            recovery::RecoveryError::CriticalFailureNoRecovery => {
                ResilienceError::RecoveryFailed("Recovery not attempted for critical failure".to_string())
            }
            recovery::RecoveryError::RecoveryActionFailed { message, .. } => {
                ResilienceError::RecoveryFailed(format!("Recovery action failed: {}", message))
            }
            recovery::RecoveryError::Timeout { duration } => {
                ResilienceError::Timeout(format!("Recovery timed out after {:?}", duration))
            }
        }
    }
}

impl From<state_sync::StateSyncError> for ResilienceError {
    fn from(err: state_sync::StateSyncError) -> Self {
        match err {
            state_sync::StateSyncError::Timeout { duration } => {
                ResilienceError::Timeout(format!("State synchronization timed out after {:?}", duration))
            }
            state_sync::StateSyncError::SizeExceeded { size, max_size } => {
                ResilienceError::SyncFailed(format!(
                    "State size ({} bytes) exceeds maximum allowed size ({} bytes)", 
                    size, max_size
                ))
            }
            state_sync::StateSyncError::ValidationFailed { message } => {
                ResilienceError::SyncFailed(format!("State validation failed: {}", message))
            }
            state_sync::StateSyncError::TargetUnavailable { target } => {
                ResilienceError::SyncFailed(format!("Target system unavailable: {}", target))
            }
            state_sync::StateSyncError::SerializationError { message } => {
                ResilienceError::SyncFailed(format!("State serialization error: {}", message))
            }
            state_sync::StateSyncError::SyncFailed { message, .. } => {
                ResilienceError::SyncFailed(format!("Synchronization failed: {}", message))
            }
        }
    }
}

/// Create a resilient operation using circuit breaker and retry
pub fn with_resilience<F, T, E>(
    circuit_breaker: &mut circuit_breaker::CircuitBreaker,
    retry: &retry::RetryMechanism,
    operation: F,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, E>,
    E: Error + Send + Sync + 'static,
{
    circuit_breaker.execute(|| {
        retry.execute(|| {
            operation().map_err(|e| {
                Box::new(ResilienceError::General(format!("{}", e))) as Box<dyn Error + Send + Sync>
            })
        })
        .map_err(|e| e.into())
    })
}

/// Create a resilient operation with recovery strategy
pub fn with_recovery<F, R, T, E>(
    recovery_strategy: &mut recovery::RecoveryStrategy,
    failure_info: recovery::FailureInfo,
    operation: F,
    recovery_action: R,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, E>,
    R: FnOnce() -> std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>,
    E: Error + Send + Sync + 'static,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(e) => {
            // Operation failed, attempt recovery
            recovery_strategy
                .handle_failure(failure_info, recovery_action)
                .map_err(|e| e.into())
        }
    }
}

/// Create a fully resilient operation using circuit breaker, retry, and recovery
pub fn with_full_resilience<F, R, T, E>(
    circuit_breaker: &mut circuit_breaker::CircuitBreaker,
    retry: &retry::RetryMechanism,
    recovery_strategy: &mut recovery::RecoveryStrategy,
    failure_info: recovery::FailureInfo,
    operation: F,
    recovery_action: R,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, E> + Clone,
    R: FnOnce() -> std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>,
    E: Error + Send + Sync + 'static,
{
    // Start with circuit breaker as the outermost protection
    circuit_breaker.execute(|| {
        // Try with retry first
        let op_clone = operation.clone();
        let retry_result = retry.execute(move || {
            op_clone().map_err(|e| {
                Box::new(ResilienceError::General(format!("{}", e))) as Box<dyn Error + Send + Sync>
            })
        });
        
        // If retry fails, attempt recovery
        match retry_result {
            Ok(result) => Ok(result),
            Err(_) => {
                // Retry failed, try recovery
                recovery_strategy
                    .handle_failure(failure_info, recovery_action)
                    .map_err(|e| e.into())
            }
        }
    })
}

/// Synchronize state using the state synchronizer
pub fn with_state_sync<T, F>(
    state_sync: &state_sync::StateSynchronizer,
    state_type: state_sync::StateType,
    state_id: &str,
    target: &str,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T>,
    T: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Execute the operation first
    let result = operation()?;
    
    // If successful, synchronize the state
    state_sync.sync_state(state_type, state_id, target, result.clone())
        .map_err(|e| e.into())?;
    
    // Return the original operation result
    Ok(result)
} 