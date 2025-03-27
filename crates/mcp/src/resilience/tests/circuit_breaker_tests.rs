use std::sync::Arc;
use std::time::Duration;

use crate::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::resilience::ResilienceError;
use super::TestError;

#[test]
fn test_circuit_breaker_success() {
    let mut circuit_breaker = CircuitBreaker::default();
    
    // Execute a successful operation
    let result = circuit_breaker.execute(|| Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42));
    
    // Verify success
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[test]
fn test_circuit_breaker_opens_after_failures() {
    // Create config with low threshold
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..CircuitBreakerConfig::default()
    };
    let mut circuit_breaker = CircuitBreaker::new(config);
    
    // First failure
    let result = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    
    // Second failure should open the circuit
    let result = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Subsequent call should be rejected immediately
    let result = circuit_breaker.execute(|| Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42));
    assert!(result.is_err());
    
    match result {
        Err(ResilienceError::CircuitOpen(_)) => (), // Expected
        _ => panic!("Expected CircuitOpen error"),
    }
}

#[test]
fn test_circuit_breaker_reset() {
    // Create config with low threshold
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..CircuitBreakerConfig::default()
    };
    let mut circuit_breaker = CircuitBreaker::new(config);
    
    // Open the circuit
    let _ = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    let _ = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Reset the circuit
    circuit_breaker.reset();
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    
    // Should work again
    let result = circuit_breaker.execute(|| Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_circuit_breaker_metrics() {
    let mut circuit_breaker = CircuitBreaker::default();
    
    // Success
    let _ = circuit_breaker.execute(|| Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42));
    
    // Get metrics
    let metrics = circuit_breaker.get_metrics();
    
    // Verify metrics
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.failure_count, 0);
    assert_eq!(metrics.open_count, 0);
    
    // Failures
    let _ = circuit_breaker.execute(|| Err::<i32, _>(Box::new(TestError::generic("test failure"))));
    let metrics = circuit_breaker.get_metrics();
    assert_eq!(metrics.success_count, 1);
    assert_eq!(metrics.failure_count, 1);
} 