// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Configuration for circuit breakers in the resilience framework

use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Configuration for a circuit breaker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakerConfig {
    /// Name of this circuit breaker for identification
    pub name: String,
    
    /// Failure threshold (0.0 - 1.0) that triggers the circuit to open
    /// e.g., 0.5 means 50% of requests must fail to open the circuit
    pub failure_threshold: f64,
    
    /// Minimum number of requests before failure threshold check is activated
    pub minimum_request_threshold: usize,
    
    /// Time in milliseconds before the circuit automatically transitions from Open to HalfOpen
    pub reset_timeout_ms: u64,
    
    /// Number of consecutive successful requests in HalfOpen state before closing the circuit
    pub half_open_success_threshold: u32,
}

impl BreakerConfig {
    /// Create a new circuit breaker configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    
    /// Set the failure threshold (0.0 - 1.0)
    pub fn with_failure_threshold(mut self, threshold: f64) -> Self {
        self.failure_threshold = threshold.clamp(0.0, 1.0);
        self
    }
    
    /// Set the minimum request threshold
    pub fn with_minimum_request_threshold(mut self, threshold: usize) -> Self {
        self.minimum_request_threshold = threshold;
        self
    }
    
    /// Set the reset timeout in milliseconds
    pub fn with_reset_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.reset_timeout_ms = timeout_ms;
        self
    }
    
    /// Set the reset timeout using a Duration
    pub fn with_reset_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout_ms = timeout.as_millis() as u64;
        self
    }
    
    /// Set the number of consecutive successes required to close the circuit
    pub fn with_half_open_success_threshold(mut self, count: u32) -> Self {
        self.half_open_success_threshold = count;
        self
    }
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            failure_threshold: 0.5,        // 50% failure rate
            minimum_request_threshold: 5,   // At least 5 requests
            reset_timeout_ms: 30000,        // 30 seconds
            half_open_success_threshold: 3, // 3 consecutive successes
        }
    }
} 