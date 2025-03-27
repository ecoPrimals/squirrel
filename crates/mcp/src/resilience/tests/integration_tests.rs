use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::error::Error as StdError;
use std::fmt;
use std::thread;

use crate::resilience::{
    with_resilience,
    with_recovery,
    with_full_resilience,
    ResilienceError
};
use crate::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use crate::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureInfo, FailureSeverity};

// A test error type
#[derive(Debug)]
struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl StdError for TestError {}

#[test]
fn test_circuit_breaker_with_retry() {
    // Create components
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test-circuit".to_string(),
        failure_threshold: 3,
        recovery_timeout_ms: 100,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
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
            &retry,
            move || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                if *count < 2 {
                    Err(TestError("Temporary failure".to_string()))
                } else {
                    Ok("Success".to_string())
                }
            }
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success".to_string());
        assert_eq!(*counter.lock().unwrap(), 2);
    }
    
    // Reset counter
    *counter.lock().unwrap() = 0;
    
    // Test operation that always fails (should trip circuit breaker)
    for _ in 0..4 {  // 4 attempts to ensure we trip the circuit
        let counter_clone = counter.clone();
        let _ = with_resilience(
            &mut circuit_breaker,
            &retry,
            move || {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                
                Err(TestError("Persistent failure".to_string()))
            }
        );
    }
    
    // Circuit should be open now
    assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
    
    // Counter should reflect 3 initial failures + 2 retries = 5 attempts
    // (The 4th call shouldn't increase the counter as the circuit is already open)
    assert_eq!(*counter.lock().unwrap(), 5);
    
    // Any further calls should be immediately rejected
    let result = with_resilience(
        &mut circuit_breaker,
        &retry,
        || Ok("This shouldn't be called".to_string())
    );
    
    assert!(matches!(result, Err(ResilienceError::CircuitOpen(..))));
}

#[test]
fn test_recovery_with_retry() {
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
            let mut count = op_counter.lock().unwrap();
            *count += 1;
            
            if *count == 1 {
                // First attempt fails
                Err(Box::new(TestError("Temporary error".to_string())))
            } else {
                // Second attempt succeeds
                Ok("Success".to_string())
            }
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success".to_string());
        assert_eq!(*operation_counter.lock().unwrap(), 2);
    }
    
    // Reset counters
    *operation_counter.lock().unwrap() = 0;
    *recovery_counter.lock().unwrap() = 0;
    
    // Second scenario: Retry fails, recovery needed
    {
        let op_counter = operation_counter.clone();
        let rec_counter = recovery_counter.clone();
        
        let operation = move || {
            let mut count = op_counter.lock().unwrap();
            *count += 1;
            
            // Always fail the operation
            Err(TestError("Persistent error".to_string()))
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
            Ok("Recovered".to_string())
        };
        
        // First try retry
        let retry_result = retry.execute(|| {
            operation().map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)
        });
        
        // Retry should fail after max attempts
        assert!(retry_result.is_err());
        
        // Then try recovery
        let result = recovery.handle_failure(failure_info, recovery_action);
        
        // Recovery should succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Recovered".to_string());
        
        // Operation should have been attempted MAX_ATTEMPTS times
        assert_eq!(*operation_counter.lock().unwrap(), 2);
        
        // Recovery should have been attempted once
        assert_eq!(*recovery_counter.lock().unwrap(), 1);
    }
}

