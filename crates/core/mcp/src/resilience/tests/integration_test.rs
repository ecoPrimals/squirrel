// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use std::sync::{Arc, Mutex, atomic::{AtomicU32, Ordering}};
use std::time::{Duration, Instant};
use std::error::Error as StdError;
use std::collections::HashMap;

use tokio::test;
use serde::{Serialize, Deserialize};

use crate::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use crate::resilience::retry::{RetryMechanism, RetryConfig, BackoffStrategy};
use crate::resilience::recovery::{RecoveryStrategy, RecoveryConfig, FailureInfo, FailureSeverity};
use crate::resilience::state_sync::{StateSynchronizer, StateSyncConfig, StateType};
use crate::resilience::health::{HealthMonitor, HealthCheck, HealthCheckConfig, HealthStatus, HealthCheckResult};
use crate::resilience::{ResilienceError, with_resilience, with_recovery, with_health_monitoring, with_complete_resilience};
use async_trait::async_trait;

// Test error for simulating failures
#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Test error: {}", self.0)
    }
}

impl StdError for TestError {}

// Test component that can be configured to fail
struct TestComponent {
    id: String,
    fail_count: Arc<AtomicU32>,
    success_after: u32,
    call_count: Arc<AtomicU32>,
    state: Arc<Mutex<String>>,
}

impl TestComponent {
    fn new(id: &str, fail_count: u32) -> Self {
        Self {
            id: id.to_string(),
            fail_count: Arc::new(AtomicU32::new(fail_count)),
            success_after: fail_count,
            call_count: Arc::new(AtomicU32::new(0)),
            state: Arc::new(Mutex::new("initial".to_string())),
        }
    }
    
    fn operate(&self) -> Result<String, Box<dyn StdError + Send + Sync>> {
        let count = self.call_count.fetch_add(1, Ordering::Relaxed);
        
        if count < self.fail_count.load(Ordering::Relaxed) {
            Err(Box::new(TestError(format!("Operation {} failed (attempt {})", self.id, count + 1))))
        } else {
            let mut state = self.state.lock().unwrap();
            *state = format!("success-{}", count + 1);
            Ok(state.clone())
        }
    }
    
    fn get_state(&self) -> String {
        self.state.lock().unwrap().clone()
    }
    
    fn set_state(&self, new_state: &str) {
        let mut state = self.state.lock().unwrap();
        *state = new_state.to_string();
    }
    
    fn reset_counters(&self) {
        self.call_count.store(0, Ordering::Relaxed);
    }
    
    fn set_fail_count(&self, count: u32) {
        self.fail_count.store(count, Ordering::Relaxed);
    }
}

// Health check implementation for TestComponent
struct TestHealthCheck {
    component: Arc<TestComponent>,
    config: HealthCheckConfig,
}

impl TestHealthCheck {
    fn new(component: Arc<TestComponent>) -> Self {
        Self {
            component,
            config: HealthCheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for TestHealthCheck {
    fn id(&self) -> &str {
        &self.component.id
    }
    
    async fn check(&self) -> HealthCheckResult {
        // Health check logic: if the component state starts with "success", it's healthy
        let state = self.component.get_state();
        
        if state.starts_with("success") {
            HealthCheckResult::new(
                self.component.id.clone(),
                HealthStatus::Healthy,
                format!("Component {} is healthy with state: {}", self.component.id, state)
            )
            .with_metric("calls", self.component.call_count.load(Ordering::Relaxed) as f64)
        } else if state.starts_with("degraded") {
            HealthCheckResult::new(
                self.component.id.clone(),
                HealthStatus::Degraded,
                format!("Component {} is degraded with state: {}", self.component.id, state)
            )
            .with_metric("calls", self.component.call_count.load(Ordering::Relaxed) as f64)
        } else {
            HealthCheckResult::new(
                self.component.id.clone(),
                HealthStatus::Unhealthy,
                format!("Component {} is unhealthy with state: {}", self.component.id, state)
            )
            .with_metric("calls", self.component.call_count.load(Ordering::Relaxed) as f64)
        }
    }
    
    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut HealthCheckConfig {
        &mut self.config
    }
}

// Serializable state for testing state synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestState {
    component_id: String,
    value: String,
    timestamp: u64,
}

// Test individual components working together
#[tokio::test]
async fn test_circuit_breaker_with_retry() {
    // Set up components
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test-cb".to_string(),
        failure_threshold: 2,
        recovery_timeout_ms: 100,
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Linear,
    });
    
    let component = TestComponent::new("test-component-1", 2);
    
    // Should succeed because retry will handle the failures
    let result = with_resilience(
        &mut circuit_breaker,
        retry,
        || component.operate(),
    ).await;
    
