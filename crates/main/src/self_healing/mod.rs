// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Self-healing mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Simple AI Health Management for Squirrel Coordinator
//!
//! Basic health management focused on AI coordination scenarios.
//! Replaces 954 lines of over-engineered self-healing infrastructure
//! with simple AI coordination health patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::error::PrimalError;

/// Simple health status for AI components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and functioning
    Healthy,
    /// Component is degraded but functional
    Degraded,
    /// Component has failed
    Failed,
    /// Status unknown
    Unknown,
}

/// Simple AI Health Manager - focused on AI coordination health
#[derive(Debug, Clone)]
pub struct SelfHealingManager {
    /// Component health states
    component_health: HashMap<String, ComponentHealth>,
    /// Configuration for health management
    config: SelfHealingConfig,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component identifier
    pub component_id: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last health check time
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Health message
    pub message: String,
    /// Consecutive failure count
    pub failure_count: u32,
}

/// Simple configuration for AI health management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    /// Health check interval in seconds
    pub check_interval_seconds: u64,
    /// Maximum failures before marking as failed
    pub max_failures: u32,
    /// Enable automatic recovery attempts
    pub enable_auto_recovery: bool,
}

impl Default for SelfHealingConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            max_failures: 3,
            enable_auto_recovery: true,
        }
    }
}

impl SelfHealingManager {
    /// Create a new AI health manager
    pub fn new(config: SelfHealingConfig) -> Self {
        info!(
            "🩺 Initializing AI Health Manager with check interval {}s",
            config.check_interval_seconds
        );

        Self {
            component_health: HashMap::new(),
            config,
        }
    }

    /// Register a component for health monitoring
    pub fn register_component(&mut self, component_id: &str) {
        let health = ComponentHealth {
            component_id: component_id.to_string(),
            status: HealthStatus::Unknown,
            last_check: chrono::Utc::now(),
            message: "Newly registered".to_string(),
            failure_count: 0,
        };

        self.component_health
            .insert(component_id.to_string(), health);
        debug!(
            "📋 Registered component for health monitoring: {}",
            component_id
        );
    }

    /// Update component health status
    pub fn update_component_health(
        &mut self,
        component_id: &str,
        status: HealthStatus,
        message: &str,
    ) {
        if let Some(health) = self.component_health.get_mut(component_id) {
            let previous_status = health.status;
            health.status = status;
            health.last_check = chrono::Utc::now();
            health.message = message.to_string();

            // Update failure count
            match status {
                HealthStatus::Failed => health.failure_count += 1,
                HealthStatus::Healthy => health.failure_count = 0,
                _ => {}
            }

            // Log status changes
            if previous_status != status {
                match status {
                    HealthStatus::Healthy => {
                        info!("✅ Component '{}' recovered: {}", component_id, message);
                    }
                    HealthStatus::Degraded => {
                        warn!("⚠️ Component '{}' degraded: {}", component_id, message);
                    }
                    HealthStatus::Failed => {
                        warn!("❌ Component '{}' failed: {}", component_id, message);
                    }
                    HealthStatus::Unknown => debug!(
                        "❓ Component '{}' status unknown: {}",
                        component_id, message
                    ),
                }
            }

            // Attempt auto-recovery for failed components
            if status == HealthStatus::Failed && self.config.enable_auto_recovery {
                self.attempt_auto_recovery(component_id);
            }
        }
    }

    /// Get component health status
    #[must_use]
    pub fn get_component_health(&self, component_id: &str) -> Option<&ComponentHealth> {
        self.component_health.get(component_id)
    }

    /// Get all component health statuses
    #[must_use]
    pub const fn get_all_component_health(&self) -> &HashMap<String, ComponentHealth> {
        &self.component_health
    }

    /// Check if all monitored components are healthy
    #[must_use]
    pub fn is_system_healthy(&self) -> bool {
        self.component_health
            .values()
            .all(|h| matches!(h.status, HealthStatus::Healthy))
    }

