// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Unit tests for circuit breaker implementation

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};

use crate::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
use crate::monitoring::create_production_monitoring_client;
use crate::error::MCPError;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    use anyhow::anyhow;
    
    use crate::monitoring::InMemoryMonitoringClient;
    use super::super::{
        BreakerConfig,
        BreakerState,
        CircuitBreaker,
        BoxFuture,
        new_circuit_breaker,
        named_circuit_breaker,
    };
    
    #[tokio::test]
    async fn test_successful_operation() {
        // Create a circuit breaker with default configuration
        let breaker = named_circuit_breaker("test-breaker", None);
        
        // Test successful operation
        let result = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("success") })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "success");
        
        // Check metrics
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.failure_count, 0);
        assert_eq!(metrics.rejection_count, 0);
        assert_eq!(metrics.current_state, BreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_failed_operation() {
        // Create a circuit breaker with default configuration
        let breaker = named_circuit_breaker("test-breaker", None);
        
        // Test failed operation
        let result = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Err("failure".to_string()) })
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().is_operation_failed());
        
        // Check metrics
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.success_count, 0);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.rejection_count, 0);
        assert_eq!(metrics.current_state, BreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        // Create a circuit breaker with low failure threshold
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(3);
        let breaker = new_circuit_breaker(config, None);
        
        // Fail 3 times to open the circuit
        for _ in 0..3 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Err("failure".to_string()) })
            }).await;
        }
        
        // Check metrics
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.failure_count, 3);
        assert_eq!(metrics.current_state, BreakerState::Open);
        
        // This should be rejected (circuit open)
        let result = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("should be rejected") })
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().is_circuit_open());
        
        // Check rejection count
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.rejection_count, 1);
    }
    
    #[tokio::test]
    async fn test_circuit_transitions_to_half_open() {
        // Create a circuit breaker with custom reset timeout
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_millis(100));
        let breaker = new_circuit_breaker(config, None);
        
        // Fail twice to open the circuit
        for _ in 0..2 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Err("failure".to_string()) })
            }).await;
        }
        
        // Check metrics - should be Open
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::Open);
        
        // Wait for the reset timeout to transition to half-open
        sleep(Duration::from_millis(150)).await;
        
        // Execute a successful operation - should be in half-open and succeed
        let result = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("success") })
        }).await;
        
        assert!(result.is_ok());
        
        // Check metrics - should be in HalfOpen
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::HalfOpen);
        assert_eq!(metrics.current_successes, 1);
    }
    
    #[tokio::test]
    async fn test_circuit_closes_after_successes() {
        // Create a circuit breaker with custom thresholds
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(2)
            .with_success_threshold(2)
            .with_reset_timeout(Duration::from_millis(100));
        let breaker = new_circuit_breaker(config, None);
        
        // Fail twice to open the circuit
        for _ in 0..2 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Err("failure".to_string()) })
            }).await;
        }
        
        // Wait for the reset timeout to transition to half-open
        sleep(Duration::from_millis(150)).await;
        
        // Success twice to close the circuit
        for _ in 0..2 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Ok("success") })
            }).await;
        }
        
        // Check metrics - should be Closed again
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_circuit_remains_open_after_failure_in_half_open() {
        // Create a circuit breaker with custom configuration
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_millis(100));
        let breaker = new_circuit_breaker(config, None);
        
        // Fail twice to open the circuit
        for _ in 0..2 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Err("failure".to_string()) })
            }).await;
        }
        
        // Wait for the reset timeout to transition to half-open
        sleep(Duration::from_millis(150)).await;
        
        // Execute a failing operation - should go back to Open
        let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Err("still failing".to_string()) })
        }).await;
        
        // Check metrics - should be Open again
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::Open);
    }
    
    #[tokio::test]
    async fn test_reset_circuit_breaker() {
        // Create a circuit breaker with custom configuration
        let config = BreakerConfig::new("test-breaker")
            .with_failure_threshold(2);
        let breaker = new_circuit_breaker(config, None);
        
        // Fail twice to open the circuit
        for _ in 0..2 {
            let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
                Box::pin(async { Err("failure".to_string()) })
            }).await;
        }
        
        // Check state - should be Open
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::Open);
        
        // Manually reset the circuit
        breaker.reset().await;
        
        // Check state - should be Closed
        let metrics = breaker.metrics().await;
        assert_eq!(metrics.current_state, BreakerState::Closed);
        
        // Execute an operation - should succeed
        let result = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("success after reset") })
        }).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_monitoring_integration() {
        // Create a mock monitoring client
        let monitoring = create_production_monitoring_client();
        
        // Create a circuit breaker with monitoring
        let config = BreakerConfig::new("monitored-breaker")
            .with_failure_threshold(2)
            .with_monitoring(true);
        let breaker = new_circuit_breaker(config, Some(monitoring.clone()));
        
        // Execute operations
        let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("success") })
        }).await;
        
        let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Err("failure".to_string()) })
        }).await;
        
        let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Err("failure".to_string()) })
        }).await;
        
        // At this point, circuit should be open
        let _ = breaker.execute(|| -> BoxFuture<Result<&'static str, String>> {
            Box::pin(async { Ok("rejected") })
        }).await;
        
        // Check if monitoring client received notifications
        assert_eq!(monitoring.get_event_count("monitored-breaker.success"), 1);
        assert_eq!(monitoring.get_event_count("monitored-breaker.failure"), 2);
        assert_eq!(monitoring.get_event_count("monitored-breaker.rejection"), 1);
    }
    
    #[tokio::test]
    async fn test_try_execute_with_different_error_type() {
        // Create a circuit breaker
        let breaker = named_circuit_breaker("test-breaker", None);
        
        // Test try_execute with anyhow error type
        let result: Result<&'static str, anyhow::Error> = breaker
            .try_execute(|| -> BoxFuture<Result<&'static str, anyhow::Error>> {
                Box::pin(async { Ok("success") })
            })
            .await;
        
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "success");
        
        // Test try_execute with failure
        let result: Result<&'static str, anyhow::Error> = breaker
            .try_execute(|| -> BoxFuture<Result<&'static str, anyhow::Error>> {
                Box::pin(async { Err(anyhow!("custom error")) })
            })
            .await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "custom error");
    }
} 