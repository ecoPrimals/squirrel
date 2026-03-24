// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Standard implementation of CircuitBreakerState
//!
//! This module provides the standard state implementation for circuit breakers.

use std::sync::RwLock;
use async_trait::async_trait;
use tokio::time::Instant;
use chrono::Utc;
use tracing::debug;

use super::config::BreakerConfig;
use super::error::{BreakerError, BreakerResult};
use super::state::{BreakerState, CircuitBreakerState};
use super::metrics::BreakerMetrics;

/// Standard implementation of circuit breaker state
pub struct StandardBreakerState {
    /// Configuration
    config: BreakerConfig,
    /// Current circuit state
    current_state: RwLock<BreakerState>,
    /// Metrics tracking
    metrics: RwLock<BreakerMetrics>,
    /// Last time the circuit was opened
    opened_time: RwLock<Option<Instant>>,
    /// Consecutive successful calls in half-open state
    half_open_success_count: RwLock<u32>,
}

impl StandardBreakerState {
    /// Create a new StandardBreakerState
    pub fn new(config: BreakerConfig) -> Self {
        let name = config.name.clone();
        let metrics = BreakerMetrics {
            name,
            state: BreakerState::Closed,
            success_count: 0,
            failure_count: 0,
            rejected_count: 0,
            timeout_count: 0,
            last_failure_time: None,
            last_success_time: None,
        };
        
        Self {
            config,
            current_state: RwLock::new(BreakerState::Closed),
            metrics: RwLock::new(metrics),
            opened_time: RwLock::new(None),
            half_open_success_count: RwLock::new(0),
        }
    }
    
    /// Check if the circuit should automatically reset
    async fn check_auto_reset(&self) -> bool {
        let opened_time = {
            let opened_guard = self.opened_time.read().map_err(|_| {
                BreakerError::internal(&self.config.name, "Failed to acquire opened_time read lock")
            });
            match opened_guard {
                Ok(guard) => match *guard {
                    Some(time) => time,
                    None => return false,
                },
                Err(_) => {
                    // If we can't read the lock, assume circuit should not reset
                    return false;
                }
            }
        };
        
        // Check if the reset timeout has passed
        let now = Instant::now();
        let reset_timeout_ms = self.config.reset_timeout_ms as u64;
        let duration_since_opened = now.duration_since(opened_time);
        
        duration_since_opened.as_millis() >= reset_timeout_ms as u128
    }
    
    /// Calculate the current failure rate
    fn calculate_failure_rate(&self, metrics: &BreakerMetrics) -> f64 {
        let total = metrics.success_count + metrics.failure_count;
        if total == 0 {
            return 0.0;
        }
        
        (metrics.failure_count as f64) / (total as f64)
    }
    
    /// Open the circuit
    async fn open_circuit(&self) {
        // Set the state to Open
        if let Ok(mut state) = self.current_state.write() {
            *state = BreakerState::Open;
        } else {
            tracing::error!("Failed to acquire write lock for circuit state in '{}'", self.config.name);
            return;
        }
        
        // Record the time when circuit opened
        if let Ok(mut opened_time) = self.opened_time.write() {
            *opened_time = Some(Instant::now());
        } else {
            tracing::error!("Failed to acquire write lock for opened_time in '{}'", self.config.name);
            return;
        }
        
        // Update metrics
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.state = BreakerState::Open;
            metrics.last_failure_time = Some(Utc::now());
        } else {
            tracing::error!("Failed to acquire write lock for metrics in '{}'", self.config.name);
            return;
        }
        
