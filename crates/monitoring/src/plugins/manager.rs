// Plugin manager for monitoring system
//
// This module provides a high-level interface for managing monitoring plugins,
// including loading, initialization, and usage.

use std::sync::Arc;
use std::sync::RwLock;
use std::collections::HashMap;
use anyhow::{Result, Context};
use tracing::{debug, error, info, warn};
use serde_json::Value;
use uuid::Uuid;

use super::common::MonitoringPlugin;
use super::registry::PluginRegistry;
use super::loader::{PluginLoader, PluginConfig};

/// Plugin manager for monitoring system
///
/// This provides a high-level interface for using plugins in the monitoring system,
/// including:
/// - Plugin registration and discovery
/// - Plugin lifecycle management
/// - Plugin configuration management
/// - Metrics collection
/// - Alert handling
#[derive(Debug)]
pub struct PluginManager {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    
    /// Plugin loader
    loader: RwLock<PluginLoader>,
    
    /// Last collected metrics per plugin
    last_metrics: RwLock<HashMap<Uuid, Value>>,
    
    /// Plugin state information
    plugin_state: RwLock<HashMap<Uuid, PluginState>>,
}

/// Plugin state information
#[derive(Debug, Clone)]
pub struct PluginState {
    /// Whether the plugin is enabled
    pub enabled: bool,
    
    /// Whether the plugin is initialized
    pub initialized: bool,
    
    /// Last initialization time
    pub last_init: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Last error message
    pub last_error: Option<String>,
    
