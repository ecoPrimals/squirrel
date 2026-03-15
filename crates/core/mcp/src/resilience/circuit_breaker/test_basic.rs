// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Basic tests for the circuit breaker implementation
//! 
//! These tests focus only on the core circuit breaker functionality
//! without integration with other resilience components.

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use futures_util::future::FutureExt;
    use tokio::time::sleep;
    
    use crate::resilience::circuit_breaker::{
        BreakerConfig, StandardCircuitBreaker, CircuitBreaker,
        BreakerError, BreakerState
    };
    
    // Custom error type for testing
    #[derive(Debug, Clone)]
    struct TestError(String);
    
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error: {}", self.0)
        }
    }
    
    impl std::error::Error for TestError {}
    
    #[tokio::test]
    async fn test_standard_circuit_breaker() {
        // Create a circuit breaker with default configuration
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(0.5)  // 50% failure threshold
            .with_reset_timeout(Duration::from_millis(500)); // 500ms reset timeout
        
        let breaker = StandardCircuitBreaker::new(config);
        
        // First test - successful operations
        for i in 0..3 {
            let result = breaker.execute(move || async move {
                Ok::<_, BreakerError>(i)
            }.boxed()).await;
            
            assert!(result.is_ok(), "Operation {} should succeed", i);
        }
        
        // Check metrics after successes
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.success_count, 3, "Should have 3 successful operations");
        assert_eq!(metrics.failure_count, 0, "Should have 0 failed operations");
        assert_eq!(metrics.state, BreakerState::Closed, "Circuit should be closed");
        
        // Next test - failing operations
        for i in 0..4 {
            let result = breaker.execute(move || async move {
                Err::<i32, _>(BreakerError::operation_failed(
                    "test-breaker",
                    &format!("Test failure {}", i)
                ))
            }.boxed()).await;
            
            assert!(result.is_err(), "Operation {} should fail", i);
        }
        
        // Check metrics after failures
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.success_count, 3, "Should still have 3 successful operations");
        assert_eq!(metrics.failure_count, 3, "Should have 3 failed operations");
        assert_eq!(metrics.state, BreakerState::Open, "Circuit should be open");
        
        // Test that the circuit rejects requests when open
        let result = breaker.execute(|| async {
            Ok::<_, BreakerError>(42)
        }.boxed()).await;
        
        assert!(result.is_err(), "Circuit should reject requests when open");
        let err = result.unwrap_err();
        assert!(matches!(err, BreakerError::CircuitOpen { .. }), "Error should be CircuitOpen, was: {}", err);
        
        // Check that rejection was counted
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.rejected_count, 2, "Should have 2 rejected operations");
        
        // Wait for the circuit to reset
        sleep(Duration::from_millis(600)).await;
        
        // Now the circuit should be half-open and allow a test request
        let result = breaker.execute(|| async {
            Ok::<_, BreakerError>(123)
        }.boxed()).await;
        
        assert!(result.is_ok(), "Circuit should allow test request in half-open state");
        assert_eq!(result.unwrap(), 123);
        
        // Run two more successful requests to close the circuit
        for i in 0..2 {
            let result = breaker.execute(move || async move {
                Ok::<_, BreakerError>(i + 200)
            }.boxed()).await;
            
            assert!(result.is_ok(), "Operation {} should succeed in half-open state", i);
        }
        
        // Check that the circuit is closed again
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.state, BreakerState::Closed, "Circuit should be closed again");
        assert_eq!(metrics.success_count, 6, "Should have 6 successful operations");
    }
    
    #[tokio::test]
    async fn test_try_execute() {
        // Create a circuit breaker
        let config = BreakerConfig::new("test-execute");
        let breaker = StandardCircuitBreaker::new(config);
        
        // Test a successful operation
        let result = breaker.execute(|| async {
            Ok::<_, BreakerError>("success")
        }.boxed()).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
    
    #[tokio::test]
    async fn test_custom_error_type() {
        // Create a circuit breaker
        let config = BreakerConfig::new("test-custom-error");
        let breaker = StandardCircuitBreaker::new(config);
        
        // First execute an operation that succeeds
        let result = breaker.execute(|| async {
            Ok::<_, BreakerError>(42) 
        }.boxed()).await;
        
        assert!(result.is_ok());
        
        // Then execute an operation that returns a custom error
        let result = breaker.execute(|| async {
            if true {
                Err::<i32, _>(BreakerError::operation_failed(
                    "test-custom-error",
                    "Custom error test"
                ))
            } else {
                Ok(0)
            }
        }.boxed()).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BreakerError::OperationFailed { .. }));
    }
}