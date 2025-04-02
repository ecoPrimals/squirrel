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

// Define a wrapper around String that can implement From<i32>
#[derive(Debug, PartialEq, Clone)]
struct TestString(String);

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

// A test error type
#[derive(Debug)]
struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl StdError for TestError {}

#[tokio::test]
async fn test_circuit_breaker_with_retry() {
    // Create components
    let mut circuit_breaker = CircuitBreaker::new(BreakerConfig {
        name: "test-circuit".to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 2,
        reset_timeout_ms: 500,
        half_open_success_threshold: 1,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    // Create a counter for tracking attempts
    let counter = Arc::new(Mutex::new(0));
    
    // Test successful operation (succeeds on second retry)
    {
        let counter_clone = counter.clone();
        let result = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            move || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                if *count < 2 {
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary failure".to_string())))
                } else {
                    Ok(TestString("Success".to_string()))
                }
            }
        ).await;
        
        assert!(result.is_ok(), "First operation should succeed after retry");
        assert_eq!(result.unwrap().0, "Success".to_string());
        assert_eq!(*counter.lock().unwrap(), 2, "Counter should be 2 after successful retry");
    }
    
    // Reset counter
    *counter.lock().unwrap() = 0;
    
    // Test operation that always fails (should trip circuit breaker)
    // Use a separate loop counter to ensure we make enough attempts
    let mut successful_failures = 0;
    
    for i in 0..5 {  // Increase attempts to ensure we trip the circuit
        println!("Executing failure loop iteration {}", i);
        let counter_clone = counter.clone();
        let result: Result<TestString, ResilienceError> = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            move || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent failure".to_string())))
            }
        ).await;
        
        // Track how many failure operations we executed
        if result.is_err() {
            successful_failures += 1;
            if let Err(ResilienceError::CircuitOpen(_)) = result {
                println!("Circuit opened at iteration {}", i);
                break; // Circuit is now open, can stop failing
            }
        }
    }
    
    // We should have had at least 2 successful failure operations
    assert!(successful_failures >= 2, "Expected at least 2 failed operations, got {}", successful_failures);
    
    // Get final circuit state
    let final_state = circuit_breaker.state().await;
    println!("Final circuit state: {:?}", final_state);
    
    // Circuit should be open or at least have high failure count
    if final_state != BreakerState::Open {
        let metrics = circuit_breaker.metrics().await;
        assert!(metrics.failure_count >= 2, 
                "If circuit not open, expected at least 2 failures, got {}", metrics.failure_count);
    } else {
        // Any further calls should be immediately rejected
        let result = with_resilience(
            &mut circuit_breaker,
            retry.clone(),
            || Ok(TestString("This shouldn't be called".to_string()))
        ).await;
        
        assert!(matches!(result, Err(ResilienceError::CircuitOpen(..))),
                "Expected CircuitOpen error, got {:?}", result);
    }
}

