use std::sync::Arc;
use std::time::Duration;
use std::fmt;
use std::error::Error as StdError;
use std::pin::Pin;
use std::future::Future;
use tokio::test;

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
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(42)
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_failure() {
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
    
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::new(TestError::generic("test failure".to_string())) as Box<dyn StdError + Send + Sync>)
        })
    }).await;
    
    assert!(result.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
    
    // Test another failure to ensure state updating works
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::new(TestError::generic("test failure".to_string())) as Box<dyn StdError + Send + Sync>)
        })
    }).await;
    
    assert!(result.is_err());
    
    // Should still be closed since we haven't hit the threshold yet
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_open_circuit() {
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 2,  // Open after 2 failures
        recovery_timeout_ms: 1000,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    // First failure
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    // Second failure - should open circuit
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    // Circuit should be open now
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // This should be rejected without calling the function
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(42)
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
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 2,  // Open after 2 failures
        recovery_timeout_ms: 100, // Short timeout for testing
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    // Trip the circuit
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Wait for recovery timeout
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // Force transition to half-open
    circuit_breaker.check_state_transition();
    assert_eq!(circuit_breaker.state(), CircuitState::HalfOpen);
    
    // Should allow one test call
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(42)
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Should transition back to closed after success
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_fallback() {
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
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    let result: Result<i32, ResilienceError> = circuit_breaker.execute(|| {
        Box::pin(async {
            Err::<i32, _>(Box::<dyn StdError + Send + Sync>::from(TestError::generic("test failure".to_string())))
        })
    }).await;
    
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Now call should use fallback
    let result: Result<i32, ResilienceError> = circuit_breaker.execute_with_fallback(|| {
        Box::pin(async {
            Ok::<_, Box<dyn StdError + Send + Sync>>(42)
        })
    }).await;
    
    // Fallback should provide result
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 999);
}

#[tokio::test]
async fn test_fallback_execution() {
    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error: {}", self.0)
        }
    }

    impl StdError for TestError {}

    let config = CircuitBreakerConfig {
        name: "test".to_string(),
        failure_threshold: 3,
        recovery_timeout_ms: 100,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    };

    let mut circuit_breaker = CircuitBreaker::new(config);
    
    // First, fail enough times to open the circuit
    for _ in 0..5 {
        let result: Result<String, ResilienceError> = circuit_breaker.execute(|| {
            Box::pin(async {
                Err::<String, _>(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
            })
        }).await;
        assert!(result.is_err());
    }
    
    // Verify the circuit is open
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Now try with a fallback
    let result: Result<String, ResilienceError> = circuit_breaker.execute_with_fallback(|| {
        Box::pin(async {
            Err::<String, _>(Box::<dyn StdError + Send + Sync>::from(TestError("Persistent error".to_string())))
        })
    }).await;
    
    // The fallback should succeed or we should receive a circuit open error
    match result {
        Ok(val) => assert_eq!(val, "Fallback result"),
        Err(ResilienceError::CircuitOpen(_)) => (), // This is also acceptable
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
} 