//! Health monitoring for ecosystem services
//!
//! This module handles health checking, status tracking, and health-based
//! decision making for services in the ecosystem.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

use crate::error::PrimalError;
use crate::monitoring::MetricsCollector;

use super::types::{ComponentHealth, HealthStatus};

/// Health monitoring coordinator
///
/// Tracks the health of all ecosystem services and components,
/// providing aggregated health status and alerting on issues.
pub struct HealthMonitor {
    health_state: Arc<RwLock<HealthState>>,
    metrics_collector: Arc<MetricsCollector>,
}

/// Internal health state
struct HealthState {
    component_health: HashMap<String, ComponentHealth>,
    last_full_check: DateTime<Utc>,
    error_count: u64,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(metrics_collector: Arc<MetricsCollector>) -> Self {
        Self {
            health_state: Arc::new(RwLock::new(HealthState {
                component_health: HashMap::new(),
                last_full_check: Utc::now(),
                error_count: 0,
            })),
            metrics_collector,
        }
    }

    /// Get current health status
    ///
    /// Returns aggregated health information for all tracked components.
    pub async fn get_health_status(&self) -> HealthStatus {
        let state = self.health_state.read().await;

        // Calculate overall health score
        let health_score = if state.component_health.is_empty() {
            0.0
        } else {
            let healthy_count = state
                .component_health
                .values()
                .filter(|c| c.status == "healthy")
                .count();
            healthy_count as f64 / state.component_health.len() as f64
        };

        // Collect health errors
        let health_errors: Vec<String> = state
            .component_health
            .values()
            .filter_map(|c| c.error.clone())
            .collect();

        HealthStatus {
            health_score,
            component_statuses: state.component_health.clone(),
            last_check: state.last_full_check,
            health_errors,
        }
    }

    /// Update health for a specific component
    ///
    /// This allows individual components to report their health status.
    pub async fn update_component_health(
        &self,
        component_id: &str,
        status: String,
        error: Option<String>,
    ) -> Result<(), PrimalError> {
        let mut state = self.health_state.write().await;

        let component_health = ComponentHealth {
            status: status.clone(),
            last_check: Utc::now(),
            error: error.clone(),
            metadata: HashMap::new(),
        };

        state
            .component_health
            .insert(component_id.to_string(), component_health);

        // Track errors
        if error.is_some() {
            state.error_count += 1;
            tracing::warn!(
                "Component {} unhealthy: {}",
                component_id,
                error.unwrap_or_default()
            );
        }

        // Record metrics
        self.metrics_collector
            .record_component_health(component_id, &status);

        Ok(())
    }

    /// Check health of a specific service (external)
    ///
    /// This performs an active health check against a service endpoint.
    pub async fn check_service_health(&self, service_id: &str) -> Result<String, PrimalError> {
        tracing::debug!("Checking health for service: {}", service_id);

        // TODO: Implement actual health check HTTP request
        // For now, return healthy status
        Ok("healthy".to_string())
    }

    /// Perform full health check of all components
    ///
    /// This initiates health checks for all registered components
    /// and updates the aggregated health status.
    pub async fn perform_full_health_check(&self) -> Result<HealthStatus, PrimalError> {
        tracing::debug!("Performing full health check");

        let mut state = self.health_state.write().await;
        state.last_full_check = Utc::now();

        // TODO: Trigger health checks for all components
        // For now, just return current status

        drop(state); // Release lock before calling get_health_status
        Ok(self.get_health_status().await)
    }

    /// Get health score (0.0 to 1.0)
    pub async fn get_health_score(&self) -> f64 {
        let status = self.get_health_status().await;
        status.health_score
    }

    /// Check if ecosystem is healthy
    ///
    /// Returns true if health score is above threshold (0.8)
    pub async fn is_healthy(&self) -> bool {
        self.get_health_score().await >= 0.8
    }
}

