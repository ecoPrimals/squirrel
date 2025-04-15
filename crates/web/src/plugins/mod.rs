//! Unified Plugin System for Squirrel Web
//!
//! This module provides a modern, type-safe plugin architecture for the Squirrel Web application.
//! It includes the core Plugin trait, WebPlugin trait, WebPluginRegistry, and related components.

pub mod model;
pub mod adapter;
pub mod example;

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::plugins::example::ExamplePlugin;

// Re-export everything from the legacy module
pub use crate::plugins as legacy;

// Re-export all public components from submodules
pub use self::model::{WebRequest, WebResponse, WebEndpoint, WebComponent, HttpMethod, ComponentType};
pub use self::adapter::LegacyWebPluginAdapter;

/// Status of a plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is active and can be used
    Active,
    /// Plugin is disabled and cannot be used
    Disabled,
    /// Plugin is pending initialization
    Pending,
    /// Plugin encountered an error
    Error,
}

/// Metadata for a plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: String,
    /// Name of the plugin
    pub name: String,
    /// Version of the plugin
    pub version: String,
    /// Description of the plugin
    pub description: String,
    /// Author of the plugin
    pub author: String,
    /// Repository URL (optional)
    pub repository: Option<String>,
    /// License (optional)
    pub license: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Core Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Get plugin status
    fn status(&self) -> PluginStatus;
    
    /// Set plugin status
    fn set_status(&mut self, status: PluginStatus);
}

/// WebPlugin trait for web-specific plugins
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Get the components provided by this plugin
    fn get_components(&self) -> Vec<WebComponent>;
    
    /// Handle a web request
    async fn handle_request(&self, request: WebRequest) -> Result<WebResponse>;
    
    /// Get component markup for rendering
    async fn get_component_markup(&self, id: uuid::Uuid, props: serde_json::Value) -> Result<String>;
}

/// Registry for plugin types that can create new instances of plugins
#[derive(Default)]
pub struct PluginTypeRegistry {
    factories: std::collections::HashMap<String, Box<dyn Fn() -> Box<dyn WebPlugin> + Send + Sync>>,
}

impl PluginTypeRegistry {
    /// Create a new plugin type registry
    pub fn new() -> Self {
        let mut registry = Self::default();
        
        // Register the example plugin factory
        registry.register_factory("example-plugin", || Box::new(ExamplePlugin::new()));
        
        registry
    }
    
    /// Register a factory function for a plugin type
    pub fn register_factory<F>(&mut self, plugin_type: &str, factory: F)
    where
        F: Fn() -> Box<dyn WebPlugin> + Send + Sync + 'static,
    {
        self.factories.insert(plugin_type.to_string(), Box::new(factory));
    }
    
    /// Create a new instance of a plugin by ID
    pub fn create_plugin(&self, plugin_id: &str) -> Option<Box<dyn WebPlugin>> {
        self.factories.get(plugin_id).map(|factory| factory())
    }
}

/// Implement a global plugin type registry for use in the WebPluginRegistry
lazy_static::lazy_static! {
    static ref PLUGIN_TYPE_REGISTRY: std::sync::RwLock<PluginTypeRegistry> = {
        std::sync::RwLock::new(PluginTypeRegistry::new())
    };
}

/// Register a plugin factory with the global registry
pub fn register_plugin_factory<F>(plugin_type: &str, factory: F)
where
    F: Fn() -> Box<dyn WebPlugin> + Send + Sync + 'static,
{
    let mut registry = PLUGIN_TYPE_REGISTRY.write().unwrap();
    registry.register_factory(plugin_type, factory);
}

/// Plugin Registry for managing plugins
pub struct WebPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Box<dyn WebPlugin>>>>,
    endpoints: Arc<RwLock<HashMap<String, HashMap<String, model::WebEndpoint>>>>,
    components: Arc<RwLock<HashMap<String, HashMap<uuid::Uuid, model::WebComponent>>>>,
}

