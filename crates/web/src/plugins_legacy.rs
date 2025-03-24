//! Web plugin integration
//!
//! This module provides integration between the web interface and the plugin system.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::State,
    http::Method,
    routing::{get, post, Router},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info};
use uuid::Uuid;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::AppState;

/// HTTP methods supported by the plugin system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

// Stub Plugin trait
#[async_trait]
pub trait Plugin: Send + Sync + std::fmt::Debug {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<()>;
    
    /// Shutdown the plugin
    async fn shutdown(&self) -> Result<()>;
    
    /// Plugin feature check
    fn has_feature(&self, feature: &str) -> bool {
        self.metadata().capabilities.contains(&feature.to_string())
    }
}

// Stub WebPlugin trait
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get the endpoints provided by this plugin
    fn get_endpoints(&self) -> Vec<WebEndpoint>;
    
    /// Get the components provided by this plugin
    fn get_components(&self) -> Vec<WebComponent>;
    
    /// Handle web endpoint request
    async fn handle_web_endpoint(&self, endpoint: &WebEndpoint, data: Option<Value>) -> Result<Value>;
}

/// Web endpoint definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebEndpoint {
    /// Path to the endpoint
    pub path: String,
    
    /// HTTP method
    pub method: HttpMethod,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Plugin metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: Uuid,
    
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: String,
    
    /// Plugin capabilities
    pub capabilities: Vec<String>,
    
    /// Plugin dependencies (IDs of plugins this plugin depends on)
    pub dependencies: Vec<Uuid>,
}

// Stub PluginManager
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<Uuid, Box<dyn WebPlugin>>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn load_plugins(&self) -> Result<usize> {
        // Stub implementation
        Ok(0) // Return 0 to indicate no plugins were loaded
    }
    
    pub async fn get_plugin(&self, _id: &Uuid) -> Option<Box<dyn WebPlugin>> {
        // Stub implementation
        None
    }
    
    pub async fn register_plugin(&self, plugin: Box<dyn WebPlugin>) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let id = plugin.metadata().id;
        plugins.insert(id, plugin);
        Ok(())
    }
    
    pub async fn get_plugins(&self) -> Vec<Uuid> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }
    
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().await;
        plugins.values()
            .map(|plugin| {
                let metadata = plugin.metadata();
                PluginInfo {
                    id: metadata.id.to_string(),
                    name: metadata.name.clone(),
                    version: metadata.version.clone(),
                }
            })
            .collect()
    }
    
    pub async fn get_endpoints<T>(&self) -> Result<Vec<(Uuid, WebEndpoint)>> {
        let plugins = self.plugins.read().await;
        let mut endpoints = Vec::new();
        
        for (id, plugin) in plugins.iter() {
            for endpoint in plugin.get_endpoints() {
                endpoints.push((*id, endpoint));
            }
        }
        
        Ok(endpoints)
    }
    
    pub async fn get_components<T>(&self) -> Result<Vec<(Uuid, WebComponent)>> {
        // Stub implementation for components
        Ok(Vec::new())
    }
    
    pub async fn initialize(&self) -> Result<()> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            plugin.initialize().await?;
        }
        Ok(())
    }
}

// Define a WebComponent type for the above method
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebComponent {
    /// Component name
    pub name: String,
    
    /// Component type
    pub component_type: String,
    
    /// Mount point
    pub mount_point: String,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
}

/// Plugin discovery directory
pub const PLUGIN_DIR: &str = "plugins";

/// Convert from plugin HTTP method to axum HTTP method
fn convert_method(method: &HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::GET,
        HttpMethod::Post => Method::POST,
        HttpMethod::Put => Method::PUT,
        HttpMethod::Delete => Method::DELETE,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Options => Method::OPTIONS,
        HttpMethod::Head => Method::HEAD,
    }
}

/// Component info response
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Component name
    pub name: String,
    
    /// Component type
    pub component_type: String,
    
    /// Mount point
    pub mount_point: String,
}

/// Endpoint info response
#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// Endpoint path
    pub path: String,
    
    /// HTTP method
    pub method: String,
    
    /// Required permissions
    pub permissions: Vec<String>,
}

