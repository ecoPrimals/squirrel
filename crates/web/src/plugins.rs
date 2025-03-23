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

use squirrel_plugins::{PluginManager, Plugin};
use squirrel_plugins::web::{
    DefaultWebPluginManager, HttpMethod, WebPlugin, WebPluginComponent, WebPluginEndpoint,
    WebPluginManager, WebPluginRoute,
};

use crate::state::AppState;

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

/// Plugin info response
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
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
    
    /// Plugin status
    pub status: String,
    
    /// Plugin capabilities
    pub capabilities: Vec<String>,
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
        match plugin_manager.load_plugins(plugin_dir).await {
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
    plugin_manager.initialize().await?;
    
    Ok(plugin_manager)
}

/// Create plugin routes
pub fn create_plugin_routes<S>(router: Router<S>, state: Arc<AppState>) -> Router<S> 
where
    S: Clone + Send + Sync + 'static,
{
    // Add our plugins API endpoints
    let plugins_router = router.clone();
    
    // Add dynamic plugin endpoint routes
    match create_plugin_endpoint_routes(state.clone()) {
        Ok(plugin_routes) => {
            plugins_router.merge(plugin_routes)
        },
        Err(e) => {
            error!("Failed to create plugin endpoint routes: {}", e);
            plugins_router
        }
    }
}

/// List all plugins
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PluginInfo>>, String> {
    let plugin_manager = &state.plugin_manager;
    let plugins = plugin_manager.get_plugins().await
        .map_err(|e| e.to_string())?;
    
    let info = plugins.iter().map(|plugin| {
        let metadata = plugin.metadata();
        PluginInfo {
            id: metadata.id,
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            author: metadata.author.clone(),
            status: "Active".to_string(), // Simplified for now
            capabilities: metadata.capabilities.clone(),
        }
    }).collect();
    
    Ok(Json(info))
}

/// List all plugin components
pub async fn list_components(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ComponentInfo>>, String> {
    let plugin_manager = &state.plugin_manager;
    let components = plugin_manager.get_components::<WebPluginComponent>().await
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
    let endpoints = plugin_manager.get_endpoints::<WebPluginEndpoint>().await
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
pub fn create_plugin_endpoint_routes(
    state: Arc<AppState>,
) -> Result<Router<Arc<AppState>>> {
    let mut router = Router::new();
    
    let plugin_manager = &state.plugin_manager;
    let endpoints = plugin_manager.get_endpoints::<WebPluginEndpoint>().await?;
    
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
                let plugins = plugin_manager.get_plugins().await
                    .map_err(|e| e.to_string())?;
                
                let plugin = plugins.iter()
                    .find(|p| p.metadata().id == plugin_id)
                    .ok_or_else(|| format!("Plugin not found: {}", plugin_id))?;
                
                // Handle the endpoint request
                let response = plugin.handle_web_endpoint(&endpoint, body).await
                    .map_err(|e| e.to_string())?;
                
                Ok::<Json<Value>, String>(Json(response))
            }
        };
        
        // Add the route based on the HTTP method
        router = match method {
            Method::GET => router.route(&path, get(handler)),
            Method::POST => router.route(&path, post(handler)),
            Method::PUT => router.route(&path, axum::routing::put(handler)),
            Method::DELETE => router.route(&path, axum::routing::delete(handler)),
            Method::PATCH => router.route(&path, axum::routing::patch(handler)),
            _ => router, // Skip other methods for now
        };
    }
    
    Ok(router)
} 