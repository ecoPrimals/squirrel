// Plugin registry for monitoring system
//
// This module provides a registry for monitoring plugins to be registered,
// discovered, and managed throughout their lifecycle.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tracing::{debug, error, info};
use uuid::Uuid;

use super::common::MonitoringPlugin;

/// Plugin registry for monitoring plugins
///
/// The registry manages the lifecycle of plugins, including:
/// - Registration of plugins
/// - Discovery of plugins by ID or capability
/// - Initialization of plugins
/// - Shutdown of plugins
/// - Plugin state management
#[derive(Debug, Default)]
pub struct PluginRegistry {
    /// Registered plugins
    plugins: RwLock<HashMap<Uuid, Arc<dyn MonitoringPlugin>>>,
    
    /// Plugin capability index for fast lookup
    capability_index: RwLock<HashMap<String, Vec<Uuid>>>,
    
    /// Initialized plugin tracking
    initialized: RwLock<Vec<Uuid>>,
    
    /// Plugin activation status
    active: RwLock<HashMap<Uuid, bool>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            capability_index: RwLock::new(HashMap::new()),
            initialized: RwLock::new(Vec::new()),
            active: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a plugin with the registry
    pub async fn register_plugin<T>(&self, plugin: Arc<T>) -> Result<()>
    where
        T: MonitoringPlugin + 'static,
    {
        let metadata = plugin.metadata();
        let plugin_id = metadata.id;
        let plugin_name = metadata.name.clone();
        
        // Check if plugin is already registered
        if self.plugins.read().unwrap().contains_key(&plugin_id) {
            return Err(anyhow!("Plugin with ID {} is already registered", plugin_id));
        }
        
        // Register plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin_id, plugin.clone() as Arc<dyn MonitoringPlugin>);
        }
        
        // Index plugin capabilities
        {
            let mut capability_index = self.capability_index.write().unwrap();
            for capability in &metadata.capabilities {
                let entry = capability_index.entry(capability.clone()).or_default();
                entry.push(plugin_id);
            }
        }
        
        // Mark plugin as not active initially
        {
            let mut active = self.active.write().unwrap();
            active.insert(plugin_id, false);
        }
        