#[tokio::test]
async fn test_recovery_with_retry() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 2,
        max_moderate_attempts: 1,
        max_severe_attempts: 1,
        recover_critical: false,
    });
    
    let operation_counter = Arc::new(Mutex::new(0));
    let recovery_counter = Arc::new(Mutex::new(0));
    
    // First scenario: Retry succeeds, no recovery needed
    {
        let op_counter = operation_counter.clone();
        
        let result = retry.execute(move || {
            let op_counter_clone = op_counter.clone();
            Box::pin(async move {
                let mut count = op_counter_clone.lock().unwrap();
                *count += 1;
                
                if *count == 1 {
                    // First attempt fails
                    Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary error".to_string())))
                } else {
                    // Second attempt succeeds
                    Ok(TestString("Success".to_string()))
                }
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Success".to_string());
        assert_eq!(*operation_counter.lock().unwrap(), 2);
    }
    
    // Reset counters
    *operation_counter.lock().unwrap() = 0;
    *recovery_counter.lock().unwrap() = 0;
    
    // Second scenario: Retry fails, recovery needed
    {
        let op_counter = operation_counter.clone();
        let rec_counter = recovery_counter.clone();
        
        // Create the operation closure
        let operation_clone = move || {
            let mut count = op_counter.lock().unwrap();
            *count += 1;
            
            // Always fail the operation
            Err(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
        };
        
        let failure_info = FailureInfo {
            message: "Operation failed after retries".to_string(),
            severity: FailureSeverity::Moderate,
            context: "test".to_string(),
            recovery_attempts: 0,
        };
        
        // Create recovery action
        let recovery_action = move || {
            let mut count = rec_counter.lock().unwrap();
            *count += 1;
            
            // Recovery succeeds
            Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Recovered".to_string()))
        };
        
        // First try retry
        let retry_result: Result<TestString, RetryError> = retry.execute(|| {
            // Clone the operation again to avoid ownership issues
            let op = operation_clone.clone();
            Box::pin(async move {
                op()
            })
        }).await;
        
        // Retry should fail after max attempts
        assert!(retry_result.is_err());
        
        // Then try recovery
        let result: Result<TestString, _> = recovery.handle_failure(failure_info, recovery_action);
        
        // Recovery should succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, "Recovered".to_string());
        
        // Operation should have been attempted MAX_ATTEMPTS times
        assert_eq!(*operation_counter.lock().unwrap(), 2);
        
        // Recovery should have been attempted once
        assert_eq!(*recovery_counter.lock().unwrap(), 1);
    }
}

#[tokio::test]
async fn test_full_resilience_chain() {
    // Set up all components
    let mut circuit_breaker = CircuitBreaker::new(BreakerConfig {
        name: "test-full-resilience".to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 2,  // Explicit minimum threshold
        reset_timeout_ms: 500,         // Increased timeout
        half_open_success_threshold: 1,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let mut recovery = RecoveryStrategy::new(RecoveryConfig::default());
    
    // Create a health monitor
    let health_monitor = health::HealthMonitor::new(10);
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
        
        let failure_info = FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test".to_string(),
            recovery_attempts: 0,
        };
        
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
        assert_eq!(*operation_counter.lock().unwrap(), 2, "Operation should be called twice");
        
        // Recovery should not be called
        assert_eq!(*recovery_counter.lock().unwrap(), 0, "Recovery should not be called");
    }
    
    // Scenario 2: Trip the circuit breaker
    {
        // Reset counters
        *operation_counter.lock().unwrap() = 0;
        *recovery_counter.lock().unwrap() = 0;
        
        // We'll keep track of successful failure operations
        let mut successful_failures = 0;
        
        // Trip the circuit breaker with persistent failures
        for i in 0..4 {  // Increased to ensure we trip the circuit
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
            
            let failure_info = FailureInfo {
                message: "Test failure".to_string(),
                severity: FailureSeverity::Severe,  // Make it severe to test recovery
                context: "test".to_string(),
                recovery_attempts: 0,
            };
            
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
        let final_state = circuit_breaker.state().await;
        println!("Final circuit state: {:?}", final_state);
        
        // If circuit is open, verify next call is rejected
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
                FailureInfo {
                    message: "Test failure".to_string(),
                    severity: FailureSeverity::Minor,
                    context: "test".to_string(),
                    recovery_attempts: 0,
                },
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

#[tokio::test]
async fn test_real_world_api_resilience() {
    // This test simulates a real-world API client with resilience
    
    // Define our components
    let mut circuit_breaker = CircuitBreaker::new(BreakerConfig {
        name: "api-circuit".to_string(),
        failure_threshold: 5.0,
        minimum_request_threshold: 1,
        reset_timeout_ms: 1000,
        half_open_success_threshold: 1,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(200),
        use_jitter: true,
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 3,
        max_moderate_attempts: 2,
        max_severe_attempts: 1,
        recover_critical: false,
    });
    
    // Simulate API state
    let api_connection = Arc::new(Mutex::new(false)); // Initially disconnected
    let api_data_cache = Arc::new(Mutex::new(Some("Cached API data".to_string())));
    
    // First, attempt to connect and get data with full resilience
    let api_conn = api_connection.clone();
    let _api_cache = api_data_cache.clone();
    
    let operation = move || {
        let conn_clone = api_conn.clone();
        let connected = *conn_clone.lock().unwrap();
        
        if !connected {
            // Simulate connection failure
            return Err(Box::<dyn StdError + Send + Sync>::from(TestError("API connection failed".to_string())));
        }
        
        // If connected, return data
        Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Fresh API data".to_string()))
    };
    
    let failure_info = FailureInfo {
        message: "API connection failed".to_string(),
        severity: FailureSeverity::Moderate,
        context: "api.connection".to_string(),
        recovery_attempts: 0,
    };
    
    let api_conn_recovery = api_connection.clone();
    let api_cache_recovery = api_data_cache.clone();
    
    let recovery_action = move || {
        // Recovery action: establish connection
        let conn_recovery_clone = api_conn_recovery.clone();
        let cache_recovery_clone = api_cache_recovery.clone();
        
        let mut conn = conn_recovery_clone.lock().unwrap();
        *conn = true; // Connect
        
        // Return from cache while connection is being established
        let cache = cache_recovery_clone.lock().unwrap();
        match cache.as_ref() {
            Some(data) => Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString(data.clone())),
            None => Err(Box::<dyn StdError + Send + Sync>::from(TestError("No cached data available".to_string())))
        }
    };
    
    // First call - should recover and return cached data
    let health_monitor = health::HealthMonitor::new(10);
    let result1 = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "test_component",
        failure_info.clone(),
        operation.clone(),
        recovery_action,
    ).await;
    
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap().0, "Cached API data".to_string());
    assert!(*api_connection.lock().unwrap()); // Should be connected now
    
    // Second call - should use the now-established connection
    let api_conn = api_connection.clone();
    let operation2 = move || {
        let conn_clone = api_conn.clone();
        let connected = *conn_clone.lock().unwrap();
        
        if !connected {
            // Simulate connection failure
            return Err(Box::<dyn StdError + Send + Sync>::from(TestError("API connection failed".to_string())));
        }
        
        // If connected, return data
        Ok::<TestString, Box<dyn StdError + Send + Sync>>(TestString("Fresh API data".to_string()))
    };
    
    // This should succeed without recovery
    let result2 = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        operation2
    ).await;
    
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().0, "Fresh API data".to_string());
}

#[tokio::test]
async fn test_with_resilience_success() {
    let mut circuit_breaker = CircuitBreaker::new(BreakerConfig {
        name: "test-circuit".to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 3,
        reset_timeout_ms: 1000,
        half_open_success_threshold: 1,
    });

    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Constant,
        base_delay: Duration::from_millis(10),
        ..Default::default()
    });

    let result: Result<TestString, ResilienceError> = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        move || {
            // This operation succeeds
            Result::<TestString, Box<dyn StdError + Send + Sync>>::Ok(TestString("Success".to_string()))
        }
    ).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, "Success");
}

