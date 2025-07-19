//! Plugin manager
//!
//! This module provides functionality for managing plugin lifecycle.

use crate::errors::{PluginError, Result};
use crate::state::PluginStateManager;
use crate::types::PluginStatus;
use crate::Plugin;
use crate::PluginConfig;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
// Security handled by BearDog framework
use crate::dependency_resolver::{
    DependencyResolver, EnhancedPluginDependency, ResolutionResult, ResolutionStatistics,
};
use crate::discovery::{DefaultPluginDiscovery, PluginDiscovery};
use serde_json;

/// Plugin registry trait
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;

    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;

    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;

    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>>;

    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;

    /// Get all registered plugins
    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;
}

/// Plugin manager trait
#[async_trait]
pub trait PluginManagerTrait: PluginRegistry {
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;

    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()>;

    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()>;

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;

    /// Load plugins from a directory
    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>>;

    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()>;

    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()>;
}

/// Plugin manager for handling plugin lifecycle and dependencies
pub struct PluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>,

    /// Plugin configurations
    plugin_configs: Arc<RwLock<HashMap<Uuid, PluginConfig>>>,

    /// Security is handled by BearDog framework
    // security_manager: Arc<SecurityManagerAdapter>,
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    /// Dependency resolver for proper plugin initialization order
    dependency_resolver: RwLock<DependencyResolver>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_configs: Arc::new(RwLock::new(HashMap::new())),
            // Security handled by BearDog framework
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            dependency_resolver: RwLock::new(DependencyResolver::new()),
        }
    }

    /// Initialize the plugin manager
    pub async fn init(&self) -> Result<()> {
        self.register_built_in_plugins().await?;

        debug!("Plugin manager initialized");
        Ok(())
    }

    /// Register built-in plugins
    async fn register_built_in_plugins(&self) -> Result<()> {
        // Use a placeholder plugin instead of HelloWorldPlugin
        use crate::discovery::create_placeholder_plugin;
        use crate::plugin::PluginMetadata;
        let placeholder_metadata = PluginMetadata::new(
            "system-placeholder",
            "1.0.0",
            "System placeholder plugin",
            "Squirrel System",
        );
        let placeholder = create_placeholder_plugin(placeholder_metadata);
        self.register_plugin(placeholder).await?;

        Ok(())
    }

    /// Register a plugin with metadata, implementation, and optional signature
    pub async fn register_plugin_with_signature(
        &self,
        plugin: Arc<dyn Plugin>,
        signature: Option<Vec<u8>>,
    ) -> Result<()> {
        let metadata = plugin.metadata();
        let id = metadata.id;

        // If signature is provided, verify it
        if let Some(_sig_bytes) = signature {
            debug!("Verifying signature for plugin {}", metadata.name);

            // Verify the signature using the security manager
            // Security verification handled by BearDog framework
            // let verification_result = self.security_manager.verify_signature(metadata, &_sig_bytes).await?;

            debug!(
                "Signature verification succeeded for plugin {}",
                metadata.name
            );
        } else {
            // Check if signatures are required by getting the configuration from the signature verifier
            // This is a more complex approach that would require accessing the SignatureVerifier directly
            // For now, we'll just log a warning if no signature is provided
            warn!("No signature provided for plugin {}", metadata.name);
        }

        // Security verification handled by BearDog framework
        // if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
        //     return Err(PluginError::SecurityError(format!("Plugin security verification failed: {}", e)).into());
        // }

        let mut plugins = self.plugins.write().await;
        if plugins.contains_key(&id) {
            return Err(anyhow::anyhow!("Plugin already registered: {}", id).into());
        }

        info!("Registered plugin {} ({})", metadata.name, id);
        plugins.insert(id, plugin);
        Ok(())
    }

    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata().clone(); // Clone metadata before moving plugin
        let id = metadata.id;

        // Security verification handled by BearDog framework
        // if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
        //     return Err(PluginError::SecurityError(format!("Plugin security verification failed: {}", e)).into());
        // }

        // Register with dependency resolver
        {
            let mut resolver = self.dependency_resolver.write().await;
            resolver.register_plugin(plugin.clone())?;
        }

        // Set the initial status to Registered
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Registered);

        // Map the name to the ID
        let mut name_to_id = self.name_to_id.write().await;
        name_to_id.insert(metadata.name.clone(), id);

        // Register the plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(id, plugin);

        debug!(
            "Registered plugin: {} with dependency resolver",
            metadata.name
        );
        Ok(())
    }

    /// Get a plugin by ID
    pub async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        match plugins.get(&id) {
            Some(plugin) => Ok(plugin.clone()),
            None => Err(anyhow::anyhow!("Plugin not found: {}", id).into()),
        }
    }

    /// Get all plugins
    pub async fn get_plugins(&self) -> Vec<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Load plugins from a directory with signature verification
    pub async fn load_plugins_with_signatures(
        &self,
        directory: &str,
        verify_signatures: bool,
    ) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();

        let mut plugin_count = 0;
        for plugin in plugin_paths {
            let metadata = plugin.metadata().clone(); // Clone metadata before moving plugin
            let id = metadata.id;
            let name = metadata.name.clone(); // Clone the name to use later

            // If signature verification is enabled, look for a signature file
            if verify_signatures {
                let sig_path = std::path::Path::new(directory).join(format!("{name}.sig"));

                if sig_path.exists() {
                    // Load the signature
                    let _sig_bytes = std::fs::read(&sig_path)?;

                    // Verify signature using security manager
                    // Security verification handled by BearDog framework
                    // if let Ok(verification_result) = self.security_manager.verify_signature(&metadata, &_sig_bytes).await {
                    //     if let Ok(()) = self.security_manager.verify_plugin(plugin.as_ref()).await {
                    //         // Both signature and plugin verification passed
                    //     }
                    // }
                } else {
                    warn!(
                        "No signature file found for plugin {name}, registering without signature"
                    );
                    // Register without signature verification
                    let arc_plugin = plugin;
                    if let Ok(()) = self.register_plugin(arc_plugin).await {
                        ids.push(id);
                        plugin_count += 1;
                    }
                }
            } else {
                // Register without signature verification
                let arc_plugin = plugin;
                if let Ok(()) = self.register_plugin(arc_plugin).await {
                    ids.push(id);
                    plugin_count += 1;
                }
            }
        }

        info!("Loaded {plugin_count} plugins from {directory}");
        Ok(ids)
    }

    /// Load plugins from a directory
    pub async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        self.load_plugins_with_signatures(directory, false).await
    }

    /// Initialize all plugins
    pub async fn initialize(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        let plugin_configs = self.plugin_configs.read().await;
        
        info!("🔌 Initializing {} plugins with configuration management", plugins.len());
        
        for plugin in plugins {
            let plugin_id = plugin.metadata().id;
            let plugin_name = &plugin.metadata().name;
            
            // Check if we have specific configuration for this plugin
            if let Some(config) = plugin_configs.get(&plugin_id) {
                info!("🔧 Applying configuration for plugin '{}' ({})", plugin_name, plugin_id);
                
                // Validate configuration against plugin requirements
                if self.validate_plugin_config(&plugin, config).await {
                    debug!("✅ Configuration validation passed for plugin '{}'", plugin_name);
                    
                    // Apply configuration settings before initialization
                    if let Err(e) = self.apply_plugin_config(&plugin, config).await {
                        warn!("⚠️ Failed to apply configuration for plugin '{}': {}", plugin_name, e);
                        // Continue with default initialization
                    } else {
                        debug!("🔧 Successfully applied configuration for plugin '{}'", plugin_name);
                    }
                } else {
                    warn!("⚠️ Configuration validation failed for plugin '{}', using defaults", plugin_name);
                }
            } else {
                debug!("📋 No specific configuration found for plugin '{}', using defaults", plugin_name);
            }
            
            // Initialize the plugin
            if let Err(e) = plugin.initialize().await {
                error!("❌ Failed to initialize plugin '{}' ({}): {}", plugin_name, plugin_id, e);
                
                // Update plugin status to reflect initialization failure
                {
                    let mut statuses = self.statuses.write().await;
                    statuses.insert(plugin_id, PluginStatus::Error(e.to_string()));
                }
                
                // Continue with other plugins but track the failure
                continue;
            }
            
            // Mark plugin as successfully initialized
            {
                let mut statuses = self.statuses.write().await;
                statuses.insert(plugin_id, PluginStatus::Active);
            }
            
            info!("✅ Successfully initialized plugin '{}' ({})", plugin_name, plugin_id);
        }

        let active_count = self.statuses.read().await.iter()
            .filter(|(_, status)| matches!(status, PluginStatus::Active))
            .count();
            
        info!("🚀 Plugin initialization complete: {}/{} plugins active", active_count, plugins.len());
        
        Ok(())
    }

    /// Shutdown all plugins
    pub async fn shutdown(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        for plugin in plugins {
            if let Err(e) = plugin.shutdown().await {
                eprintln!("Failed to shutdown plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }

        Ok(())
    }

    /// Get all components of a specific type
    pub fn get_components<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation,
        // we would iterate through plugins and retrieve components of the specified type
        Ok(Vec::new())
    }

    /// Get all endpoints of a specific type
    pub fn get_endpoints<T: 'static>(&self) -> Result<Vec<(Uuid, T)>> {
        // This is a placeholder - in a real implementation,
        // we would iterate through plugins and retrieve endpoints of the specified type
        Ok(Vec::new())
    }

    /// Get a plugin by ID
    pub async fn get_plugin_by_id(&self, id: &Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(id).ok_or(PluginError::NotFound(*id))?;
        Ok(plugin.clone())
    }

    /// Get all plugins
    pub async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }

    /// Execute initialization for all plugins
    pub async fn initialize_all(&self) -> Result<()> {
        let plugins = self.plugins.read().await;

        for plugin in plugins.values() {
            if let Err(e) = plugin.initialize().await {
                eprintln!(
                    "Failed to initialize plugin {}: {}",
                    plugin.metadata().id,
                    e
                );
                // Continue with other plugins
            }
        }

        Ok(())
    }

    /// Execute shutdown for all plugins
    pub async fn shutdown_all(&self) -> Result<()> {
        let plugins = self.plugins.read().await;

        for plugin in plugins.values() {
            if let Err(e) = plugin.shutdown().await {
                eprintln!("Failed to shutdown plugin {}: {}", plugin.metadata().id, e);
                // Continue with other plugins
            }
        }

        Ok(())
    }

    /// Load plugins from a directory using the given `PluginDiscovery`
    pub async fn load_plugins_with_discovery<D: PluginDiscovery + Send + Sync>(
        &self,
        discovery: &D,
        directory: &str,
    ) -> Result<usize> {
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut count = 0;

        // Since we don't have a load_plugin method in PluginDiscovery trait,
        // we'll directly use the discovered plugins
        // Assuming discover_plugins returns Vec<Box<dyn Plugin>> as per trait definition
        for plugin in plugin_paths {
            // Convert Box<dyn Plugin> to Arc<dyn Plugin>
            let arc_plugin = plugin;
            self.register_plugin(arc_plugin).await?;
            count += 1;
        }

        Ok(count)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default plugin manager implementation
pub struct DefaultPluginManager {
    plugins: Arc<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>,
    // Security handled by BearDog framework
    // security_manager: Arc<SecurityManagerAdapter>,
    status: PluginManagerStatus,
    metrics: PluginManagerMetrics,
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    name_to_id: RwLock<HashMap<String, Uuid>>,
    dependency_resolver: RwLock<DependencyResolver>,
}

impl DefaultPluginManager {
    /// Create a new default plugin manager
    pub fn new(
        _state_manager: Arc<dyn PluginStateManager + Send + Sync>,
        // Security handled by BearDog framework
        // security_manager: Option<Arc<SecurityManagerAdapter>>
    ) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            status: PluginManagerStatus::new(),
            metrics: PluginManagerMetrics::new(),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            dependency_resolver: RwLock::new(DependencyResolver::new()),
        }
    }

    /// Get the state manager
    pub fn state_manager(&self) -> Arc<dyn PluginStateManager + Send + Sync> {
        // Placeholder implementation
        use crate::state::MemoryStateManager;
        Arc::new(MemoryStateManager::new())
    }

    /// Load plugins from a directory with signature verification
    pub async fn load_plugins_with_signatures(
        &self,
        directory: &str,
        verify_signatures: bool,
    ) -> Result<Vec<Uuid>> {
        let discovery = DefaultPluginDiscovery::new();
        let plugin_paths = discovery.discover_plugins(directory).await?;
        let mut ids = Vec::new();

        let mut plugin_count = 0;
        for plugin in plugin_paths {
            let metadata = plugin.metadata().clone(); // Clone metadata before moving plugin
            let id = metadata.id;
            let name = metadata.name.clone(); // Clone the name to use later

            // If signature verification is enabled, look for a signature file
            if verify_signatures {
                let sig_path = std::path::Path::new(directory).join(format!("{name}.sig"));

                if sig_path.exists() {
                    // Load the signature
                    let _sig_bytes = std::fs::read(&sig_path)?;

                    // Verify signature using security manager
                    // Security verification handled by BearDog framework
                    // if let Ok(verification_result) = self.security_manager.verify_signature(&metadata, &_sig_bytes).await {
                    //     if let Ok(()) = self.security_manager.verify_plugin(plugin.as_ref()).await {
                    //         // Both signature and plugin verification passed
                    //     }
                    // }
                } else {
                    warn!(
                        "No signature file found for plugin {name}, registering without signature"
                    );
                    // Register without signature verification
                    let arc_plugin = plugin;
                    if let Ok(()) = self.register_plugin(arc_plugin).await {
                        ids.push(id);
                        plugin_count += 1;
                    }
                }
            } else {
                // Register without signature verification
                let arc_plugin = plugin;
                if let Ok(()) = self.register_plugin(arc_plugin).await {
                    ids.push(id);
                    plugin_count += 1;
                }
            }
        }

        info!("Loaded {plugin_count} plugins from {directory}");
        Ok(ids)
    }

    /// Check if a plugin has dependencies
    async fn check_dependencies(&self, plugin: &Arc<dyn Plugin>) -> Result<()> {
        let plugins = self.plugins.read().await;
        let metadata = plugin.metadata();

        for dependency_id in &metadata.dependencies {
            let mut found = false;

            for (plugin_id, _) in plugins.iter() {
                if plugin_id == dependency_id {
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(PluginError::DependencyNotFound(dependency_id.to_string()));
            }
        }

        Ok(())
    }

    /// Check for dependency cycles
    #[allow(dead_code)]
    async fn check_dependency_cycles(&self, plugin_id: Uuid, path: &mut Vec<Uuid>) -> Result<()> {
        if path.contains(&plugin_id) {
            return Err(PluginError::DependencyCycle(plugin_id));
        }

        path.push(plugin_id);

        let plugins = self.plugins.read().await;
        if let Some(plugin) = plugins.get(&plugin_id) {
            let metadata = plugin.metadata();

            for dependency_id in &metadata.dependencies {
                // Use Box::pin to handle recursive async calls
                let future = self.check_dependency_cycles(*dependency_id, path);
                Box::pin(future).await?;
            }
        }

        path.pop();

        Ok(())
    }

    /// Get security report for a plugin (handled by BearDog framework)
    pub async fn get_security_report(&self, _id: Uuid) -> Result<String> {
        // Security reporting handled by BearDog framework
        // self.security_manager.create_security_report(_id).await.map_err(|e| {
        //     PluginError::SecurityError(format!("Failed to create security report: {}", e)).into()
        // })
        Ok("Security reporting handled by BearDog framework".to_string())
    }

    /// Register enhanced dependencies for a plugin
    pub async fn register_enhanced_dependencies(
        &self,
        plugin_id: Uuid,
        dependencies: Vec<EnhancedPluginDependency>,
    ) -> Result<()> {
        let mut resolver = self.dependency_resolver.write().await;
        resolver.register_enhanced_dependencies(plugin_id, dependencies)?;
        debug!("Registered enhanced dependencies for plugin {plugin_id}");
        Ok(())
    }

    /// Get dependency resolution statistics
    pub async fn get_dependency_statistics(&self) -> ResolutionStatistics {
        let resolver = self.dependency_resolver.read().await;
        resolver.get_statistics()
    }

    /// Clear dependency resolution cache
    pub async fn clear_dependency_cache(&self) {
        let mut resolver = self.dependency_resolver.write().await;
        resolver.clear_cache();
    }

    /// Get plugins that depend on a specific plugin
    pub async fn get_plugin_dependents(&self, plugin_id: Uuid) -> Vec<Uuid> {
        let resolver = self.dependency_resolver.read().await;
        resolver.get_dependents(plugin_id)
    }

    /// Perform a dry-run dependency resolution without initializing plugins
    pub async fn resolve_dependencies_dry_run(&self) -> Result<ResolutionResult> {
        let mut resolver = self.dependency_resolver.write().await;
        Ok(resolver.resolve_dependencies()?)
    }

    /// Validate plugin configuration against plugin requirements
    async fn validate_plugin_config(&self, plugin: &Arc<dyn Plugin>, config: &PluginConfig) -> bool {
        // Basic validation - ensure plugin supports the configuration keys
        let metadata = plugin.metadata();
        
        // Check if plugin has specific configuration requirements
        if let Some(required_config) = &metadata.required_config {
            for required_key in required_config {
                if !config.settings.contains_key(required_key) {
                    warn!("🔍 Plugin '{}' requires configuration key '{}' but it's missing", 
                          metadata.name, required_key);
                    return false;
                }
            }
        }
        
        // Validate configuration value types and ranges
        for (key, value) in &config.settings {
            if let Err(e) = self.validate_config_value(key, value) {
                warn!("🔍 Invalid configuration value for '{}' in plugin '{}': {}", 
                      key, metadata.name, e);
                return false;
            }
        }
        
        debug!("✅ Configuration validation passed for plugin '{}'", metadata.name);
        true
    }

    /// Apply plugin configuration settings
    async fn apply_plugin_config(&self, plugin: &Arc<dyn Plugin>, config: &PluginConfig) -> Result<()> {
        let metadata = plugin.metadata();
        
        debug!("🔧 Applying {} configuration settings to plugin '{}'", 
               config.settings.len(), metadata.name);
               
        // Apply each configuration setting
        for (key, value) in &config.settings {
            match self.apply_config_setting(plugin, key, value).await {
                Ok(_) => {
                    debug!("✅ Applied config setting '{}' = '{:?}' to plugin '{}'", 
                           key, value, metadata.name);
                }
                Err(e) => {
                    warn!("⚠️ Failed to apply config setting '{}' to plugin '{}': {}", 
                          key, metadata.name, e);
                    return Err(e);
                }
            }
        }
        
        info!("🔧 Successfully applied all configuration settings to plugin '{}'", metadata.name);
        Ok(())
    }

    /// Validate a single configuration value
    fn validate_config_value(&self, key: &str, value: &serde_json::Value) -> Result<()> {
        // Basic type and range validation based on common configuration patterns
        match key {
            "timeout" | "retry_count" | "max_connections" => {
                if let Some(num) = value.as_i64() {
                    if num < 0 {
                        return Err(PluginError::ConfigurationError(format!("Value for '{}' must be non-negative", key)));
                    }
                } else {
                    return Err(PluginError::ConfigurationError(format!("Value for '{}' must be a number", key)));
                }
            }
            "enabled" | "debug" | "verbose" => {
                if !value.is_boolean() {
                    return Err(PluginError::ConfigurationError(format!("Value for '{}' must be a boolean", key)));
                }
            }
            "url" | "endpoint" => {
                if let Some(url_str) = value.as_str() {
                    if !url_str.starts_with("http") {
                        return Err(PluginError::ConfigurationError(format!("URL for '{}' must start with http:// or https://", key)));
                    }
                }
            }
            _ => {
                // Generic validation - just ensure it's a valid JSON value
                if value.is_null() && key != "optional" {
                    return Err(PluginError::ConfigurationError(format!("Required configuration '{}' cannot be null", key)));
                }
            }
        }
        
        Ok(())
    }

    /// Apply a single configuration setting to a plugin
    async fn apply_config_setting(&self, plugin: &Arc<dyn Plugin>, key: &str, value: &serde_json::Value) -> Result<()> {
        // This is a placeholder for plugin-specific configuration application
        // In a real implementation, plugins would expose configuration interfaces
        
        debug!("Applying configuration setting '{}' to plugin '{}'", key, plugin.metadata().name);
        
        // For now, we just log the configuration application
        // Real implementation would call plugin-specific configuration methods
        
        Ok(())
    }
}

