//! Circuit breaker implementation for the MCP resilience framework
//! 
//! This module provides a circuit breaker pattern implementation to prevent
//! cascading failures in distributed systems.

use std::time::Instant;
use std::fmt;
use std::error::Error;
use crate::resilience::{ResilienceError, Result};

/// Error type for circuit breaker operations
#[derive(Debug)]
pub enum CircuitBreakerError {
    /// Circuit is open, operation not allowed
    CircuitOpen,
    /// Operation failed
    OperationFailed(String),
}

impl fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CircuitOpen => write!(f, "Circuit is open"),
            Self::OperationFailed(msg) => write!(f, "Operation failed: {msg}"),
        }
    }
}

impl Error for CircuitBreakerError {}

/// The current state of the circuit breaker
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, operations proceed normally
    Closed,
    /// Circuit is open, operations are rejected
    Open,
    /// Circuit is half-open, limited operations are allowed to test recovery
    HalfOpen,
}

/// Configuration for the circuit breaker
pub struct CircuitBreakerConfig {
    /// Name of the circuit breaker for identification
    pub name: String,
    /// Number of failures before the circuit opens
    pub failure_threshold: u32,
    /// Duration in milliseconds to wait before transitioning from open to half-open
    pub recovery_timeout_ms: u64,
    /// Number of successful calls needed in half-open state to close the circuit
    pub half_open_success_threshold: u32,
    /// Maximum number of calls allowed in half-open state
    pub half_open_allowed_calls: u32,
    /// Fallback function to use when circuit is open
    pub fallback: Option<Box<dyn Fn() -> i32 + Send + Sync>>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            failure_threshold: 5,
            recovery_timeout_ms: 5000,
            half_open_success_threshold: 2,
            half_open_allowed_calls: 3,
            fallback: None,
        }
    }
}

impl std::fmt::Debug for CircuitBreakerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreakerConfig")
            .field("name", &self.name)
            .field("failure_threshold", &self.failure_threshold)
            .field("recovery_timeout_ms", &self.recovery_timeout_ms)
            .field("half_open_success_threshold", &self.half_open_success_threshold)
            .field("half_open_allowed_calls", &self.half_open_allowed_calls)
            .field("fallback", &if self.fallback.is_some() { "Some(Fn)" } else { "None" })
            .finish()
    }
}

impl Clone for CircuitBreakerConfig {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            failure_threshold: self.failure_threshold,
            recovery_timeout_ms: self.recovery_timeout_ms,
            half_open_success_threshold: self.half_open_success_threshold,
            half_open_allowed_calls: self.half_open_allowed_calls,
            fallback: None, // We don't clone the fallback function
        }
    }
}

