//! Circuit breaker implementation
//!
//! This module defines the core CircuitBreaker trait and its standard implementation.

use std::sync::Arc;
use async_trait::async_trait;
use futures_util::future::BoxFuture as FuturesBoxFuture;
use crate::monitoring::MonitoringClient;

// Import the BreakerError and BreakerResult from error module instead
use super::error::BreakerResult;
use super::config::BreakerConfig;
use super::state::{BreakerState, CircuitBreakerState};
use super::metrics::BreakerMetrics;
use super::standard_state::StandardBreakerState;

/// BoxFuture type for circuit breaker async operations
pub type BoxFuture<'a, T> = FuturesBoxFuture<'a, T>;

/// Basic object-safe circuit breaker trait
#[async_trait]
pub trait CircuitBreaker: Send + Sync {
    /// Get the name of the circuit breaker
    fn name(&self) -> &str;
    
    /// Execute an async operation with circuit breaker protection
    async fn execute<T, F>(&self, operation: F) -> BreakerResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> BoxFuture<'static, BreakerResult<T>> + Send + 'static;
    
    /// Get the current circuit breaker metrics
    async fn metrics(&self) -> BreakerMetrics;
    
    /// Get the circuit breaker configuration
    fn config(&self) -> &BreakerConfig;
    
    /// Get the current circuit state
    async fn state(&self) -> BreakerState;
}

/// Standard implementation of a circuit breaker
#[derive(Clone)]
pub struct StandardCircuitBreaker {
    /// The name of this circuit breaker
    name: String,
    /// The circuit breaker state machine
    state: Arc<Box<dyn CircuitBreakerState + Send + Sync>>,
    /// The monitoring client for reporting metrics
    monitoring: Option<Arc<dyn MonitoringClient + Send + Sync>>,
}

impl StandardCircuitBreaker {
    /// Create a new StandardCircuitBreaker with the given configuration
    pub fn new(config: BreakerConfig) -> Self {
        let name = config.name.clone();
        let state = Box::new(StandardBreakerState::new(config.clone()));
        
        Self {
            name,
            state: Arc::new(state),
            monitoring: None,
        }
    }
    
    /// Set a monitoring client for this circuit breaker
    pub fn with_monitoring(mut self, client: Arc<dyn MonitoringClient + Send + Sync>) -> Self {
        self.monitoring = Some(client);
        self
    }
}

#[async_trait]
impl CircuitBreaker for StandardCircuitBreaker {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn config(&self) -> &BreakerConfig {
        self.state.config()
    }
    
    async fn state(&self) -> BreakerState {
        self.state.state().await
    }
    
    async fn execute<T, F>(&self, operation: F) -> BreakerResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> BoxFuture<'static, BreakerResult<T>> + Send + 'static,
    {
        // First check if the circuit is open
        self.state.try_request().await?;
        
        // Execute the operation
        let result = operation().await;
        
        // Update state based on result
        match &result {
            Ok(_) => {
                self.state.on_success().await;
            }
            Err(err) => {
                // Box the error so it can be passed to on_error
                let boxed_err = Box::new(err.clone());
                self.state.on_error(boxed_err).await;
            }
        }
        
        result
    }
    
    async fn metrics(&self) -> BreakerMetrics {
        self.state.metrics().await
    }
} 