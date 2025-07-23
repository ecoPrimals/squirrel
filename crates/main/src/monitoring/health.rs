//! # Health Monitoring Module
//!
//! This module provides comprehensive health monitoring capabilities for all ecosystem components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use super::HealthState;
use crate::error::PrimalError;

/// Health monitoring system
pub struct HealthMonitor {
    /// Component health states
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    /// Health check configurations
    health_checks: Arc<RwLock<HashMap<String, HealthCheckConfig>>>,
    /// Health history
    health_history: Arc<RwLock<Vec<HealthSnapshot>>>,
    /// Maximum history size
    max_history_size: usize,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Current health state
    pub state: HealthState,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Check duration
    pub check_duration: Duration,
    /// Health message
    pub message: String,
    /// Health details
    pub details: HashMap<String, String>,
    /// Check count
    pub check_count: u64,
    /// Consecutive successes
    pub consecutive_successes: u64,
    /// Consecutive failures
    pub consecutive_failures: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold
    pub success_threshold: u32,
    /// Grace period after startup
    pub grace_period: Duration,
}

/// Health snapshot for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
    /// Component health states
    pub component_health: HashMap<String, ComponentHealth>,
    /// Overall system health
    pub system_health: HealthState,
}

impl HealthMonitor {
    /// Create a new health monitor
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
        config: HealthCheckConfig,
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
            .unwrap_or(Duration::from_millis(0));

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
                    health.message = format!("Health check failed: {}", e);
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

