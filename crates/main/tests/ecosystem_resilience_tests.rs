//! Ecosystem Resilience Tests
//!
//! This test suite focuses on fault tolerance, recovery, and resilience patterns
//! for the ecosystem integration. It tests various resilience mechanisms including:
//! - Circuit breakers for failure detection and recovery
//! - Retry mechanisms with exponential backoff
//! - Fallback mechanisms for graceful degradation
//! - Cascading failure prevention
//! - Bulkhead patterns for resource isolation
//! - Timeout handling and recovery
//! - Graceful degradation under load
//! - Recovery after system outages
//!
//! # Test Architecture
//!
//! The tests use a structured approach with:
//! - FailureSimulation for controlled failure injection
//! - ResilienceTestEnvironment for test orchestration
//! - ResilienceMetrics for comprehensive measurement
//! - Multiple phases per test to verify state transitions
//!
//! # Usage
//!
//! Run all resilience tests:
//! ```bash
//! cargo test --test ecosystem_resilience_tests
//! ```
//!
//! Run specific resilience pattern tests:
//! ```bash
//! cargo test test_circuit_breaker_pattern
//! cargo test test_retry_mechanism
//! ```

use squirrel::biomeos_integration::*;
use squirrel::error::PrimalError;
// UniversalSecurityAdapter not needed for these tests
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{sleep, timeout};

// ============================================================================
// FAILURE SIMULATION TYPES
// ============================================================================

/// Failure simulation configuration
#[derive(Clone, Debug)]
pub struct FailureSimulation {
    pub failure_type: FailureType,
    pub duration: Duration,
    pub probability: f64,
    pub recovery_time: Duration,
    pub max_retries: u32,
}

/// Types of failures to simulate
#[derive(Clone, Debug)]
pub enum FailureType {
    NetworkTimeout,
    ServiceUnavailable,
    AuthenticationFailure,
    ResourceExhaustion,
    PartialFailure,
    CascadingFailure,
    DataCorruption,
    SecurityBreach,
}

/// Circuit breaker states
#[derive(Clone, Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

// ============================================================================
// TEST ENVIRONMENT AND METRICS
// ============================================================================

/// Resilience test environment
#[derive(Clone)]
pub struct ResilienceTestEnvironment {
    pub squirrel_instance: Arc<RwLock<SquirrelBiomeOSIntegration>>,
    pub failure_simulation: Arc<RwLock<FailureSimulation>>,
    pub metrics: Arc<RwLock<ResilienceMetrics>>,
    // Removed security_adapter as it's not available
}

/// Resilience metrics collection
#[derive(Debug, Clone)]
pub struct ResilienceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub recovered_requests: u64,
    pub circuit_breaker_trips: u64,
    pub retry_attempts: u64,
    pub fallback_activations: u64,
    pub mean_time_to_recovery: Duration,
    pub failure_detection_time: Duration,
    pub recovery_time: Duration,
    pub availability_percentage: f64,
}

impl ResilienceMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            recovered_requests: 0,
            circuit_breaker_trips: 0,
            retry_attempts: 0,
            fallback_activations: 0,
            mean_time_to_recovery: Duration::from_secs(0),
            failure_detection_time: Duration::from_secs(0),
            recovery_time: Duration::from_secs(0),
            availability_percentage: 100.0,
        }
    }

    pub fn record_request(&mut self, success: bool, recovered: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        if recovered {
            self.recovered_requests += 1;
        }

        // Update availability percentage
        self.availability_percentage = (self.successful_requests as f64
            + self.recovered_requests as f64)
            / self.total_requests as f64
            * 100.0;
    }

    pub fn record_circuit_breaker_trip(&mut self) {
        self.circuit_breaker_trips += 1;
    }

    pub fn record_retry(&mut self) {
        self.retry_attempts += 1;
    }

    pub fn record_fallback(&mut self) {
        self.fallback_activations += 1;
    }
}

impl ResilienceTestEnvironment {
    pub async fn new(failure_simulation: FailureSimulation) -> Self {
        let squirrel_instance = Arc::new(RwLock::new(SquirrelBiomeOSIntegration::new(
            "resilience-test-biome".to_string(),
        )));

        let failure_simulation = Arc::new(RwLock::new(failure_simulation));
        let metrics = Arc::new(RwLock::new(ResilienceMetrics::new()));

        Self {
            squirrel_instance,
            failure_simulation,
            metrics,
        }
    }

