//! Monitoring plugins module
//!
//! This module contains the plugins implementation for the monitoring crate.

mod system_metrics;
mod health_reporter;
mod alert_handler;
mod prelude;
mod common;

pub use system_metrics::SystemMetricsPlugin;
pub use health_reporter::HealthReporterPlugin;
pub use alert_handler::AlertHandlerPlugin;
pub use alert_handler::AlertHandler;
pub use common::{PluginMetadata, MonitoringPlugin};

use anyhow::{Result, anyhow};
use std::sync::Arc;

/// Register monitoring plugins with the plugin registry
pub async fn register_plugins<T>(registry: Arc<T>) -> Result<()>
where
    T: MonitoringPluginRegistry + Send + Sync + 'static,
{
    // Register system metrics plugin
    let system_metrics = Arc::new(SystemMetricsPlugin::new());
    registry.register_monitoring_plugin(system_metrics).await.map_err(|e| anyhow!("Failed to register system metrics plugin: {}", e))?;
    
    // Register health reporter plugin
    let health_reporter = Arc::new(HealthReporterPlugin::new());
    registry.register_monitoring_plugin(health_reporter).await.map_err(|e| anyhow!("Failed to register health reporter plugin: {}", e))?;
    
    // Register alert handler plugin
    let alert_handler = Arc::new(AlertHandlerPlugin::new());
    registry.register_monitoring_plugin(alert_handler).await.map_err(|e| anyhow!("Failed to register alert handler plugin: {}", e))?;
    
    Ok(())
}

/// Simplified plugin registry trait for monitoring plugins
#[async_trait::async_trait]
pub trait MonitoringPluginRegistry {
    /// Register a plugin with the registry
    async fn register_monitoring_plugin<T>(&self, plugin: Arc<T>) -> anyhow::Result<()>
    where
        T: MonitoringPlugin + Send + Sync + 'static;
}

// Tests for the monitoring plugins
#[cfg(test)]
mod tests; 