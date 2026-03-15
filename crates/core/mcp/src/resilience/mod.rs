// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Resilience framework for the MCP protocol
//! 
//! This module provides mechanisms for enhancing fault tolerance and reliability
//! in MCP systems. It includes circuit breakers, retry mechanisms, recovery strategies,
//! state synchronization, and health monitoring.
//!
//! The resilience framework is designed to:
//! - Prevent cascading failures using circuit breakers
//! - Handle transient errors with retry mechanisms
//! - Recover from failures using configurable strategies
//! - Synchronize state between primary and backup systems
//! - Monitor system health and trigger automatic recovery
//! - Isolate failures using bulkhead pattern
//! - Protect services from overload using rate limiting
//!
//! The main components include:
//! - `CircuitBreaker`: Prevents repeated failures by temporarily disabling operations
//! - `RetryMechanism`: Automatically retries failed operations with configurable backoff
//! - `RecoveryStrategy`: Implements recovery procedures for different types of failures
//! - `StateSynchronizer`: Manages state synchronization between distributed components
//! - `HealthMonitor`: Tracks component health and triggers recovery when needed
//! - `Bulkhead`: Isolates failures by limiting concurrent operations
//! - `RateLimiter`: Protects services from overload by limiting operations per time period

use std::fmt;
use std::error::Error as StdError;
use std::sync::Arc;
use tracing::debug;
use std::time::Duration;
use rand::Rng;

// Import directly from modules instead of re-exporting
use futures_util::future::BoxFuture;

// Import from our modules
use crate::resilience::circuit_breaker::{
    BreakerError, CircuitBreaker, StandardCircuitBreaker,
};
use crate::resilience::recovery::{FailureInfo, RecoveryStrategy};

// Import the resilience components directly
use crate::resilience::bulkhead::Bulkhead;
use crate::resilience::rate_limiter::RateLimiter;

// Import BreakerResult for type conversions

pub mod circuit_breaker;
pub mod retry;
pub mod recovery;
/// State synchronization mechanisms for resilient distributed systems
pub mod state_sync;
/// Health monitoring capabilities for system components
pub mod health;
/// Bulkhead isolation pattern for limiting concurrent calls
pub mod bulkhead;
/// Rate limiting pattern for controlling access rates
pub mod rate_limiter;
/// Error types and handling for resilience operations
pub mod resilience_error;
/// Usage examples for the resilience framework
pub mod examples;

pub use circuit_breaker::{
    BreakerState,
    BreakerMetrics,
    CircuitBreakerState,
    new_circuit_breaker,
};
pub use recovery::{FailureSeverity};
pub use retry::{
    RetryMechanism,
    RetryConfig,
    RetryMetrics,
    RetryError,
    BackoffStrategy,
};
pub use examples::{
    run_circuit_breaker_example,
    run_retry_example,
};

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

// Update the implementation to use our new BreakerError
impl From<circuit_breaker::BreakerError> for ResilienceError {
    fn from(err: circuit_breaker::BreakerError) -> Self {
        match err {
            circuit_breaker::BreakerError::CircuitOpen { name, reset_time_ms } => {
                Self::CircuitOpen(format!("Circuit '{}' is open, reset time: {}ms", name, reset_time_ms))
            }
            circuit_breaker::BreakerError::Timeout { name, timeout_ms } => {
                Self::Timeout(format!("Operation on circuit '{}' timed out after {}ms", name, timeout_ms))
            }
            circuit_breaker::BreakerError::OperationFailed { name, reason } => {
                Self::OperationFailed(format!("Operation on circuit '{}' failed: {}", name, reason))
            }
            circuit_breaker::BreakerError::Internal { name, details } => {
                Self::General(format!("Internal error in circuit '{}': {}", name, details))
            }
        }
    }
}

