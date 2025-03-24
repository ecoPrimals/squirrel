// Plugin loader for monitoring system
//
// This module provides functionality for loading plugins from different sources,
// including built-in plugins, external crates, and configuration.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use tracing::{debug, error, info, warn};

use super::registry::PluginRegistry;
use super::common::MonitoringPlugin;
use super::{SystemMetricsPlugin, HealthReporterPlugin, AlertHandlerPlugin};

/// Plugin loader for monitoring plugins
///
/// The loader provides functionality for:
/// - Loading built-in plugins
/// - Loading externally defined plugins
/// - Loading plugins from configuration
#[derive(Debug)]
pub struct PluginLoader {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    
    /// Plugin configurations
    configs: HashMap<String, PluginConfig>,
    
    /// Whether built-in plugins are loaded
    built_in_loaded: bool,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin ID or name
    pub id: String,
    
    /// Whether the plugin is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Plugin-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_enabled() -> bool {
    true
}

impl PluginLoader {
    /// Create a new plugin loader with the given registry
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            registry,
            configs: HashMap::new(),
            built_in_loaded: false,
        }
    }
    
    /// Load built-in plugins
    pub async fn load_built_in_plugins(&mut self) -> Result<()> {
        if self.built_in_loaded {
            debug!("Built-in plugins already loaded");
            return Ok(());
        }
        
        // System metrics plugin
        let system_metrics = Arc::new(SystemMetricsPlugin::new());
        self.registry.register_plugin(system_metrics).await
            .context("Failed to register system metrics plugin")?;
        
        // Health reporter plugin
        let health_reporter = Arc::new(HealthReporterPlugin::new());
        self.registry.register_plugin(health_reporter).await
            .context("Failed to register health reporter plugin")?;
        
        // Alert handler plugin
        let alert_handler = Arc::new(AlertHandlerPlugin::new());
        self.registry.register_plugin(alert_handler).await
            .context("Failed to register alert handler plugin")?;
        
        self.built_in_loaded = true;
        info!("Loaded built-in plugins");
        
        Ok(())
    }
    
    /// Add a plugin configuration
    pub fn add_config(&mut self, config: PluginConfig) {
        self.configs.insert(config.id.clone(), config);
    }
    
    /// Add multiple plugin configurations
    pub fn add_configs(&mut self, configs: Vec<PluginConfig>) {
        for config in configs {
            self.add_config(config);
        }
    }
    
    /// Load configurations from a file
    pub fn load_configs_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = std::fs::File::open(path)
            .context("Failed to open plugin configuration file")?;
        
        let configs: Vec<PluginConfig> = serde_json::from_reader(file)
            .context("Failed to parse plugin configurations")?;
        
        self.add_configs(configs);
        Ok(())
    }
    
    /// Configure a plugin with its configuration
    pub async fn configure_plugin<T>(&self, plugin: Arc<T>, _config: &PluginConfig) -> Result<()>
    where
        T: MonitoringPlugin + 'static,
    {
        // If plugin has a custom configuration method, use it
        // Otherwise, initialize with default configuration
        plugin.initialize().await
            .context("Failed to initialize plugin")?;
        
        debug!("Configured plugin: {}", plugin.metadata().name);
        Ok(())
    }
    
    /// Load and initialize all enabled plugins from configurations
    pub async fn load_and_initialize_plugins(&self) -> Result<()> {
        // Get all registered plugins
        let plugins = self.registry.get_active_plugins();
        
        // Initialize plugins with their configurations
        for plugin in plugins {
            let metadata = plugin.metadata();
            let plugin_name = &metadata.name;
            
            // Look up configuration by plugin name or ID
            let config = self.configs.get(plugin_name)
                .or_else(|| self.configs.get(&metadata.id.to_string()));
            
            if let Some(config) = config {
                if !config.enabled {
                    warn!("Plugin {} is disabled in configuration", plugin_name);
                    continue;
                }
                
                // Initialize plugin with configuration
                plugin.initialize().await
                    .with_context(|| format!("Failed to initialize plugin {}", plugin_name))?;
                
                info!("Initialized plugin: {}", plugin_name);
            } else {
                // No configuration found, use default configuration
                plugin.initialize().await
                    .with_context(|| format!("Failed to initialize plugin {}", plugin_name))?;
                
                debug!("Initialized plugin with default configuration: {}", plugin_name);
            }
        }
        
        Ok(())
    }
    
    /// Load a custom plugin implementation
    pub async fn load_custom_plugin<T>(&self, plugin: Arc<T>) -> Result<()>
    where
        T: MonitoringPlugin + 'static,
    {
        // Register plugin with registry
        self.registry.register_plugin(plugin.clone()).await
            .context("Failed to register custom plugin")?;
        
        // Get plugin metadata
        let metadata = plugin.metadata();
        let plugin_name = metadata.name.clone(); // Clone the name to avoid borrowing issues
        
        // Look up configuration by plugin name or ID
        let config = self.configs.get(&metadata.name)
            .or_else(|| self.configs.get(&metadata.id.to_string()));
        
        // Initialize plugin with configuration if available
        if let Some(config) = config {
            if !config.enabled {
                warn!("Custom plugin {} is disabled in configuration", plugin_name);
                return Ok(());
            }
            
            // Use clone to avoid borrowing issues
            self.configure_plugin(plugin.clone(), config).await
                .with_context(|| format!("Failed to configure custom plugin {}", plugin_name))?;
        } else {
            // No configuration found, use default configuration
            plugin.initialize().await
                .with_context(|| format!("Failed to initialize custom plugin {}", plugin_name))?;
            
            debug!("Initialized custom plugin with default configuration: {}", plugin_name);
        }
        
        info!("Loaded custom plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Get the plugin registry
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use async_trait::async_trait;
    
    // Custom test plugin
    #[derive(Debug)]
    struct TestPlugin {
        metadata: super::super::common::PluginMetadata,
        initialized: AtomicBool,
    }
    
    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                metadata: super::super::common::PluginMetadata::new(
                    name,
                    "1.0.0",
                    "Test plugin",
                    "Test Author",
                ),
                initialized: AtomicBool::new(false),
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
        
        async fn collect_metrics(&self) -> Result<serde_json::Value> {
            Ok(json!({ "test": "metrics" }))
        }
        
        fn get_monitoring_targets(&self) -> Vec<String> {
            vec!["test".to_string()]
        }
        
        async fn handle_alert(&self, _alert: serde_json::Value) -> Result<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_load_built_in_plugins() {
        let registry = Arc::new(PluginRegistry::new());
        let mut loader = PluginLoader::new(registry.clone());
        
        // Load built-in plugins
        loader.load_built_in_plugins().await.unwrap();
        
        // Verify plugins were loaded
        assert!(registry.plugin_count() > 0);
        assert_eq!(registry.plugin_count(), 3); // System metrics, health reporter, alert handler
    }
    
    #[tokio::test]
    async fn test_load_custom_plugin() {
        let registry = Arc::new(PluginRegistry::new());
        let loader = PluginLoader::new(registry.clone());
        
        // Create custom plugin
        let plugin = Arc::new(TestPlugin::new("Custom Plugin"));
        
        // Load custom plugin
        loader.load_custom_plugin(plugin.clone()).await.unwrap();
        
        // Verify plugin was loaded and initialized
        assert_eq!(registry.plugin_count(), 1);
        assert!(plugin.initialized.load(Ordering::SeqCst));
    }
    
    #[tokio::test]
    async fn test_plugin_configuration() {
        let registry = Arc::new(PluginRegistry::new());
        let mut loader = PluginLoader::new(registry.clone());
        
        // Add configuration
        let config = PluginConfig {
            id: "Custom Plugin".to_string(),
            enabled: true,
            config: json!({
                "setting1": "value1",
                "setting2": 42
            }),
        };
        
        loader.add_config(config);
        
        // Create custom plugin
        let plugin = Arc::new(TestPlugin::new("Custom Plugin"));
        
        // Load custom plugin
        loader.load_custom_plugin(plugin.clone()).await.unwrap();
        
        // Verify plugin was loaded and initialized
        assert_eq!(registry.plugin_count(), 1);
        assert!(plugin.initialized.load(Ordering::SeqCst));
    }
} 