    /// Last successful metrics collection time
    pub last_metrics_collection: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Last successful alert handling time
    pub last_alert_handling: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            last_init: None,
            last_error: None,
            last_metrics_collection: None,
            last_alert_handling: None,
        }
    }
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        let registry = Arc::new(PluginRegistry::new());
        let loader = PluginLoader::new(registry.clone());
        
        Self {
            registry,
            loader: RwLock::new(loader),
            last_metrics: RwLock::new(HashMap::new()),
            plugin_state: RwLock::new(HashMap::new()),
        }
    }
    
    /// Initialize the plugin manager
    pub async fn initialize(&self) -> Result<()> {
        // Load built-in plugins
        {
            let mut loader = self.loader.write().unwrap();
            loader.load_built_in_plugins().await
                .context("Failed to load built-in plugins")?;
        }
        
        // Initialize all plugins
        let plugins = self.registry.get_active_plugins();
        for plugin in plugins {
            let metadata = plugin.metadata();
            let plugin_id = metadata.id;
            
            // Initialize plugin
            if let Err(e) = plugin.initialize().await {
                error!("Failed to initialize plugin {}: {}", metadata.name, e);
                
                // Update plugin state
                let mut plugin_state = self.plugin_state.write().unwrap();
                let state = plugin_state.entry(plugin_id).or_default();
                state.initialized = false;
                state.last_error = Some(e.to_string());
                
                continue;
            }
            
            // Update plugin state
            {
                let mut plugin_state = self.plugin_state.write().unwrap();
                let state = plugin_state.entry(plugin_id).or_default();
                state.initialized = true;
                state.last_init = Some(chrono::Utc::now());
                state.last_error = None;
            }
            
            info!("Initialized plugin: {}", metadata.name);
        }
        
        Ok(())
    }
    
    /// Shut down all plugins
    pub async fn shutdown(&self) -> Result<()> {
        // Shut down all plugins
        self.registry.shutdown_all_plugins().await
            .context("Failed to shut down all plugins")?;
        
        // Clear plugin state
        {
            let mut plugin_state = self.plugin_state.write().unwrap();
            for (_, state) in plugin_state.iter_mut() {
                state.initialized = false;
            }
        }
        
        info!("Shut down all plugins");
        Ok(())
    }
    
    /// Load a plugin configuration
    pub fn load_config(&self, config: PluginConfig) {
        let mut loader = self.loader.write().unwrap();
        loader.add_config(config);
    }
    
    /// Load plugin configurations from a file
    pub fn load_configs_from_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut loader = self.loader.write().unwrap();
        loader.load_configs_from_file(path)?;
        Ok(())
    }
    
    /// Register a custom plugin
    pub async fn register_plugin<T>(&self, plugin: Arc<T>) -> Result<()>
    where
        T: MonitoringPlugin + 'static,
    {
        // Register plugin with registry
        self.registry.register_plugin(plugin.clone()).await
            .context("Failed to register custom plugin")?;
        
        // Add plugin state
        let metadata = plugin.metadata();
        let plugin_id = metadata.id;
        {
            let mut plugin_state = self.plugin_state.write().unwrap();
            plugin_state.insert(plugin_id, PluginState::default());
        }
        
        info!("Registered plugin: {}", metadata.name);
        Ok(())
    }
    
    /// Initialize a plugin
    pub async fn initialize_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Initialize plugin
        self.registry.initialize_plugin(plugin_id).await
            .with_context(|| format!("Failed to initialize plugin {}", plugin_id))?;
        
        // Update plugin state
        {
            let mut plugin_state = self.plugin_state.write().unwrap();
            let state = plugin_state.entry(plugin_id).or_default();
            state.initialized = true;
            state.last_init = Some(chrono::Utc::now());
            state.last_error = None;
        }
        
        debug!("Initialized plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Enable a plugin
    pub fn enable_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Check if plugin exists
        let _ = self.registry.get_plugin_by_id(plugin_id)?;
        
        // Update plugin state
        {
            let mut plugin_state = self.plugin_state.write().unwrap();
            let state = plugin_state.entry(plugin_id).or_default();
            state.enabled = true;
        }
        
        debug!("Enabled plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Disable a plugin
    pub fn disable_plugin(&self, plugin_id: Uuid) -> Result<()> {
        // Check if plugin exists
        let _ = self.registry.get_plugin_by_id(plugin_id)?;
        
        // Update plugin state
        {
            let mut plugin_state = self.plugin_state.write().unwrap();
            let state = plugin_state.entry(plugin_id).or_default();
            state.enabled = false;
        }
        
        debug!("Disabled plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Collect metrics from all enabled plugins
    pub async fn collect_metrics(&self) -> Result<HashMap<String, Value>> {
        let mut metrics = HashMap::new();
        
        // Get all enabled plugins
        let plugins = self.get_enabled_plugins();
        
        // Collect metrics from each plugin
        for plugin in plugins {
            let metadata = plugin.metadata();
            let plugin_id = metadata.id;
            
            // Skip uninitialized plugins
            if !self.is_plugin_initialized(plugin_id) {
                continue;
            }
            
            // Collect metrics from plugin
            match plugin.collect_metrics().await {
                Ok(plugin_metrics) => {
                    // Update last metrics
                    {
                        let mut last_metrics = self.last_metrics.write().unwrap();
                        last_metrics.insert(plugin_id, plugin_metrics.clone());
                    }
                    
                    // Update plugin state
                    {
                        let mut plugin_state = self.plugin_state.write().unwrap();
                        if let Some(state) = plugin_state.get_mut(&plugin_id) {
                            state.last_metrics_collection = Some(chrono::Utc::now());
                            state.last_error = None;
                        }
                    }
                    
                    // Add metrics to result
                    metrics.insert(metadata.name.clone(), plugin_metrics);
                }
                Err(e) => {
                    error!("Failed to collect metrics from plugin {}: {}", metadata.name, e);
                    
                    // Update plugin state
                    {
                        let mut plugin_state = self.plugin_state.write().unwrap();
                        if let Some(state) = plugin_state.get_mut(&plugin_id) {
                            state.last_error = Some(e.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(metrics)
    }
    
    /// Handle an alert with all enabled plugins
    pub async fn handle_alert(&self, alert: Value) -> Result<()> {
        // Get all enabled plugins
        let plugins = self.get_enabled_plugins();
        
        // Handle alert with each plugin
        for plugin in plugins {
            let metadata = plugin.metadata();
            let plugin_id = metadata.id;
            
            // Skip uninitialized plugins
            if !self.is_plugin_initialized(plugin_id) {
                continue;
            }
            
            // Handle alert with plugin
            if let Err(e) = plugin.handle_alert(alert.clone()).await {
                error!("Failed to handle alert with plugin {}: {}", metadata.name, e);
                
                // Update plugin state
                {
                    let mut plugin_state = self.plugin_state.write().unwrap();
                    if let Some(state) = plugin_state.get_mut(&plugin_id) {
                        state.last_error = Some(e.to_string());
                    }
                }
            } else {
                // Update plugin state
                {
                    let mut plugin_state = self.plugin_state.write().unwrap();
                    if let Some(state) = plugin_state.get_mut(&plugin_id) {
                        state.last_alert_handling = Some(chrono::Utc::now());
                        state.last_error = None;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get all enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<Arc<dyn MonitoringPlugin>> {
        let plugin_state = self.plugin_state.read().unwrap();
        let plugins = self.registry.get_active_plugins();
        
        plugins.into_iter()
            .filter(|plugin| {
                let plugin_id = plugin.metadata().id;
                plugin_state.get(&plugin_id)
                    .map(|state| state.enabled)
                    .unwrap_or(true)
            })
            .collect()
    }
    
    /// Get plugin state
    pub fn get_plugin_state(&self, plugin_id: Uuid) -> Option<PluginState> {
        let plugin_state = self.plugin_state.read().unwrap();
        plugin_state.get(&plugin_id).cloned()
    }
    
    /// Get all plugin states
    pub fn get_all_plugin_states(&self) -> HashMap<Uuid, PluginState> {
        let plugin_state = self.plugin_state.read().unwrap();
        plugin_state.clone()
    }
    
    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, plugin_id: Uuid) -> bool {
        let plugin_state = self.plugin_state.read().unwrap();
        plugin_state.get(&plugin_id)
            .map(|state| state.enabled)
            .unwrap_or(false)
    }
    
    /// Check if a plugin is initialized
    pub fn is_plugin_initialized(&self, plugin_id: Uuid) -> bool {
        let plugin_state = self.plugin_state.read().unwrap();
        plugin_state.get(&plugin_id)
            .map(|state| state.initialized)
            .unwrap_or(false)
    }
    
    /// Get last collected metrics for a plugin
    pub fn get_last_metrics(&self, plugin_id: Uuid) -> Option<Value> {
        let last_metrics = self.last_metrics.read().unwrap();
        last_metrics.get(&plugin_id).cloned()
    }
    
    /// Get the plugin registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use async_trait::async_trait;
    use serde_json::json;
    
    // Test plugin for manager tests
    #[derive(Debug)]
    struct TestPlugin {
        metadata: super::super::common::PluginMetadata,
        initialized: AtomicBool,
        metrics: Value,
    }
    
    impl TestPlugin {
        fn new(name: &str, metrics: Value) -> Self {
            Self {
                metadata: super::super::common::PluginMetadata::new(
                    name,
                    "1.0.0",
                    "Test plugin",
                    "Test Author",
                ),
                initialized: AtomicBool::new(false),
                metrics,
            }
        }
    }
    
    #[async_trait]
    impl super::super::common::MonitoringPlugin for TestPlugin {
        fn metadata(&self) -> &super::super::common::PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            self.initialized.store(true, Ordering::SeqCst);
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            self.initialized.store(false, Ordering::SeqCst);
            Ok(())
        }
        
        async fn collect_metrics(&self) -> Result<Value> {
            Ok(self.metrics.clone())
        }
        
        fn get_monitoring_targets(&self) -> Vec<String> {
            vec!["test".to_string()]
        }
        
        async fn handle_alert(&self, _alert: Value) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_plugin_manager_initialization() {
        let manager = PluginManager::new();
        
        // Initialize manager (loads built-in plugins)
        manager.initialize().await.unwrap();
        
        // Verify built-in plugins are loaded and initialized
        assert!(manager.registry.plugin_count() > 0);
    }
    
    #[tokio::test]
    async fn test_plugin_manager_custom_plugins() {
        let manager = PluginManager::new();
        
        // Register a custom plugin
        let plugin = Arc::new(TestPlugin::new(
            "Custom Plugin",
            json!({ "test": "metrics" }),
        ));
        let plugin_id = plugin.metadata().id;
        
        // Register plugin
        manager.register_plugin(plugin.clone()).await.unwrap();
        
        // Initialize plugin
        manager.initialize_plugin(plugin_id).await.unwrap();
        
        // Verify plugin state
        assert!(manager.is_plugin_enabled(plugin_id));
        assert!(manager.is_plugin_initialized(plugin_id));
        
        // Collect metrics
        let metrics = manager.collect_metrics().await.unwrap();
        assert!(metrics.contains_key("Custom Plugin"));
        
        // Verify last metrics
        let last_metrics = manager.get_last_metrics(plugin_id).unwrap();
        assert_eq!(last_metrics, json!({ "test": "metrics" }));
    }
    
    #[tokio::test]
    async fn test_plugin_manager_enable_disable() {
        let manager = PluginManager::new();
        
        // Register a custom plugin
        let plugin = Arc::new(TestPlugin::new(
            "Custom Plugin",
            json!({ "test": "metrics" }),
        ));
        let plugin_id = plugin.metadata().id;
        
        // Register and initialize plugin
        manager.register_plugin(plugin.clone()).await.unwrap();
        manager.initialize_plugin(plugin_id).await.unwrap();
        
        // Disable plugin
        manager.disable_plugin(plugin_id).unwrap();
        assert!(!manager.is_plugin_enabled(plugin_id));
        
        // Collect metrics (plugin disabled, no metrics should be collected)
        let metrics = manager.collect_metrics().await.unwrap();
        assert!(!metrics.contains_key("Custom Plugin"));
        
        // Enable plugin again
        manager.enable_plugin(plugin_id).unwrap();
        assert!(manager.is_plugin_enabled(plugin_id));
        
        // Collect metrics again (plugin enabled, metrics should be collected)
        let metrics = manager.collect_metrics().await.unwrap();
        assert!(metrics.contains_key("Custom Plugin"));
    }
} 