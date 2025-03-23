//! Plugin registration for the app crate
//!
//! This module provides functionality for registering plugins with the app.

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};

use squirrel_interfaces::plugins::{Plugin, PluginRegistry};

/// Initialize the plugin system
pub async fn initialize_plugin_system() -> Result<Arc<dyn PluginRegistry>> {
    info!("Initializing plugin system");
    
    // Create the plugin registry
    let registry = Arc::new(squirrel_plugins::registry::PluginRegistry::new());
    
    debug!("Plugin registry created");
    
    Ok(registry)
}

/// Register core plugins
pub async fn register_core_plugins(registry: &dyn PluginRegistry) -> Result<()> {
    info!("Registering core plugins");
    
    // Register plugins based on available features
    #[cfg(feature = "commands")]
    {
        debug!("Registering commands plugin");
        let plugin_id = squirrel_commands::register_plugin(registry).await?;
        debug!("Commands plugin registered with ID: {}", plugin_id);
    }
    
    info!("Core plugins registered");
    Ok(())
} 