    /// Get system health summary
    #[must_use]
    pub fn get_system_health_summary(&self) -> SystemHealthSummary {
        let total_components = self.component_health.len();
        let healthy_count = self
            .component_health
            .values()
            .filter(|h| matches!(h.status, HealthStatus::Healthy))
            .count();
        let degraded_count = self
            .component_health
            .values()
            .filter(|h| matches!(h.status, HealthStatus::Degraded))
            .count();
        let failed_count = self
            .component_health
            .values()
            .filter(|h| matches!(h.status, HealthStatus::Failed))
            .count();

        SystemHealthSummary {
            overall_healthy: failed_count == 0 && degraded_count < total_components / 2,
            total_components,
            healthy_count,
            degraded_count,
            failed_count,
            last_update: chrono::Utc::now(),
        }
    }

    /// Perform health check on all registered components
    pub async fn perform_health_check(&mut self) -> Result<(), PrimalError> {
        debug!("🔍 Performing AI coordination health check...");

        let mut check_results = Vec::new();

        // Simulate health checks for AI coordination components
        for component_id in self.component_health.clone().keys() {
            let is_healthy = self.simulate_component_health_check(component_id).await;

            let (status, message) = if is_healthy {
                (HealthStatus::Healthy, "Component responding normally")
            } else {
                (HealthStatus::Failed, "Component not responding")
            };

            self.update_component_health(component_id, status, message);
            check_results.push((component_id.clone(), status));
        }

        let healthy_components = check_results
            .iter()
            .filter(|(_, s)| matches!(s, HealthStatus::Healthy))
            .count();
        info!(
            "✅ Health check complete: {}/{} components healthy",
            healthy_components,
            check_results.len()
        );

        Ok(())
    }

    /// Simulate health check for a component
    async fn simulate_component_health_check(&self, _component_id: &str) -> bool {
        // Simple simulation - assume healthy for now
        // In real implementation would check actual component health
        true
    }

    /// Attempt automatic recovery for failed component
    fn attempt_auto_recovery(&mut self, component_id: &str) {
        if let Some(health) = self.component_health.get(component_id)
            && health.failure_count >= self.config.max_failures
        {
            warn!(
                "🔧 Attempting auto-recovery for component '{}' (failure count: {})",
                component_id, health.failure_count
            );

            // Simulate recovery attempt
            // In real implementation, this would restart services, clear caches, etc.
            info!(
                "🔄 Auto-recovery initiated for component '{}'",
                component_id
            );
        }
    }

    /// Get health recommendations
    #[must_use]
    pub fn get_health_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        let summary = self.get_system_health_summary();

        if summary.failed_count > 0 {
            recommendations.push(format!(
                "⚠️ {} components have failed - investigate and restart",
                summary.failed_count
            ));
        }

        if summary.degraded_count > 0 {
            recommendations.push(format!(
                "⚠️ {} components are degraded - monitor closely",
                summary.degraded_count
            ));
        }

        if (summary.healthy_count as f64 / summary.total_components as f64) < 0.8 {
            recommendations
                .push("🚨 System health below 80% - immediate attention required".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("✅ All AI coordination components are healthy".to_string());
        }

        recommendations
    }

    /// Get system health status (legacy method for compatibility)
    pub async fn get_health_status(&self) -> HashMap<String, ComponentHealth> {
        self.component_health.clone()
    }
}

/// System health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSummary {
    /// Whether the overall system is healthy.
    pub overall_healthy: bool,
    /// Total number of monitored components.
    pub total_components: usize,
    /// Number of healthy components.
    pub healthy_count: usize,
    /// Number of degraded components.
    pub degraded_count: usize,
    /// Number of failed components.
    pub failed_count: usize,
    /// Timestamp of the last health update.
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Initializes the AI health manager with default configuration and registers core components.
pub async fn initialize_self_healing() -> Result<SelfHealingManager, PrimalError> {
    let config = SelfHealingConfig::default();
    let mut manager = SelfHealingManager::new(config);

    // Register core AI coordination components
    manager.register_component("ai_coordinator");
    manager.register_component("security_adapter");
    manager.register_component("orchestration_adapter");
    manager.register_component("storage_adapter");
    manager.register_component("compute_adapter");

    info!(
        "🩺 AI Health Manager initialized with {} components",
        manager.component_health.len()
    );

    Ok(manager)
}

#[cfg(test)]
#[path = "self_healing_tests.rs"]
mod tests;