impl From<recovery::RecoveryError> for ResilienceError {
    fn from(err: recovery::RecoveryError) -> Self {
        match err {
            recovery::RecoveryError::MaxAttemptsExceeded { severity, attempts, max_attempts } => {
                Self::RecoveryFailed(format!(
                    "Maximum recovery attempts ({max_attempts}) exceeded for {severity} failure: {attempts} attempts made"
                ))
            }
            recovery::RecoveryError::CriticalFailureNoRecovery => {
                Self::RecoveryFailed("Recovery not attempted for critical failure".to_string())
            }
            recovery::RecoveryError::RecoveryActionFailed { message, .. } => {
                Self::RecoveryFailed(format!("Recovery action failed: {message}"))
            }
            recovery::RecoveryError::Timeout { duration } => {
                Self::Timeout(format!("Recovery timed out after {duration:?}"))
            }
        }
    }
}

impl From<state_sync::StateSyncError> for ResilienceError {
    fn from(err: state_sync::StateSyncError) -> Self {
        match err {
            state_sync::StateSyncError::Timeout { duration } => {
                Self::Timeout(format!("State synchronization timed out after {duration:?}"))
            }
            state_sync::StateSyncError::SizeExceeded { size, max_size } => {
                Self::SyncFailed(format!(
                    "State size ({size} bytes) exceeds maximum allowed size ({max_size} bytes)"
                ))
            }
            state_sync::StateSyncError::ValidationFailed { message } => {
                Self::SyncFailed(format!("State validation failed: {message}"))
            }
            state_sync::StateSyncError::TargetUnavailable { target } => {
                Self::SyncFailed(format!("Target system unavailable: {target}"))
            }
            state_sync::StateSyncError::SerializationError { message } => {
                Self::SyncFailed(format!("State serialization error: {message}"))
            }
            state_sync::StateSyncError::SyncFailed { message, .. } => {
                Self::SyncFailed(format!("Synchronization failed: {message}"))
            }
            state_sync::StateSyncError::NotFound(message) => {
                Self::SyncFailed(format!("State not found: {message}"))
            }
            state_sync::StateSyncError::DeserializationFailed(message) => {
                Self::SyncFailed(format!("State deserialization failed: {message}"))
            }
        }
    }
}

impl From<health::HealthCheckError> for ResilienceError {
    fn from(err: health::HealthCheckError) -> Self {
        match err {
            health::HealthCheckError::Timeout { component_id, duration } => {
                Self::Timeout(format!(
                    "Health check for component '{component_id}' timed out after {duration:?}"
                ))
            },
            health::HealthCheckError::CheckFailed { component_id, message } => {
                Self::General(format!(
                    "Health check for component '{component_id}' failed: {message}"
                ))
            },
            health::HealthCheckError::ComponentUnavailable { component_id } => {
                Self::General(format!(
                    "Component '{component_id}' is unavailable for health check"
                ))
            },
        }
    }
}

/// Execute an operation with bulkhead isolation
///
/// This function uses the bulkhead pattern to isolate failures and control
/// the impact of failures in one part of the application.
///
/// # Arguments
///
/// * `bulkhead` - The bulkhead instance to use
/// * `operation` - The operation to execute
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Errors
///
/// Returns an error if:
/// * The bulkhead has reached maximum concurrent calls
/// * The operation times out while waiting in the queue
/// * The operation times out during execution
/// * The operation itself fails
pub async fn with_bulkhead<F, T>(
    bulkhead: &bulkhead::Bulkhead,
    operation: F,
) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send + 'static,
    T: Send + 'static,
{
    bulkhead.execute(operation).await
}

