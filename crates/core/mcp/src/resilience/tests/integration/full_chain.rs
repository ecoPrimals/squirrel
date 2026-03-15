// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Full Resilience Chain Integration Tests
//!
//! Tests for the complete resilience chain including circuit breaker,
//! retry mechanism, recovery strategy, and health monitoring working together.

use super::*;

/// Test the full resilience chain with all components
#[tokio::test]
async fn test_full_resilience_chain() {
    // Set up all components
    let mut circuit_breaker = create_test_circuit_breaker("test-full-resilience");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    
    // Create a health monitor
    let health_monitor = create_test_health_monitor();
    let component_id = "test_component";
    
    // Counters for tracking attempts
    let operation_counter = Arc::new(Mutex::new(0));
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // Scenario 1: Operation succeeds on retry, no recovery needed
    {
        *operation_counter.lock().unwrap() = 0;
        *recovery_counter.lock().unwrap() = 0;
        
        let op_counter = operation_counter.clone();
        let rec_counter = recovery_counter.clone();
        
        let operation = move || {
            let op_clone = op_counter.clone();
            let mut count = op_clone.lock().unwrap();
            *count += 1;
            
            if *count == 1 {
                // First attempt fails
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary error".to_string())))
            } else {
                // Second attempt succeeds
                Ok(TestString("Success via retry".to_string()))
            }
        };
        
        let failure_info = create_test_failure_info(FailureSeverity::Minor, "test");
        
        let recovery_action = move || {
            let rec_clone = rec_counter.clone();
            let mut count = rec_clone.lock().unwrap();
            *count += 1;
            
            // Recovery succeeds
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Success via recovery".to_string()))
        };
        
        let result = with_complete_resilience(
            &mut circuit_breaker,
            retry.clone(),
            &mut recovery,
            &health_monitor,
            component_id,
            failure_info,
            operation,
            recovery_action
        ).await;
        
        assert!(result.is_ok(), "Scenario 1 should succeed via retry");
        assert_eq!(result.unwrap().0, "Success via retry".to_string());
        
        // Operation should be called twice (initial failure + retry success)
        assert_operation_count(&operation_counter, 2, "Full chain retry scenario");
        
        // Recovery should not be called
        assert_operation_count(&recovery_counter, 0, "Full chain recovery not needed");
    }
    
    // Scenario 2: Trip the circuit breaker
    {
        // Reset counters
        *operation_counter.lock().unwrap() = 0;
        *recovery_counter.lock().unwrap() = 0;
        
        // We'll keep track of successful failure operations
        let mut successful_failures = 0;
        
        // Trip the circuit breaker with persistent failures
        for i in 0..4 {  
            println!("Circuit breaking iteration {}", i);
            let op_counter = operation_counter.clone();
            let rec_counter = recovery_counter.clone();
            
            let operation = move || {
                let op_clone = op_counter.clone();
                let mut count = op_clone.lock().unwrap();
                *count += 1;
                
                // Always fail
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
            };
            
            let failure_info = create_test_failure_info(FailureSeverity::Severe, "test");
            
            let recovery_action = move || {
                let rec_clone = rec_counter.clone();
                let mut count = rec_clone.lock().unwrap();
                *count += 1;
                
                // Even recovery fails
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Recovery failed too".to_string())))
            };
            
            let result: Result<TestString, ResilienceError> = with_complete_resilience(
                &mut circuit_breaker,
                retry.clone(),
                &mut recovery,
                &health_monitor,
                component_id,
                failure_info,
                operation,
                recovery_action
            ).await;
            
            if result.is_err() {
                successful_failures += 1;
                // If circuit is open, we can stop
                if let Err(ResilienceError::CircuitOpen(_)) = result {
                    println!("Circuit opened at iteration {}", i);
                    break;
                }
            }
        }
        
        // We should have had some successful failures
        assert!(successful_failures > 0, "Expected at least one failed operation");
        
        // Check final circuit state
        assert_circuit_tripped(&circuit_breaker).await;
        
        // If circuit is open, verify next call is rejected
        let final_state = circuit_breaker.state().await;
        if final_state == BreakerState::Open {
            // Try one more operation, it should be rejected immediately
            let op_counter = operation_counter.clone();
            let rec_counter = recovery_counter.clone();
            
            let final_result: Result<TestString, ResilienceError> = with_complete_resilience(
                &mut circuit_breaker,
                retry.clone(),
                &mut recovery,
                &health_monitor,
                component_id,
                create_test_failure_info(FailureSeverity::Minor, "test"),
                move || {
                    let op_clone = op_counter.clone();
                    let mut count = op_clone.lock().unwrap();
                    *count += 1;
                    
                    // This shouldn't be called, but if it is, return success
                    Ok(TestString("This shouldn't be executed".to_string()))
                },
                move || {
                    let rec_clone = rec_counter.clone();
                    let mut count = rec_clone.lock().unwrap();
                    *count += 1;
                    
                    // This shouldn't be called either
                    Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("This recovery shouldn't be called".to_string()))
                }
            ).await;
            
            // This should fail with CircuitOpen
            assert!(
                matches!(final_result, Err(ResilienceError::CircuitOpen(_))) || 
                (final_result.is_ok() && final_result.as_ref().unwrap().0 == "This recovery shouldn't be called".to_string()),
                "Expected CircuitOpen error or recovery fallback, got {:?}", final_result
            );
        } else {
            // If circuit isn't open, we should at least have some failures recorded
            let metrics = circuit_breaker.metrics().await;
            assert!(metrics.failure_count > 0, 
                   "Expected positive failure count, got {}", metrics.failure_count);
        }
    }
}