    assert!(result.is_ok());
    assert_eq!(component.get_state(), "success-3");
    assert_eq!(circuit_breaker.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_opens_with_too_many_failures() {
    // Set up components
    let mut circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
        name: "test-cb".to_string(),
        failure_threshold: 2,
        recovery_timeout_ms: 1000, // Long timeout so it stays open
        half_open_success_threshold: 1,
        half_open_allowed_calls: 1,
        fallback: None,
    });
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2, // Not enough retries to overcome all failures
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Linear,
    });
    
    let component = TestComponent::new("test-component-2", 4); // Will always fail
    
    // First operation should fail but not open circuit
    let result1 = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || component.operate(),
    ).await;
    
    assert!(result1.is_err());
    
    // Second operation should fail and open circuit
    let result2 = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || component.operate(),
    ).await;
    
    assert!(result2.is_err());
    assert_eq!(circuit_breaker.state(), CircuitState::Open);
    
    // Third operation should be rejected by circuit breaker
    let result3 = with_resilience(
        &mut circuit_breaker,
        retry.clone(),
        || component.operate(),
    ).await;
    
    assert!(result3.is_err());
    match result3 {
        Err(ResilienceError::CircuitOpen(_)) => (),
        _ => panic!("Expected CircuitOpen error"),
    }
}

#[tokio::test]
async fn test_recovery_strategy() {
    // Set up components
    let mut recovery = RecoveryStrategy::new(RecoveryConfig {
        max_minor_attempts: 2,
        max_moderate_attempts: 1,
        max_severe_attempts: 1,
        recover_critical: false,
    });
    
    let component = TestComponent::new("test-component-3", 1);
    component.set_state("failed");
    
    // Create failure info
    let failure_info = FailureInfo {
        message: "Test failure".to_string(),
        severity: FailureSeverity::Minor,
        context: "test-context".to_string(),
        recovery_attempts: 0,
    };
    
    // Recovery should work
    let result = with_recovery(
        &mut recovery,
        failure_info,
        || component.operate(),
        || {
            // Recovery action
            component.set_fail_count(0); // Make it succeed next time
            component.reset_counters();
            Err(Box::new(TestError("Recovery action still fails".to_string())) as Box<dyn StdError + Send + Sync>)
        }
    );
    
    assert!(result.is_err()); // The recovery action itself fails
    
    // But the component should be set up to succeed next time
    let direct_result = component.operate();
    assert!(direct_result.is_ok());
}

#[tokio::test]
async fn test_health_monitoring() {
    // Set up components
    let component = Arc::new(TestComponent::new("test-component-4", 0));
    let health_check = TestHealthCheck::new(component.clone());
    
    let mut health_monitor = HealthMonitor::default();
    health_monitor.register(health_check);
    
    // Initial health should be unhealthy (initial state)
    let result = health_monitor.check_component("test-component-4").await.unwrap();
    assert_eq!(result.status, HealthStatus::Unhealthy);
    
    // Update state to healthy
    component.operate().unwrap();
    
    // Now health should be healthy
    let result = health_monitor.check_component("test-component-4").await.unwrap();
    assert_eq!(result.status, HealthStatus::Healthy);
    
    // Test operation with health monitoring
    let operation_result = with_health_monitoring(
        &health_monitor,
        "test-component-4",
        || component.operate(),
    ).await;
    
    assert!(operation_result.is_ok());
    
    // Set to unhealthy and try again
    component.set_state("broken");
    
    // Update health status
    health_monitor.check_component("test-component-4").await.unwrap();
    
    // Operation should be prevented
    let operation_result = with_health_monitoring(
        &health_monitor,
        "test-component-4",
        || component.operate(),
    ).await;
    
    assert!(operation_result.is_err());
}

