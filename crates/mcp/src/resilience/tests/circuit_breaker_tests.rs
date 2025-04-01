use std::fmt;
use std::error::Error as StdError;

use crate::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::resilience::ResilienceError;

#[derive(Debug)]
pub enum TestError {
    Generic(String),
}

impl TestError {
    pub fn generic(message: String) -> Self {
        TestError::Generic(message)
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestError::Generic(msg) => write!(f, "Generic test error: {}", msg),
        }
    }
}

impl StdError for TestError {}

#[tokio::test]
async fn test_circuit_breaker_success() {
    // Define a wrapper around i32 that can implement From<i32>
    #[derive(Debug, PartialEq)]
    struct TestInt(i32);
    
    impl From<i32> for TestInt {
        fn from(value: i32) -> Self {
            TestInt(value)
        }
    }

    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(TestInt(42))
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestInt(42));
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_failure() {
    // Define a wrapper around i32 that can implement From<i32>
    #[derive(Debug, PartialEq)]
    struct TestInt(i32);
    
    impl From<i32> for TestInt {
        fn from(value: i32) -> Self {
            TestInt(value)
        }
    }

    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::new(TestError::generic("test failure".to_string())) as Box<dyn StdError + Send + Sync>)
        })
    }).await;
    
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    
    // Test another failure to ensure state updating works
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::new(TestError::generic("test failure".to_string())) as Box<dyn StdError + Send + Sync>)
        })
    }).await;
    
    assert!(result.is_err());
    
    // Should still be closed since we haven't hit the threshold yet
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_open_circuit() {
    // Define a wrapper around i32 that can implement From<i32>
    #[derive(Debug, PartialEq)]
    struct TestInt(i32);
    
    impl From<i32> for TestInt {
        fn from(value: i32) -> Self {
            TestInt(value)
        }
    }

    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 2,  // Open after 2 failures
        recovery_timeout_ms: 1000,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    // First failure
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    // Second failure - should open circuit
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    // Circuit should be open now
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // This should be rejected without calling the function
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(TestInt(42))
        })
    }).await;
    
    assert!(result.is_err());
    
    match result {
        Err(ResilienceError::CircuitOpen(_)) => (), // Expected
        _ => panic!("Expected CircuitOpen error"),
    }
}

#[tokio::test]
async fn test_circuit_breaker_half_open() {
    // Define a wrapper around i32 that can implement From<i32>
    #[derive(Debug, PartialEq)]
    struct TestInt(i32);
    
    impl From<i32> for TestInt {
        fn from(value: i32) -> Self {
            TestInt(value)
        }
    }

    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 2,  // Open after 2 failures
        recovery_timeout_ms: 100, // Short timeout for testing
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    // Trip the circuit
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // Force transition to half-open
    circuit_breaker.check_state_transition();
    assert_eq!(circuit_breaker.state(), CircuitState::HalfOpen);
    
    // Should allow one test call
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(TestInt(42))
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestInt(42));
    
    // Should transition back to closed after success
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_fallback() {
    // Define a wrapper around i32 that can implement From<i32>
    #[derive(Debug, PartialEq)]
    struct TestInt(i32);
    
    impl From<i32> for TestInt {
        fn from(value: i32) -> Self {
            TestInt(value)
        }
    }

    let fallback_fn = || 999;
    
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 2,  // Open after 2 failures
        recovery_timeout_ms: 1000,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: Some(Box::new(fallback_fn)),
    });
    
    // Trip the circuit
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    let _result: Result<TestInt, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestInt, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Now call should use fallback
    let result: Result<TestInt, ResilienceError> = circuit_breaker.execute_with_fallback(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(TestInt(42))
        })
    }).await;
    
    // Fallback should provide result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestInt(999));
}

#[tokio::test]
async fn test_fallback_execution() {
    // Test error type
    #[derive(Debug)]
    struct TestError(String);
    
    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestError: {}", self.0)
        }
    }
    
    impl StdError for TestError {}
    
    // Create a test wrapper type
    #[derive(Debug, PartialEq)]
    struct TestString(String);
    
    impl From<i32> for TestString {
        fn from(value: i32) -> Self {
            TestString(format!("Fallback result: {}", value))
        }
    }
    
    // Create a circuit breaker with a fallback function
    let config = CircuitBreakerConfig {
        name: "test-cb".to_string(),
        failure_threshold: 3,
        recovery_timeout_ms: 5000,
        half_open_success_threshold: 2,
        half_open_allowed_calls: 4,
        fallback: Some(Box::new(|| 999)),
    };

    let mut circuit_breaker = CircuitBreaker::new(config);
    
    // First, fail enough times to just reach the failure threshold
    for i in 0..3 {
        let result: Result<TestString, ResilienceError> = circuit_breaker.execute(|| {
            Box::pin(async {
                Err::<TestString, _>(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
            })
        }).await;
        
        println!("Iteration {}: Circuit state: {:?}, Result: {:?}", i, circuit_breaker.state(), result);
        
        if i < 2 {
            // First two iterations should fail with the operation error
            assert!(result.is_err());
        } else {
            // The third iteration triggers the circuit to open
            assert_eq!(circuit_breaker.state(), CircuitState::Open);
        }
    }
    
    // Verify the circuit is open
    let current_state = circuit_breaker.state();
    println!("After failures, circuit state: {:?}", current_state);
    assert_eq!(current_state, CircuitState::Open);
    
    // Now try with explicit execute_with_fallback when circuit is open
    let result: Result<TestString, ResilienceError> = circuit_breaker.execute_with_fallback(|| {
        Box::pin(async {
            Err::<TestString, _>(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
        })
    }).await;
    
    println!("Fallback result: {:?}", result);
    
    // The fallback should succeed with the string conversion from i32
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestString(String::from("Fallback result: 999")));
    
    // Now try with a normal execute when circuit is open - should use fallback
    let result: Result<TestString, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<TestString, _>(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
        })
    }).await;
    
    println!("Normal execute with open circuit result: {:?}", result);
    
    // Should also get the fallback result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), TestString(String::from("Fallback result: 999")));
} 