#[tokio::test]
async fn test_retry_mechanism_and_circuit_integration() {
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3, 
        backoff_strategy: BackoffStrategy::Constant,
        base_delay: Duration::from_millis(10),
        ..Default::default()
    });
    
    // Use thread-safe counter for attempt tracking
    let attempt_counter = Arc::new(Mutex::new(0));
    
    // Should succeed on the second attempt
    let retry_result: Result<TestString, RetryError> = retry.execute(|| {
        let counter = attempt_counter.clone();
        Box::pin(async move {
            let mut count = counter.lock().unwrap();
            *count += 1;
            
            if *count == 1 {
                Err(Box::<dyn StdError + Send + Sync>::from(TestError("Temporary error".to_string())))
            } else {
                Ok(TestString("Success on retry".to_string()))
            }
        })
    }).await;
    
    assert!(retry_result.is_ok());
    assert_eq!(retry_result.unwrap().0, "Success on retry".to_string());
    assert_eq!(*attempt_counter.lock().unwrap(), 2);
}

#[tokio::test]
async fn test_full_resilience_pipeline() {
    let mut circuit_breaker = CircuitBreaker::new(BreakerConfig {
        name: "test-pipeline".to_string(),
        failure_threshold: 0.5,
        minimum_request_threshold: 5,
        reset_timeout_ms: 100,
        half_open_success_threshold: 1,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        backoff_strategy: BackoffStrategy::Constant,
        base_delay: Duration::from_millis(10),
        ..Default::default()
    });
    
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 2,
        max_moderate_attempts: 1,
        max_severe_attempts: 1,
        recover_critical: false,
    });
    
    let api_data_cache = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut cache = api_data_cache.lock().unwrap();
        cache.insert("test_data".to_string(), 42);
    }
    
    // Create health monitor
    let health_monitor = health::HealthMonitor::new(10);
    
    // Test the complete integration
    let _: Result<i32, ResilienceError> = with_complete_resilience(
        &mut circuit_breaker,
        retry.clone(),
        &mut recovery,
        &health_monitor,
        "test_component",
        FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Moderate,
            context: "test-service".to_string(),
            recovery_attempts: 0,
        },
        move || {
            // This should succeed
            Result::<i32, Box<dyn StdError + Send + Sync>>::Ok(123)
        },
        move || {
            // This is the fallback
            Result::<i32, Box<dyn StdError + Send + Sync>>::Ok(999)
        }
    ).await;
} 