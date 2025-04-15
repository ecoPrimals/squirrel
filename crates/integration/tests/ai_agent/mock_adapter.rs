//! Mock implementation of the AI Agent adapter for testing
//!
//! This module provides a simplified mock implementation for testing
//! circuit breaker, rate limiting, and caching behavior.

use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;
use uuid::Uuid;

use crate::ai_agent::{
    CircuitBreakerState,
    CircuitBreakerConfig,
    AIAgentConfig,
};

/// A mock adapter for testing the AI Agent functionality
#[derive(Clone)]
pub struct MockAIAgent {
    config: AIAgentConfig,
    circuit_breaker_state: Arc<Mutex<CircuitBreakerState>>,
    failure_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    should_fail: Arc<Mutex<bool>>,
    is_initialized: Arc<Mutex<bool>>,
}

impl MockAIAgent {
    /// Create a new mock adapter with the given configuration
    pub fn new(config: AIAgentConfig) -> Self {
        Self {
            config,
            circuit_breaker_state: Arc::new(Mutex::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            success_count: Arc::new(AtomicUsize::new(0)),
            should_fail: Arc::new(Mutex::new(false)),
            is_initialized: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Initialize the adapter
    pub async fn initialize(&self) -> Result<(), String> {
        let mut initialized = self.is_initialized.lock().unwrap();
        *initialized = true;
        Ok(())
    }
    
    /// Set whether the mock should fail on requests
    pub fn set_failure_mode(&self, should_fail: bool) {
        let mut fail_state = self.should_fail.lock().unwrap();
        *fail_state = should_fail;
    }
    
    /// Force set the circuit breaker state (for testing)
    pub fn set_circuit_breaker_state(&self, state: CircuitBreakerState) {
        let mut cb_state = self.circuit_breaker_state.lock().unwrap();
        *cb_state = state;
    }
    
    /// Get the success count
    pub fn get_success_count(&self) -> usize {
        self.success_count.load(Ordering::SeqCst)
    }
    
    /// Get the failure count
    pub fn get_failure_count(&self) -> usize {
        self.failure_count.load(Ordering::SeqCst)
    }
    
    /// Process a simulated request
    pub async fn process_request(&self, prompt: &str) -> Result<String, String> {
        // Check if initialized
        let initialized = *self.is_initialized.lock().unwrap();
        if !initialized {
            return Err("Adapter not initialized".to_string());
        }
        
        // Process through circuit breaker
        self.process_through_circuit_breaker(prompt).await
    }
    
    /// Get the current status of the adapter
    pub async fn get_status(&self) -> MockStatus {
        let circuit_breaker_state = {
            let state = self.circuit_breaker_state.lock().unwrap();
            *state
        };
        
        let initialized = *self.is_initialized.lock().unwrap();
        
        MockStatus {
            operational: initialized && circuit_breaker_state == CircuitBreakerState::Closed,
            circuit_breaker_state,
            error_count: self.failure_count.load(Ordering::SeqCst) as u64,
            request_count: (self.success_count.load(Ordering::SeqCst) + self.failure_count.load(Ordering::SeqCst)) as u64,
        }
    }
    
    /// Check if the circuit breaker should open
    fn should_open_circuit(&self) -> bool {
        let failure_count = self.failure_count.load(Ordering::SeqCst);
        failure_count >= self.config.circuit_breaker.failure_threshold as usize
    }
    
    /// Helper to process a request through the circuit breaker
    async fn process_through_circuit_breaker(&self, prompt: &str) -> Result<String, String> {
        // Check circuit breaker state
        let state = {
            let state = self.circuit_breaker_state.lock().unwrap();
            *state
        };
        
        match state {
            CircuitBreakerState::Open => {
                // Circuit is open, fail fast
                Err(format!("Circuit breaker is open for operation"))
            },
            CircuitBreakerState::HalfOpen => {
                // Allow test requests in half-open state
                self.process_with_failure_simulation(prompt).await
            },
            CircuitBreakerState::Closed => {
                // Normal operation
                self.process_with_failure_simulation(prompt).await
            }
        }
    }
    
    /// Process a request with configurable failure simulation
    async fn process_with_failure_simulation(&self, prompt: &str) -> Result<String, String> {
        // Get failure setting
        let should_fail = *self.should_fail.lock().unwrap();
        
        if should_fail {
            // Increment failure counter
            self.failure_count.fetch_add(1, Ordering::SeqCst);
            
            // Check if we should open the circuit
            if self.should_open_circuit() {
                let mut state = self.circuit_breaker_state.lock().unwrap();
                *state = CircuitBreakerState::Open;
            }
            
            Err("Simulated service failure".to_string())
        } else {
            // Increment success counter
            self.success_count.fetch_add(1, Ordering::SeqCst);
            
            // If in half-open state and success threshold reached, close the circuit
            let (is_half_open, max_half_open_calls) = {
                let state = self.circuit_breaker_state.lock().unwrap();
                (*state == CircuitBreakerState::HalfOpen, self.config.circuit_breaker.half_open_max_calls)
            };
            
            if is_half_open && self.success_count.load(Ordering::SeqCst) >= max_half_open_calls as usize {
                let mut state = self.circuit_breaker_state.lock().unwrap();
                *state = CircuitBreakerState::Closed;
            }
            
            // Generate mock response
            Ok(format!("Mock response for: {}", prompt))
        }
    }
}

/// Status information for the mock adapter
pub struct MockStatus {
    pub operational: bool,
    pub circuit_breaker_state: CircuitBreakerState,
    pub error_count: u64,
    pub request_count: u64,
} 