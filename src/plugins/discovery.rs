// Plugin Discovery Module
//
// This module provides functionality for discovering plugins.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;
use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata};

/// Plugin discovery
///
/// This trait defines the interface for discovering plugins.
#[async_trait]
pub trait PluginDiscovery: Send + Sync + Debug {
    /// Discover plugins in a directory
    async fn discover_directory(&self, path: &Path) -> Result<Vec<PluginMetadata>>;
    
    /// Discover plugins by metadata
    async fn discover_metadata(&self, metadata: &PluginMetadata) -> Result<Vec<PluginMetadata>>;
    
    /// Get all discovered plugins
    async fn get_all(&self) -> Result<Vec<PluginMetadata>>;
    
    /// Get a plugin by ID
    async fn get_by_id(&self, id: Uuid) -> Result<PluginMetadata>;
    
    /// Get plugins by capability
    async fn get_by_capability(&self, capability: &str) -> Result<Vec<PluginMetadata>>;
    
    /// Load a plugin
    async fn load_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;
    
    /// Load all plugins
    async fn load_all(&self) -> Result<Vec<Arc<dyn Plugin>>>;
}

/// Plugin discovery implementation
#[derive(Debug)]
pub struct FileSystemDiscovery {
    /// Discovered plugins
    discovered: RwLock<HashMap<Uuid, PluginMetadata>>,
    
    /// Plugin loaders
    loaders: Vec<Arc<dyn PluginLoader>>,
    
    /// Plugin cache
    cache: RwLock<HashMap<Uuid, Arc<dyn Plugin>>>,
    
    /// Root directories
    root_dirs: Vec<PathBuf>,
}

impl FileSystemDiscovery {
    /// Create a new file system discovery
    pub fn new(root_dirs: Vec<PathBuf>) -> Self {
        Self {
            discovered: RwLock::new(HashMap::new()),
            loaders: Vec::new(),
            cache: RwLock::new(HashMap::new()),
            root_dirs,
        }
    }
    
    /// Add a plugin loader
    pub fn add_loader(&mut self, loader: Arc<dyn PluginLoader>) {
        self.loaders.push(loader);
    }
    
    /// Discover plugins in a directory recursively
    async fn discover_recursive(&self, path: &Path, results: &mut Vec<PluginMetadata>) -> Result<()> {
        if !path.exists() {
            return Err(PluginError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Directory not found",
            )));
        }
        
        if !path.is_dir() {
            return Err(PluginError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path is not a directory",
            )));
        }
        
        let entries = match tokio::fs::read_dir(path).await {
            Ok(entries) => entries,
            Err(e) => return Err(PluginError::IoError(e)),
        };
        
        let mut entries_vec = Vec::new();
        
        // Collect all entries
        let mut entries = entries;
        while let Ok(Some(entry)) = entries.next_entry().await {
            entries_vec.push(entry);
        }
        
        // Process entries
        for entry in entries_vec {
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively discover plugins in subdirectories
                self.discover_recursive(&path, results).await?;
            } else {
                // Try to load plugin metadata from file
                for loader in &self.loaders {
                    if loader.can_load(&path).await? {
                        match loader.load_metadata(&path).await {
                            Ok(metadata) => {
                                results.push(metadata);
                                break;
                            }
                            Err(e) => {
                                warn!("Failed to load plugin metadata from {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Add discovered plugins to the registry
    async fn add_discovered(&self, plugins: Vec<PluginMetadata>) -> Result<Vec<PluginMetadata>> {
        let mut discovered = self.discovered.write().await;
        let mut new_plugins = Vec::new();
        
        for metadata in plugins {
            let id = metadata.id;
            
            if !discovered.contains_key(&id) {
                discovered.insert(id, metadata.clone());
                new_plugins.push(metadata);
            }
        }
        
        Ok(new_plugins)
    }
}

#[async_trait]
impl PluginDiscovery for FileSystemDiscovery {
    async fn discover_directory(&self, path: &Path) -> Result<Vec<PluginMetadata>> {
        let mut results = Vec::new();
        
        // Discover plugins recursively
        self.discover_recursive(path, &mut results).await?;
        
        // Add discovered plugins to the registry
        let new_plugins = self.add_discovered(results).await?;
        
        Ok(new_plugins)
    }
    
    async fn discover_metadata(&self, metadata: &PluginMetadata) -> Result<Vec<PluginMetadata>> {
        let id = metadata.id;
        
        // Add metadata to the registry
        let mut discovered = self.discovered.write().await;
        
        if !discovered.contains_key(&id) {
            discovered.insert(id, metadata.clone());
            Ok(vec![metadata.clone()])
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn get_all(&self) -> Result<Vec<PluginMetadata>> {
        let discovered = self.discovered.read().await;
        
        Ok(discovered.values().cloned().collect())
    }
    
    async fn get_by_id(&self, id: Uuid) -> Result<PluginMetadata> {
        let discovered = self.discovered.read().await;
        
        match discovered.get(&id) {
            Some(metadata) => Ok(metadata.clone()),
            None => Err(PluginError::NotFound(id)),
        }
    }
    
    async fn get_by_capability(&self, capability: &str) -> Result<Vec<PluginMetadata>> {
        let discovered = self.discovered.read().await;
        
        let mut result = Vec::new();
        
        for metadata in discovered.values() {
            if metadata.capabilities.contains(&capability.to_string()) {
                result.push(metadata.clone());
            }
        }
        
        Ok(result)
    }
    
    async fn load_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        // Check cache
        {
            let cache = self.cache.read().await;
            
            if let Some(plugin) = cache.get(&id) {
                return Ok(plugin.clone());
            }
        }
        
        // Get metadata
        let metadata = self.get_by_id(id).await?;
        
        // Find a suitable loader
        for loader in &self.loaders {
            match loader.load_plugin(&metadata).await {
                Ok(plugin) => {
                    // Cache the plugin
                    let mut cache = self.cache.write().await;
                    cache.insert(id, plugin.clone());
                    
                    return Ok(plugin);
                }
                Err(e) => {
                    warn!("Failed to load plugin {}: {}", id, e);
                }
            }
        }
        
        Err(PluginError::LoadError(format!(
            "No suitable loader found for plugin {}",
            id
        )))
    }
    
    async fn load_all(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let discovered = self.discovered.read().await;
        
        let mut plugins = Vec::new();
        
        for id in discovered.keys() {
            match self.load_plugin(*id).await {
                Ok(plugin) => {
                    plugins.push(plugin);
                }
                Err(e) => {
                    warn!("Failed to load plugin {}: {}", id, e);
                }
            }
        }
        
        Ok(plugins)
    }
}

/// Plugin loader
///
/// This trait defines the interface for loading plugins.
#[async_trait]
pub trait PluginLoader: Send + Sync + Debug {
    /// Check if this loader can load a plugin from a path
    async fn can_load(&self, path: &Path) -> Result<bool>;
    
    /// Load plugin metadata from a path
    async fn load_metadata(&self, path: &Path) -> Result<PluginMetadata>;
    
    /// Load a plugin
    async fn load_plugin(&self, metadata: &PluginMetadata) -> Result<Arc<dyn Plugin>>;
} 