impl WebPluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            endpoints: Arc::new(RwLock::new(HashMap::new())),
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a plugin with the registry
    pub async fn register_plugin<P: WebPlugin + Clone + 'static>(&self, plugin: P) -> Result<()> {
        let plugin_id = plugin.metadata().id.clone();
        let plugin_name = plugin.metadata().name.clone();
        
        info!("Registering plugin: {} ({})", plugin_name, plugin_id);
        
        // Register with the plugin type registry if not already registered
        let registry = PLUGIN_TYPE_REGISTRY.read().unwrap();
        if registry.create_plugin(&plugin_id).is_none() {
            drop(registry);  // Release the read lock before acquiring write lock
            
            // Register a factory for this plugin type
            let mut registry = PLUGIN_TYPE_REGISTRY.write().unwrap();
            
            // Clone the plugin before moving it into the closure
            let plugin_clone = plugin.clone();
            registry.register_factory(&plugin_id, move || {
                // This creates a new clone of the plugin
                Box::new(plugin_clone.clone())
            });
            
            // Get back to a clean state for the rest of the function
            drop(registry);
        }
        
        // Store the plugin
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(plugin_id.clone(), Box::new(plugin) as Box<dyn WebPlugin>);
        }
        
        // Register endpoints
        {
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&plugin_id).unwrap();
            let endpoints = plugin.get_endpoints();
            
            let mut registry_endpoints = self.endpoints.write().await;
            let mut plugin_endpoints = HashMap::new();
            
            for endpoint in endpoints {
                debug!("Registering endpoint: {} {}", endpoint.method as u8, endpoint.path);
                plugin_endpoints.insert(endpoint.path.clone(), endpoint);
            }
            
            registry_endpoints.insert(plugin_id.clone(), plugin_endpoints);
        }
        
        // Register components
        {
            let plugins = self.plugins.read().await;
            let plugin = plugins.get(&plugin_id).unwrap();
            let components = plugin.get_components();
            
            let mut registry_components = self.components.write().await;
            let mut plugin_components = HashMap::new();
            
            for component in components {
                debug!("Registering component: {}", component.name);
                plugin_components.insert(component.id, component);
            }
            
            registry_components.insert(plugin_id.clone(), plugin_components);
        }
        
        Ok(())
    }
    
    /// Unregister a plugin from the registry
    pub async fn unregister_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Unregistering plugin: {}", plugin_id);
        
        // Remove the plugin
        {
            let mut plugins = self.plugins.write().await;
            plugins.remove(plugin_id);
        }
        
        // Remove endpoints
        {
            let mut registry_endpoints = self.endpoints.write().await;
            registry_endpoints.remove(plugin_id);
        }
        
        // Remove components
        {
            let mut registry_components = self.components.write().await;
            registry_components.remove(plugin_id);
        }
        
        Ok(())
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Enabling plugin: {}", plugin_id);
        
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.set_status(PluginStatus::Active);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", plugin_id))
        }
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        info!("Disabling plugin: {}", plugin_id);
        
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.set_status(PluginStatus::Disabled);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin not found: {}", plugin_id))
        }
    }
    
    /// Get all plugins with Active status
    pub async fn get_plugins(&self) -> Vec<Box<dyn WebPlugin>> {
        let mut plugins = Vec::new();
        
        let plugins_registry = self.plugins.read().await;
        
        for (id, plugin) in plugins_registry.iter() {
            if plugin.status() == PluginStatus::Active {
                let plugin_ref: &Box<dyn WebPlugin> = plugin;
                let plugin_clone: Box<dyn WebPlugin> = match plugin_ref.metadata().id.as_str() {
                    // Create a new instance of ExamplePlugin as a special case
                    id if id == "example-plugin" => {
                        // Clone the example plugin if that's what we have
                        Box::new(ExamplePlugin::new())
                    },
                    _ => {
                        // For other plugins, we'd need proper cloning - this is simplified
                        // In a real implementation, plugins should provide a method for cloning
                        // or we would store them differently
                        debug!("Cloning plugin reference for {}", id);
                        plugin_ref.clone()
                    }
                };
                plugins.push(plugin_clone);
            }
        }
        
        plugins
    }
    
    /// Get all disabled plugins
    pub async fn get_disabled_plugins(&self) -> Vec<Box<dyn WebPlugin>> {
        let mut plugins = Vec::new();
        
        let plugins_registry = self.plugins.read().await;
        
        for (id, plugin) in plugins_registry.iter() {
            if plugin.status() == PluginStatus::Disabled {
                let plugin_ref: &Box<dyn WebPlugin> = plugin;
                let plugin_clone: Box<dyn WebPlugin> = match plugin_ref.metadata().id.as_str() {
                    // Create a new instance of ExamplePlugin as a special case
                    id if id == "example-plugin" => {
                        // Clone the example plugin if that's what we have
                        Box::new(ExamplePlugin::new())
                    },
                    _ => {
                        // For other plugins, we'd need proper cloning - this is simplified
                        debug!("Cloning plugin reference for {}", id);
                        plugin_ref.clone()
                    }
                };
                plugins.push(plugin_clone);
            }
        }
        
        plugins
    }
    
    /// Get all endpoints from all plugins
    pub async fn get_endpoints(&self) -> Vec<(String, model::WebEndpoint)> {
        let endpoints = self.endpoints.read().await;
        let mut result = Vec::new();
        
        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for (_path, endpoint) in plugin_endpoints {
                result.push((plugin_id.clone(), endpoint.clone()));
            }
        }
        
        result
    }
    
    /// Get all components from all plugins
    pub async fn get_components(&self) -> Vec<(String, model::WebComponent)> {
        let components = self.components.read().await;
        let mut result = Vec::new();
        
        for (plugin_id, plugin_components) in components.iter() {
            for (_, component) in plugin_components {
                result.push((plugin_id.clone(), component.clone()));
            }
        }
        
        result
    }
    
    /// Handle a web request
    pub async fn handle_request(&self, request: model::WebRequest) -> Result<model::WebResponse> {
        debug!("Handling request: {} {}", request.method as u8, request.path);
        
        let endpoints = self.endpoints.read().await;
        let plugins = self.plugins.read().await;
        
        // Find the plugin that can handle this request
        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for (endpoint_path, _) in plugin_endpoints {
                if endpoint_path == &request.path {
                    // Found matching endpoint
                    if let Some(plugin) = plugins.get(plugin_id) {
                        // Check if plugin is active
                        if plugin.status() != PluginStatus::Active {
                            return Ok(model::WebResponse::service_unavailable());
                        }
                        
                        // Handle the request
                        return plugin.handle_request(request.clone()).await;
                    }
                }
            }
        }
        
        // No plugin found that can handle this request
        Ok(model::WebResponse::not_found())
    }
    
    /// Get component markup for rendering
    pub async fn get_component_markup(&self, component_id: uuid::Uuid, props: serde_json::Value) -> Result<String> {
        debug!("Getting component markup for: {}", component_id);
        
        let components = self.components.read().await;
        let plugins = self.plugins.read().await;
        
        // Find the plugin that owns this component
        for (plugin_id, plugin_components) in components.iter() {
            if let Some(_) = plugin_components.get(&component_id) {
                // Found the component
                if let Some(plugin) = plugins.get(plugin_id) {
                    // Check if plugin is active
                    if plugin.status() != PluginStatus::Active {
                        return Err(anyhow::anyhow!("Plugin is not active"));
                    }
                    
                    // Get the markup
                    return plugin.get_component_markup(component_id, props).await;
                }
            }
        }
        
        // No component found with this ID
        Err(anyhow::anyhow!("Component not found: {}", component_id))
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins_from_directory(&self, directory: &str) -> Result<usize> {
        info!("Loading plugins from directory: {}", directory);
        
        // Create Path from directory string
        let dir_path = std::path::Path::new(directory);
        
        // Ensure the directory exists
        if !dir_path.exists() || !dir_path.is_dir() {
            warn!("Plugin directory does not exist or is not a directory: {}", directory);
            return Ok(0);
        }
        
        let mut loaded_count = 0;
        
        // Read the directory
        match std::fs::read_dir(dir_path) {
            Ok(entries) => {
                // Process each entry
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        
                        // We'll handle .so files on Unix and .dll files on Windows
                        if path.is_file() && 
                           ((cfg!(target_os = "windows") && path.extension() == Some(std::ffi::OsStr::new("dll"))) ||
                            (cfg!(not(target_os = "windows")) && path.extension() == Some(std::ffi::OsStr::new("so")))) {
                            
                            // Load the plugin
                            if let Err(e) = self.load_plugin_from_file(&path).await {
                                error!("Failed to load plugin from file {}: {}", path.display(), e);
                            } else {
                                loaded_count += 1;
                                info!("Successfully loaded plugin from: {}", path.display());
                            }
                        } else if path.is_file() && 
                                  (path.extension() == Some(std::ffi::OsStr::new("js")) || 
                                   path.extension() == Some(std::ffi::OsStr::new("py"))) {
                            
                            // Handle script-based plugins (JavaScript or Python)
                            if let Err(e) = self.load_plugin_from_script(&path).await {
                                error!("Failed to load script plugin from file {}: {}", path.display(), e);
                            } else {
                                loaded_count += 1;
                                info!("Successfully loaded script plugin from: {}", path.display());
                            }
                        }
                    }
                }
            },
            Err(e) => {
                error!("Failed to read plugin directory {}: {}", directory, e);
                return Err(anyhow::anyhow!("Failed to read plugin directory: {}", e));
            }
        }
        
        info!("Loaded {} plugins from directory {}", loaded_count, directory);
        Ok(loaded_count)
    }
    
    /// Load a plugin from a dynamic library file
    async fn load_plugin_from_file(&self, path: &std::path::Path) -> Result<()> {
        #[cfg(feature = "dynamic-plugins")]
        {
            use libloading::{Library, Symbol};
            
            // This would be the actual implementation for dynamic loading
            unsafe {
                // Load the library
                let lib = Library::new(path).map_err(|e| anyhow::anyhow!("Failed to load library: {}", e))?;
                
                // Look up the create_plugin symbol
                let create_plugin: Symbol<unsafe extern "C" fn() -> *mut dyn WebPlugin> = 
                    lib.get(b"create_plugin")
                       .map_err(|e| anyhow::anyhow!("Failed to find create_plugin symbol: {}", e))?;
                
                // Call the function to create the plugin
                let plugin_ptr = create_plugin();
                
                // Convert the raw pointer to a Box
                let plugin = Box::from_raw(plugin_ptr);
                
                // Register the plugin
                self.register_plugin(*plugin).await?;
                
                // Don't drop the library! We need to keep it loaded as long as the plugin is in use
                // We should store the library handle somewhere to prevent it from being dropped
                // For now, we'll just leak it to keep it loaded
                std::mem::forget(lib);
            }
            
            Ok(())
        }
        #[cfg(not(feature = "dynamic-plugins"))]
        {
            // In development or when dynamic loading is not enabled, load the example plugin
            warn!("Dynamic plugin loading not enabled, loading ExamplePlugin instead of {}", path.display());
            
            // Create and register an example plugin
            let example = ExamplePlugin::new();
            self.register_plugin(example).await?;
            
            Ok(())
        }
    }
    
    /// Load a plugin from a script file (JavaScript or Python)
    async fn load_plugin_from_script(&self, path: &std::path::Path) -> Result<()> {
        // This is a placeholder for script-based plugin loading
        // In a real implementation, we would use a JavaScript or Python runtime
        // to load and execute the script, which would register the plugin
        
        #[cfg(feature = "script-plugins")]
        {
            // Actual implementation would go here
            unimplemented!("Script plugin loading not yet implemented");
        }
        
        #[cfg(not(feature = "script-plugins"))]
        {
            // In development or when script plugins are not enabled, load the example plugin
            warn!("Script plugin loading not enabled, loading ExamplePlugin instead of {}", path.display());
            
            // Create and register an example plugin
            let example = ExamplePlugin::new();
            self.register_plugin(example).await?;
            
            Ok(())
        }
    }
}

// Implement Clone for Box<dyn WebPlugin> using the plugin type registry
impl Clone for Box<dyn WebPlugin> {
    fn clone(&self) -> Self {
        let plugin_id = self.metadata().id.clone();
        
        // Try to create a new instance using the plugin type registry
        let registry = PLUGIN_TYPE_REGISTRY.read().unwrap();
        if let Some(plugin) = registry.create_plugin(&plugin_id) {
            return plugin;
        }
        
        // Fallback to ExamplePlugin if the type isn't registered
        debug!("Failed to clone plugin: {}, returning ExamplePlugin", plugin_id);
        Box::new(ExamplePlugin::new())
    }
} 