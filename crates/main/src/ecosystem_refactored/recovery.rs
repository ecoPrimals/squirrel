//! Error recovery and resilience for ecosystem services
//!
//! This module handles failure detection, recovery strategies, and
//! circuit breaker patterns for resilient service operation.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

use crate::error::PrimalError;
use crate::monitoring::MetricsCollector;

/// Recovery and resilience coordinator
///
/// Implements circuit breaker patterns, retry logic, and failover
/// strategies to ensure ecosystem resilience.
pub struct RecoveryManager {
    state: Arc<RwLock<RecoveryState>>,
    metrics_collector: Arc<MetricsCollector>,
}

/// Internal recovery state
struct RecoveryState {
    circuit_breakers: std::collections::HashMap<String, CircuitBreakerState>,
    failure_counts: std::collections::HashMap<String, u64>,
}

/// Circuit breaker state for a service
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    status: CircuitStatus,
    failure_count: u64,
    last_failure: Option<chrono::DateTime<chrono::Utc>>,
    last_success: Option<chrono::DateTime<chrono::Utc>>,
}

/// Circuit breaker status
#[derive(Debug, Clone, PartialEq)]
enum CircuitStatus {
    Closed,  // Normal operation
    Open,    // Failures detected, blocking requests
    HalfOpen, // Testing if service recovered
}

impl RecoveryManager {
    /// Create new recovery manager
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            state: Arc::new(RwLock::new(RecoveryState {
                circuit_breakers: std::collections::HashMap::new(),
                failure_counts: std::collections::HashMap::new(),
            })),
            metrics_collector,
        }
    }

    /// Handle service failure
    ///
    /// Records failure and initiates recovery if needed.
    pub async fn handle_failure(
        &self,
        service_id: &str,
        error: &PrimalError,
    ) -> Result<(), PrimalError> {
        tracing::warn!("Service failure detected: {} - {}", service_id, error);

        let mut state = self.state.write().await;

        // Increment failure count
        let failure_count = state
            .failure_counts
            .entry(service_id.to_string())
            .or_insert(0);
        *failure_count += 1;

        // Update circuit breaker
        let breaker = state
            .circuit_breakers
            .entry(service_id.to_string())
            .or_insert_with(|| CircuitBreakerState {
                status: CircuitStatus::Closed,
                failure_count: 0,
                last_failure: None,
                last_success: None,
            });

        breaker.failure_count += 1;
        breaker.last_failure = Some(chrono::Utc::now());

        // Open circuit if too many failures
        if breaker.failure_count >= 5 {
            breaker.status = CircuitStatus::Open;
            tracing::error!("Circuit breaker opened for service: {}", service_id);
        }

        // Record metrics
        self.metrics_collector
            .record_service_failure(service_id, &error.to_string());

        Ok(())
    }

    /// Attempt recovery for a service
    ///
    /// Tries to recover a failed service using various strategies.
    pub async fn attempt_recovery(&self, service_id: &str) -> Result<(), PrimalError> {
        tracing::info!("Attempting recovery for service: {}", service_id);

        let state = self.state.read().await;

        // Check circuit breaker state
        if let Some(breaker) = state.circuit_breakers.get(service_id) {
            match breaker.status {
                CircuitStatus::Open => {
                    tracing::debug!("Circuit breaker is open, attempting half-open transition");
                    drop(state);
                    self.transition_to_half_open(service_id).await?;
                }
                CircuitStatus::HalfOpen => {
                    tracing::debug!("Circuit breaker is half-open, testing service");
                    // TODO: Implement actual health check
                }
                CircuitStatus::Closed => {
                    tracing::debug!("Circuit breaker is closed, service healthy");
                }
            }
        }

        Ok(())
    }

    /// Transition circuit breaker to half-open state
    async fn transition_to_half_open(&self, service_id: &str) -> Result<(), PrimalError> {
        let mut state = self.state.write().await;

        if let Some(breaker) = state.circuit_breakers.get_mut(service_id) {
            breaker.status = CircuitStatus::HalfOpen;
            tracing::info!("Transitioned to half-open: {}", service_id);
        }

        Ok(())
    }

    /// Record successful service call
    ///
    /// Updates circuit breaker state on successful operations.
    pub async fn record_success(&self, service_id: &str) -> Result<(), PrimalError> {
        let mut state = self.state.write().await;

        if let Some(breaker) = state.circuit_breakers.get_mut(service_id) {
            breaker.last_success = Some(chrono::Utc::now());

            // Close circuit if in half-open and success
            if breaker.status == CircuitStatus::HalfOpen {
                breaker.status = CircuitStatus::Closed;
                breaker.failure_count = 0;
                tracing::info!("Circuit breaker closed for service: {}", service_id);
            }
        }

        // Reset failure count
        state.failure_counts.insert(service_id.to_string(), 0);

        Ok(())
    }

    /// Check if service can accept requests
    ///
    /// Returns true if circuit breaker allows requests.
    pub async fn can_call_service(&self, service_id: &str) -> bool {
        let state = self.state.read().await;

        if let Some(breaker) = state.circuit_breakers.get(service_id) {
            !matches!(breaker.status, CircuitStatus::Open)
        } else {
            true // No breaker = allow calls
        }
    }

    /// Get failure count for a service
    pub async fn get_failure_count(&self, service_id: &str) -> u64 {
        let state = self.state.read().await;
        state.failure_counts.get(service_id).copied().unwrap_or(0)
    }

    /// Reset circuit breaker for a service
    ///
    /// Manually resets the circuit breaker, useful for admin operations.
    pub async fn reset_circuit_breaker(&self, service_id: &str) -> Result<(), PrimalError> {
        let mut state = self.state.write().await;

        if let Some(breaker) = state.circuit_breakers.get_mut(service_id) {
            breaker.status = CircuitStatus::Closed;
            breaker.failure_count = 0;
            tracing::info!("Circuit breaker reset for service: {}", service_id);
        }

        state.failure_counts.insert(service_id.to_string(), 0);

        Ok(())
    }
}