/// Execute an operation with rate limiting
///
/// This function ensures that operations don't exceed a configured rate limit,
/// using a token bucket algorithm to control the rate.
///
/// # Arguments
///
/// * `rate_limiter` - Rate limiter to control operation throughput
/// * `operation` - The operation to execute
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Errors
///
/// Returns an error if:
/// * The rate limit is exceeded and waiting is disabled
/// * The operation times out while waiting for a rate limit permit
/// * The operation itself fails
pub async fn with_rate_limiting<F, T>(
    rate_limiter: &rate_limiter::RateLimiter,
    operation: F,
) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, Box<dyn StdError + Send + Sync>>> + Send + 'static,
    T: Send + 'static,
{
    // Convert the operation to work with RateLimitError by using the debug formatting of Box<dyn Error>
    let adapted_operation = async {
        match operation.await {
            Ok(value) => Ok(value),
            Err(error) => Err(rate_limiter::RateLimitError::OperationFailed(format!("{:?}", error)))
        }
    };
    
    rate_limiter.execute(adapted_operation).await
}

/// Execute an operation with resilience, using a circuit breaker and retry mechanism
///
/// This function combines circuit breaker and retry mechanism to enhance fault tolerance 
/// for operations that might fail transiently.
///
/// # Arguments
///
/// * `circuit_breaker` - Circuit breaker to prevent cascading failures
/// * `retry` - Retry mechanism to handle transient failures
/// * `operation` - The operation to execute
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Errors
///
/// Returns an error if the operation fails after retries or if the circuit breaker is open
pub async fn with_resilience<F, T, CB>(
    circuit_breaker: &mut CB,
    retry: retry::RetryMechanism,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
    T: Send + 'static,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    // Default component ID for error messaging
    let component_id = "resilience_component";
    
    // Execute with circuit breaker and retry
    circuit_breaker.execute(move || {
        let operation_clone = operation.clone();
        
        Box::pin(async move {
            // Use retry mechanism
            let retry_result = retry.execute(|| {
                let op = operation_clone.clone();
                
                Box::pin(async move {
                    op()
                })
            }).await;
            
            match retry_result {
                Ok(value) => Ok(value),
                Err(e) => Err(BreakerError::operation_failed(component_id, e.to_string()))
            }
        })
    }).await.map_err(|e| ResilienceError::from(e))
}

/// Create a resilient operation with recovery strategy
///
/// # Errors
///
/// This function will return an error in the following cases:
/// - If the operation fails and the recovery strategy fails to recover
/// - If the recovery action fails to execute properly
///
/// # Panics
///
/// This function might panic if:
/// - The operation closure panics during execution
/// - The recovery action closure panics during execution
/// - The recovery strategy's internal state becomes inconsistent
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
    // Try the normal operation first
    match operation() {
        Ok(value) => Ok(value),
        Err(error) => {
            debug!("Operation failed, attempting recovery: {}", error);
            
            // Try recovery
            recovery_strategy
                .handle_failure(failure_info, recovery_action)
                .map_err(std::convert::Into::into)
        }
    }
}

/// Execute an operation with health monitoring
///
/// This function checks component health before executing the operation.
/// If the component is in a critical state, it prevents the operation from executing.
///
/// # Errors
///
/// Returns an error if:
/// * The component is in critical health state
/// * The operation itself fails
pub async fn with_health_monitoring<F, T>(
    health_monitor: &health::HealthMonitor,
    component_id: &str,
    operation: F,
) -> Result<T>
where
    F: Fn() -> std::result::Result<T, Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    T: Send + 'static,
{
    // Check health status first
    let status = health_monitor.get_component_status(component_id);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::General(format!(
            "Cannot execute operation: component '{component_id}' is in critical state"
        )));
    }
    
    // Execute the operation
    match operation() {
        Ok(value) => Ok(value),
        Err(e) => Err(ResilienceError::General(format!("{e}"))),
    }
}