/// Circuit breaker for fault tolerance
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration for this circuit breaker
    config: CircuitBreakerConfig,
    /// Current state of the circuit
    state: CircuitState,
    /// Time when the circuit entered the open state
    open_time: Option<Instant>,
    /// Number of consecutive failures in closed state
    failure_count: u32,
    /// Number of successful calls in half-open state
    half_open_success_count: u32,
    /// Number of calls attempted in half-open state
    half_open_call_count: u32,
    /// Total number of successful calls
    success_count: u64,
    /// Total number of failed calls
    failure_total_count: u64,
    /// Number of times the circuit has tripped open
    open_count: u64,
    /// Number of times the fallback was used
    fallback_count: u64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the specified configuration
    #[must_use] pub const fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            open_time: None,
            failure_count: 0,
            half_open_success_count: 0,
            half_open_call_count: 0,
            success_count: 0,
            failure_total_count: 0,
            open_count: 0,
            fallback_count: 0,
        }
    }
    
    /// Create a new circuit breaker with default configuration
    #[must_use] pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
    
    /// Get the current state of the circuit breaker
    #[must_use] pub const fn state(&self) -> CircuitState {
        self.state
    }
    
    /// Reset the circuit breaker to its initial state
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.open_time = None;
        self.failure_count = 0;
        self.half_open_success_count = 0;
        self.half_open_call_count = 0;
    }
    
    /// Execute an operation with the circuit breaker
    ///
    /// Runs an asynchronous operation using the circuit breaker pattern to prevent
    /// cascading failures. The operation will only be executed if the circuit is closed
    /// or in a half-open test state.
    /// 
    /// # Arguments
    /// 
    /// * `operation` - The operation to execute, provided as a closure that returns a future
    ///
    /// # Returns
    /// 
    /// The result of the operation if successful, or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The circuit breaker is open (`CircuitBreakerError::CircuitOpen`)
    /// * The operation fails (`CircuitBreakerError::OperationFailed`)
    /// * The fallback function returns an error when the circuit is open
    pub async fn execute<F, T>(&mut self, operation: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
        T: Send + 'static + From<i32>,
    {
        // Check circuit state
        match self.state {
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                if let Some(open_time) = self.open_time {
                    #[allow(clippy::cast_possible_truncation)] // u128 -> u64 for elapsed millis is safe here
                    let elapsed = open_time.elapsed().as_millis() as u64;
                    if elapsed >= self.config.recovery_timeout_ms {
                        // Transition to half-open state
                        self.state = CircuitState::HalfOpen;
                        self.half_open_call_count = 0;
                        self.half_open_success_count = 0;
                    } else {
                        // Circuit is still open
                        return self.handle_open_circuit();
                    }
                } else {
                    // No open time recorded, should not happen
                    self.state = CircuitState::Closed;
                }
            }
            CircuitState::HalfOpen => {
                // Check if we've reached the limit of allowed calls in half-open state
                if self.half_open_call_count >= self.config.half_open_allowed_calls {
                    return Err(ResilienceError::from(CircuitBreakerError::CircuitOpen));
                }
                self.half_open_call_count += 1;
            }
            CircuitState::Closed => {
                // Proceed normally
            }
        }
        
        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.handle_success();
                Ok(result)
            }
            Err(error) => {
                self.handle_failure();
                Err(ResilienceError::from(CircuitBreakerError::OperationFailed(error.to_string())))
            }
        }
    }
    
    // Handle successful operation
    fn handle_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count = 0;
                self.success_count += 1;
            }
            CircuitState::HalfOpen => {
                // Count successes in half-open state
                self.half_open_success_count += 1;
                self.success_count += 1;
                
                // Check if we've reached the threshold to close the circuit
                if self.half_open_success_count >= self.config.half_open_success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                }
            }
            CircuitState::Open => {
                // Should not reach here, but handle gracefully
                self.success_count += 1;
            }
        }
    }
    
    // Handle operation failure
    fn handle_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Increment failure count
                self.failure_count += 1;
                self.failure_total_count += 1;
                
                // Check if we've reached the threshold to open the circuit
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open;
                    self.open_time = Some(Instant::now());
                    self.open_count += 1;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                self.state = CircuitState::Open;
                self.open_time = Some(Instant::now());
                self.failure_total_count += 1;
                self.open_count += 1;
            }
            CircuitState::Open => {
                // Should not reach here, but handle gracefully
                self.failure_total_count += 1;
            }
        }
    }
    
    // Handle open circuit by returning an error or using fallback
    fn handle_open_circuit<T>(&mut self) -> Result<T>
    where
        T: From<i32>,
    {
        // Use fallback if available
        if let Some(fallback) = &self.config.fallback {
            let fallback_value = fallback();
            self.fallback_count += 1;
            return Ok(T::from(fallback_value));
        }
        
        // Return error if we get here (no fallback)
        Err(ResilienceError::from(CircuitBreakerError::CircuitOpen))
    }
    
    /// Get metrics about the circuit breaker
    #[must_use] pub fn get_metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            name: self.config.name.clone(),
            state: self.state,
            success_count: self.success_count,
            failure_count: self.failure_total_count,
            open_count: self.open_count,
            fallback_count: self.fallback_count,
        }
    }

    /// Manually check and update the circuit state based on timeouts
    /// 
    /// This method checks if an open circuit should transition to half-open
    /// based on the recovery timeout.
    pub fn check_state_transition(&mut self) {
        if self.state == CircuitState::Open {
            if let Some(open_time) = self.open_time {
                #[allow(clippy::cast_possible_truncation)] // u128 -> u64 for elapsed millis is safe here
                let elapsed = open_time.elapsed().as_millis() as u64;
                if elapsed >= self.config.recovery_timeout_ms {
                    // Transition to half-open state
                    self.state = CircuitState::HalfOpen;
                    self.half_open_call_count = 0;
                    self.half_open_success_count = 0;
                }
            }
        }
    }
    
    /// Execute an operation with the circuit breaker using a fallback if available
    ///
    /// Similar to `execute`, but will always attempt to use the fallback function
    /// if the circuit is open or the operation fails.
    /// 
    /// # Arguments
    /// 
    /// * `operation` - The operation to execute, provided as a closure that returns a future
    ///
    /// # Returns
    /// 
    /// The result of the operation if successful, or the fallback result if available,
    /// or an error
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The circuit breaker is open and no fallback is available
    /// * The operation fails and no fallback is available
    pub async fn execute_with_fallback<F, T>(&mut self, operation: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
        T: Send + 'static + From<i32>,
    {
        // Check circuit state
        if self.state == CircuitState::Open {
            // If the circuit is open, try to use the fallback
            if let Some(fallback) = &self.config.fallback {
                let fallback_value = fallback();
                self.fallback_count += 1;
                return Ok(T::from(fallback_value));
            }
            return Err(ResilienceError::from(CircuitBreakerError::CircuitOpen));
        }
        
        // Try the regular execution
        match self.execute(operation).await {
            Ok(result) => Ok(result),
            Err(error) => {
                // On failure, try the fallback
                if let Some(fallback) = &self.config.fallback {
                    let fallback_value = fallback();
                    self.fallback_count += 1;
                    Ok(T::from(fallback_value))
                } else {
                    Err(error)
                }
            }
        }
    }
}

/// Metrics for the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    /// Name of the circuit breaker
    pub name: String,
    /// Current state of the circuit
    pub state: CircuitState,
    /// Total number of successful calls
    pub success_count: u64,
    /// Total number of failed calls
    pub failure_count: u64,
    /// Number of times the circuit has tripped open
    pub open_count: u64,
    /// Number of times the fallback was used
    pub fallback_count: u64,
} 