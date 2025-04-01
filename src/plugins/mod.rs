//! Plugin system for Squirrel
//!
//! This module provides the core plugin system infrastructure for Squirrel,
//! including a plugin registry, loader, and manager.

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
use squirrel_interfaces::plugins::PluginRegistry;

mod registry;
mod loader;

pub use registry::{DefaultPluginRegistry, create_plugin_registry};
pub use loader::{PluginLoader, create_plugin_loader};

/// Main plugin system manager
///
/// This struct provides high-level functions for managing the plugin system,
/// including initialization, plugin registration, discovery, and shutdown.
pub struct PluginManager {
    /// The plugin registry
    registry: Arc<DefaultPluginRegistry>,
    /// The plugin loader
    loader: Arc<PluginLoader>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let registry = create_plugin_registry();
        let loader = create_plugin_loader(registry.clone());
        
        Self {
            registry,
            loader,
        }
    }
    
    /// Initialize the plugin system
    ///
    /// This loads all built-in plugins and plugins from the provided
    /// directories, then initializes all plugins.
    ///
    /// # Arguments
    ///
    /// * `plugin_dirs` - Directories to search for plugins
    ///
    /// # Returns
    ///
    /// A result containing a list of loaded plugin IDs
    pub async fn initialize<P: AsRef<Path>>(&self, plugin_dirs: &[P]) -> Result<Vec<String>> {
        info!("Initializing plugin system");
        let mut plugin_ids = Vec::new();
        
        // Load built-in plugins
        let builtin_ids = self.loader.load_builtin_plugins().await?;
        plugin_ids.extend(builtin_ids);
        
        // Load plugins from directories
        for dir in plugin_dirs {
            let ids = self.loader.load_plugins_from_directory(dir).await?;
            plugin_ids.extend(ids);
        }
        
        // Initialize all plugins
        self.loader.initialize_all_plugins().await?;
        
        info!("Plugin system initialized with {} plugins", plugin_ids.len());
        Ok(plugin_ids)
    }
    
    /// Shutdown the plugin system
    ///
    /// This shuts down all loaded plugins and cleans up resources.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down plugin system");
        self.loader.shutdown_all_plugins().await?;
        info!("Plugin system shutdown complete");
        Ok(())
    }
    
    /// Get the plugin registry
    pub fn registry(&self) -> Arc<DefaultPluginRegistry> {
        self.registry.clone()
    }
    
    /// Get a list of all loaded plugins
    pub async fn list_plugins(&self) -> Vec<Arc<dyn squirrel_interfaces::plugins::Plugin>> {
        self.registry.list_plugins().await
    }
}

/// Create a new plugin manager
pub fn create_plugin_manager() -> Arc<PluginManager> {
    Arc::new(PluginManager::new())
}

// Example plugins module for demonstration purposes
#[cfg(feature = "plugins")]
pub mod examples {
    use std::sync::Arc;
    // use squirrel_example_plugins::create_utility_plugin;
    
    /// Create an example utility plugin
    pub fn create_example_utility_plugin() -> Arc<dyn squirrel_interfaces::plugins::Plugin> {
        // Placeholder for actual plugin - commented out until the example plugin is available
        // create_utility_plugin()
        panic!("Example utility plugin not implemented yet")
    }
} 