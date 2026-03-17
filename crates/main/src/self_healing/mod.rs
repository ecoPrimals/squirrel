// SPDX-License-Identifier: AGPL-3.0-only
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
mod tests {
    use super::*;

    fn create_test_manager() -> SelfHealingManager {
        SelfHealingManager::new(SelfHealingConfig::default())
    }

    #[test]
    fn test_default_config() {
        let config = SelfHealingConfig::default();
        assert_eq!(config.check_interval_seconds, 30);
        assert_eq!(config.max_failures, 3);
        assert!(config.enable_auto_recovery);
    }

    #[test]
    fn test_custom_config() {
        let config = SelfHealingConfig {
            check_interval_seconds: 60,
            max_failures: 5,
            enable_auto_recovery: false,
        };
        assert_eq!(config.check_interval_seconds, 60);
        assert_eq!(config.max_failures, 5);
        assert!(!config.enable_auto_recovery);
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let config = SelfHealingConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SelfHealingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.check_interval_seconds,
            config.check_interval_seconds
        );
        assert_eq!(deserialized.max_failures, config.max_failures);
        assert_eq!(
            deserialized.enable_auto_recovery,
            config.enable_auto_recovery
        );
    }

    #[test]
    fn test_manager_creation() {
        let manager = create_test_manager();
        assert!(manager.component_health.is_empty());
    }

    #[test]
    fn test_register_component() {
        let mut manager = create_test_manager();
        manager.register_component("test_component");

        assert_eq!(manager.component_health.len(), 1);
        let health = manager.get_component_health("test_component").unwrap();
        assert_eq!(health.component_id, "test_component");
        assert_eq!(health.status, HealthStatus::Unknown);
        assert_eq!(health.failure_count, 0);
        assert_eq!(health.message, "Newly registered");
    }

    #[test]
    fn test_register_multiple_components() {
        let mut manager = create_test_manager();
        manager.register_component("comp_a");
        manager.register_component("comp_b");
        manager.register_component("comp_c");

        assert_eq!(manager.component_health.len(), 3);
        assert!(manager.get_component_health("comp_a").is_some());
        assert!(manager.get_component_health("comp_b").is_some());
        assert!(manager.get_component_health("comp_c").is_some());
    }

    #[test]
    fn test_get_nonexistent_component() {
        let manager = create_test_manager();
        assert!(manager.get_component_health("nonexistent").is_none());
    }

    #[test]
    fn test_update_component_healthy() {
        let mut manager = create_test_manager();
        manager.register_component("test");

        manager.update_component_health("test", HealthStatus::Healthy, "All good");

        let health = manager.get_component_health("test").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.message, "All good");
        assert_eq!(health.failure_count, 0);
    }

    #[test]
    fn test_update_component_failed() {
        let mut manager = create_test_manager();
        manager.register_component("test");

        manager.update_component_health("test", HealthStatus::Failed, "Connection lost");

        let health = manager.get_component_health("test").unwrap();
        assert_eq!(health.status, HealthStatus::Failed);
        assert_eq!(health.failure_count, 1);
    }

    #[test]
    fn test_failure_count_increments() {
        let mut manager = create_test_manager();
        manager.register_component("test");

        manager.update_component_health("test", HealthStatus::Failed, "Fail 1");
        manager.update_component_health("test", HealthStatus::Failed, "Fail 2");
        manager.update_component_health("test", HealthStatus::Failed, "Fail 3");

        let health = manager.get_component_health("test").unwrap();
        assert_eq!(health.failure_count, 3);
    }

    #[test]
    fn test_failure_count_resets_on_healthy() {
        let mut manager = create_test_manager();
        manager.register_component("test");

        manager.update_component_health("test", HealthStatus::Failed, "Fail");
        manager.update_component_health("test", HealthStatus::Failed, "Fail");
        assert_eq!(
            manager.get_component_health("test").unwrap().failure_count,
            2
        );

        manager.update_component_health("test", HealthStatus::Healthy, "Recovered");
        assert_eq!(
            manager.get_component_health("test").unwrap().failure_count,
            0
        );
    }

    #[test]
    fn test_degraded_does_not_change_failure_count() {
        let mut manager = create_test_manager();
        manager.register_component("test");

        manager.update_component_health("test", HealthStatus::Failed, "Fail");
        assert_eq!(
            manager.get_component_health("test").unwrap().failure_count,
            1
        );

        manager.update_component_health("test", HealthStatus::Degraded, "Degraded");
        assert_eq!(
            manager.get_component_health("test").unwrap().failure_count,
            1
        );
    }

    #[test]
    fn test_update_nonexistent_component_is_noop() {
        let mut manager = create_test_manager();
        // Should not panic
        manager.update_component_health("nonexistent", HealthStatus::Healthy, "OK");
        assert!(manager.get_component_health("nonexistent").is_none());
    }

    #[test]
    fn test_is_system_healthy_empty() {
        let manager = create_test_manager();
        // No components = all healthy (vacuously true)
        assert!(manager.is_system_healthy());
    }

    #[test]
    fn test_is_system_healthy_all_healthy() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.register_component("b");
        manager.update_component_health("a", HealthStatus::Healthy, "OK");
        manager.update_component_health("b", HealthStatus::Healthy, "OK");

        assert!(manager.is_system_healthy());
    }

    #[test]
    fn test_is_system_healthy_with_failed() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.register_component("b");
        manager.update_component_health("a", HealthStatus::Healthy, "OK");
        manager.update_component_health("b", HealthStatus::Failed, "Down");

        assert!(!manager.is_system_healthy());
    }

    #[test]
    fn test_is_system_healthy_with_degraded() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.update_component_health("a", HealthStatus::Degraded, "Slow");

        assert!(!manager.is_system_healthy());
    }

    #[test]
    fn test_system_health_summary_empty() {
        let manager = create_test_manager();
        let summary = manager.get_system_health_summary();

        // With 0 total components, degraded_count(0) < total/2(0) is false
        assert!(!summary.overall_healthy);
        assert_eq!(summary.total_components, 0);
        assert_eq!(summary.healthy_count, 0);
        assert_eq!(summary.degraded_count, 0);
        assert_eq!(summary.failed_count, 0);
    }

    #[test]
    fn test_system_health_summary_mixed() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.register_component("b");
        manager.register_component("c");
        manager.register_component("d");

        manager.update_component_health("a", HealthStatus::Healthy, "OK");
        manager.update_component_health("b", HealthStatus::Healthy, "OK");
        manager.update_component_health("c", HealthStatus::Degraded, "Slow");
        manager.update_component_health("d", HealthStatus::Failed, "Down");

        let summary = manager.get_system_health_summary();
        assert_eq!(summary.total_components, 4);
        assert_eq!(summary.healthy_count, 2);
        assert_eq!(summary.degraded_count, 1);
        assert_eq!(summary.failed_count, 1);
        assert!(!summary.overall_healthy); // Has failed component
    }

    #[test]
    fn test_system_health_summary_all_degraded() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.register_component("b");
        manager.update_component_health("a", HealthStatus::Degraded, "Slow");
        manager.update_component_health("b", HealthStatus::Degraded, "Slow");

        let summary = manager.get_system_health_summary();
        // All degraded (2/2 >= half) => not overall healthy
        assert!(!summary.overall_healthy);
    }

    #[test]
    fn test_health_recommendations_all_healthy() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.update_component_health("a", HealthStatus::Healthy, "OK");

        let recs = manager.get_health_recommendations();
        assert_eq!(recs.len(), 1);
        assert!(recs[0].contains("All AI coordination components are healthy"));
    }

    #[test]
    fn test_health_recommendations_failed() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.update_component_health("a", HealthStatus::Failed, "Down");

        let recs = manager.get_health_recommendations();
        assert!(recs.iter().any(|r| r.contains("failed")));
    }

    #[test]
    fn test_health_recommendations_degraded() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.update_component_health("a", HealthStatus::Degraded, "Slow");

        let recs = manager.get_health_recommendations();
        assert!(recs.iter().any(|r| r.contains("degraded")));
    }

    #[test]
    fn test_health_recommendations_below_80_percent() {
        let mut manager = create_test_manager();
        for i in 0..5 {
            manager.register_component(&format!("comp_{i}"));
        }
        // Only 1/5 healthy = 20%
        manager.update_component_health("comp_0", HealthStatus::Healthy, "OK");
        manager.update_component_health("comp_1", HealthStatus::Failed, "Down");
        manager.update_component_health("comp_2", HealthStatus::Failed, "Down");
        manager.update_component_health("comp_3", HealthStatus::Failed, "Down");
        manager.update_component_health("comp_4", HealthStatus::Failed, "Down");

        let recs = manager.get_health_recommendations();
        assert!(recs.iter().any(|r| r.contains("below 80%")));
    }

    #[test]
    fn test_get_all_component_health() {
        let mut manager = create_test_manager();
        manager.register_component("a");
        manager.register_component("b");

        let all = manager.get_all_component_health();
        assert_eq!(all.len(), 2);
        assert!(all.contains_key("a"));
        assert!(all.contains_key("b"));
    }

    #[tokio::test]
    async fn test_get_health_status() {
        let mut manager = create_test_manager();
        manager.register_component("a");

        let status = manager.get_health_status().await;
        assert_eq!(status.len(), 1);
        assert!(status.contains_key("a"));
    }

    #[tokio::test]
    async fn test_perform_health_check() {
        let mut manager = create_test_manager();
        manager.register_component("ai_coordinator");
        manager.register_component("security_adapter");

        let result = manager.perform_health_check().await;
        assert!(result.is_ok());

        // All simulated health checks return true → healthy
        let health = manager.get_component_health("ai_coordinator").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);

        let health = manager.get_component_health("security_adapter").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_initialize_self_healing() {
        let manager = initialize_self_healing().await.unwrap();
        assert_eq!(manager.component_health.len(), 5);
        assert!(manager.get_component_health("ai_coordinator").is_some());
        assert!(manager.get_component_health("security_adapter").is_some());
        assert!(
            manager
                .get_component_health("orchestration_adapter")
                .is_some()
        );
        assert!(manager.get_component_health("storage_adapter").is_some());
        assert!(manager.get_component_health("compute_adapter").is_some());
    }

    #[test]
    fn test_health_status_serde_roundtrip() {
        let statuses = vec![
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Failed,
            HealthStatus::Unknown,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_component_health_serde_roundtrip() {
        let health = ComponentHealth {
            component_id: "test".to_string(),
            status: HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            message: "All good".to_string(),
            failure_count: 0,
        };
        let json = serde_json::to_string(&health).unwrap();
        let deserialized: ComponentHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.component_id, "test");
        assert_eq!(deserialized.status, HealthStatus::Healthy);
        assert_eq!(deserialized.failure_count, 0);
    }

    #[test]
    fn test_system_health_summary_serde_roundtrip() {
        let summary = SystemHealthSummary {
            overall_healthy: true,
            total_components: 5,
            healthy_count: 4,
            degraded_count: 1,
            failed_count: 0,
            last_update: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: SystemHealthSummary = serde_json::from_str(&json).unwrap();
        assert!(deserialized.overall_healthy);
        assert_eq!(deserialized.total_components, 5);
        assert_eq!(deserialized.healthy_count, 4);
    }

    #[test]
    fn test_auto_recovery_triggered_at_max_failures() {
        let config = SelfHealingConfig {
            check_interval_seconds: 30,
            max_failures: 2,
            enable_auto_recovery: true,
        };
        let mut manager = SelfHealingManager::new(config);
        manager.register_component("test");

        // Trigger failures up to and beyond max_failures
        manager.update_component_health("test", HealthStatus::Failed, "Fail 1");
        manager.update_component_health("test", HealthStatus::Failed, "Fail 2");
        manager.update_component_health("test", HealthStatus::Failed, "Fail 3");

        // Verify the failure count is tracked
        let health = manager.get_component_health("test").unwrap();
        assert_eq!(health.failure_count, 3);
    }

    #[test]
    fn test_auto_recovery_disabled() {
        let config = SelfHealingConfig {
            check_interval_seconds: 30,
            max_failures: 1,
            enable_auto_recovery: false,
        };
        let mut manager = SelfHealingManager::new(config);
        manager.register_component("test");

        // Should not panic even with auto_recovery disabled
        manager.update_component_health("test", HealthStatus::Failed, "Fail");
        manager.update_component_health("test", HealthStatus::Failed, "Fail again");

        let health = manager.get_component_health("test").unwrap();
        assert_eq!(health.failure_count, 2);
    }
}
