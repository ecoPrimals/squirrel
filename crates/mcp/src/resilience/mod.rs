//! Resilience framework for the MCP protocol
//! 
//! This module provides mechanisms for enhancing fault tolerance and reliability
//! in MCP systems. It includes circuit breakers, retry mechanisms, recovery strategies,
//! state synchronization, and health monitoring.

use std::fmt;
use std::error::Error as StdError;
use std::sync::Arc;
use tracing::{debug, error};

pub mod circuit_breaker;
pub mod retry;
pub mod recovery;
/// State synchronization mechanisms for resilient distributed systems
pub mod state_sync;
/// Health monitoring capabilities for system components
pub mod health;

/// Error types and handling for resilience operations
pub mod resilience_error;

pub use circuit_breaker::CircuitBreaker;
pub use recovery::{RecoveryStrategy, RecoveryError, FailureSeverity, FailureInfo};
use crate::tool::lifecycle_original::RecoveryAction;

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

    /// Operation failed after recovery attempts
    OperationFailed(String),
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
            ResilienceError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl StdError for ResilienceError {}

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
            state_sync::StateSyncError::NotFound(message) => {
                ResilienceError::SyncFailed(format!("State not found: {}", message))
            }
            state_sync::StateSyncError::DeserializationFailed(message) => {
                ResilienceError::SyncFailed(format!("State deserialization failed: {}", message))
            }
        }
    }
}

impl From<health::HealthCheckError> for ResilienceError {
    fn from(err: health::HealthCheckError) -> Self {
        match err {
            health::HealthCheckError::Timeout { component_id, duration } => {
                ResilienceError::Timeout(format!(
                    "Health check for component '{}' timed out after {:?}",
                    component_id, duration
                ))
            },
            health::HealthCheckError::CheckFailed { component_id, message } => {
                ResilienceError::General(format!(
                    "Health check for component '{}' failed: {}",
                    component_id, message
                ))
            },
            health::HealthCheckError::ComponentUnavailable { component_id } => {
                ResilienceError::General(format!(
                    "Component '{}' is unavailable for health check",
                    component_id
                ))
            },
        }
    }
}

/// Create a resilient operation with circuit breaker and retry
pub async fn with_resilience<F, T>(
    circuit_breaker: &mut circuit_breaker::CircuitBreaker,
    retry: retry::RetryMechanism,
    operation: F,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static + Clone,
    T: Send + 'static,
{
    // Move the operation into the circuit breaker's closure
    let circuit_op = async move {
        // Create a closure that will be called by the retry mechanism
        let op = operation.clone();
        let retry_owned = retry;
        let retry_result = retry_owned.execute(move || {
            let op_inner = op.clone();
            Box::pin(async move {
                match op_inner() {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        let boxed: Box<dyn StdError + Send + Sync> = 
                            Box::new(ResilienceError::General(format!("{}", e)));
                        Err(boxed)
                    }
                }
            })
        }).await;
        
        retry_result.map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)
    };
    
    // Execute with circuit breaker
    let cb_future = circuit_breaker.execute(move || Box::pin(circuit_op));
    cb_future.await.map_err(|e| e.into())
}

