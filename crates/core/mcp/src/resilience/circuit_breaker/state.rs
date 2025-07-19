//! Circuit breaker state management
//!
//! This module defines the state types and traits for circuit breakers.

use std::sync::Arc;
use std::fmt;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use super::metrics::BreakerMetrics;
use super::config::BreakerConfig;
use super::error::BreakerResult;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakerState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

impl fmt::Display for BreakerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "Closed"),
            Self::Open => write!(f, "Open"),
            Self::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Interface for the circuit breaker state machine
#[async_trait]
pub trait CircuitBreakerState: Send + Sync {
    /// Get the current state
    async fn state(&self) -> BreakerState;
    
    /// Get the configuration
    fn config(&self) -> &BreakerConfig;
    
    /// Try to make a request
    async fn try_request(&self) -> BreakerResult<()>;
    
    /// Report a successful request
    async fn on_success(&self);
    
    /// Report a failed request
    async fn on_error(&self, err: Box<dyn std::error::Error + Send + Sync>);
    
    /// Get metrics
    async fn metrics(&self) -> BreakerMetrics;
}

/// Create a boxed circuit breaker state trait object from any implementation
pub fn boxed_state<S: CircuitBreakerState + 'static>(state: S) -> Box<dyn CircuitBreakerState> {
    Box::new(state)
}

/// Create an Arc-wrapped circuit breaker state trait object from any implementation
pub fn shared_state<S: CircuitBreakerState + 'static>(state: S) -> Arc<dyn CircuitBreakerState> {
    Arc::new(state)
}

// Implement CircuitBreakerState for Box<dyn CircuitBreakerState>
#[async_trait]
impl CircuitBreakerState for Box<dyn CircuitBreakerState + Send + Sync> {
    async fn state(&self) -> BreakerState {
        (**self).state().await
    }
    
    fn config(&self) -> &BreakerConfig {
        (**self).config()
    }
    
    async fn try_request(&self) -> BreakerResult<()> {
        (**self).try_request().await
    }
    
    async fn on_success(&self) {
        (**self).on_success().await
    }
    
    async fn on_error(&self, err: Box<dyn std::error::Error + Send + Sync>) {
        (**self).on_error(err).await
    }
    
    async fn metrics(&self) -> BreakerMetrics {
        (**self).metrics().await
    }
} 