//! Web plugin system
//!
//! This module provides functionality for web interface plugins.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;
use crate::core::Plugin;

/// HTTP methods for web plugin endpoints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// DELETE method
    Delete,
    /// PATCH method
    Patch,
    /// OPTIONS method
    Options,
    /// HEAD method
    Head,
}

/// Web plugin route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPluginRoute {
    /// Route path
    pub path: String,
    
    /// HTTP method
    pub method: HttpMethod,
    
    /// Route handler name
    pub handler: String,
    
    /// Required permissions
    pub permissions: Vec<String>,
    
    /// Route metadata
    pub metadata: serde_json::Value,
}

/// Web plugin component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPluginComponent {
    /// Component name
    pub name: String,
    
    /// Component type
    pub component_type: String,
    
    /// Mount point in the UI
    pub mount_point: String,
    
    /// Component props
    pub props: serde_json::Value,
    
    /// Component metadata
    pub metadata: serde_json::Value,
}

/// Web plugin endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPluginEndpoint {
    /// Endpoint path
    pub path: String,
    
    /// HTTP method
    pub method: HttpMethod,
    
    /// Endpoint handler name
    pub handler: String,
    
    /// Required permissions
    pub permissions: Vec<String>,
    
    /// Input schema
    pub input_schema: Option<serde_json::Value>,
    
    /// Output schema
    pub output_schema: Option<serde_json::Value>,
    
    /// Endpoint metadata
    pub metadata: serde_json::Value,
}

/// Web plugin manager trait
#[async_trait]
pub trait WebPluginManager: Send + Sync {
    /// Initialize the web plugin system
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the web plugin system
    async fn shutdown(&self) -> Result<()>;
    
    /// Register a web plugin
    async fn register_plugin(&self, plugin: Arc<dyn WebPlugin>) -> Result<()>;
    
    /// Unregister a web plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;
    
    /// Get all web plugins
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn WebPlugin>>>;
    
    /// Get all web plugin routes
    async fn get_routes(&self) -> Result<Vec<(Uuid, WebPluginRoute)>>;
    
    /// Get all web plugin components
    async fn get_components(&self) -> Result<Vec<(Uuid, WebPluginComponent)>>;
    
    /// Get all web plugin endpoints
    async fn get_endpoints(&self) -> Result<Vec<(Uuid, WebPluginEndpoint)>>;
}

/// Web plugin trait
#[async_trait]
pub trait WebPlugin: Plugin + Send + Sync {
    /// Get the web plugin assets directory
    fn get_assets_dir(&self) -> Option<&str>;
    
    /// Get the web plugin routes
    fn get_routes(&self) -> Vec<WebPluginRoute>;
    
    /// Get the web plugin UI components
    fn get_ui_components(&self) -> Vec<WebPluginComponent>;
    
    /// Get the web plugin API endpoints
    fn get_api_endpoints(&self) -> Vec<WebPluginEndpoint>;
    
    /// Initialize the web plugin
    async fn web_initialize(&self) -> Result<()>;
    
    /// Shutdown the web plugin
    async fn web_shutdown(&self) -> Result<()>;
    
    /// Handle a web plugin route request
    async fn handle_route(
        &self,
        route: &WebPluginRoute,
        request: serde_json::Value,
    ) -> Result<serde_json::Value>;
    
    /// Handle a web plugin API endpoint request
    async fn handle_endpoint(
        &self,
        endpoint: &WebPluginEndpoint,
        request: serde_json::Value,
    ) -> Result<serde_json::Value>;
}

/// Default web plugin implementation
#[derive(Debug)]
pub struct DefaultWebPluginManager {
    /// Web plugins
    plugins: tokio::sync::RwLock<Vec<Arc<dyn WebPlugin>>>,
}

impl DefaultWebPluginManager {
    /// Create a new web plugin manager
    pub fn new() -> Self {
        Self {
            plugins: tokio::sync::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl WebPluginManager for DefaultWebPluginManager {
    /// Initialize the web plugin system
    async fn initialize(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        for plugin in plugins.iter() {
            plugin.web_initialize().await?;
        }
        
        Ok(())
    }
    
    /// Shutdown the web plugin system
    async fn shutdown(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        
        for plugin in plugins.iter() {
            plugin.web_shutdown().await?;
        }
        
        Ok(())
    }
    
    /// Register a web plugin
    async fn register_plugin(&self, plugin: Arc<dyn WebPlugin>) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        plugins.push(plugin);
        Ok(())
    }
    
    /// Unregister a web plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let index = plugins
            .iter()
            .position(|p| p.metadata().id == id)
            .ok_or(crate::PluginError::NotFound(id))?;
        
        plugins.remove(index);
        Ok(())
    }
    
    /// Get all web plugins
    async fn get_plugins(&self) -> Result<Vec<Arc<dyn WebPlugin>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.clone())
    }
    
    /// Get all web plugin routes
    async fn get_routes(&self) -> Result<Vec<(Uuid, WebPluginRoute)>> {
        let plugins = self.plugins.read().await;
        let mut routes = Vec::new();
        
        for plugin in plugins.iter() {
            let id = plugin.metadata().id;
            
            for route in plugin.get_routes() {
                routes.push((id, route));
            }
        }
        
        Ok(routes)
    }
    
    /// Get all web plugin components
    async fn get_components(&self) -> Result<Vec<(Uuid, WebPluginComponent)>> {
        let plugins = self.plugins.read().await;
        let mut components = Vec::new();
        
        for plugin in plugins.iter() {
            let id = plugin.metadata().id;
            
            for component in plugin.get_ui_components() {
                components.push((id, component));
            }
        }
        
        Ok(components)
    }
    
    /// Get all web plugin endpoints
    async fn get_endpoints(&self) -> Result<Vec<(Uuid, WebPluginEndpoint)>> {
        let plugins = self.plugins.read().await;
        let mut endpoints = Vec::new();
        
        for plugin in plugins.iter() {
            let id = plugin.metadata().id;
            
            for endpoint in plugin.get_api_endpoints() {
                endpoints.push((id, endpoint));
            }
        }
        
        Ok(endpoints)
    }
} 