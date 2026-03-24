// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Basic Resilience Scenarios Integration Tests
//!
//! Tests for fundamental resilience scenarios including success cases,
//! basic error handling, and simple integration patterns.

use super::*;

/// Test basic success scenario with resilience framework
#[tokio::test]
async fn test_with_resilience_success() {
    let mut circuit_breaker = create_lenient_circuit_breaker("test-circuit");
    let retry = create_exponential_retry_mechanism();

    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        move || {
            // This operation succeeds
            Result::<TestString, Box<dyn StdError + Send + Sync>>::Ok(TestString("Success".to_string()))
        }
    ).await;

    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed").0, "Success");
}

/// Test basic failure and retry scenario
#[tokio::test]
async fn test_basic_failure_and_retry() {
    let mut circuit_breaker = create_lenient_circuit_breaker("basic-retry");
    let retry = create_test_retry_mechanism();
    
    let attempt_counter = Arc::new(Mutex::new(0));
    
    let counter = attempt_counter.clone();
    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        move || {
            let counter_clone = counter.clone();
            let mut count = counter_clone.lock().expect("should succeed");
            *count += 1;
            
            if *count == 1 {
                // First attempt fails
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("First attempt failed".to_string())))
            } else {
                // Second attempt succeeds
                Ok(TestString("Retry succeeded".to_string()))
            }
        }
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed").0, "Retry succeeded".to_string());
    assert_operation_count(&attempt_counter, 2, "Basic retry scenario");
}

/// Test success without any failures
#[tokio::test]
async fn test_immediate_success() {
    let mut circuit_breaker = create_test_circuit_breaker("immediate-success");
    let retry = create_test_retry_mechanism();
    let attempt_counter = Arc::new(Mutex::new(0));
    
    let counter = attempt_counter.clone();
    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        move || {
            let counter_clone = counter.clone();
            let mut count = counter_clone.lock().expect("should succeed");
            *count += 1;
            
            // Always succeed immediately
            Ok(TestString("Immediate success".to_string()))
        }
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed").0, "Immediate success".to_string());
    assert_operation_count(&attempt_counter, 1, "Immediate success scenario");
}

/// Test basic circuit breaker functionality
#[tokio::test]
async fn test_basic_circuit_breaker() {
    let mut circuit_breaker = create_test_circuit_breaker("basic-circuit");
    let retry = create_test_retry_mechanism();
    
    // First, verify circuit starts closed
    let initial_state = circuit_breaker.state().await;
    assert_eq!(initial_state, BreakerState::Closed);
    
    // Execute a successful operation
    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || Ok(TestString("Circuit breaker working".to_string()))
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed").0, "Circuit breaker working".to_string());
    
    // Circuit should still be closed after success
    let final_state = circuit_breaker.state().await;
    assert_eq!(final_state, BreakerState::Closed);
}

/// Test error propagation through resilience framework
#[tokio::test]
async fn test_error_propagation() {
    let mut circuit_breaker = create_lenient_circuit_breaker("error-propagation");
    let retry = create_test_retry_mechanism();
    
    let attempt_counter = Arc::new(Mutex::new(0));
    
    // Operation that always fails
    let counter = attempt_counter.clone();
    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        move || {
            let counter_clone = counter.clone();
            let mut count = counter_clone.lock().expect("should succeed");
            *count += 1;
            
            // Always fail
            Err(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
        }
    ).await;
    
    // Should fail after all retries
    assert!(result.is_err());
    
    // Should have attempted max_attempts times
    assert_operation_count(&attempt_counter, 2, "Error propagation scenario"); // max_attempts = 2
}

/// Test resilience with different error types
#[tokio::test]
async fn test_different_error_types() {
    let mut circuit_breaker = create_test_circuit_breaker("error-types");
    let retry = create_test_retry_mechanism();
    
    // Test with custom error
    let result1: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || Err(Box::<dyn StdError + Send + Sync>::from(TestError("Custom error".to_string())))
    ).await;
    
    assert!(result1.is_err());
    
    // Test with different error type
    let result2: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || {
            Err(Box::<dyn StdError + Send + Sync>::from(
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused")
            ))
        }
    ).await;
    
    assert!(result2.is_err());
    
    // Test successful operation after errors
    let result3: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        || Ok(TestString("Success after errors".to_string()))
    ).await;
    
    assert!(result3.is_ok());
    assert_eq!(result3.expect("should succeed").0, "Success after errors".to_string());
}

/// Test resilience framework with empty operations
#[tokio::test]
async fn test_minimal_operations() {
    let mut circuit_breaker = create_test_circuit_breaker("minimal");
    let retry = create_test_retry_mechanism();
    
    // Test with minimal successful operation
    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        || Ok(TestString("".to_string())) // Empty but valid result
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.expect("should succeed").0, "");
}

/// Test circuit breaker metrics tracking
#[tokio::test]
async fn test_circuit_breaker_metrics() {
    let mut circuit_breaker = create_test_circuit_breaker("metrics");
    let retry = create_test_retry_mechanism();
    
    // Get initial metrics
    let initial_metrics = circuit_breaker.metrics().await;
    let initial_success = initial_metrics.success_count;
    let initial_failure = initial_metrics.failure_count;
    
    // Execute successful operation
    let _result1: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || Ok(TestString("Success 1".to_string()))
    ).await;
    
    // Execute failing operation
    let _result2: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry,
        || Err(Box::<dyn StdError + Send + Sync>::from(TestError("Failure".to_string())))
    ).await;
    
    // Check updated metrics
    let final_metrics = circuit_breaker.metrics().await;
    
    // Should have at least one more success and some failures
    assert!(final_metrics.success_count > initial_success, 
           "Expected success count to increase from {} to {}", 
           initial_success, final_metrics.success_count);
    
    assert!(final_metrics.failure_count > initial_failure,
           "Expected failure count to increase from {} to {}", 
           initial_failure, final_metrics.failure_count);
} 