//! Standard implementation of the CircuitBreaker trait

use std::collections::VecDeque;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{debug, error, info, warn};
use anyhow::anyhow;

use super::{BreakerConfig, BreakerError, BreakerMetrics, BreakerState, CircuitBreaker};

/// Internal state of the circuit breaker
struct BreakerInternalState {
    /// Current state of the circuit breaker
    current_state: BreakerState,
    
    /// Time when the circuit opened
    open_time: Option<Instant>,
    
    /// Count of consecutive failures (in Closed state)
    failure_count: u32,
    
    /// Count of consecutive successes (in HalfOpen state)
    success_count: u32,
    
    /// Count of calls in HalfOpen state
    half_open_call_count: u32,
    
    /// Recent failures with timestamps for sliding window
    recent_failures: VecDeque<DateTime<Utc>>,
    
    /// Metrics for this circuit breaker
    metrics: BreakerMetrics,
}

/// Standard implementation of the CircuitBreaker trait
pub struct StandardCircuitBreaker {
    /// Configuration for the circuit breaker
    config: BreakerConfig,
    
    /// Internal state of the circuit breaker
    state: Arc<RwLock<BreakerInternalState>>,
}

impl StandardCircuitBreaker {
    /// Creates a new circuit breaker with the given configuration
    pub fn new(config: BreakerConfig) -> Self {
        let metrics = BreakerMetrics::new(config.name.clone());
        
        Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(BreakerInternalState {
                current_state: BreakerState::Closed,
                open_time: None,
                failure_count: 0,
                success_count: 0,
                half_open_call_count: 0,
                recent_failures: VecDeque::new(),
                metrics,
            })),
        }
    }
    
    /// Check if the circuit should transition state
    async fn check_state_transition(&self) {
        let mut state = self.state.write().await;
        let now = Instant::now();
        
        match state.current_state {
            BreakerState::Open => {
                if let Some(open_time) = state.open_time {
                    if now.duration_since(open_time) >= self.config.reset_timeout {
                        debug!(
                            "Circuit breaker '{}' transitioning from Open to HalfOpen after {}ms",
                            self.config.name(), self.config.reset_timeout.as_millis()
                        );
                        
                        state.current_state = BreakerState::HalfOpen;
                        state.half_open_call_count = 0;
                        state.success_count = 0;
                        state.metrics.record_state_transition(BreakerState::HalfOpen);
                    }
                }
            },
            _ => {}
        }
    }
    
    /// Check if the circuit should open based on failure threshold
    async fn check_failure_threshold(&self) -> bool {
        let mut state = self.state.write().await;
        
        // Only check in closed state
        if state.current_state != BreakerState::Closed {
            return false;
        }
        
        // Check if we're using a failure window
        if let Some(window) = self.config.failure_window {
            if self.config.sliding_window {
                // Using sliding window approach
                let now = Utc::now();
                
                // Remove failures outside the window
                while let Some(time) = state.recent_failures.front() {
                    if (now - *time).to_std().unwrap_or_else(|_| Duration::from_secs(0)) > window {
                        state.recent_failures.pop_front();
                    } else {
                        break;
                    }
                }
                
                // Check if we've exceeded the threshold
                if state.recent_failures.len() >= self.config.failure_threshold as usize {
                    self.transition_to_open(&mut state).await;
                    return true;
                }
            } else {
                // Check consecutive failures
                if state.failure_count >= self.config.failure_threshold {
                    self.transition_to_open(&mut state).await;
                    return true;
                }
            }
        } else {
            // No window, just check consecutive failures
            if state.failure_count >= self.config.failure_threshold {
                self.transition_to_open(&mut state).await;
                return true;
            }
        }
        
        false
    }
    
    /// Helper method to transition to open state
    async fn transition_to_open(&self, state: &mut BreakerInternalState) {
        debug!(
            "Circuit breaker '{}' transitioning to Open state after {} failures",
            self.config.name(), state.failure_count
        );
        
        state.current_state = BreakerState::Open;
        state.open_time = Some(Instant::now());
        state.metrics.record_state_transition(BreakerState::Open);
    }
    
    /// Record a successful operation
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        
        match state.current_state {
            BreakerState::Closed => {
                state.failure_count = 0;
                state.metrics.record_success();
            },
            BreakerState::HalfOpen => {
                state.success_count += 1;
                state.metrics.record_success();
                
                if state.success_count >= self.config.success_threshold {
                    debug!(
                        "Circuit breaker '{}' transitioning from HalfOpen to Closed after {} successes",
                        self.config.name(), state.success_count
                    );
                    
                    state.current_state = BreakerState::Closed;
                    state.failure_count = 0;
                    state.recent_failures.clear();
                    state.metrics.record_state_transition(BreakerState::Closed);
                }
            },
            BreakerState::Open => {
                // Should not happen, but handle gracefully
                state.metrics.record_success();
            }
        }
    }
    
    /// Record a failed operation
    async fn record_failure(&self, error_msg: String) {
        let mut state = self.state.write().await;
        
        match state.current_state {
            BreakerState::Closed => {
                state.failure_count += 1;
                
                // Record the failure timestamp if we're using a window
                if self.config.failure_window.is_some() && self.config.sliding_window {
                    state.recent_failures.push_back(Utc::now());
                }
                
                state.metrics.record_failure(error_msg);
            },
            BreakerState::HalfOpen => {
                debug!(
                    "Circuit breaker '{}' transitioning back to Open after failure in HalfOpen state",
                    self.config.name()
                );
                
                state.current_state = BreakerState::Open;
                state.open_time = Some(Instant::now());
                state.metrics.record_failure(error_msg);
                state.metrics.record_state_transition(BreakerState::Open);
            },
            BreakerState::Open => {
                // Should not happen, but handle gracefully
                state.metrics.record_failure(error_msg);
            }
        }
    }
    
    /// Record a rejected operation
    async fn record_rejection(&self) {
        let mut state = self.state.write().await;
        state.metrics.record_rejection();
    }
    
    /// Record a timeout
    async fn record_timeout(&self, duration: Duration) {
        let mut state = self.state.write().await;
        state.metrics.record_timeout();
        state.metrics.record_failure(format!("Operation timed out after {:?}", duration));
    }
}