/// Execute an operation with complete resilience
///
/// This function combines circuit breaker, retry mechanism, recovery strategy, and health monitoring
/// to provide comprehensive protection against failures.
///
/// # Arguments
///
/// * `circuit_breaker` - Circuit breaker to prevent cascading failures
/// * `retry` - Retry mechanism to handle transient failures
/// * `recovery` - Recovery strategy to recover from failures
/// * `health_monitor` - Health monitor to track component health
/// * `component_id` - ID of the component executing the operation
/// * `failure_info` - Information about the failure to aid recovery
/// * `operation` - The operation to execute
/// * `recovery_action` - Action to take for recovery if the operation fails
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Errors
///
/// Returns an error if any of the resilience mechanisms fail
pub async fn with_complete_resilience<F, T, CB, RA>(
    circuit_breaker: &mut CB,
    retry: retry::RetryMechanism,
    recovery: &mut RecoveryStrategy,
    health_monitor: &health::HealthMonitor,
    component_id: &str,
    failure_info: FailureInfo,
    operation: F,
    recovery_action: RA,
) -> Result<T>
where
    F: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
    RA: FnOnce() -> std::result::Result<T, Box<dyn StdError + Send + Sync>> + Send + Sync + 'static,
    T: Send + 'static,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    // Clone component_id to avoid lifetime issues
    let component_id_owned = component_id.to_string();
    
    // Check health status first
    let status = health_monitor.get_component_status(&component_id_owned);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::HealthCheck(format!(
            "Cannot execute operation: component '{component_id_owned}' is in critical state"
        )));
    }
    
    // First try to execute with circuit breaker and retry
    let result = circuit_breaker.execute(move || {
        let operation_clone = operation.clone();
        let component_id_clone = component_id_owned.clone();
        
        Box::pin(async move {
            // Use retry mechanism
            let retry_result = retry.execute(|| {
                let op = operation_clone.clone();
                
                Box::pin(async move {
                    op()
                })
            }).await;
            
            match retry_result {
                Ok(value) => Ok(value),
                Err(e) => Err(BreakerError::operation_failed(&component_id_clone, e.to_string()))
            }
        })
    }).await;
    
    // Handle the result
    match result {
        Ok(value) => Ok(value),
        Err(_breaker_err) => {
            // If operation failed, try recovery
            let recovery_result = recovery.recover(
                failure_info,
                recovery_action,
            ).await;
            
            match recovery_result {
                Ok(recovery_value) => Ok(recovery_value),
                Err(recovery_err) => Err(ResilienceError::from(recovery_err)),
            }
        }
    }
}

