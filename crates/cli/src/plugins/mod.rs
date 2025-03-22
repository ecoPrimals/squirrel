//! Plugin system for the Squirrel CLI
//!
//! This module implements a plugin system that allows extending the
//! CLI functionality with custom commands and features.

pub mod plugin;
pub mod manager;
pub mod state;
pub mod error;
#[cfg(test)]
mod tests;

pub use plugin::{PluginItem, PluginStatus, PluginMetadata};
pub use manager::PluginManager;
pub use error::PluginError;

// This is the entry point for initializing the plugin system
pub fn initialize_plugins() -> Result<(), error::PluginError> {
    // Initialize the plugin system
    // This function is called at application startup to load installed plugins
    
    // Get the plugin manager singleton
    let _plugin_manager = state::get_plugin_manager();
    
    // TODO: Implement plugin discovery and loading logic
    // - Scan plugin directories
    // - Load plugin metadata
    // - Register plugins with the plugin manager
    
    Ok(())
} 