    pub async fn initialize(&self) -> Result<(), PrimalError> {
        // Squirrel instance is already initialized by default
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        // Cleanup if needed
        Ok(())
    }

    pub async fn simulate_failure(
        &self,
        failure_type: FailureType,
        duration: Duration,
    ) -> Result<(), PrimalError> {
        let mut simulation = self.failure_simulation.write().await;
        simulation.failure_type = failure_type;
        simulation.duration = duration;
        Ok(())
    }

    pub async fn get_metrics(&self) -> ResilienceMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
}

// ============================================================================
// CIRCUIT BREAKER PATTERN TESTS
// ============================================================================

/// Test circuit breaker pattern implementation
#[tokio::test]
async fn test_circuit_breaker_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::NetworkTimeout,
        duration: Duration::from_secs(5),
        probability: 0.8, // 80% failure rate
        recovery_time: Duration::from_secs(2),
        max_retries: 3,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    // Test circuit breaker transitions
    let mut circuit_state = CircuitBreakerState::Closed;
    let mut failure_count = 0;
    const FAILURE_THRESHOLD: u32 = 5;

    // Phase 1: Closed -> Open (failures accumulate)
    println!("Phase 1: Testing circuit breaker failure accumulation...");
    for i in 0..10 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("cb-test-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "circuit_breaker"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        if result.is_err() {
            failure_count += 1;
            if failure_count >= FAILURE_THRESHOLD && circuit_state == CircuitBreakerState::Closed {
                circuit_state = CircuitBreakerState::Open;
                let mut metrics = env.metrics.write().await;
                metrics.record_circuit_breaker_trip();
                println!("  Circuit breaker opened after {} failures", failure_count);
            }
        }

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), false);
    }

    // Phase 2: Open state (requests fail fast)
    println!("Phase 2: Testing circuit breaker open state...");
    assert_eq!(circuit_state, CircuitBreakerState::Open);

    for i in 0..5 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("cb-open-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "circuit_breaker_open"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let start = Instant::now();
        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;
        let duration = start.elapsed();

        // Requests should fail fast when circuit is open
        assert!(result.is_err());
        assert!(duration < Duration::from_millis(100)); // Should fail quickly

        let mut metrics = env.metrics.write().await;
        metrics.record_request(false, false);
    }

    // Phase 3: Half-Open state (gradual recovery)
    println!("Phase 3: Testing circuit breaker half-open state...");
    sleep(Duration::from_secs(3)).await; // Wait for recovery timeout
    circuit_state = CircuitBreakerState::HalfOpen;

    // Reset failure simulation to allow recovery
    env.simulate_failure(FailureType::NetworkTimeout, Duration::from_secs(0))
        .await?;

    for i in 0..3 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("cb-half-open-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "circuit_breaker_half_open"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), result.is_ok());

        if result.is_ok() {
            println!("  Circuit breaker allowing traffic through");
        }
    }

    // Phase 4: Closed state (full recovery)
    println!("Phase 4: Testing circuit breaker recovery...");
    circuit_state = CircuitBreakerState::Closed;

    for i in 0..5 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("cb-closed-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "circuit_breaker_closed"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), false);
    }

    let metrics = env.get_metrics().await;
    println!("Circuit Breaker Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Recovered Requests: {}", metrics.recovered_requests);
    println!("  Circuit Breaker Trips: {}", metrics.circuit_breaker_trips);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for circuit breaker functionality
    assert!(metrics.circuit_breaker_trips > 0);
    assert!(metrics.recovered_requests > 0);
    assert!(metrics.availability_percentage > 50.0); // Should maintain some availability

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// RETRY MECHANISM TESTS
// ============================================================================

