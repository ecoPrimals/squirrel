//! Health reporter plugin implementation

use crate::plugins::common::{MonitoringPlugin, PluginMetadata};
use crate::health::component::{ComponentHealth, HealthCheck};
use crate::health::status::Status;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, error};
use serde_json::Value;
use std::fmt::Debug;

/// Health reporter plugin for monitoring and reporting component health.
/// This plugin exposes health status of various components and capabilities
/// to register health checks.
#[derive(Debug)]
pub struct HealthReporterPlugin {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Health checks registry
    health_checks: Arc<RwLock<HashMap<String, Arc<dyn HealthCheck>>>>,
}

impl Default for HealthReporterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthReporterPlugin {
    /// Create a new health reporter plugin
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "Health Reporter Plugin",
                "1.0.0",
                "Reports health status of registered components",
                "DataScienceBioLab",
            )
            .with_capability("health.status")
            .with_capability("health.reporting")
            .with_capability("health.monitoring"),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a health check with the plugin
    pub async fn register_health_check(&self, health_check: Box<dyn HealthCheck>) {
        let name = health_check.name().to_string();
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(name.clone(), Arc::from(health_check));
        info!("Registered health check: {}", name);
    }
    
    /// Run all registered health checks
    pub async fn run_health_checks(&self) -> Result<Vec<ComponentHealth>> {
        let health_checks = self.health_checks.read().await;
        let mut results = Vec::new();
        
        for (name, health_check) in health_checks.iter() {
            match health_check.check().await {
                Ok(health) => {
                    results.push(health);
                }
                Err(e) => {
                    error!("Failed to run health check {}: {}", name, e);
                    // Create a health entry with error status
                    results.push(ComponentHealth {
                        name: name.clone(),
                        status: Status::Unhealthy,
                        message: Some(format!("Health check failed: {}", e)),
                        last_check: chrono::Utc::now(),
                        details: HashMap::new(),
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    /// Collect health status metrics of all components
    pub async fn collect_health_metrics(&self) -> anyhow::Result<Value> {
        let health_checks = self.run_health_checks().await?;
        
        // Count components by status
        let mut healthy_count = 0;
        let mut warning_count = 0;
        let mut critical_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unknown_count = 0;
        
        for health in &health_checks {
            match health.status {
                Status::Healthy => healthy_count += 1,
                Status::Warning => warning_count += 1,
                Status::Critical => critical_count += 1,
                Status::Degraded => degraded_count += 1,
                Status::Unhealthy => unhealthy_count += 1,
                Status::Unknown => unknown_count += 1,
            }
        }
        
        // Convert health checks to JSON
        let health_json: Vec<Value> = health_checks
            .into_iter()
            .map(|health| serde_json::to_value(health).unwrap_or_else(|_| serde_json::json!({})))
            .collect();
        
        // Return health metrics
        Ok(serde_json::json!({
            "components": health_json,
            "summary": {
                "total": health_json.len(),
                "healthy": healthy_count,
                "warning": warning_count,
                "critical": critical_count,
                "degraded": degraded_count,
                "unhealthy": unhealthy_count,
                "unknown": unknown_count,
            },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }))
    }
}

#[async_trait]
impl MonitoringPlugin for HealthReporterPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing Health Reporter Plugin");
        Ok(())
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        info!("Shutting down Health Reporter Plugin");
        Ok(())
    }
    
    async fn collect_metrics(&self) -> anyhow::Result<Value> {
        let health_checks = self.run_health_checks().await?;
        let health_json: Vec<Value> = health_checks
            .into_iter()
            .map(|health| serde_json::to_value(health).unwrap_or_else(|_| serde_json::json!({})))
            .collect();
        
        Ok(serde_json::json!(health_json))
    }
    
    fn get_monitoring_targets(&self) -> Vec<String> {
        vec!["health".to_string()]
    }
    
    async fn handle_alert(&self, _alert: Value) -> anyhow::Result<()> {
        // Health reporter doesn't handle alerts directly
        Ok(())
    }
} 