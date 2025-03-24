//! Web plugin registry
//!
//! This module provides the registry for web plugins, which manages the
//! lifecycle of plugins and provides access to plugin functionality.

use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use serde_json::Value;

use crate::plugins::core::{Plugin, PluginStatus, PluginRef};
use crate::plugins::model::{WebPlugin, WebEndpoint, WebComponent, WebRequest, WebResponse, HttpMethod};

/// Route definition for plugin endpoints
#[derive(Debug, Clone)]
pub struct Route {
    /// Original route pattern
    pub pattern: String,
    /// Parameter names in the route
    pub params: Vec<String>,
    /// Regular expression for matching
    pub regex: regex::Regex,
}

impl Route {
    /// Create a new route from a pattern
    pub fn new(pattern: &str) -> Result<Self> {
        let mut params = Vec::new();
        let mut regex_str = "^".to_string();
        
        // Split the pattern into segments
        let segments = pattern.split('/').filter(|s| !s.is_empty());
        
        for segment in segments {
            regex_str.push_str(r"/");
            
            // Check if this is a parameter segment
            if segment.starts_with(':') {
                let param = segment[1..].to_string();
                params.push(param);
                regex_str.push_str(r"([^/]+)");
            } else {
                // Escape special characters in the segment
                let escaped = regex::escape(segment);
                regex_str.push_str(&escaped);
            }
        }
        
        // Add end of string anchor
        regex_str.push_str(r"$");
        
        // Compile the regex
        let regex = regex::Regex::new(&regex_str)
            .context("Failed to compile route regex")?;
        
        Ok(Self {
            pattern: pattern.to_string(),
            params,
            regex,
        })
    }
    
    /// Check if a path matches this route
    pub fn matches(&self, path: &str) -> bool {
        self.regex.is_match(path)
    }
    
    /// Extract parameters from a path
    pub fn extract_params(&self, path: &str) -> Option<HashMap<String, String>> {
        let captures = self.regex.captures(path)?;
        
        let mut params = HashMap::new();
        for (i, name) in self.params.iter().enumerate() {
            let value = captures.get(i + 1)?.as_str().to_string();
            params.insert(name.clone(), value);
        }
        
        Some(params)
    }
}

/// Web plugin registry
#[derive(Debug)]
pub struct WebPluginRegistry {
    /// Registry for plugins
    plugins: RwLock<HashMap<Uuid, PluginRef<dyn WebPlugin>>>,
    /// Endpoints by plugin ID
    endpoints: RwLock<HashMap<Uuid, Vec<WebEndpoint>>>,
    /// Components by plugin ID
    components: RwLock<HashMap<Uuid, Vec<WebComponent>>>,
    /// Routes for fast matching
    routes: RwLock<HashMap<String, Route>>,
}

impl WebPluginRegistry {
    /// Create a new web plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            endpoints: RwLock::new(HashMap::new()),
            components: RwLock::new(HashMap::new()),
            routes: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a plugin
    pub async fn register_plugin<T>(&self, plugin: T) -> Result<()>
    where
        T: WebPlugin + 'static,
    {
        let plugin_ref = Arc::new(plugin) as PluginRef<dyn WebPlugin>;
        let plugin_id = plugin_ref.metadata().id;
        let plugin_name = plugin_ref.metadata().name.clone();
        
        // Initialize the plugin
        debug!("Initializing plugin: {}", plugin_name);
        plugin_ref.initialize().await
            .context(format!("Failed to initialize plugin: {}", plugin_name))?;
        
        // Register plugin endpoints
        let endpoints = plugin_ref.get_endpoints();
        debug!("Plugin {} has {} endpoints", plugin_name, endpoints.len());
        
        let mut routes_lock = self.routes.write().await;
        for endpoint in &endpoints {
            // Create route for the endpoint
            let route = Route::new(&endpoint.path)
                .context(format!("Failed to create route for endpoint: {}", endpoint.path))?;
            
            let route_key = format!("{}:{}", endpoint.method as u8, endpoint.path);
            routes_lock.insert(route_key, route);
        }
        
        // Store the plugin endpoints
        let mut endpoints_lock = self.endpoints.write().await;
        endpoints_lock.insert(plugin_id, endpoints);
        
        // Register plugin components
        let components = plugin_ref.get_components();
        debug!("Plugin {} has {} components", plugin_name, components.len());
        
        // Store the plugin components
        let mut components_lock = self.components.write().await;
        components_lock.insert(plugin_id, components);
        
        // Register the plugin
        let mut plugins_lock = self.plugins.write().await;
        plugins_lock.insert(plugin_id, plugin_ref);
        
        info!("Registered plugin: {}", plugin_name);
        Ok(())
    }
    