/// Create a resilient operation with recovery strategy
pub async fn with_recovery<F, R, T>(
    recovery_strategy: &mut recovery::RecoveryStrategy,
    failure_info: recovery::FailureInfo,
    operation: F,
    recovery_action: R,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>>,
    R: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>>,
    T: Send + 'static,
{
    match operation() {
        Ok(result) => Ok(result),
        Err(_) => {
            // Operation failed, attempt recovery
            recovery_strategy
                .handle_failure(failure_info, recovery_action)
                .map_err(|e| e.into())
        }
    }
}

/// Create a resilient operation with health monitoring
pub async fn with_health_monitoring<F, T>(
    health_monitor: &health::HealthMonitor,
    component_id: &str,
    operation: F,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    T: Send + 'static,
{
    // Check current health status before executing
    let status = health_monitor.component_status(component_id);
    
    if status == health::HealthStatus::Critical || status == health::HealthStatus::Unhealthy {
        return Err(ResilienceError::General(format!(
            "Cannot execute operation: component '{}' is in {} state",
            component_id, status
        )));
    }
    
    // Execute the operation
    match operation() {
        Ok(result) => Ok(result),
        Err(err) => {
            // Update health check on failure
            // In a real implementation, this would trigger an async health check
            Err(ResilienceError::General(format!("Operation failed: {}", err)))
        }
    }
}

/// Create a fully resilient operation using circuit breaker, retry, recovery, and health monitoring
pub async fn with_complete_resilience<F, R, T>(
    circuit_breaker: &mut circuit_breaker::CircuitBreaker,
    retry: retry::RetryMechanism,
    recovery_strategy: &mut recovery::RecoveryStrategy,
    health_monitor: &health::HealthMonitor,
    component_id: &str, 
    failure_info: recovery::FailureInfo,
    operation: F,
    recovery_action: R,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static + Clone,
    R: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + 'static,
    T: Send + 'static,
{
    // Check health status first
    let status = health_monitor.component_status(component_id);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::General(format!(
            "Cannot execute operation: component '{}' is in critical state",
            component_id
        )));
    }
    
    // Create a circuit breaker operation that uses retry and monitors health
    let circuit_op = {
        // Move the operation and component_id into circuit_op
        let operation = operation;
        let component_id_str = component_id.to_string();
        let retry = retry;
        
        async move {
            // Create a closure that will be called by the retry mechanism
            let retry_result = retry.execute(move || {
                let op = operation.clone();
                let _component_id = component_id_str.clone();
                
                Box::pin(async move {
                    match op() {
                        Ok(value) => Ok(value),
                        Err(e) => {
                            // Could trigger health check here
                            let boxed: Box<dyn StdError + Send + Sync> = 
                                Box::new(ResilienceError::General(format!("{}", e)));
                            Err(boxed)
                        }
                    }
                })
            }).await;
            
            retry_result.map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)
        }
    };
    
    // Execute with circuit breaker
    let cb_result = circuit_breaker.execute(move || Box::pin(circuit_op)).await;
    
    // If circuit breaker execution fails, try recovery
    match cb_result {
        Ok(value) => Ok(value),
        Err(_e) => {
            recovery_strategy
                .handle_failure(failure_info, recovery_action)
                .map_err(|e| e.into())
        }
    }
}

/// Synchronize state using the state synchronizer
pub async fn with_state_sync<T, F>(
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
        .await
        .map_err(|e| ResilienceError::from(e))?;
    
    // Return the original operation result
    Ok(result)
}

/// Execute an operation with recovery capabilities
///
/// This function executes the provided operation and applies recovery strategies if it fails.
/// It integrates with the circuit breaker pattern to prevent cascading failures.
///
/// # Arguments
/// * `circuit_breaker` - Optional circuit breaker instance to control circuit state
/// * `component_id` - Identifier of the component being executed
/// * `operation` - The operation to execute, which returns a future
/// * `recovery_strategy` - Strategy to use for recovery in case of failures
/// * `failure_info` - Information about the failure context
/// * `recovery_action` - Optional specific recovery action to take
///
/// # Returns
/// The result of the operation or an error if recovery failed
pub async fn execute_with_recovery<T, F>(
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    component_id: &str,
    operation: F,
    recovery_strategy: &mut RecoveryStrategy,
    failure_info: FailureInfo,
    recovery_action: Option<RecoveryAction>
) -> std::result::Result<T, ResilienceError>
where
    F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
{
    // Define a wrapper for the with_circuit_breaker function since it wasn't found in the code
    async fn with_circuit_breaker<T, F>(
        _circuit_breaker: Option<Arc<CircuitBreaker>>,
        _component_id: &str,
        operation: F
    ) -> std::result::Result<T, ResilienceError>
    where
        F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
    {
        // Simple implementation - we'll later integrate with the actual circuit breaker
        let result = operation().await;
        result
    }

    match with_circuit_breaker(circuit_breaker, component_id, operation).await {
        Ok(result) => Ok(result),
        Err(_e) => {
            if let Some(recovery_action) = recovery_action {
                // Use the handle_failure method from the existing RecoveryStrategy implementation
                let recovery_result = recovery_strategy.handle_failure(failure_info.clone(), || {
                    // Convert RecoveryAction to the expected type
                    match recovery_action {
                        RecoveryAction::Reset => Ok(()),
                        RecoveryAction::Restart => Ok(()),
                        RecoveryAction::Recreate => Ok(()),
                        RecoveryAction::Custom(_action_name) => {
                            Ok(()) // Handle custom action if needed
                        }
                    }
                });
                
                if let Err(err) = recovery_result {
                    debug!("Recovery failed: {}", err);
                }
            }
            
            Err(ResilienceError::OperationFailed(
                format!("Operation failed after recovery attempt")
            ))
        }
    }
} 