/// Initialize the plugin system
pub async fn init_plugin_system() -> Result<PluginManager> {
    let plugin_manager = PluginManager::new();
    
    // Load plugins from the plugins directory
    let plugin_dir = Path::new(PLUGIN_DIR);
    if plugin_dir.exists() {
        info!("Loading plugins from {:?}", plugin_dir);
        match plugin_manager.load_plugins().await {
            Ok(count) => {
                info!("Successfully loaded {} plugins", count);
            },
            Err(e) => {
                error!("Failed to load plugins: {}", e);
                // Continue with the plugins that could be loaded, if any
            }
        }
    } else {
        info!("Plugin directory {:?} doesn't exist, skipping plugin loading", plugin_dir);
    }
    
    // Initialize the plugin system
    if let Err(e) = plugin_manager.initialize().await {
        error!("Failed to initialize plugin system: {}", e);
    }
    
    Ok(plugin_manager)
}

/// Create plugin routes
pub async fn create_plugin_routes<S>(router: Router<S>, _state: Arc<AppState>) -> Router<S> 
where
    S: Clone + Send + Sync + 'static,
{
    // For now, just return the original router
    // In the future, we would add plugin routes here
    router
}

/// List all plugins
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PluginInfo>>, String> {
    let plugin_manager = &state.plugin_manager;
    let plugins = plugin_manager.list_plugins().await;
    
    Ok(Json(plugins))
}

/// List all plugin components
pub async fn list_components(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ComponentInfo>>, String> {
    let plugin_manager = &state.plugin_manager;
    let components = plugin_manager.get_components::<WebComponent>().await
        .map_err(|e| e.to_string())?;
    
    let info = components.iter().map(|(plugin_id, component)| {
        ComponentInfo {
            plugin_id: *plugin_id,
            name: component.name.clone(),
            component_type: component.component_type.clone(),
            mount_point: component.mount_point.clone(),
        }
    }).collect();
    
    Ok(Json(info))
}

/// List all plugin endpoints
pub async fn list_endpoints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<EndpointInfo>>, String> {
    let plugin_manager = &state.plugin_manager;
    let endpoints = plugin_manager.get_endpoints::<WebEndpoint>().await
        .map_err(|e| e.to_string())?;
    
    let info = endpoints.iter().map(|(plugin_id, endpoint)| {
        EndpointInfo {
            plugin_id: *plugin_id,
            path: endpoint.path.clone(),
            method: format!("{:?}", endpoint.method),
            permissions: endpoint.permissions.clone(),
        }
    }).collect();
    
    Ok(Json(info))
}

/// Create API endpoint routes for plugins
pub async fn create_plugin_endpoint_routes(
    state: Arc<AppState>,
) -> Result<Router<Arc<AppState>>> {
    let mut router = Router::new();
    
    let plugin_manager = &state.plugin_manager;
    let endpoints = plugin_manager.get_endpoints::<WebEndpoint>().await?;
    
    for (plugin_id, endpoint) in endpoints {
        let plugin_id_clone = plugin_id;
        let endpoint_clone = endpoint.clone();
        let path = format!("/api/plugins/{}/endpoints{}", plugin_id, endpoint.path);
        let method = convert_method(&endpoint.method);
        
        debug!("Adding plugin endpoint route: {} {}", method, path);
        
        let handler = move |State(app_state): State<Arc<AppState>>, Json(body): Json<Value>| {
            let plugin_id = plugin_id_clone;
            let endpoint = endpoint_clone.clone();
            async move {
                let plugin_manager = &app_state.plugin_manager;
                let plugin = plugin_manager.get_plugin(&plugin_id).await
                    .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
                
                let result = plugin.handle_web_endpoint(&endpoint, Some(body)).await
                    .map_err(|e| format!("Plugin error: {}", e))?;
                
                Ok::<Json<Value>, String>(Json(result))
            }
        };
        
        // Create a router with the appropriate HTTP method
        let route = match method {
            Method::GET => Router::new().route("/", get(handler)),
            Method::POST => Router::new().route("/", post(handler)),
            Method::PUT => Router::new().route("/", axum::routing::put(handler)),
            Method::DELETE => Router::new().route("/", axum::routing::delete(handler)),
            Method::PATCH => Router::new().route("/", axum::routing::patch(handler)),
            Method::OPTIONS => Router::new().route("/", axum::routing::options(handler)),
            Method::HEAD => Router::new().route("/", axum::routing::head(handler)),
            _ => Router::new(),
        };
        
        router = router.nest(&path, route);
    }
    
    Ok(router)
} 