#[tokio::test]
async fn test_state_synchronization() {
    // Set up components
    let state_sync = StateSynchronizer::new(StateSyncConfig {
        sync_timeout: Duration::from_secs(1),
        max_state_size: 1024,
        validate_state: true,
    });
    
    let component = TestComponent::new("test-component-5", 0);
    component.set_state("state-to-sync");
    
    // Create test state
    let test_state = TestState {
        component_id: component.id.clone(),
        value: component.get_state(),
        timestamp: 12345,
    };
    
    // Sync the state
    let result = state_sync.sync_state(
        StateType::Runtime,
        "test-state",
        "test-target",
        test_state.clone(),
    ).await;
    
    assert!(result.is_ok());
    
    // Verify metrics
    let metrics = state_sync.get_metrics();
    assert_eq!(*metrics.successful_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
}

#[tokio::test]
async fn test_complete_resilience_pipeline() {
    // Set up all components
    let mut circuit_breaker = CircuitBreaker::default();
    
    let retry = RetryMechanism::new(RetryConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(50),
        use_jitter: false,
        backoff_strategy: BackoffStrategy::Exponential,
    });
    
    let mut recovery = RecoveryStrategy::default();
    
    let component = Arc::new(TestComponent::new("test-component-6", 3));
    let health_check = TestHealthCheck::new(component.clone());
    
    let mut health_monitor = HealthMonitor::default();
    health_monitor.register(health_check);
    
    // Set up failure info
    let failure_info = FailureInfo {
        message: "Operation failed".to_string(),
        severity: FailureSeverity::Minor,
        context: "test-context".to_string(),
        recovery_attempts: 0,
    };
    
    // Run the operation with full resilience
    let result = with_complete_resilience(
        &mut circuit_breaker,
        retry,
        &mut recovery,
        &health_monitor,
        "test-component-6",
        failure_info,
        || component.operate(),
        || {
            // Recovery action
            component.set_fail_count(0);
            component.reset_counters();
            component.operate()
        }
    ).await;
    
    // Should succeed via recovery
    assert!(result.is_ok());
    
    // Component should have new state
    assert_eq!(component.get_state(), "success-1");
}

// Test simulating real-world cascading failures and recovery
#[tokio::test]
async fn test_complex_failure_scenario() {
    // Components that depend on each other
    let database = Arc::new(TestComponent::new("database", 2));
    let api_service = Arc::new(TestComponent::new("api-service", 0));
    let user_service = Arc::new(TestComponent::new("user-service", 0));
    
    // Health checks
    let db_health = TestHealthCheck::new(database.clone());
    let api_health = TestHealthCheck::new(api_service.clone());
    let user_health = TestHealthCheck::new(user_service.clone());
    
    // Circuit breakers
    let mut db_circuit = CircuitBreaker::default();
    let mut api_circuit = CircuitBreaker::default();
    let mut user_circuit = CircuitBreaker::default();
    
    // Retry mechanisms
    let db_retry = RetryMechanism::default();
    let api_retry = RetryMechanism::default();
    let user_retry = RetryMechanism::default();
    
    // Recovery strategies
    let mut db_recovery = RecoveryStrategy::default();
    let mut api_recovery = RecoveryStrategy::default();
    let mut user_recovery = RecoveryStrategy::default();
    
    // Health monitor
    let mut health_monitor = HealthMonitor::default();
    health_monitor.register(db_health);
    health_monitor.register(api_health);
    health_monitor.register(user_health);
    
    // State synchronizer for recovery
    let state_sync = StateSynchronizer::default();
    
    // Initialize states
    api_service.set_state("success-init");
    user_service.set_state("success-init");
    
    // User service depends on API service which depends on database
    // Database will fail initially, causing a cascade
    
    // 1. Try to operate the database - should fail but recover
    let db_result = with_resilience(
        &mut db_circuit,
        db_retry.clone(),
        || database.operate(),
    ).await;
    
    // Should still fail because retry isn't enough
    assert!(db_result.is_err());
    
    // 2. Database failure affects API service
    api_service.set_state("degraded-db-dependency");
    
    // 3. Update health status
    health_monitor.check_component("database").await.unwrap();
    health_monitor.check_component("api-service").await.unwrap();
    health_monitor.check_component("user-service").await.unwrap();
    
    // 4. Attempt recovery on database
    let db_recovery_result = with_recovery(
        &mut db_recovery,
        FailureInfo {
            message: "Database failure".to_string(),
            severity: FailureSeverity::Moderate,
            context: "database".to_string(),
            recovery_attempts: 0,
        },
        || Err(Box::new(TestError("DB still failing".to_string())) as Box<dyn StdError + Send + Sync>),
        || {
            // Recovery action
            database.set_fail_count(0);
            database.reset_counters();
            Ok(())
        }
    );
    
    assert!(db_recovery_result.is_ok());
    
    // 5. Database should work now
    let db_result = database.operate();
    assert!(db_result.is_ok());
    
    // 6. Update API service state based on healthy database
    api_service.set_state("success-restored");
    
    // 7. Update health status
    health_monitor.check_all().await;
    
    // 8. Now everything should be healthy
    assert_eq!(health_monitor.component_status("database"), HealthStatus::Healthy);
    assert_eq!(health_monitor.component_status("api-service"), HealthStatus::Healthy);
    
    // 9. Operation with health check should succeed
    let api_result = with_health_monitoring(
        &health_monitor,
        "api-service",
        || api_service.operate(),
    ).await;
    
    assert!(api_result.is_ok());
    
    // Metrics should show the recovery
    let metrics = health_monitor.get_metrics();
    assert!(metrics.total_checks > 0);
} 