/// Execute an operation with state synchronization
///
/// # Errors
///
/// This function will return an error in the following cases:
/// - If the state synchronization fails
/// - If the operation fails
///
/// # Panics
///
/// This function might panic if:
/// - The operation closure panics during execution
/// - The state synchronizer's internal state becomes inconsistent
pub async fn with_state_sync<T, F>(
    state_sync: &state_sync::StateSynchronizer,
    state_type: state_sync::StateType,
    state_id: &str,
    target: &str,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send,
    T: serde::Serialize + Clone + Send + Sync + 'static,
{
    // Execute the operation
    let result = operation().await?;
    
    // Synchronize the state
    state_sync.sync_state(state_type, state_id, target, &result).await?;
    
    Ok(result)
}

/// Execute an operation with comprehensive resilience
///
/// This function combines all resilience mechanisms including bulkhead isolation
/// and rate limiting to provide comprehensive protection against failures.
///
/// # Arguments
///
/// * `circuit_breaker` - Circuit breaker to prevent cascading failures
/// * `bulkhead` - Bulkhead to isolate failures
/// * `rate_limiter` - Rate limiter to protect from overload
/// * `timeout` - Maximum time to wait for the operation to complete
/// * `component_id` - ID of the component executing the operation
/// * `operation` - The operation to execute
///
/// # Returns
///
/// The result of the operation if successful
///
/// # Errors
///
/// Returns an error if any of the resilience mechanisms fail
pub async fn with_comprehensive_resilience<'a, F, R, T, CB>(
    circuit_breaker: &'a mut CB,
    bulkhead: &'a Bulkhead,
    rate_limiter: &'a RateLimiter,
    _retry_policy: R,
    timeout: Duration,
    component_id: &'a str,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> BoxFuture<'static, Result<T>> + Send + Sync + Clone + 'static,
    R: RetryPolicy + Send + Sync + 'static,
    T: Send + 'static + Clone,
    CB: circuit_breaker::CircuitBreaker + Send + Sync,
{
    // Clone component_id to avoid lifetime issues
    let component_id_owned = component_id.to_string();
    
    // Check health status first
    let _health_monitor = health::HealthMonitor::new(100);
    let status = _health_monitor.get_component_status(&component_id_owned);
    if status == health::HealthStatus::Critical {
        return Err(ResilienceError::HealthCheck(format!(
            "Cannot execute operation: component '{component_id_owned}' is in critical state"
        )));
    }
    
    // Check rate limiter
    if !rate_limiter.try_acquire().await {
        return Err(ResilienceError::RateLimit(format!(
            "Rate limit exceeded for component '{component_id_owned}'"
        )));
    }
    
    // Check bulkhead
    if !bulkhead.try_enter().await {
        return Err(ResilienceError::Bulkhead(format!(
            "Bulkhead capacity exceeded for component '{component_id_owned}'"
        )));
    }
    
    // Create a timeout wrapper
    let timeout_result = tokio::time::timeout(timeout, async {
        // Execute with circuit breaker
        circuit_breaker.execute(move || {
            let op = operation.clone();
            let component_id_str = component_id_owned.clone();
            
            Box::pin(async move {
                // Execute the operation directly
                op().await.map_err(|e| 
                    circuit_breaker::BreakerError::operation_failed(&component_id_str, e.to_string())
                )
            })
        }).await
    }).await;
    
    // Handle timeout and error conversion
    match timeout_result {
        Ok(breaker_result) => breaker_result.map_err(Into::into),
        Err(_) => Err(ResilienceError::Timeout(format!(
            "Operation timed out after {:?} for component '{}'",
            timeout, component_id
        ))),
    }
}

/// Execute with recovery and circuit breaker
/// 
/// This function forwards to the newer function definitions above
pub async fn execute_with_recovery<T, F>(
    circuit_breaker: Option<StandardCircuitBreaker>,
    component_id: &str,
    operation: F,
    _recovery_strategy: &mut RecoveryStrategy,
    _failure_info: FailureInfo,
    _recovery_action: Option<String>
) -> std::result::Result<T, ResilienceError>
where
    F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
    T: Send + 'static,
{
    // Clone component_id to avoid borrow issues
    let component_id_owned = component_id.to_string();
    
    with_circuit_breaker(circuit_breaker, &component_id_owned, operation).await
}

// Helper function for the above
#[doc(hidden)]
async fn with_circuit_breaker<T, F>(
    mut circuit_breaker: Option<StandardCircuitBreaker>,
    component_id: &str,
    operation: F
) -> std::result::Result<T, ResilienceError>
where
    F: FnOnce() -> core::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, ResilienceError>> + Send>> + Send,
    T: Send + 'static,
{
    // Clone the component_id to avoid borrow issues
    let component_id_owned = component_id.to_string();
    
    match circuit_breaker {
        Some(ref mut cb) => {
            // Define future for the circuit breaker to execute
            let fut = operation();
            
            // Use the circuit breaker
            cb.execute(move || {
                use futures_util::FutureExt;
                
                // Convert future to BreakerResult
                async move {
                    match fut.await {
                        Ok(result) => Ok(result),
                        Err(err) => Err(circuit_breaker::BreakerError::operation_failed(
                            &component_id_owned, err.to_string()
                        ))
                    }
                }.boxed()
            }).await.map_err(|e| e.into())
        },
        None => {
            // No circuit breaker, just run the operation directly
            operation().await
        }
    }
}

/// A builder for resilience components
pub struct ResilienceBuilder {
    bulkhead: Option<Arc<Bulkhead>>,
    rate_limiter: Option<Arc<RateLimiter>>,
    circuit_breaker: Option<StandardCircuitBreaker>,
    retry_policy: Option<Box<dyn RetryPolicy + Send + Sync>>,
    timeout: Option<Duration>,
}

// Manual Debug implementation to avoid requiring Debug for all fields
impl fmt::Debug for ResilienceBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResilienceBuilder")
            .field("bulkhead", &format_args!("<bulkhead>"))
            .field("rate_limiter", &format_args!("<rate_limiter>"))
            .field("circuit_breaker", &format_args!("<circuit_breaker>"))
            .field("retry_policy", &format_args!("<retry_policy>"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

// Manual Clone implementation since trait objects don't implement Clone
impl Clone for ResilienceBuilder {
    fn clone(&self) -> Self {
        Self {
            bulkhead: self.bulkhead.clone(),
            rate_limiter: self.rate_limiter.clone(),
            circuit_breaker: self.circuit_breaker.clone(),
            retry_policy: None, // Can't clone trait objects, so create a new one when needed
            timeout: self.timeout,
        }
    }
}

// Builder methods
impl ResilienceBuilder {
    pub fn new() -> Self {
        Self {
            bulkhead: None,
            rate_limiter: None,
            circuit_breaker: None,
            retry_policy: None,
            timeout: None,
        }
    }
    
    pub fn with_bulkhead(mut self, bulkhead: Arc<Bulkhead>) -> Self {
        self.bulkhead = Some(bulkhead);
        self
    }
    
    pub fn with_rate_limiter(mut self, rate_limiter: Arc<RateLimiter>) -> Self {
        self.rate_limiter = Some(rate_limiter);
        self
    }
    
    pub fn with_circuit_breaker(mut self, circuit_breaker: StandardCircuitBreaker) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }
    
    pub fn with_retry_policy(mut self, retry_policy: Box<dyn RetryPolicy + Send + Sync>) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn build(self) -> Self {
        self
    }
}

/// Policy for determining if and how an operation should be retried
pub trait RetryPolicy: Send + Sync {
    /// Determines if a failed operation should be retried
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool;
    
    /// Determines the backoff duration for a retry attempt
    fn backoff_duration(&self, attempt: usize) -> Duration;
}

/// Standard implementation of the RetryPolicy trait
pub struct StandardRetryPolicy {
    /// Maximum number of retry attempts
    max_retries: usize,
    /// Base delay for retry attempts
    base_delay: Duration,
    /// Maximum delay for any retry attempt
    max_delay: Duration,
    /// Backoff strategy for calculating delays
    backoff_strategy: BackoffStrategy,
    /// Whether to use jitter to avoid retry storms
    use_jitter: bool,
}

impl StandardRetryPolicy {
    /// Creates a new retry policy with the specified parameters
    pub fn new(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
        backoff_strategy: BackoffStrategy,
        use_jitter: bool,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_strategy,
            use_jitter,
        }
    }
    
    /// Creates a new retry policy with default parameters
    pub fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_strategy: BackoffStrategy::Exponential,
            use_jitter: true,
        }
    }
    
    /// Creates a retry policy with the specified maximum retries
    pub fn with_max_retries(max_retries: usize) -> Self {
        Self {
            max_retries,
            ..Self::default()
        }
    }
    
    /// Creates a retry policy with exponential backoff
    pub fn with_exponential_backoff(
        max_retries: usize,
        base_delay: Duration,
        max_delay: Duration,
    ) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
            backoff_strategy: BackoffStrategy::Exponential,
            use_jitter: true,
        }
    }
    
    /// Applies jitter to a delay value
    fn apply_jitter(&self, delay: Duration) -> Duration {
        if self.use_jitter {
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.0..1.0);
            let jitter_millis = (delay.as_millis() as f64 * jitter_factor) as u64;
            Duration::from_millis(jitter_millis)
        } else {
            delay
        }
    }
}