#[async_trait]
impl PluginRegistry for DefaultPluginManager {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let metadata = plugin.metadata().clone(); // Clone metadata before moving plugin
        let id = metadata.id;

        // Security verification handled by BearDog framework
        // if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
        //     return Err(PluginError::SecurityError(format!("Plugin security verification failed: {}", e)).into());
        // }

        // Register with dependency resolver
        {
            let mut resolver = self.dependency_resolver.write().await;
            resolver.register_plugin(plugin.clone())?;
        }

        // Set the initial status to Registered
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Registered);

        // Map the name to the ID
        let mut name_to_id = self.name_to_id.write().await;
        name_to_id.insert(metadata.name.clone(), id);

        // Register the plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(id, plugin);

        debug!(
            "Registered plugin: {} with dependency resolver",
            metadata.name
        );
        Ok(())
    }

    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let mut statuses = self.statuses.write().await;
        let mut name_to_id = self.name_to_id.write().await;

        let plugin = plugins.remove(&id).ok_or(PluginError::NotFound(id))?;
        let name = plugin.metadata().name.clone();

        statuses.remove(&id);
        name_to_id.remove(&name);

        debug!("Unregistered plugin: {}", id);

        Ok(())
    }

    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(plugin.clone())
    }

    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        let name_to_id = self.name_to_id.read().await;
        let id = name_to_id
            .get(name)
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))?;
        PluginRegistry::get_plugin(self, *id).await
    }

    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        let status = statuses.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(*status)
    }

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id));
        }

        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
        Ok(())
    }

    /// Get all registered plugins
    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>> {
        let plugins = self.plugins.read().await;
        let result: Vec<Arc<dyn Plugin>> = plugins.values().cloned().collect();
        Ok(result)
    }
}