#[test]
fn test_full_resilience_chain() {
    // Set up all components
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test-full-resilience".to_string(),
        failure_threshold: 2,
        recovery_timeout_ms: 100,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Constant,
    });
    
    let mut recovery = RecoveryStrategy::new(RecoveryConfig::default());
    
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
            let mut count = op_counter.lock().unwrap();
            *count += 1;
            
            if *count == 1 {
                // First attempt fails
                Err(TestError("Temporary error".to_string()))
            } else {
                // Second attempt succeeds
                Ok("Success via retry".to_string())
            }
        };
        
        let failure_info = FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test".to_string(),
            recovery_attempts: 0,
        };
        
        let recovery_action = move || {
            let mut count = rec_counter.lock().unwrap();
            *count += 1;
            
            // Recovery succeeds
            Ok("Success via recovery".to_string())
        };
        
        let result = with_full_resilience(
            &mut circuit_breaker,
            &retry,
            &mut recovery,
            failure_info,
            operation,
            recovery_action
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success via retry".to_string());
        assert_eq!(*operation_counter.lock().unwrap(), 2); // Operation tried twice
        assert_eq!(*recovery_counter.lock().unwrap(), 0);  // Recovery not needed
    }
    
    // Scenario 2: Operation fails on all retries, recovery needed
    {
        *operation_counter.lock().unwrap() = 0;
        *recovery_counter.lock().unwrap() = 0;
        
        let op_counter = operation_counter.clone();
        let rec_counter = recovery_counter.clone();
        
        let operation = move || {
            let mut count = op_counter.lock().unwrap();
            *count += 1;
            
            // Always fail
            Err(TestError("Persistent error".to_string()))
        };
        
        let failure_info = FailureInfo {
            message: "Test failure".to_string(),
            severity: FailureSeverity::Minor,
            context: "test".to_string(),
            recovery_attempts: 0,
        };
        
        let recovery_action = move || {
            let mut count = rec_counter.lock().unwrap();
            *count += 1;
            
            // Recovery succeeds
            Ok("Success via recovery".to_string())
        };
        
        let result = with_full_resilience(
            &mut circuit_breaker,
            &retry,
            &mut recovery,
            failure_info,
            operation,
            recovery_action
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success via recovery".to_string());
        assert_eq!(*operation_counter.lock().unwrap(), 2); // Operation tried twice (max retries)
        assert_eq!(*recovery_counter.lock().unwrap(), 1);  // Recovery used once
    }
    
    // Scenario 3: Everything fails, circuit trips
    {
        // Reset counters
        *operation_counter.lock().unwrap() = 0;
        *recovery_counter.lock().unwrap() = 0;
        
        // Trip the circuit breaker with persistent failures
        for _ in 0..2 {  // 2 is the failure threshold
            let op_counter = operation_counter.clone();
            let rec_counter = recovery_counter.clone();
            
            let operation = move || {
                let mut count = op_counter.lock().unwrap();
                *count += 1;
                
                // Always fail
                Err(TestError("Persistent error".to_string()))
            };
            
            let failure_info = FailureInfo {
                message: "Test failure".to_string(),
                severity: FailureSeverity::Severe,  // Make it severe to test recovery
                context: "test".to_string(),
                recovery_attempts: 0,
            };
            
            let recovery_action = move || {
                let mut count = rec_counter.lock().unwrap();
                *count += 1;
                
                // Even recovery fails
                Err(Box::new(TestError("Recovery failed too".to_string())))
            };
            
            let _ = with_full_resilience(
                &mut circuit_breaker,
                &retry,
                &mut recovery,
                failure_info,
                operation,
                recovery_action
            );
        }
        
        // Circuit should be open now
        assert_eq!(circuit_breaker.get_state(), CircuitState::Open);
        
        // Try one more operation, it should be rejected immediately
        let result = with_full_resilience(
            &mut circuit_breaker,
            &retry,
            &mut recovery,
            FailureInfo {
                message: "Test failure".to_string(),
                severity: FailureSeverity::Minor,
                context: "test".to_string(),
                recovery_attempts: 0,
            },
            || Ok("This shouldn't be called".to_string()),
            || Ok("Recovery shouldn't be called".to_string())
        );
        
        assert!(matches!(result, Err(ResilienceError::CircuitOpen(..))));
        
        // Operation counter should reflect previous attempts only
        assert_eq!(*operation_counter.lock().unwrap(), 4); // 2 attempts × 2 failures
        
        // Recovery counter should also reflect previous attempts only
        assert_eq!(*recovery_counter.lock().unwrap(), 2); // 1 attempt × 2 failures
    }
}

#[test]
fn test_real_world_api_resilience() {
    // This test simulates a real-world API client with resilience
    
    // Define our components
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "api-circuit".to_string(),
        failure_threshold: 5,
        recovery_timeout_ms: 5000,
        half_open_success_threshold: 2,
        half_open_allowed_calls: 2,
        fallback: None,
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
    let api_cache = api_data_cache.clone();
    
    let operation = move || {
        let connected = *api_conn.lock().unwrap();
        
        if !connected {
            // Simulate connection failure
            return Err(TestError("API connection failed".to_string()));
        }
        
        // If connected, return data
        Ok("Fresh API data".to_string())
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
        let mut conn = api_conn_recovery.lock().unwrap();
        *conn = true; // Connect
        
        // Return from cache while connection is being established
        let cache = api_cache_recovery.lock().unwrap();
        match cache.as_ref() {
            Some(data) => Ok(data.clone()),
            None => Err(Box::new(TestError("No cached data available".to_string())))
        }
    };
    
    // First call - should recover and return cached data
    let result1 = with_full_resilience(
        &mut circuit_breaker,
        &retry,
        &mut recovery,
        failure_info.clone(),
        operation.clone(),
        recovery_action
    );
    
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), "Cached API data".to_string());
    assert!(*api_connection.lock().unwrap()); // Should be connected now
    
    // Second call - should use the now-established connection
    let api_conn = api_connection.clone();
    let operation2 = move || {
        let connected = *api_conn.lock().unwrap();
        
        if !connected {
            // Simulate connection failure
            return Err(TestError("API connection failed".to_string()));
        }
        
        // If connected, return data
        Ok("Fresh API data".to_string())
    };
    
    // This should succeed without recovery
    let result2 = with_resilience(
        &mut circuit_breaker,
        &retry,
        operation2
    );
    
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), "Fresh API data".to_string());
} 