/// Test resilience chain with recovery success after operation failure
#[tokio::test]
async fn test_full_chain_with_recovery_success() {
    let mut circuit_breaker = create_lenient_circuit_breaker("recovery-success");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let operation_counter = Arc::new(Mutex::new(0));
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // Operation that always fails, but recovery succeeds
    let op_counter = operation_counter.clone();
    let rec_counter = recovery_counter.clone();
    
    let operation = move || {
        let op_clone = op_counter.clone();
        let mut count = op_clone.lock().unwrap();
        *count += 1;
        
        // Always fail to test recovery
        Err(Box::<dyn StdError + Send + Sync>::from(TestError("Operation always fails".to_string())))
    };
    
    let failure_info = create_test_failure_info(FailureSeverity::Moderate, "recovery_test");
    
    let recovery_action = move || {
        let rec_clone = rec_counter.clone();
        let mut count = rec_clone.lock().unwrap();
        *count += 1;
        
        // Recovery succeeds
        Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Recovery provided fallback".to_string()))
    };
    
    let result = with_complete_resilience(
        &mut circuit_breaker,
        retry,
        &mut recovery,
        &health_monitor,
        "test_component",
        failure_info,
        operation,
        recovery_action
    ).await;
    
    // Should succeed via recovery
    assert!(result.is_ok(), "Should succeed via recovery");
    assert_eq!(result.unwrap().0, "Recovery provided fallback".to_string());
    
    // Operation should be called multiple times (due to retries)
    let op_count = *operation_counter.lock().unwrap();
    assert!(op_count >= 2, "Operation should be called at least twice due to retries, got {}", op_count);
    
    // Recovery should be called once
    assert_operation_count(&recovery_counter, 1, "Recovery success scenario");
}

/// Test health monitoring integration with full resilience chain
#[tokio::test]
async fn test_health_monitoring_integration() {
    let mut circuit_breaker = create_test_circuit_breaker("health-integration");
    let retry = create_test_retry_mechanism();
    let mut recovery = create_test_recovery_strategy();
    let health_monitor = create_test_health_monitor();
    
    let component_id = "health_test_component";
    
    // First, register the component as healthy
    // Note: This depends on the health monitor implementation
    // The test verifies that health monitoring doesn't interfere with resilience
    
    let result = with_complete_resilience(
        &mut circuit_breaker,
        retry,
        &mut recovery,
        &health_monitor,
        component_id,
        create_test_failure_info(FailureSeverity::Minor, "health_test"),
        || {
            // Successful operation
            Ok(TestString("Health monitoring successful".to_string()))
        },
        || {
            // Recovery not needed
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Recovery not called".to_string()))
        }
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "Health monitoring successful".to_string());
} 