        info!("Registered plugin: {} ({})", plugin_name, plugin_id);
        Ok(())
    }
    
    /// Initialize a plugin by ID
    pub async fn initialize_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.get_plugin_by_id(plugin_id)?;
        
        // Initialize plugin
        plugin.initialize().await?;
        
        // Mark plugin as initialized
        {
            let mut initialized = self.initialized.write().unwrap();
            if !initialized.contains(&plugin_id) {
                initialized.push(plugin_id);
            }
        }
        
        // Mark plugin as active
        {
            let mut active = self.active.write().unwrap();
            active.insert(plugin_id, true);
        }
        
        debug!("Plugin initialized: {}", plugin_id);
        Ok(())
    }
    
    /// Initialize all registered plugins
    pub async fn initialize_all_plugins(&self) -> Result<()> {
        let plugin_ids: Vec<Uuid> = {
            self.plugins.read()
                .unwrap()
                .keys()
                .cloned()
                .collect()
        };
        
        for plugin_id in plugin_ids {
            if let Err(e) = self.initialize_plugin(plugin_id).await {
                error!("Failed to initialize plugin {}: {}", plugin_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Shutdown a plugin by ID
    pub async fn shutdown_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Get plugin
        let plugin = self.get_plugin_by_id(plugin_id)?;
        
        // Shutdown plugin
        plugin.shutdown().await?;
        
        // Mark plugin as not initialized
        {
            let mut initialized = self.initialized.write().unwrap();
            initialized.retain(|id| *id != plugin_id);
        }
        
        // Mark plugin as not active
        {
            let mut active = self.active.write().unwrap();
            active.insert(plugin_id, false);
        }
        
        debug!("Plugin shutdown: {}", plugin_id);
        Ok(())
    }
    
    /// Shutdown all initialized plugins
    pub async fn shutdown_all_plugins(&self) -> Result<()> {
        let plugin_ids: Vec<Uuid> = {
            self.initialized.read()
                .unwrap()
                .clone()
        };
        
        for plugin_id in plugin_ids {
            if let Err(e) = self.shutdown_plugin(plugin_id).await {
                error!("Failed to shutdown plugin {}: {}", plugin_id, e);
            }
        }
        
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get_plugin_by_id(&self, plugin_id: Uuid) -> Result<Arc<dyn MonitoringPlugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(&plugin_id)
            .cloned()
            .ok_or_else(|| anyhow!("Plugin with ID {} not found", plugin_id))
    }
    
    /// Get plugins by capability
    pub fn get_plugins_by_capability(&self, capability: &str) -> Vec<Arc<dyn MonitoringPlugin>> {
        let capability_index = self.capability_index.read().unwrap();
        let plugins = self.plugins.read().unwrap();
        
        match capability_index.get(capability) {
            Some(plugin_ids) => {
                plugin_ids.iter()
                    .filter_map(|id| plugins.get(id).cloned())
                    .collect()
            }
            None => Vec::new(),
        }
    }
    
    /// Get all active plugins
    pub fn get_active_plugins(&self) -> Vec<Arc<dyn MonitoringPlugin>> {
        let plugins = self.plugins.read().unwrap();
        let active = self.active.read().unwrap();
        
        active.iter()
            .filter_map(|(id, is_active)| {
                if *is_active {
                    plugins.get(id).cloned()
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get the total number of registered plugins
    pub fn plugin_count(&self) -> usize {
        self.plugins.read().unwrap().len()
    }
    
    /// Get the number of active plugins
    pub fn active_plugin_count(&self) -> usize {
        self.active.read().unwrap().values().filter(|v| **v).count()
    }
    
    /// Check if a plugin is active
    pub fn is_plugin_active(&self, plugin_id: Uuid) -> bool {
        self.active.read().unwrap().get(&plugin_id).copied().unwrap_or(false)
    }
}

#[async_trait]
impl super::MonitoringPluginRegistry for PluginRegistry {
    async fn register_monitoring_plugin<T>(&self, plugin: Arc<T>) -> Result<()>
    where
        T: MonitoringPlugin + Send + Sync + 'static,
    {
        self.register_plugin(plugin).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use serde_json::json;
    
    // Mock plugin for testing
    #[derive(Debug)]
    struct MockPlugin {
        metadata: PluginMetadata,
        initialized: std::sync::atomic::AtomicBool,
        shutdown: std::sync::atomic::AtomicBool,
    }
    
    impl MockPlugin {
        fn new(name: &str, capabilities: Vec<&str>) -> Self {
            let mut metadata = PluginMetadata::new(
                name,
                "1.0.0",
                "Mock plugin for testing",
                "Test Author",
            );
            
            for capability in capabilities {
                metadata = metadata.with_capability(capability);
            }
            
            Self {
                metadata,
                initialized: std::sync::atomic::AtomicBool::new(false),
                shutdown: std::sync::atomic::AtomicBool::new(false),
            }
        }
    }
    
    #[async_trait]
    impl MonitoringPlugin for MockPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            self.initialized.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            self.shutdown.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        
        async fn collect_metrics(&self) -> Result<serde_json::Value> {
            Ok(json!({
                "test": "metrics"
            }))
        }
        
        fn get_monitoring_targets(&self) -> Vec<String> {
            vec!["test".to_string()]
        }
        
        async fn handle_alert(&self, _alert: serde_json::Value) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_plugin_registration() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("Test Plugin", vec!["test"]));
        
        // Register plugin
        assert!(registry.register_plugin(plugin.clone()).await.is_ok());
        
        // Verify plugin count
        assert_eq!(registry.plugin_count(), 1);
        
        // Try to register the same plugin again (should fail)
        assert!(registry.register_plugin(plugin.clone()).await.is_err());
    }
    
    #[tokio::test]
    async fn test_plugin_initialization() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("Test Plugin", vec!["test"]));
        let plugin_id = plugin.metadata().id;
        
        // Register plugin
        registry.register_plugin(plugin.clone()).await.unwrap();
        
        // Initialize plugin
        assert!(registry.initialize_plugin(plugin_id).await.is_ok());
        
        // Verify plugin is active
        assert!(registry.is_plugin_active(plugin_id));
        assert_eq!(registry.active_plugin_count(), 1);
    }
    
    #[tokio::test]
    async fn test_plugin_capability_lookup() {
        let registry = PluginRegistry::new();
        
        // Create plugins with different capabilities
        let plugin1 = Arc::new(MockPlugin::new("Plugin 1", vec!["metrics", "system"]));
        let plugin2 = Arc::new(MockPlugin::new("Plugin 2", vec!["metrics", "network"]));
        let plugin3 = Arc::new(MockPlugin::new("Plugin 3", vec!["alerts"]));
        
        // Register plugins
        registry.register_plugin(plugin1.clone()).await.unwrap();
        registry.register_plugin(plugin2.clone()).await.unwrap();
        registry.register_plugin(plugin3.clone()).await.unwrap();
        
        // Get plugins by capability
        let metrics_plugins = registry.get_plugins_by_capability("metrics");
        let system_plugins = registry.get_plugins_by_capability("system");
        let network_plugins = registry.get_plugins_by_capability("network");
        let alerts_plugins = registry.get_plugins_by_capability("alerts");
        let nonexistent_plugins = registry.get_plugins_by_capability("nonexistent");
        
        // Verify capability lookup
        assert_eq!(metrics_plugins.len(), 2);
        assert_eq!(system_plugins.len(), 1);
        assert_eq!(network_plugins.len(), 1);
        assert_eq!(alerts_plugins.len(), 1);
        assert_eq!(nonexistent_plugins.len(), 0);
    }
    
    #[tokio::test]
    async fn test_plugin_shutdown() {
        let registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("Test Plugin", vec!["test"]));
        let plugin_id = plugin.metadata().id;
        
        // Register and initialize plugin
        registry.register_plugin(plugin.clone()).await.unwrap();
        registry.initialize_plugin(plugin_id).await.unwrap();
        
        // Shutdown plugin
        assert!(registry.shutdown_plugin(plugin_id).await.is_ok());
        
        // Verify plugin is not active
        assert!(!registry.is_plugin_active(plugin_id));
        assert_eq!(registry.active_plugin_count(), 0);
    }
} 