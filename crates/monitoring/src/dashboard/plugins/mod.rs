//! Dashboard plugins module
//!
//! This module contains the dashboard plugin system that enables extending
//! the dashboard with custom visualization components and data sources.

pub mod example;
pub mod registry;
pub mod types;

// Re-exports
pub use registry::{DashboardPluginRegistryImpl, create_dashboard_plugin_registry, register_plugin_with_manager};
pub use types::{
    DashboardPlugin, DashboardPluginType, VisualizationPlugin, 
    DataSourcePlugin, PluginEvent, DashboardPluginRegistry, PluginMetadata
};
pub use example::ExamplePlugin;

use crate::dashboard::manager::DashboardManager;
use tracing::info;
use std::sync::Arc;
use squirrel_core::error::Result;

use crate::dashboard::DashboardComponent;

/// Register a dashboard component (plugin) with the dashboard manager
/// 
/// This function takes a dashboard component and registers it with the manager.
/// Returns a Result indicating success or failure.
pub async fn register_plugin(
    manager: &DashboardManager,
    component: Arc<dyn DashboardComponent>,
) -> Result<()> {
    info!("Registering dashboard component: {}", component.id());
    manager.register_component(component).await
}

/// Register all default plugins with the dashboard manager
///
/// # Arguments
///
/// * `manager` - Dashboard manager
///
/// # Returns
///
/// * `Result<()>` - Result indicating success or failure
pub async fn register_default_plugins(manager: &DashboardManager) -> Result<()> {
    // Register plugins with dashboard manager
    let visualization_plugin = Arc::new(ExamplePlugin::new());
    
    // Register the plugin directly using the correct component registration
    manager.register_component(visualization_plugin).await?;
    
    Ok(())
}

/// Creates an example visualization plugin
pub fn create_example_plugin() -> ExamplePlugin {
    info!("Creating example plugin");
    ExamplePlugin::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::config::DashboardConfig;
    
    #[tokio::test]
    async fn test_plugin_registration() {
        // Create dashboard manager
        let manager = Arc::new(DashboardManager::new(DashboardConfig::default()));
        
        // Initialize plugin registry
        manager.set_test_plugin_registry().await.unwrap();
        
        // Create plugin
        let plugin = Arc::new(example::ExamplePlugin::new());
        
        // Register plugin with plugin registry
        let plugin_registry = manager.get_plugin_registry().await.unwrap();
        let reg_result = plugin_registry.register_plugin(plugin.clone()).await;
        assert!(reg_result.is_ok());
        
        // Also register as a component
        let comp_result = manager.register_component(plugin).await;
        assert!(comp_result.is_ok());
        
        // Verify component was added
        let components = manager.get_components().await;
        assert_eq!(components.len(), 1);
    }

    #[tokio::test]
    async fn test_register_plugin_with_manager() {
        // Create manager
        let config = DashboardConfig::default();
        let manager = DashboardManager::new(config);
        let arc_manager = Arc::new(manager);

        // Initialize plugin registry
        arc_manager.set_test_plugin_registry().await.unwrap();

        // Create plugin
        let plugin = Arc::new(example::ExamplePlugin::new());
        
        // Register plugin
        let result = register_plugin_with_manager(&arc_manager, plugin).await;
        
        // Assert
        assert!(result.is_ok());
    }
} 