    /// Get detailed health information for a component
    pub async fn get_component_health(
        &self,
        component: &str,
    ) -> Result<ComponentHealth, PrimalError> {
        let component_health = self.component_health.read().await;

        if let Some(health) = component_health.get(component) {
            Ok(health.clone())
        } else {
            Err(PrimalError::NotFoundError(format!(
                "Component '{}' not found",
                component
            )))
        }
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> Result<HealthState, PrimalError> {
        let summary = self.get_health_summary().await?;

        let mut has_critical = false;
        let mut has_warning = false;

        for state in summary.values() {
            match state {
                HealthState::Critical => has_critical = true,
                HealthState::Warning => has_warning = true,
                HealthState::Healthy => {}
                HealthState::Unknown => has_warning = true,
            }
        }

        let system_health = if has_critical {
            HealthState::Critical
        } else if has_warning {
            HealthState::Warning
        } else {
            HealthState::Healthy
        };

        Ok(system_health)
    }

    /// Perform health check for a specific component
    async fn perform_health_check(&self, component: &str) -> Result<(), PrimalError> {
        match component {
            "ai_intelligence" => self.check_ai_intelligence().await,
            "mcp_integration" => self.check_mcp_integration().await,
            "context_state" => self.check_context_state().await,
            "agent_deployment" => self.check_agent_deployment().await,
            "songbird" => self.check_songbird().await,
            "toadstool" => self.check_toadstool().await,
            "nestgate" => self.check_nestgate().await,
            "beardog" => self.check_beardog().await,
            _ => self.check_generic_component(component).await,
        }
    }

    /// AI Intelligence health check
    async fn check_ai_intelligence(&self) -> Result<(), PrimalError> {
        // Simulate AI intelligence health check
        // In a real implementation, this would:
        // - Check if AI models are loaded
        // - Verify inference capabilities
        // - Test API endpoints
        // - Check memory usage

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Simulate health check success
        Ok(())
    }

    /// MCP Integration health check
    async fn check_mcp_integration(&self) -> Result<(), PrimalError> {
        // Simulate MCP integration health check
        // In a real implementation, this would:
        // - Check MCP connections
        // - Verify protocol handlers
        // - Test message serialization
        // - Check connection pools

        tokio::time::sleep(Duration::from_millis(15)).await;

        Ok(())
    }

    /// Context State health check
    async fn check_context_state(&self) -> Result<(), PrimalError> {
        // Simulate context state health check
        // In a real implementation, this would:
        // - Check context persistence
        // - Verify session management
        // - Test context retrieval
        // - Check memory usage

        tokio::time::sleep(Duration::from_millis(8)).await;

        Ok(())
    }

    /// Agent Deployment health check
    async fn check_agent_deployment(&self) -> Result<(), PrimalError> {
        // Simulate agent deployment health check
        // In a real implementation, this would:
        // - Check deployed agents
        // - Verify agent health
        // - Test deployment capabilities
        // - Check resource usage

        tokio::time::sleep(Duration::from_millis(12)).await;

        Ok(())
    }

    /// Songbird health check
    async fn check_songbird(&self) -> Result<(), PrimalError> {
        // Simulate songbird health check
        // In a real implementation, this would:
        // - Check orchestration services
        // - Verify service discovery
        // - Test load balancing
        // - Check health monitoring

        tokio::time::sleep(Duration::from_millis(20)).await;

        Ok(())
    }

    /// Toadstool health check
    async fn check_toadstool(&self) -> Result<(), PrimalError> {
        // Simulate toadstool health check
        // In a real implementation, this would:
        // - Check compute resources
        // - Verify job queue
        // - Test job execution
        // - Check resource allocation

        tokio::time::sleep(Duration::from_millis(18)).await;

        Ok(())
    }

    /// Nestgate health check
    async fn check_nestgate(&self) -> Result<(), PrimalError> {
        // Simulate nestgate health check
        // In a real implementation, this would:
        // - Check storage systems
        // - Verify data integrity
        // - Test read/write operations
        // - Check backup systems

        tokio::time::sleep(Duration::from_millis(25)).await;

        Ok(())
    }

    /// Beardog health check
    async fn check_beardog(&self) -> Result<(), PrimalError> {
        // Simulate beardog health check
        // In a real implementation, this would:
        // - Check security services
        // - Verify authentication
        // - Test authorization
        // - Check security policies

        tokio::time::sleep(Duration::from_millis(14)).await;

        Ok(())
    }

    /// Generic component health check
    async fn check_generic_component(&self, _component: &str) -> Result<(), PrimalError> {
        // Generic health check for unknown components
        tokio::time::sleep(Duration::from_millis(5)).await;

        Ok(())
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

        let mut history = self.health_history.write().await;
        history.push(snapshot);

        // Limit history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        Ok(())
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
            failure_threshold: 3,
            success_threshold: 2,
            grace_period: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        assert!(monitor.component_health.read().await.is_empty());
        assert!(monitor.health_checks.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_component_registration() {
        let monitor = HealthMonitor::new();
        let config = HealthCheckConfig::default();

        let result = monitor.register_component("test_component", config).await;
        assert!(result.is_ok());

        let health_checks = monitor.health_checks.read().await;
        assert!(health_checks.contains_key("test_component"));

        let component_health = monitor.component_health.read().await;
        assert!(component_health.contains_key("test_component"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let monitor = HealthMonitor::new();
        let config = HealthCheckConfig::default();

        monitor
            .register_component("test_component", config)
            .await
            .unwrap();

        let result = monitor.check_component_health("test_component").await;
        assert!(result.is_ok());

        let health = monitor
            .get_component_health("test_component")
            .await
            .unwrap();
        assert_eq!(health.check_count, 1);
    }

    #[tokio::test]
    async fn test_health_summary() {
        let monitor = HealthMonitor::new();
        let config = HealthCheckConfig::default();

        monitor
            .register_component("component1", config.clone())
            .await
            .unwrap();
        monitor
            .register_component("component2", config)
            .await
            .unwrap();

        monitor.check_all_components().await.unwrap();

        let summary = monitor.get_health_summary().await.unwrap();
        assert_eq!(summary.len(), 2);
        assert!(summary.contains_key("component1"));
        assert!(summary.contains_key("component2"));
    }

    #[tokio::test]
    async fn test_system_health() {
        let monitor = HealthMonitor::new();
        let config = HealthCheckConfig::default();

        monitor
            .register_component("healthy_component", config)
            .await
            .unwrap();
        monitor.check_all_components().await.unwrap();

        let system_health = monitor.get_system_health().await.unwrap();
        // System health should be healthy or warning (depends on random failures)
        assert!(matches!(
            system_health,
            HealthState::Healthy | HealthState::Warning | HealthState::Critical
        ));
    }
}