#[async_trait]
impl CircuitBreaker for StandardCircuitBreaker {
    async fn execute<F, T, E>(&self, operation: F) -> Result<T, BreakerError<E>>
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: Send + 'static,
        E: Error + Send + Sync + 'static,
    {
        // Check for state transitions first
        self.check_state_transition().await;
        
        // Check if circuit is open
        let state_result = {
            let state = self.state.read().await;
            state.current_state
        };
        
        if state_result == BreakerState::Open {
            self.record_rejection().await;
            return Err(BreakerError::circuit_open(
                format!("Circuit breaker '{}' is open", self.config.name())
            ));
        }
        
        // Check if we can make a call in half-open state
        if state_result == BreakerState::HalfOpen {
            let can_call = {
                let mut state = self.state.write().await;
                state.half_open_call_count < self.config.half_open_max_calls
            };
            
            if !can_call {
                self.record_rejection().await;
                return Err(BreakerError::circuit_open(
                    format!("Circuit breaker '{}' is half-open and call limit reached", self.config.name())
                ));
            }
            
            // Increment half-open call count
            {
                let mut state = self.state.write().await;
                state.half_open_call_count += 1;
            }
        }
        
        // Apply timeout if configured
        let operation_result = if let Some(timeout_duration) = self.config.operation_timeout {
            match timeout(timeout_duration, operation).await {
                Ok(result) => result,
                Err(_) => {
                    self.record_timeout(timeout_duration).await;
                    return Err(BreakerError::timeout(timeout_duration));
                }
            }
        } else {
            operation.await
        };
        
        // Handle the operation result
        match operation_result {
            Ok(value) => {
                self.record_success().await;
                Ok(value)
            }
            Err(error) => {
                self.record_failure(error.to_string()).await;
                
                // Check if we need to trip the circuit
                self.check_failure_threshold().await;
                
                Err(BreakerError::OperationFailed(error))
            }
        }
    }
    
    async fn state(&self) -> BreakerState {
        let state = self.state.read().await;
        state.current_state
    }
    
    async fn reset(&self) -> Result<(), BreakerError<anyhow::Error>> {
        let mut state = self.state.write().await;
        
        state.current_state = BreakerState::Closed;
        state.open_time = None;
        state.failure_count = 0;
        state.success_count = 0;
        state.half_open_call_count = 0;
        state.recent_failures.clear();
        state.metrics.record_state_transition(BreakerState::Closed);
        
        debug!("Circuit breaker '{}' manually reset to Closed state", self.config.name());
        
        Ok(())
    }
    
    async fn trip(&self) -> Result<(), BreakerError<anyhow::Error>> {
        let mut state = self.state.write().await;
        
        state.current_state = BreakerState::Open;
        state.open_time = Some(Instant::now());
        state.metrics.record_state_transition(BreakerState::Open);
        
        debug!("Circuit breaker '{}' manually tripped to Open state", self.config.name());
        
        Ok(())
    }
    
    async fn metrics(&self) -> BreakerMetrics {
        let mut state = self.state.write().await;
        
        // Update time in state before returning metrics
        state.metrics.update_time_in_state();
        
        state.metrics.clone()
    }
} 