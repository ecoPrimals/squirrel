use std::collections::HashMap;
use std::path::PathBuf;

use crate::plugins::{PluginItem, PluginMetadata, PluginStatus};
use crate::plugins::error::PluginError;

/// A manager for Squirrel plugins
pub struct PluginManager {
    plugins: HashMap<String, PluginItem>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// List all installed plugins
    pub fn list_plugins(&self) -> Vec<&PluginItem> {
        self.plugins.values().collect()
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Result<&PluginItem, PluginError> {
        self.plugins
            .get(name)
            .ok_or_else(|| PluginError::plugin_not_found(name))
    }

    /// Get a mutable reference to a plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Result<&mut PluginItem, PluginError> {
        self.plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::plugin_not_found(name))
    }

    /// Add a new plugin to the manager
    pub fn add_plugin(&mut self, metadata: PluginMetadata, path: PathBuf, status: PluginStatus) -> Result<&PluginItem, PluginError> {
        let name = metadata.name.clone();
        
        if self.plugins.contains_key(&name) {
            return Err(PluginError::plugin_already_exists(&name));
        }
        
        let plugin = PluginItem::new(metadata, path, status);
        self.plugins.insert(name.clone(), plugin);
        
        Ok(self.plugins.get(&name).unwrap())
    }

    /// Remove a plugin from the manager
    pub fn remove_plugin(&mut self, name: &str) -> Result<(), PluginError> {
        if !self.plugins.contains_key(name) {
            return Err(PluginError::plugin_not_found(name));
        }
        
        self.plugins.remove(name);
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default plugin manager
pub fn create_plugin_manager() -> PluginManager {
    PluginManager::new()
}

/// Initialize the plugin system
pub fn initialize_plugins() -> Result<(), PluginError> {
    Ok(())
} 