// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Resilience Integration Tests
//!
//! This module contains integration tests for resilience components including
//! circuit breakers, retry mechanisms, recovery strategies, and their combinations.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::error::Error as StdError;
use std::fmt;
use crate::resilience::recovery::{FailureInfo, FailureSeverity, RecoveryStrategy, RecoveryConfig};

use crate::resilience::{
    with_resilience,
    with_complete_resilience,
    ResilienceError,
    CircuitBreaker as CircuitBreakerTrait
};
use crate::resilience::circuit_breaker::{StandardCircuitBreaker as CircuitBreaker, BreakerConfig, BreakerState};
use crate::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy, RetryError};
use crate::resilience::health;

// Re-export all test modules
pub mod circuit_retry;
pub mod recovery;
pub mod full_chain;
pub mod api_simulation;
pub mod basic_scenarios;
pub mod pipeline;

// ----- Test Utilities and Helper Types -----

/// A wrapper around String that can implement From<i32> for testing
#[derive(Debug, PartialEq, Clone)]
pub struct TestString(pub String);

impl From<i32> for TestString {
    fn from(value: i32) -> Self {
        TestString(format!("Fallback result: {}", value))
    }
}

impl AsRef<str> for TestString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A test error type for simulating failures
#[derive(Debug)]
pub struct TestError(pub String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl StdError for TestError {}

// ----- Test Configuration Helpers -----

/// Create a standard circuit breaker for testing
pub fn create_test_circuit_breaker(name: &str) -> CircuitBreaker {
    CircuitBreaker::new(BreakerConfig {
        name: name.to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 2,
        reset_timeout_ms: 500,
        half_open_success_threshold: 1,
    })
}

/// Create a strict circuit breaker that trips easily
pub fn create_strict_circuit_breaker(name: &str) -> CircuitBreaker {
    CircuitBreaker::new(BreakerConfig {
        name: name.to_string(),
        failure_threshold: 5.0,
        minimum_request_threshold: 1,
        reset_timeout_ms: 1000,
        half_open_success_threshold: 1,
    })
}

/// Create a lenient circuit breaker for testing
pub fn create_lenient_circuit_breaker(name: &str) -> CircuitBreaker {
    CircuitBreaker::new(BreakerConfig {
        name: name.to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 5,
        reset_timeout_ms: 100,
        half_open_success_threshold: 1,
    })
}

/// Create a standard retry mechanism for testing
pub fn create_test_retry_mechanism() -> RetryMechanism {
    RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    })
}

/// Create a retry mechanism with exponential backoff
pub fn create_exponential_retry_mechanism() -> RetryMechanism {
    RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(200),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    })
}

/// Create a standard recovery strategy for testing
pub fn create_test_recovery_strategy() -> RecoveryStrategy {
    RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 2,
        max_moderate_attempts: 1,
        max_severe_attempts: 1,
        recover_critical: false,
    })
}

/// Create a more aggressive recovery strategy
pub fn create_aggressive_recovery_strategy() -> RecoveryStrategy {
    RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 3,
        max_moderate_attempts: 2,
        max_severe_attempts: 1,
        recover_critical: false,
    })
}

/// Create a health monitor for testing
pub fn create_test_health_monitor() -> health::HealthMonitor {
    health::HealthMonitor::new(10)
}

// ----- Test Simulation Helpers -----

/// Simulated API connection state
pub struct MockApiConnection {
    pub connected: Arc<Mutex<bool>>,
    pub data_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl MockApiConnection {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        cache.insert("test_data".to_string(), "Cached API data".to_string());
        
        Self {
            connected: Arc::new(Mutex::new(false)),
            data_cache: Arc::new(Mutex::new(cache)),
        }
    }
    
    pub fn connect(&self) {
        let mut conn = self.connected.lock().unwrap();
        *conn = true;
    }
    
    pub fn disconnect(&self) {
        let mut conn = self.connected.lock().unwrap();
        *conn = false;
    }
    
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
    
    pub fn get_cached_data(&self, key: &str) -> Option<String> {
        let cache = self.data_cache.lock().unwrap();
        cache.get(key).cloned()
    }
}

/// Create a failure info for testing
pub fn create_test_failure_info(severity: FailureSeverity, context: &str) -> FailureInfo {
    FailureInfo {
        message: format!("Test failure in {}", context),
        severity,
        context: context.to_string(),
        recovery_attempts: 0,
    }
}

// ----- Test Assertion Helpers -----

/// Assert that a circuit breaker has tripped
pub async fn assert_circuit_tripped(circuit_breaker: &CircuitBreaker) {
    let state = circuit_breaker.state().await;
    let metrics = circuit_breaker.metrics().await;
    
    assert!(
        state == BreakerState::Open || metrics.failure_count >= 2,
        "Expected circuit to be open or have high failure count. State: {:?}, Failures: {}",
        state, metrics.failure_count
    );
}

/// Assert that an operation counter matches expected value
pub fn assert_operation_count(counter: &Arc<Mutex<i32>>, expected: i32, context: &str) {
    let count = *counter.lock().unwrap();
    assert_eq!(count, expected, "{}: Expected {} operations, got {}", context, expected, count);
}

/// Execute multiple failure operations to trip a circuit
pub async fn trip_circuit_with_failures(
    circuit_breaker: &mut CircuitBreaker,
    retry: RetryMechanism,
    max_attempts: usize,
) -> usize {
    let mut successful_failures = 0;
    
    for i in 0..max_attempts {
        let result: Result<TestString, ResilienceError> = with_resilience(
            circuit_breaker,
            retry.clone(),
            move || {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError(
                    format!("Persistent failure {}", i)
                )))
            }
        ).await;
        
        if result.is_err() {
            successful_failures += 1;
            if let Err(ResilienceError::CircuitOpen(_)) = result {
                break;
            }
        }
    }
    
    successful_failures
} 