//! Plugin registry module
//!
//! This module provides a registry for plugins.

use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::plugin::{Plugin, PluginMetadata};
use crate::PluginError;

/// Plugin dependency
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency ID
    pub id: Uuid,
    
    /// Dependency name
    pub name: String,
    
    /// Minimum version required
    pub min_version: String,
    
    /// Maximum version supported
    pub max_version: Option<String>,
    
    /// Whether the dependency is optional
    pub optional: bool,
}

/// Plugin registry
pub struct PluginRegistry {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
    
    /// Plugin metadata
    metadata: RwLock<HashMap<Uuid, PluginMetadata>>,
    
    /// Plugin index by name
    index: RwLock<BTreeMap<String, Vec<Uuid>>>,
    
    /// Plugin dependencies
    dependencies: RwLock<HashMap<Uuid, Vec<PluginDependency>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            metadata: RwLock::new(HashMap::new()),
            index: RwLock::new(BTreeMap::new()),
            dependencies: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a plugin
    pub async fn register<P: Plugin + 'static>(&self, plugin: Arc<P>) -> Result<()> {
        let metadata = plugin.metadata().clone();
        let id = metadata.id;
        let name = metadata.name.clone();
        
        // Check if plugin is already registered
        {
            let plugins = self.plugins.read().await;
            if plugins.contains_key(&id) {
                return Err(PluginError::AlreadyRegistered(id).into());
            }
        }
        
        // Update mappings
        {
            let mut plugins = self.plugins.write().await;
            let mut metadata_map = self.metadata.write().await;
            let mut index = self.index.write().await;
            
            // Store plugin
            plugins.insert(id, plugin.clone() as Arc<dyn Plugin>);
            
            // Store metadata
            metadata_map.insert(id, metadata.clone());
            
            // Update index
            let entry = index.entry(name).or_insert_with(Vec::new);
            if !entry.contains(&id) {
                entry.push(id);
            }
        }
        
        // Parse and store dependencies
        {
            let mut deps = self.dependencies.write().await;
            let plugin_deps: Vec<PluginDependency> = metadata.dependencies.iter()
                .map(|dep_id| {
                    PluginDependency {
                        id: *dep_id,
                        name: String::new(), // To be resolved later
                        min_version: "0.0.0".to_string(), // Default
                        max_version: None,
                        optional: false,
                    }
                })
                .collect();
            
            if !plugin_deps.is_empty() {
                deps.insert(id, plugin_deps);
            }
        }
        
        Ok(())
    }
    
    /// Unregister a plugin
    pub async fn unregister(&self, id: Uuid) -> Result<()> {
        // Check if plugin exists
        let plugin_name = {
            let metadata = self.metadata.read().await;
            match metadata.get(&id) {
                Some(meta) => meta.name.clone(),
                None => return Err(PluginError::NotFound(id).into()),
            }
        };
        
        // Update mappings
        {
            let mut plugins = self.plugins.write().await;
            let mut metadata_map = self.metadata.write().await;
            let mut index = self.index.write().await;
            let mut deps = self.dependencies.write().await;
            
            // Remove plugin
            plugins.remove(&id);
            
            // Remove metadata
            metadata_map.remove(&id);
            
            // Update index
            if let Some(entries) = index.get_mut(&plugin_name) {
                entries.retain(|&entry_id| entry_id != id);
                if entries.is_empty() {
                    index.remove(&plugin_name);
                }
            }
            
            // Remove dependencies
            deps.remove(&id);
        }
        
        Ok(())
    }
    
    /// Get plugin by ID
    pub async fn get_plugin(&self, id: Uuid) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(&id).cloned()
    }
    
    /// Get plugin by name
    pub async fn get_plugin_by_name(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let index = self.index.read().await;
        let plugins = self.plugins.read().await;
        
        match index.get(name) {
            Some(ids) if !ids.is_empty() => {
                // Return first plugin with the given name
                plugins.get(&ids[0]).cloned()
            }
            _ => None,
        }
    }
    
    /// List all plugins
    pub async fn list_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }
    
    /// Resolve plugin dependencies
    pub async fn resolve_dependencies(&self, id: Uuid) -> Result<Vec<Arc<dyn Plugin>>> {
        let deps = {
            let deps_map = self.dependencies.read().await;
            match deps_map.get(&id) {
                Some(deps) => deps.clone(),
                None => Vec::new(),
            }
        };
        
        let mut result = Vec::new();
        let plugins = self.plugins.read().await;
        
        for dep in deps {
            match plugins.get(&dep.id) {
                Some(plugin) => result.push(plugin.clone()),
                None => {
                    if !dep.optional {
                        return Err(PluginError::DependencyNotFoundUuid(dep.id).into());
                    }
                }
            }
        }
        
        Ok(result)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
} 