impl RetryPolicy for StandardRetryPolicy {
    fn should_retry(&self, attempt: usize, error: &ResilienceError) -> bool {
        // Don't retry if we've exceeded the maximum attempts
        if attempt >= self.max_retries {
            return false;
        }
        
        // Retry based on error type
        match error {
            // Don't retry if the circuit is open
            ResilienceError::CircuitOpen(_) => false,
            
            // Always retry timeouts
            ResilienceError::Timeout(_) => true,
            
            // Retry rate limit errors (these are often transient)
            ResilienceError::RateLimit(_) => true,
            
            // For operation failures, retry by default
            ResilienceError::OperationFailed(_) => true,
            
            // For synchronization failures, retry by default
            ResilienceError::SyncFailed(_) => true,
            
            // Don't retry if we've already exceeded retry attempts
            ResilienceError::RetryExceeded(_) => false,
            
            // Don't retry recovery failures
            ResilienceError::RecoveryFailed(_) => false,
            
            // Don't retry bulkhead isolation errors
            ResilienceError::Bulkhead(_) => false,
            
            // Don't retry health check failures
            ResilienceError::HealthCheck(_) => false,
            
            // For general errors, retry by default
            ResilienceError::General(_) => true,
        }
    }
    
    fn backoff_duration(&self, attempt: usize) -> Duration {
        let base_delay = match self.backoff_strategy {
            BackoffStrategy::Constant => self.base_delay,
            
            BackoffStrategy::Linear => {
                self.base_delay.mul_f32(attempt as f32)
            },
            
            BackoffStrategy::Exponential => {
                // 2^attempt scaling for exponential backoff
                let scale = 2u32.pow(attempt as u32) as f32;
                self.base_delay.mul_f32(scale)
            },
            
            BackoffStrategy::Fibonacci => {
                // Calculate Fibonacci number
                let mut a = 1;
                let mut b = 1;
                for _ in 0..attempt {
                    let temp = a;
                    a = b;
                    b = temp + b;
                }
                self.base_delay.mul_f32(a as f32)
            },
        };
        
        // Apply jitter and respect max delay
        let delay_with_jitter = self.apply_jitter(base_delay);
        if delay_with_jitter > self.max_delay {
            self.max_delay
        } else {
            delay_with_jitter
        }
    }
}