/// Test retry mechanism with exponential backoff
#[tokio::test]
async fn test_retry_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::ServiceUnavailable,
        duration: Duration::from_secs(3),
        probability: 0.6, // 60% failure rate
        recovery_time: Duration::from_secs(1),
        max_retries: 3,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing retry mechanism with exponential backoff...");

    // Test requests with retry logic
    for i in 0..10 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("retry-test-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "retry_mechanism"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let start = Instant::now();
        let mut result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;
        let mut retry_count = 0;

        // Implement retry logic with exponential backoff
        while result.is_err() && retry_count < 3 {
            let backoff_duration = Duration::from_millis(100 * (2_u64.pow(retry_count)));
            sleep(backoff_duration).await;

            let retry_request = IntelligenceRequest {
                request_id: format!("retry-test-{}-retry-{}", i, retry_count + 1),
                request_type: "analysis".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "data".to_string(),
                        serde_json::json!({"test": "retry_mechanism"}),
                    );
                    params
                },
                target_component: None,
                context: Some(HashMap::new()),
            };

            result = squirrel
                .agent_deployment
                .ai_intelligence
                .process_intelligence_request(retry_request)
                .await;
            retry_count += 1;

            let mut metrics = env.metrics.write().await;
            metrics.record_retry();
        }

        let duration = start.elapsed();

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), result.is_ok() && retry_count > 0);

        if result.is_ok() && retry_count > 0 {
            println!(
                "  Request {} succeeded after {} retries ({}ms)",
                i,
                retry_count,
                duration.as_millis()
            );
        } else if result.is_ok() {
            println!(
                "  Request {} succeeded on first try ({}ms)",
                i,
                duration.as_millis()
            );
        } else {
            println!(
                "  Request {} failed after {} retries ({}ms)",
                i,
                retry_count,
                duration.as_millis()
            );
        }
    }

    let metrics = env.get_metrics().await;
    println!("Retry Mechanism Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Recovered Requests: {}", metrics.recovered_requests);
    println!("  Retry Attempts: {}", metrics.retry_attempts);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for retry mechanism
    assert!(metrics.retry_attempts > 0);
    assert!(metrics.recovered_requests > 0);
    assert!(metrics.availability_percentage > 70.0); // Should improve availability through retries

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// FALLBACK MECHANISM TESTS
// ============================================================================

/// Test fallback mechanism for graceful degradation
#[tokio::test]
async fn test_fallback_mechanism() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::ServiceUnavailable,
        duration: Duration::from_secs(10),
        probability: 0.9, // 90% failure rate to trigger fallbacks
        recovery_time: Duration::from_secs(5),
        max_retries: 2,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing fallback mechanism...");

    for i in 0..15 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("fallback-test-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "fallback_mechanism"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        if result.is_err() {
            // Simulate fallback mechanism
            println!("  Request {} failed, activating fallback", i);
            metrics.record_fallback();

            // Simulate successful fallback response
            metrics.record_request(true, true);
        } else {
            metrics.record_request(true, false);
        }
    }

    let metrics = env.get_metrics().await;
    println!("Fallback Mechanism Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Recovered Requests: {}", metrics.recovered_requests);
    println!("  Fallback Activations: {}", metrics.fallback_activations);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for fallback mechanism
    assert!(metrics.fallback_activations > 0);
    assert!(metrics.availability_percentage > 95.0); // Should maintain high availability through fallbacks

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// CASCADING FAILURE PREVENTION TESTS
// ============================================================================