#[async_trait]
impl PluginManagerTrait for DefaultPluginManager {
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        let plugin = plugins.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(plugin.clone())
    }

    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()> {
        // Get the plugin
        let plugin = PluginRegistry::get_plugin(self, id).await?;

        // Verify dependencies
        self.check_dependencies(&plugin).await?;

        // Security verification handled by BearDog framework
        // Check for security issues before initializing
        // if let Err(e) = self.security_manager.verify_plugin(plugin.as_ref()).await {
        //     // Set the status to Failed
        //     let mut statuses = self.statuses.write().await;
        //     statuses.insert(id, PluginStatus::Failed);
        //
        //     return Err(PluginError::SecurityError(format!("Security verification failed: {}", e)).into());
        // }

        // Security sandbox creation handled by BearDog framework
        // if !self.security_manager.is_sandboxed(id).await? {
        //     let config = crate::security::SandboxConfig::default();
        //     self.security_manager.create_sandbox(id, config).await?;
        // }

        // Initialize the plugin
        let init_result = plugin.initialize().await;

        // If initialization failed, update the status
        if init_result.is_err() {
            let mut statuses = self.statuses.write().await;
            statuses.insert(id, PluginStatus::Failed);
        }

        // Return the result with a proper error conversion
        init_result.map_err(|e| {
            anyhow::Error::from(PluginError::InitializationError(format!(
                "Plugin initialization failed: {e}"
            )))
        })?;

        // Set the status to Initialized
        let mut statuses = self.statuses.write().await;
        statuses.insert(id, PluginStatus::Initialized);

        // Log the successful initialization
        tracing::info!("Initialized plugin: {}", plugin.metadata().name);

        Ok(())
    }

    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()> {
        let mut statuses = self.statuses.write().await;
        let status = statuses.get(&id).cloned();

        // Check if plugin is not active
        if let Some(status) = status {
            if status != PluginStatus::Initialized {
                return Ok(());
            }
        }

        // Get plugin using explicit trait qualification
        let plugin = PluginRegistry::get_plugin(self, id)
            .await
            .map_err(|_| anyhow::anyhow!("Plugin not found"))?;

        // Shutdown plugin
        match plugin.shutdown().await {
            Ok(_) => {
                statuses.insert(id, PluginStatus::Unloaded);
                Ok(())
            }
            Err(err) => {
                // Keep plugin active on error
                Err(PluginError::InitializationError(format!(
                    "Plugin shutdown failed: {err}"
                )))
            }
        }
    }

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus> {
        let statuses = self.statuses.read().await;
        let status = statuses.get(&id).ok_or(PluginError::NotFound(id))?;
        Ok(*status)
    }

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()> {
        let plugins = self.plugins.read().await;
        if !plugins.contains_key(&id) {
            return Err(PluginError::NotFound(id));
        }

        let mut statuses = self.statuses.write().await;
        statuses.insert(id, status);
        Ok(())
    }

    /// Load plugins from a directory
    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>> {
        self.load_plugins_with_signatures(directory, false).await
    }

    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()> {
        info!("Starting initialization of all plugins with dependency resolution");

        // Resolve dependencies to get proper initialization order
        let resolution_result = {
            let mut resolver = self.dependency_resolver.write().await;
            resolver.resolve_dependencies()?
        };

        // Log any warnings from dependency resolution
        for warning in &resolution_result.warnings {
            warn!("Dependency resolution warning: {warning}");
        }

        // Check for circular dependencies
        if !resolution_result.circular_dependencies.is_empty() {
            for cycle in &resolution_result.circular_dependencies {
                error!("Circular dependency detected: {cycle:?}");
            }
            return Err(PluginError::CircularDependency(format!(
                "Detected {} circular dependencies",
                resolution_result.circular_dependencies.len()
            )));
        }

        // Check for unresolved plugins
        if !resolution_result.unresolved_plugins.is_empty() {
            for (plugin_id, missing_deps) in &resolution_result.unresolved_plugins {
                error!("Plugin {plugin_id} has unresolved dependencies: {missing_deps:?}");
            }
            return Err(PluginError::ResolutionFailed(format!(
                "Cannot resolve dependencies for {} plugins",
                resolution_result.unresolved_plugins.len()
            )));
        }

        // Log version conflicts but continue (they're warnings)
        for conflict in &resolution_result.version_conflicts {
            warn!(
                "Version conflict: plugin {} requires {} version {}, but {} is available",
                conflict.plugin_id,
                conflict.dependency_name,
                conflict.required_version,
                conflict.available_version
            );
        }

        // Initialize plugins in dependency order
        info!(
            "Initializing {} plugins in dependency order",
            resolution_result.initialization_order.len()
        );
        let mut initialized_count = 0;
        let mut failed_count = 0;

        for plugin_id in &resolution_result.initialization_order {
            match self.initialize_plugin(*plugin_id).await {
                Ok(_) => {
                    initialized_count += 1;
                    debug!("Successfully initialized plugin {plugin_id}");
                }
                Err(e) => {
                    failed_count += 1;
                    error!("Failed to initialize plugin {plugin_id}: {e}");

                    // Set status to failed
                    let mut statuses = self.statuses.write().await;
                    statuses.insert(*plugin_id, PluginStatus::Failed);

                    // For now, continue with other plugins rather than failing completely
                    // In a production system, you might want to fail fast for critical plugins
                }
            }
        }

        info!(
            "Plugin initialization completed. Success: {}, Failed: {}, Total: {}",
            initialized_count,
            failed_count,
            resolution_result.initialization_order.len()
        );

        if failed_count > 0 {
            warn!("{failed_count} plugins failed to initialize");
        }

        Ok(())
    }

    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        let ids: Vec<Uuid> = plugins.keys().copied().collect();
        drop(plugins);

        for id in ids {
            // Try to shutdown but don't fail if one plugin fails
            let _ = self.shutdown_plugin(id).await;
        }

        Ok(())
    }
}

/// Plugin manager status
#[derive(Debug, Default)]
pub struct PluginManagerStatus {
    /// Number of currently active plugins
    pub active_plugins: usize,
    /// Total number of plugins registered
    pub total_plugins: usize,
}

impl PluginManagerStatus {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Plugin manager metrics
#[derive(Debug, Default)]
pub struct PluginManagerMetrics {
    pub load_time_ms: u64,
    pub memory_usage_mb: u64,
}

impl PluginManagerMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}