/// Handles an error and converts it to a resilience error
fn handle_resilience_error<E: std::error::Error + Send + Sync + 'static>(error: E, component_id: &str) -> ResilienceError {
    // Try to convert known error types
    let error_string = error.to_string();
    
    if error_string.contains("circuit is open") || error_string.contains("Circuit") {
        return ResilienceError::CircuitOpen(
            format!("Circuit for component '{}' is open", component_id)
        );
    } else if error_string.contains("timeout") || error_string.contains("Timeout") {
        return ResilienceError::Timeout(
            format!("Operation for component '{}' timed out", component_id)
        );
    } else if error_string.contains("retry") || error_string.contains("Retry") {
        return ResilienceError::RetryExceeded(
            format!("Maximum retry attempts for component '{}' exceeded", component_id)
        );
    } else if error_string.contains("bulkhead") || error_string.contains("Bulkhead") {
        return ResilienceError::Bulkhead(
            format!("Bulkhead for component '{}' rejected request", component_id)
        );
    } else if error_string.contains("rate limit") || error_string.contains("Rate") {
        return ResilienceError::RateLimit(
            format!("Rate limit for component '{}' exceeded", component_id)
        );
    }
    
    // Unknown error type
    ResilienceError::General(format!("Component '{}' error: {}", component_id, error))
}

pub async fn execute_with_resilience_components<'a>(
    component_id: &'a str,
    operation: impl FnOnce() -> std::result::Result<(), Box<dyn StdError + Send + Sync>> + Clone + Send + Sync + 'static,
) -> Result<()> {
    // Create resilience components
    let mut circuit_breaker = circuit_breaker::new_circuit_breaker(component_id);
    
    // Create retry mechanism
    let retry = retry::RetryMechanism::default();
    
    // Execute with resilience using new signature
    with_resilience(
        &mut circuit_breaker,
        retry,
        operation,
    ).await
}
