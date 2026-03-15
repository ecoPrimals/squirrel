// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Circuit breaker state management
//!
//! This module defines the state types and traits for circuit breakers.

use std::future::Future;
use std::sync::Arc;
use std::fmt;
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
pub trait CircuitBreakerState: Send + Sync {
    /// Get the current state
    fn state(&self) -> impl Future<Output = BreakerState> + Send;
    
    /// Get the configuration
    fn config(&self) -> &BreakerConfig;
    
    /// Try to make a request
    fn try_request(&self) -> impl Future<Output = BreakerResult<()>> + Send;
    
    /// Report a successful request
    fn on_success(&self) -> impl Future<Output = ()> + Send;
    
    /// Report a failed request
    fn on_error(&self, err: Box<dyn std::error::Error + Send + Sync>) -> impl Future<Output = ()> + Send;
    
    /// Get metrics
    fn metrics(&self) -> impl Future<Output = BreakerMetrics> + Send;
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
impl CircuitBreakerState for Box<dyn CircuitBreakerState + Send + Sync> {
    fn state(&self) -> impl Future<Output = BreakerState> + Send {
        (**self).state()
    }
    
    fn config(&self) -> &BreakerConfig {
        (**self).config()
    }
    
    fn try_request(&self) -> impl Future<Output = BreakerResult<()>> + Send {
        (**self).try_request()
    }
    
    fn on_success(&self) -> impl Future<Output = ()> + Send {
        (**self).on_success()
    }
    
    fn on_error(&self, err: Box<dyn std::error::Error + Send + Sync>) -> impl Future<Output = ()> + Send {
        (**self).on_error(err)
    }
    
    fn metrics(&self) -> impl Future<Output = BreakerMetrics> + Send {
        (**self).metrics()
    }
} 