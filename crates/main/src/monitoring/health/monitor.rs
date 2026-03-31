// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health monitoring system
//!
//! This module provides the core health monitor implementation.

use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{debug, error, info};

use super::HealthState;
use super::types::{ComponentHealth, HealthSnapshot, MonitoringHealthCheckConfig};
use crate::error::PrimalError;

/// Health monitoring system
pub struct HealthMonitor {
    /// Component health states
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    /// Health check configurations
    health_checks: Arc<RwLock<HashMap<String, MonitoringHealthCheckConfig>>>,
    /// Health history
    health_history: Arc<RwLock<Vec<HealthSnapshot>>>,
    /// Maximum history size
    max_history_size: usize,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    /// Create a new health monitor
    #[must_use]
    pub fn new() -> Self {
        Self {
            component_health: Arc::new(RwLock::new(HashMap::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            health_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 100,
        }
    }

    /// Register a component for health monitoring
    pub async fn register_component(
        &self,
        name: &str,
        config: MonitoringHealthCheckConfig,
    ) -> Result<(), PrimalError> {
        let mut health_checks = self.health_checks.write().await;
        let mut component_health = self.component_health.write().await;

        health_checks.insert(name.to_string(), config);

        component_health.insert(
            name.to_string(),
            ComponentHealth {
                name: name.to_string(),
                state: HealthState::Unknown,
                last_check: Utc::now(),
                check_duration: Duration::from_millis(0),
                message: "Component registered".to_string(),
                details: HashMap::new(),
                check_count: 0,
                consecutive_successes: 0,
                consecutive_failures: 0,
            },
        );

        info!("Registered component for health monitoring: {}", name);
        Ok(())
    }

    /// Check health of all components
    pub async fn check_all_components(&self) -> Result<(), PrimalError> {
        debug!("Checking health of all components");

        let components: Vec<String> = self.component_health.read().await.keys().cloned().collect();

        for component in components {
            if let Err(e) = self.check_component_health(&component).await {
                error!("Failed to check health of component {}: {}", component, e);
            }
        }

        // Create health snapshot
        self.create_health_snapshot().await?;

        Ok(())
    }

    /// Check health of a specific component
    pub async fn check_component_health(&self, component: &str) -> Result<(), PrimalError> {
        let check_start = Utc::now();

        // Perform the actual health check
        let health_result = self.perform_health_check(component).await;

        let check_duration = Utc::now()
            .signed_duration_since(check_start)
            .to_std()
            .unwrap_or_else(|_| Duration::from_millis(0));

        // Update component health
        let mut component_health = self.component_health.write().await;
        if let Some(health) = component_health.get_mut(component) {
            health.check_count += 1;
            health.last_check = check_start;
            health.check_duration = check_duration;

            match health_result {
                Ok(()) => {
                    health.state = HealthState::Healthy;
                    health.message = "Component is healthy".to_string();
                    health.consecutive_successes += 1;
                    health.consecutive_failures = 0;
                    health
                        .details
                        .insert("status".to_string(), "healthy".to_string());
                }
                Err(e) => {
                    health.state = HealthState::Critical;
                    health.message = format!("Health check failed: {e}");
                    health.consecutive_failures += 1;
                    health.consecutive_successes = 0;
                    health.details.insert("error".to_string(), e.to_string());
                }
            }

            debug!(
                "Health check for {} completed in {:?}: {:?}",
                component, check_duration, health.state
            );
        }

        Ok(())
    }

    /// Get health summary for all components
    pub async fn get_health_summary(&self) -> Result<HashMap<String, HealthState>, PrimalError> {
        let component_health = self.component_health.read().await;

        let mut summary = HashMap::new();
        for (name, health) in component_health.iter() {
            summary.insert(name.clone(), health.state.clone());
        }

        Ok(summary)
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> Result<HealthState, PrimalError> {
        let component_health = self.component_health.read().await;

        if component_health.is_empty() {
            return Ok(HealthState::Unknown);
        }

        let mut has_critical = false;
        let mut has_degraded = false;

        for health in component_health.values() {
            match health.state {
                HealthState::Critical => has_critical = true,
                HealthState::Warning | HealthState::Unknown => has_degraded = true,
                HealthState::Healthy => {}
            }
        }

        Ok(if has_critical {
            HealthState::Critical
        } else if has_degraded {
            HealthState::Warning
        } else {
            HealthState::Healthy
        })
    }

    /// Get health information for a specific component
    pub async fn get_component_health(
        &self,
        component: &str,
    ) -> Result<ComponentHealth, PrimalError> {
        let component_health = self.component_health.read().await;

        component_health
            .get(component)
            .cloned()
            .ok_or_else(|| PrimalError::NotFoundError(format!("Component not found: {component}")))
    }

    /// Create a health snapshot
    async fn create_health_snapshot(&self) -> Result<(), PrimalError> {
        let component_health = self.component_health.read().await;
        let system_health = self.get_system_health().await?;

        let snapshot = HealthSnapshot {
            timestamp: Utc::now(),
            component_health: component_health.clone(),
            system_health,
        };

        let mut health_history = self.health_history.write().await;
        health_history.push(snapshot);

        // Trim history if needed
        let history_len = health_history.len();
        if history_len > self.max_history_size {
            health_history.drain(0..history_len - self.max_history_size);
        }

        Ok(())
    }

    /// Perform actual health check: system resource thresholds and optional Unix socket probes.
    ///
    /// Uses [`universal_constants::sys_info`] for CPU and memory on supported platforms.
    /// Unix sockets listed in `AI_PROVIDER_SOCKETS` / `SONGBIRD_SOCKET` and the resolved Squirrel
    /// socket path are probed **only when the path exists**, so idle tests without listeners stay green.
    async fn perform_health_check(&self, component: &str) -> Result<(), PrimalError> {
        let check_timeout = self
            .health_checks
            .read()
            .await
            .get(component)
            .map_or_else(|| Duration::from_secs(2), |c| c.timeout);

        let cpu = universal_constants::sys_info::system_cpu_usage_percent().unwrap_or(0.0);
        let mem_pct = universal_constants::sys_info::memory_info()
            .ok()
            .filter(|m| m.total > 0)
            .map_or(0.0, |m| (m.used as f64 / m.total as f64) * 100.0);

        const CPU_CRITICAL: f64 = 99.5;
        const MEM_CRITICAL: f64 = 99.5;
        if cpu >= CPU_CRITICAL {
            return Err(PrimalError::Internal(format!(
                "CPU utilization critically high: {cpu:.1}%"
            )));
        }
        if mem_pct >= MEM_CRITICAL {
            return Err(PrimalError::Internal(format!(
                "Memory utilization critically high: {mem_pct:.1}%"
            )));
        }

        let mut socket_paths: Vec<String> = Vec::new();
        socket_paths.push(crate::rpc::unix_socket::get_socket_path(
            &crate::rpc::unix_socket::get_node_id(),
        ));
        if let Ok(extra) = std::env::var("AI_PROVIDER_SOCKETS") {
            for p in extra.split(',') {
                let p = p.trim();
                if !p.is_empty() {
                    socket_paths.push(p.to_string());
                }
            }
        }
        if let Ok(p) = std::env::var("SONGBIRD_SOCKET")
            && !p.is_empty()
        {
            socket_paths.push(p);
        }

        for path in socket_paths {
            if !Path::new(&path).exists() {
                continue;
            }
            match timeout(check_timeout, UnixStream::connect(&path)).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    return Err(PrimalError::Network(format!(
                        "Unix socket not accepting connections at {path}: {e}"
                    )));
                }
                Err(_) => {
                    return Err(PrimalError::Network(format!(
                        "Unix socket connection to {path} exceeded {check_timeout:?}"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get health history
    pub async fn get_health_history(&self) -> Vec<HealthSnapshot> {
        self.health_history.read().await.clone()
    }

    /// Clear health history
    pub async fn clear_health_history(&self) {
        self.health_history.write().await.clear();
    }

    /// Set maximum history size
    pub const fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PrimalError;
    use crate::monitoring::health::types::MonitoringHealthCheckConfig;

    fn sample_config() -> MonitoringHealthCheckConfig {
        MonitoringHealthCheckConfig {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(2),
            failure_threshold: 2,
            success_threshold: 1,
            grace_period: Duration::from_secs(5),
        }
    }

    #[tokio::test]
    async fn default_and_register_component() {
        let m = HealthMonitor::default();

        m.register_component("db", sample_config())
            .await
            .expect("register");

        let h = m.get_component_health("db").await.expect("get");
        assert_eq!(h.name, "db");
        assert!(matches!(h.state, HealthState::Unknown));
        assert_eq!(h.message, "Component registered");
    }

    #[tokio::test]
    async fn check_component_updates_health_and_summary() {
        let m = HealthMonitor::new();
        m.register_component("api", sample_config())
            .await
            .expect("should succeed");
        m.check_component_health("api")
            .await
            .expect("should succeed");

        let h = m.get_component_health("api").await.expect("should succeed");
        assert!(matches!(h.state, HealthState::Healthy));
        assert_eq!(h.check_count, 1);
        assert_eq!(h.consecutive_successes, 1);
        assert_eq!(h.consecutive_failures, 0);

        let summary = m.get_health_summary().await.expect("should succeed");
        assert_eq!(summary.get("api"), Some(&HealthState::Healthy));
    }

    #[tokio::test]
    async fn system_health_empty_unknown_before_check() {
        let m = HealthMonitor::new();
        assert_eq!(
            m.get_system_health().await.expect("should succeed"),
            HealthState::Unknown
        );

        m.register_component("a", sample_config())
            .await
            .expect("should succeed");
        // Before any check, components are Unknown → aggregated Warning
        assert_eq!(
            m.get_system_health().await.expect("should succeed"),
            HealthState::Warning
        );

        m.check_all_components().await.expect("should succeed");
        assert_eq!(
            m.get_system_health().await.expect("should succeed"),
            HealthState::Healthy
        );
    }

    #[tokio::test]
    async fn get_component_health_not_found() {
        let m = HealthMonitor::new();
        let err = m.get_component_health("nope").await.unwrap_err();
        assert!(matches!(err, PrimalError::NotFoundError(_)));
    }

    #[tokio::test]
    async fn health_history_snapshot_and_trim() {
        let mut m = HealthMonitor::new();
        m.set_max_history_size(2);
        m.register_component("c1", sample_config())
            .await
            .expect("should succeed");
        m.check_all_components().await.expect("should succeed");
        m.check_all_components().await.expect("should succeed");
        m.check_all_components().await.expect("should succeed");

        let hist = m.get_health_history().await;
        assert!(hist.len() <= 2);
        assert!(!hist.is_empty());

        m.clear_health_history().await;
        assert!(m.get_health_history().await.is_empty());
    }
}
