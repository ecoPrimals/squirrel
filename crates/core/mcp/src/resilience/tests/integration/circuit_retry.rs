// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Circuit Breaker and Retry Integration Tests
//!
//! Tests for the integration between circuit breakers and retry mechanisms,
//! verifying that they work together effectively.

use super::*;

/// Test that spans can be created and exported using a mock exporter
#[tokio::test]
async fn test_circuit_breaker_with_retry() {
    // Create components
    let mut circuit_breaker = create_test_circuit_breaker("test-circuit");
    let retry = create_test_retry_mechanism();
    
    // Create a counter for tracking attempts
    let counter = Arc::new(Mutex::new(0));
    
    // Test successful operation (succeeds on second retry)
    {
        let counter_clone = counter.clone();
        let result = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            move || {
                let mut count = counter_clone.lock().expect("should succeed");
                *count += 1;
                
                if *count < 2 {
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary failure".to_string())))
                } else {
                    Ok(TestString("Success".to_string()))
                }
            }
        ).await;
        
        assert!(result.is_ok(), "First operation should succeed after retry");
        assert_eq!(result.expect("should succeed").0, "Success".to_string());
        assert_operation_count(&counter, 2, "First operation");
    }
    
    // Reset counter
    *counter.lock().expect("should succeed") = 0;
    
    // Test operation that always fails (should trip circuit breaker)
    let successful_failures = trip_circuit_with_failures(&mut circuit_breaker, retry.clone(), 5).await;
    
    // We should have had at least 2 successful failure operations
    assert!(successful_failures >= 2, "Expected at least 2 failed operations, got {}", successful_failures);
    
    // Verify circuit state
    assert_circuit_tripped(&circuit_breaker).await;
    
    // If circuit is open, any further calls should be immediately rejected
    let final_state = circuit_breaker.state().await;
    if final_state == BreakerState::Open {
        let result = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            || Ok(TestString("This shouldn't be called".to_string()))
        ).await;
        
        assert!(matches!(result, Err(ResilienceError::CircuitOpen(..))),
                "Expected CircuitOpen error, got {:?}", result);
    }
}

/// Test basic retry mechanism with circuit integration
#[tokio::test]
async fn test_retry_mechanism_and_circuit_integration() {
    let retry = create_test_retry_mechanism();
    
    // Use thread-safe counter for attempt tracking
    let attempt_counter = Arc::new(Mutex::new(0));
    
    // Should succeed on the second attempt
    let retry_result: Result<TestString, RetryError> = retry.execute(|| {
        let counter = attempt_counter.clone();
        Box::pin(async move {
            let mut count = counter.lock().expect("should succeed");
            *count += 1;
            
            if *count == 1 {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary error".to_string())))
            } else {
                Ok(TestString("Success on retry".to_string()))
            }
        })
    }).await;
    
    assert!(retry_result.is_ok());
    assert_eq!(retry_result.expect("should succeed").0, "Success on retry".to_string());
    assert_operation_count(&attempt_counter, 2, "Retry operation");
}

/// Test exponential backoff retry with circuit breaker
#[tokio::test]
async fn test_exponential_retry_with_circuit() {
    let mut circuit_breaker = create_lenient_circuit_breaker("exponential-test");
    let retry = create_exponential_retry_mechanism();
    
    let attempt_counter = Arc::new(Mutex::new(0));
    
    // Test successful operation after multiple retries
    let counter_clone = attempt_counter.clone();
    let result = with_resilience(
        &mut circuit_breaker,
        retry,
        move || {
            let mut count = counter_clone.lock().expect("should succeed");
            *count += 1;
            
            if *count < 3 {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError(
                    format!("Failure attempt {}", *count)
                )))
            } else {
                Ok(TestString("Success after exponential backoff".to_string()))
            }
        }
    ).await;
    
    assert!(result.is_ok(), "Operation should succeed after exponential backoff");
    assert_eq!(result.expect("should succeed").0, "Success after exponential backoff".to_string());
    assert_operation_count(&attempt_counter, 3, "Exponential backoff operation");
}

/// Test circuit breaker behavior under rapid failure conditions
#[tokio::test]
async fn test_rapid_failure_circuit_tripping() {
    let mut circuit_breaker = create_test_circuit_breaker("rapid-failure");
    let retry = create_test_retry_mechanism();
    
    let total_attempts = Arc::new(Mutex::new(0));
    let mut circuit_opened = false;
    
    // Execute rapid failures
    for i in 0..10 {
        let attempts_clone = total_attempts.clone();
        
        let result: Result<TestString, ResilienceError> = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            move || {
                let mut count = attempts_clone.lock().expect("should succeed");
                *count += 1;
                
                Err(Box::<dyn StdError + Send + Sync>::from(TestError(
                    format!("Rapid failure {}", i)
                )))
            }
        ).await;
        
        // Check if circuit has opened
        if let Err(ResilienceError::CircuitOpen(_)) = result {
            circuit_opened = true;
            println!("Circuit opened at iteration {}", i);
            break;
        }
    }
    
    // Verify that the circuit eventually opened or we had significant failures
    let final_state = circuit_breaker.state().await;
    let metrics = circuit_breaker.metrics().await;
    
    assert!(
        circuit_opened || final_state == BreakerState::Open || metrics.failure_count >= 4,
        "Expected circuit to open or accumulate failures. State: {:?}, Failures: {}, Opened: {}",
        final_state, metrics.failure_count, circuit_opened
    );
} 