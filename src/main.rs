//! Main entry point for the Squirrel application

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, error};
use squirrel_interfaces::plugins::PluginRegistry;
use squirrel_mcp::plugins::PluginManager;
use std::sync::Arc;

mod plugins;
mod adapter;
// mod lib; // Removed as src/lib/ directory was deleted during refactoring

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Squirrel application");
    
    // Initialize the plugin system
    let plugin_manager = plugins::create_plugin_manager();
    let plugin_dirs = vec![PathBuf::from("./plugins")];
    
    match plugin_manager.initialize(&plugin_dirs).await {
        Ok(plugin_ids) => {
            info!("Plugin system initialized with {} plugins", plugin_ids.len());
            
            // Log loaded plugins
            for (i, plugin) in plugin_manager.list_plugins().await.iter().enumerate() {
                let metadata = plugin.metadata();
                info!(
                    "Plugin {}: {} ({}), capabilities: {:?}",
                    i + 1,
                    metadata.name,
                    metadata.version,
                    metadata.capabilities
                );
            }
        }
        Err(e) => {
            error!("Failed to initialize plugin system: {}", e);
            // Continue execution even if plugin system fails
        }
    }
    
    // Example: access a plugin by capability
    let registry = plugin_manager.registry();
    if let Some(cmd_plugin) = registry.get_plugin_by_capability("command_execution").await {
        info!(
            "Found command execution plugin: {}",
            cmd_plugin.metadata().name
        );
    }
    
    // Shut down the plugin system before exiting
    if let Err(e) = plugin_manager.shutdown().await {
        error!("Error shutting down plugin system: {}", e);
    }
    
    info!("Squirrel application shutting down");
    Ok(())
} 