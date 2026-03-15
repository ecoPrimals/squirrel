// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Example of using the circuit breaker pattern
//!
//! This example demonstrates how to use the circuit breaker pattern
//! to protect against cascading failures.

use std::time::Duration;
use tokio::time::sleep;
use futures_util::future::FutureExt;

use tracing::{info, error};

// Import the error and state modules correctly
use crate::resilience::circuit_breaker::{StandardCircuitBreaker, BreakerConfig, BreakerError, CircuitBreaker};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("This example is deprecated. Use run_circuit_breaker_example instead.");
    Err("Use run_circuit_breaker_example instead".into())
}

/// Example showing how to use the circuit breaker implementation.
pub async fn run_circuit_breaker_example() {
    // Create a circuit breaker with a 50% failure threshold
    let config = BreakerConfig::new("example-breaker")
        .with_failure_threshold(0.5)
        .with_reset_timeout(Duration::from_secs(5));
    
    let breaker = StandardCircuitBreaker::new(config);
    
    info!("Starting circuit breaker example");
    
    // Run 10 operations, where the first 3 succeed and the rest fail
    for i in 1..=10 {
        let i_clone = i;
        let result = if i <= 3 {
            // Operations 1-3 succeed
            breaker.execute(move || async move { 
                info!("Operation {} succeeded", i_clone);
                Ok::<_, BreakerError>(i_clone) 
            }.boxed()).await
        } else {
            // Operations 4-10 fail
            breaker.execute(move || async move { 
                error!("Operation {} failed", i_clone);
                Err::<i32, _>(BreakerError::operation_failed(
                    "example-breaker",
                    &format!("Operation {} failed", i_clone)
                )) 
            }.boxed()).await
        };
        
        match result {
            Ok(value) => info!("Result: {}", value),
            Err(err) => info!("Error: {}", err),
        }
        
        // Print metrics every 3 operations
        if i % 3 == 0 || i == 10 {
            let metrics = breaker.metrics().await;
            info!("Current metrics: {:?}", metrics);
            info!("Circuit state: {}", metrics.state);
        }
    }
    
    // The circuit should be open now due to failure rate > 50%
    info!("Circuit is open, waiting for reset timeout...");
    
    // Sleep for 5 seconds to allow the circuit breaker to reset
    sleep(Duration::from_secs(6)).await;
    
    info!("Testing recovery after reset timeout...");
    
    // Try 5 more operations, which should succeed
    for i in 1..=5 {
        let i_clone = i;
        let result = breaker.execute(move || async move {
            info!("Recovery operation {} succeeded", i_clone);
            Ok::<_, BreakerError>(i_clone + 100)
        }.boxed()).await;
        
        match result {
            Ok(value) => info!("Recovery result: {}", value),
            Err(err) => info!("Recovery error: {}", err),
        }
    }
    
    // Print final metrics
    let final_metrics = breaker.metrics().await;
    info!("Final metrics: {:?}", final_metrics);
    info!("Final circuit state: {}", final_metrics.state);
} 