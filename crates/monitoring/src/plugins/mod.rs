//! Monitoring plugins module
//!
//! This module contains the plugins implementation for the monitoring crate.
//! It provides a plugin system that allows for extending the functionality
//! of the monitoring system.

mod system_metrics;
mod health_reporter;
mod alert_handler;
mod common;
mod prelude;
mod registry;
mod loader;
mod manager;
pub mod examples;

pub use system_metrics::SystemMetricsPlugin;
pub use health_reporter::HealthReporterPlugin;
pub use alert_handler::AlertHandlerPlugin;
pub use alert_handler::AlertHandler;
pub use common::{PluginMetadata, MonitoringPlugin};
pub use registry::PluginRegistry;
pub use loader::{PluginLoader, PluginConfig};
pub use manager::{PluginManager, PluginState};

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

/// Default plugin factory for creating and initializing plugins
pub struct DefaultPluginFactory;

impl DefaultPluginFactory {
    /// Create a new plugin manager with all built-in plugins
    pub async fn create_plugin_manager() -> Result<PluginManager> {
        let manager = PluginManager::new();
        
        // Initialize the manager with built-in plugins
        manager.initialize().await?;
        
        Ok(manager)
    }
    
    /// Create a custom plugin manager with specific plugins
    pub fn create_custom_manager() -> PluginManager {
        PluginManager::new()
    }
}

/// Create a default plugin manager with all built-in plugins
pub async fn create_default_plugin_manager() -> Result<PluginManager> {
    DefaultPluginFactory::create_plugin_manager().await
}

// Export prelude for easy importing
pub mod exports {
    pub use super::common::{PluginMetadata, MonitoringPlugin};
    pub use super::registry::PluginRegistry;
    pub use super::loader::{PluginLoader, PluginConfig};
    pub use super::manager::{PluginManager, PluginState};
    pub use super::DefaultPluginFactory;
    pub use super::create_default_plugin_manager;
}

// Tests for the monitoring plugins
#[cfg(test)]
mod tests; 