    /// Get a plugin by ID
    pub async fn get_plugin(&self, id: &Uuid) -> Option<PluginRef<dyn WebPlugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(id).cloned()
    }
    
    /// Get all plugins
    pub async fn get_plugins(&self) -> Vec<PluginRef<dyn WebPlugin>> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }
    
    /// Get all plugin IDs
    pub async fn get_plugin_ids(&self) -> Vec<Uuid> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }
    
    /// Get all plugin endpoints
    pub async fn get_endpoints(&self) -> Vec<(Uuid, WebEndpoint)> {
        let endpoints = self.endpoints.read().await;
        let mut result = Vec::new();
        
        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for endpoint in plugin_endpoints {
                result.push((*plugin_id, endpoint.clone()));
            }
        }
        
        result
    }
    
    /// Get all plugin components
    pub async fn get_components(&self) -> Vec<(Uuid, WebComponent)> {
        let components = self.components.read().await;
        let mut result = Vec::new();
        
        for (plugin_id, plugin_components) in components.iter() {
            for component in plugin_components {
                result.push((*plugin_id, component.clone()));
            }
        }
        
        result
    }
    
    /// Find an endpoint by path and method
    pub async fn find_endpoint(&self, path: &str, method: HttpMethod) -> Option<(Uuid, WebEndpoint)> {
        // First, try exact path match
        let endpoints = self.endpoints.read().await;
        
        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for endpoint in plugin_endpoints {
                if endpoint.path == path && endpoint.method == method {
                    return Some((*plugin_id, endpoint.clone()));
                }
            }
        }
        
        // If no exact match, try route matching
        let routes = self.routes.read().await;
        
        for (plugin_id, plugin_endpoints) in endpoints.iter() {
            for endpoint in plugin_endpoints {
                if endpoint.method != method {
                    continue;
                }
                
                let route_key = format!("{}:{}", endpoint.method as u8, endpoint.path);
                if let Some(route) = routes.get(&route_key) {
                    if route.matches(path) {
                        return Some((*plugin_id, endpoint.clone()));
                    }
                }
            }
        }
        
        None
    }
    
    /// Find a component by ID
    pub async fn find_component(&self, component_id: &Uuid) -> Option<(Uuid, WebComponent)> {
        let components = self.components.read().await;
        
        for (plugin_id, plugin_components) in components.iter() {
            for component in plugin_components {
                if &component.id == component_id {
                    return Some((*plugin_id, component.clone()));
                }
            }
        }
        
        None
    }
    
    /// Handle a request
    pub async fn handle_request(&self, request: WebRequest) -> Result<WebResponse> {
        // Find the endpoint
        let (plugin_id, endpoint) = self.find_endpoint(&request.path, request.method)
            .ok_or_else(|| anyhow!("Endpoint not found: {} {}", request.method as u8, request.path))?;
        
        // Check permissions
        for permission in &endpoint.permissions {
            if !request.permissions.contains(permission) {
                return Ok(WebResponse::forbidden());
            }
        }
        
        // Get the plugin
        let plugin = self.get_plugin(&plugin_id).await
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        // Handle the request
        plugin.handle_request(request).await
    }
    
    /// Get component markup
    pub async fn get_component_markup(&self, component_id: Uuid, props: Value) -> Result<String> {
        // Find the component
        let (plugin_id, _) = self.find_component(&component_id).await
            .ok_or_else(|| anyhow!("Component not found: {}", component_id))?;
        
        // Get the plugin
        let plugin = self.get_plugin(&plugin_id).await
            .ok_or_else(|| anyhow!("Plugin not found: {}", plugin_id))?;
        
        // Get the markup
        plugin.get_component_markup(component_id, props).await
    }
    
    /// Initialize all plugins
    pub async fn initialize(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        
        for plugin in plugins {
            let plugin_name = plugin.metadata().name.clone();
            debug!("Initializing plugin: {}", plugin_name);
            
            plugin.initialize().await
                .context(format!("Failed to initialize plugin: {}", plugin_name))?;
        }
        
        Ok(())
    }
    
    /// Shutdown all plugins
    pub async fn shutdown(&self) -> Result<()> {
        let plugins = self.get_plugins().await;
        
        for plugin in plugins {
            let plugin_name = plugin.metadata().name.clone();
            debug!("Shutting down plugin: {}", plugin_name);
            
            if let Err(err) = plugin.shutdown().await {
                warn!("Error shutting down plugin {}: {}", plugin_name, err);
            }
        }
        
        Ok(())
    }
    
    /// Get plugin status
    pub async fn get_plugin_status(&self, id: &Uuid) -> Result<PluginStatus> {
        let plugin = self.get_plugin(id).await
            .ok_or_else(|| anyhow!("Plugin not found: {}", id))?;
        
        Ok(plugin.status().await)
    }
    
    /// Load plugins from a directory
    pub async fn load_plugins_from_directory(&self, directory: impl AsRef<Path>) -> Result<usize> {
        let directory = directory.as_ref();
        info!("Loading plugins from directory: {}", directory.display());
        
        if !directory.exists() {
            warn!("Plugin directory does not exist: {}", directory.display());
            return Ok(0);
        }
        
        // This would normally load dynamic plugins from the directory
        // For now, it's just a placeholder as the actual plugin loading
        // would depend on the specific plugin system implementation
        
        Ok(0)
    }
}

impl Default for WebPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
} 