        debug!("Circuit '{}' opened", self.config.name);
    }
    
    /// Half-open the circuit for testing
    async fn half_open_circuit(&self) {
        // Reset the success counter
        if let Ok(mut success_count) = self.half_open_success_count.write() {
            *success_count = 0;
        } else {
            tracing::error!("Failed to acquire write lock for success_count in '{}'", self.config.name);
            return;
        }
        
        // Set the state to HalfOpen
        if let Ok(mut state) = self.current_state.write() {
            *state = BreakerState::HalfOpen;
        } else {
            tracing::error!("Failed to acquire write lock for circuit state in '{}'", self.config.name);
            return;
        }
        
        // Update metrics
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.state = BreakerState::HalfOpen;
        } else {
            tracing::error!("Failed to acquire write lock for metrics in '{}'", self.config.name);
            return;
        }
        
        debug!("Circuit '{}' half-opened", self.config.name);
    }
    
    /// Close the circuit
    async fn close_circuit(&self) {
        // Set the state to Closed
        {
            let mut state = self.current_state.write().expect("circuit breaker state lock poisoned");
            *state = BreakerState::Closed;
        }
        
        // Clear opened time
        {
            let mut opened_time = self.opened_time.write().expect("circuit breaker opened_time lock poisoned");
            *opened_time = None;
        }
        
        // Reset the success counter
        {
            let mut success_count = self.half_open_success_count.write().expect("circuit breaker success count lock poisoned");
            *success_count = 0;
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().expect("circuit breaker metrics lock poisoned");
            metrics.state = BreakerState::Closed;
        }
        
        debug!("Circuit '{}' closed", self.config.name);
    }
}

#[async_trait]
impl CircuitBreakerState for StandardBreakerState {
    async fn state(&self) -> BreakerState {
        *self.current_state.read().expect("circuit breaker state lock poisoned")
    }
    
    fn config(&self) -> &BreakerConfig {
        &self.config
    }
    
    async fn try_request(&self) -> BreakerResult<()> {
        // Check current state
        let current_state = { *self.current_state.read().expect("circuit breaker state lock poisoned") };
        
        match current_state {
            BreakerState::Closed => {
                // Closed circuit - allow requests
                Ok(())
            },
            BreakerState::Open => {
                // Check if we should auto-reset
                if self.check_auto_reset().await {
                    // Time to test the circuit
                    self.half_open_circuit().await;
                    
                    // Allow this single request
                    Ok(())
                } else {
                    // Still open, reject request
                    let reset_time_ms = self.config.reset_timeout_ms;
                    
                    // Increment rejection count
                    {
                        let mut metrics = self.metrics.write().expect("circuit breaker metrics lock poisoned");
                        metrics.rejected_count += 1;
                    }
                    
                    Err(BreakerError::circuit_open(&self.config.name, reset_time_ms))
                }
            },
            BreakerState::HalfOpen => {
                // Half-open circuit - allow limited requests
                Ok(())
            },
        }
    }
    
    async fn on_success(&self) {
        // Get current state
        let current_state = { *self.current_state.read().expect("circuit breaker state lock poisoned") };
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().expect("circuit breaker metrics lock poisoned");
            metrics.success_count += 1;
            metrics.last_success_time = Some(Utc::now());
        }
        
        // If in half-open state, check if we can close the circuit
        if current_state == BreakerState::HalfOpen {
            // Increment success counter
            let success_count = {
                let mut count = self.half_open_success_count.write().expect("circuit breaker success count lock poisoned");
                *count += 1;
                *count
            };
            
            debug!("Half-open success: {}/{}", success_count, self.config.half_open_success_threshold);
            
            // If we've had enough successes, close the circuit
            if success_count >= self.config.half_open_success_threshold {
                self.close_circuit().await;
            }
        }
    }
    
    async fn on_error(&self, _err: Box<dyn std::error::Error + Send + Sync>) {
        // Get current state
        let current_state = { *self.current_state.read().expect("circuit breaker state lock poisoned") };
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().expect("circuit breaker metrics lock poisoned");
            metrics.failure_count += 1;
            metrics.last_failure_time = Some(Utc::now());
        }
        
        match current_state {
            BreakerState::Closed => {
                // In closed state, check failure threshold
                let failure_rate = {
                    let metrics = self.metrics.read().expect("circuit breaker metrics lock poisoned");
                    self.calculate_failure_rate(&metrics)
                };
                
                if failure_rate >= self.config.failure_threshold {
                    debug!(
                        "Failure rate {:.1}% exceeds threshold {:.1}%, opening circuit", 
                        failure_rate * 100.0, 
                        self.config.failure_threshold * 100.0
                    );
                    self.open_circuit().await;
                }
            },
            BreakerState::HalfOpen => {
                // If we get an error in half-open state, open the circuit again
                debug!("Error in half-open state, reopening circuit");
                self.open_circuit().await;
            },
            BreakerState::Open => {
                // Already open, just update the metrics
            },
        }
    }
    
    async fn metrics(&self) -> BreakerMetrics {
        self.metrics.read().expect("circuit breaker metrics lock poisoned").clone()
    }
} 