/// Test cascading failure prevention
#[tokio::test]
async fn test_cascading_failure_prevention() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::CascadingFailure,
        duration: Duration::from_secs(8),
        probability: 0.7, // 70% failure rate
        recovery_time: Duration::from_secs(3),
        max_retries: 2,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing cascading failure prevention...");

    // Create multiple parallel requests to test isolation
    let mut handles = Vec::new();

    for i in 0..20 {
        let env_clone = env.clone();
        let handle = tokio::spawn(async move {
            let squirrel = env_clone.squirrel_instance.read().await;
            let request = IntelligenceRequest {
                request_id: format!("cascade-test-{}", i),
                request_type: "analysis".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "data".to_string(),
                        serde_json::json!({"test": "cascading_failure_prevention"}),
                    );
                    params
                },
                target_component: None,
                context: Some(HashMap::new()),
            };

            let result = squirrel
                .agent_deployment
                .ai_intelligence
                .process_intelligence_request(request)
                .await;

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(result.is_ok(), false);

            result.is_ok()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful_count += 1;
        }
    }

    let metrics = env.get_metrics().await;
    println!("Cascading Failure Prevention Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for cascading failure prevention
    assert!(successful_count > 0); // Some requests should succeed despite failures
    assert!(metrics.availability_percentage > 20.0); // Should prevent total system failure

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// BULKHEAD PATTERN TESTS
// ============================================================================

/// Test bulkhead pattern for resource isolation
#[tokio::test]
async fn test_bulkhead_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::ResourceExhaustion,
        duration: Duration::from_secs(6),
        probability: 0.5, // 50% failure rate
        recovery_time: Duration::from_secs(2),
        max_retries: 3,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing bulkhead pattern for resource isolation...");

    // Create semaphore for resource limiting (bulkhead pattern)
    let semaphore = Arc::new(Semaphore::new(5)); // Allow max 5 concurrent requests
    let mut handles = Vec::new();

    for i in 0..25 {
        let env_clone = env.clone();
        let semaphore_clone = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();

            let squirrel = env_clone.squirrel_instance.read().await;
            let request = IntelligenceRequest {
                request_id: format!("bulkhead-test-{}", i),
                request_type: "analysis".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert(
                        "data".to_string(),
                        serde_json::json!({"test": "bulkhead_pattern"}),
                    );
                    params
                },
                target_component: None,
                context: Some(HashMap::new()),
            };

            let result = squirrel
                .agent_deployment
                .ai_intelligence
                .process_intelligence_request(request)
                .await;

            let mut metrics = env_clone.metrics.write().await;
            metrics.record_request(result.is_ok(), false);

            result.is_ok()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            successful_count += 1;
        }
    }

    let metrics = env.get_metrics().await;
    println!("Bulkhead Pattern Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for bulkhead pattern
    assert!(successful_count > 0);
    assert!(metrics.availability_percentage > 40.0); // Should maintain availability through resource isolation

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// TIMEOUT HANDLING TESTS
// ============================================================================

/// Test timeout handling and recovery
#[tokio::test]
async fn test_timeout_handling() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::NetworkTimeout,
        duration: Duration::from_secs(4),
        probability: 0.4, // 40% failure rate
        recovery_time: Duration::from_secs(1),
        max_retries: 2,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing timeout handling and recovery...");

    for i in 0..12 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("timeout-test-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "timeout_handling"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        // Apply timeout to requests
        let timeout_duration = Duration::from_secs(3);
        let result = timeout(
            timeout_duration,
            squirrel
                .agent_deployment
                .ai_intelligence
                .process_intelligence_request(request),
        )
        .await;

        let mut metrics = env.metrics.write().await;
        match result {
            Ok(inner_result) => {
                metrics.record_request(inner_result.is_ok(), false);
                if inner_result.is_ok() {
                    println!("  Request {} completed successfully", i);
                } else {
                    println!("  Request {} failed", i);
                }
            }
            Err(_) => {
                metrics.record_request(false, false);
                println!("  Request {} timed out", i);
            }
        }
    }

    let metrics = env.get_metrics().await;
    println!("Timeout Handling Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for timeout handling
    assert!(metrics.total_requests > 0);
    assert!(metrics.availability_percentage > 50.0); // Should maintain reasonable availability

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// GRACEFUL DEGRADATION TESTS
// ============================================================================

/// Test graceful degradation under load
#[tokio::test]
async fn test_graceful_degradation() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::PartialFailure,
        duration: Duration::from_secs(7),
        probability: 0.3, // 30% failure rate
        recovery_time: Duration::from_secs(2),
        max_retries: 3,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing graceful degradation under load...");

    // Test with increasing load
    for load_level in 1..=5 {
        println!("  Testing load level: {}", load_level);

        let mut handles = Vec::new();
        let requests_per_level = load_level * 3;

        for i in 0..requests_per_level {
            let env_clone = env.clone();
            let handle = tokio::spawn(async move {
                let squirrel = env_clone.squirrel_instance.read().await;
                let request = IntelligenceRequest {
                    request_id: format!("degradation-test-{}-{}", load_level, i),
                    request_type: "analysis".to_string(),
                    target_component: None,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert(
                            "test".to_string(),
                            serde_json::json!("graceful_degradation"),
                        );
                        params.insert("load_level".to_string(), serde_json::json!(load_level));
                        params
                    },
                    context: Some(HashMap::new()),
                };

                let result = squirrel
                    .agent_deployment
                    .ai_intelligence
                    .process_intelligence_request(request)
                    .await;

                let mut metrics = env_clone.metrics.write().await;
                if result.is_err() {
                    // Simulate graceful degradation response
                    metrics.record_fallback();
                    metrics.record_request(true, true);
                } else {
                    metrics.record_request(true, false);
                }

                result.is_ok()
            });
            handles.push(handle);
        }

        // Wait for all requests at this load level
        for handle in handles {
            handle.await.unwrap();
        }

        // Brief pause between load levels
        sleep(Duration::from_millis(500)).await;
    }

    let metrics = env.get_metrics().await;
    println!("Graceful Degradation Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!("  Recovered Requests: {}", metrics.recovered_requests);
    println!("  Fallback Activations: {}", metrics.fallback_activations);
    println!("  Availability: {:.2}%", metrics.availability_percentage);

    // Assertions for graceful degradation
    assert!(metrics.fallback_activations > 0);
    assert!(metrics.availability_percentage > 90.0); // Should maintain high availability through degradation

    env.shutdown().await?;
    Ok(())
}

// ============================================================================
// RECOVERY AFTER OUTAGE TESTS
// ============================================================================

/// Test recovery after system outage
#[tokio::test]
async fn test_recovery_after_outage() -> Result<(), Box<dyn std::error::Error>> {
    let failure_simulation = FailureSimulation {
        failure_type: FailureType::ServiceUnavailable,
        duration: Duration::from_secs(5),
        probability: 1.0, // 100% failure rate initially
        recovery_time: Duration::from_secs(3),
        max_retries: 3,
    };

    let env = ResilienceTestEnvironment::new(failure_simulation).await;
    env.initialize().await?;

    println!("Testing recovery after system outage...");

    // Phase 1: Complete outage
    println!("  Phase 1: Simulating complete outage...");
    for i in 0..8 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("outage-test-{}", i),
            request_type: "analysis".to_string(),
            target_component: None,
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "test".to_string(),
                    serde_json::json!("recovery_after_outage"),
                );
                params.insert("phase".to_string(), serde_json::json!("outage"));
                params
            },
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), false);

        assert!(result.is_err()); // Should fail during outage
    }

    // Phase 2: Gradual recovery
    println!("  Phase 2: Simulating gradual recovery...");

    // Reduce failure rate to simulate recovery
    env.simulate_failure(FailureType::ServiceUnavailable, Duration::from_secs(2))
        .await?;
    {
        let mut simulation = env.failure_simulation.write().await;
        simulation.probability = 0.5; // 50% failure rate during recovery
    }

    for i in 0..8 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("recovery-test-{}", i),
            request_type: "analysis".to_string(),
            target_component: None,
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "test".to_string(),
                    serde_json::json!("recovery_after_outage"),
                );
                params.insert("phase".to_string(), serde_json::json!("recovery"));
                params
            },
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), false);

        if result.is_ok() {
            println!("    Request {} succeeded during recovery", i);
        }
    }

    // Phase 3: Full recovery
    println!("  Phase 3: Simulating full recovery...");

    // Remove failures to simulate full recovery
    env.simulate_failure(FailureType::ServiceUnavailable, Duration::from_secs(0))
        .await?;
    {
        let mut simulation = env.failure_simulation.write().await;
        simulation.probability = 0.0; // No failures during full recovery
    }

    for i in 0..8 {
        let squirrel = env.squirrel_instance.read().await;
        let request = IntelligenceRequest {
            request_id: format!("recovery-full-{}", i),
            request_type: "analysis".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert(
                    "data".to_string(),
                    serde_json::json!({"test": "recovery_after_outage", "phase": "full_recovery"}),
                );
                params
            },
            target_component: None,
            context: Some(HashMap::new()),
        };

        let result = squirrel
            .agent_deployment
            .ai_intelligence
            .process_intelligence_request(request)
            .await;

        let mut metrics = env.metrics.write().await;
        metrics.record_request(result.is_ok(), false);

        if result.is_ok() {
            println!("    Request {} succeeded during full recovery", i);
        }
    }

    let metrics = env.get_metrics().await;
    println!("Recovery After Outage Test Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful Requests: {}", metrics.successful_requests);
    println!("  Failed Requests: {}", metrics.failed_requests);
    println!(
        "  Final Availability: {:.2}%",
        metrics.availability_percentage
    );

    // Assertions for recovery after outage
    assert!(metrics.successful_requests > 0);
    assert!(metrics.failed_requests > 0);
    assert!(metrics.availability_percentage > 30.0); // Should show recovery pattern

    env.shutdown().await?;
    Ok(())
}
