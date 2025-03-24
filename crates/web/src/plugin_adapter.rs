//! Plugin adapter module
//!
//! This module provides adapter functionality to bridge between the existing 
//! web plugin system and the unified plugin system.

use std::sync::Arc;
use anyhow::{Result, Context};
use axum::{
    Router,
    routing::{get, post},
    extract::State,
    Json,
};
use serde::{Serialize, Deserialize};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::AppState;
use crate::plugins::{
    WebEndpoint as LegacyWebEndpoint,
    HttpMethod as LegacyHttpMethod,
    WebComponent as LegacyWebComponent,
    PluginInfo as LegacyPluginInfo,
    EndpointInfo as LegacyEndpointInfo,
    ComponentInfo as LegacyComponentInfo,
};

// These imports will be uncommented once the unified plugin system is available
// use squirrel_plugins::registry::PluginRegistry;
// use squirrel_plugins::web::{WebPlugin, WebEndpoint, HttpMethod, WebComponent};

/// Initialize the plugin system with the unified registry
///
/// This function will initialize the plugin system using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn init_plugin_system() -> Result<crate::plugins::PluginManager> {
    // For now, initialize the legacy plugin system
    // In the future, this will use the unified plugin registry
    info!("Initializing web plugin system (adapter)");
    
    // Create plugin directory if it doesn't exist
    if !std::path::Path::new(crate::plugins::PLUGIN_DIR).exists() {
        std::fs::create_dir_all(crate::plugins::PLUGIN_DIR)
            .context("Failed to create plugin directory")?;
    }
    
    // Initialize the plugin manager
    let plugin_manager = crate::plugins::PluginManager::new();
    
    // When migrating to the unified system, we'll add code like:
    // let registry = state.plugin_registry.clone();
    // Load web plugins from the unified registry
    
    info!("Web plugin system initialized (adapter)");
    Ok(plugin_manager)
}

/// Create plugin routes with the unified registry
///
/// This function will create routes for plugins using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn create_plugin_routes<S>(router: Router<S>, state: Arc<AppState>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // For now, use the legacy plugin routes
    // In the future, this will use the unified plugin registry
    debug!("Creating plugin routes (adapter)");
    
    // Create state-specific handlers that match the router's state type
    let state_clone = state.clone();
    let list_plugins_handler = move || async move {
        let inner_state = state_clone.clone();
        list_plugins(State(inner_state)).await
    };
    
    let state_clone = state.clone();
    let list_endpoints_handler = move || async move {
        let inner_state = state_clone.clone();
        list_endpoints(State(inner_state)).await
    };
    
    let state_clone = state.clone();
    let list_components_handler = move || async move {
        let inner_state = state_clone.clone();
        list_components(State(inner_state)).await
    };
    
    router
        .route("/api/plugins", get(list_plugins_handler))
        .route("/api/plugins/endpoints", get(list_endpoints_handler))
        .route("/api/plugins/components", get(list_components_handler))
}

/// List all available plugins
///
/// This function will list all available plugins using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LegacyPluginInfo>>, String> {
    // For now, use the legacy plugin manager
    // In the future, this will use the unified plugin registry
    debug!("Listing plugins (adapter)");
    
    let plugins = state.plugin_manager
        .list_plugins()
        .await;
    
    Ok(Json(plugins))
}

/// List all available plugin endpoints
///
/// This function will list all available plugin endpoints using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_endpoints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LegacyEndpointInfo>>, String> {
    // For now, use the legacy plugin manager
    // In the future, this will use the unified plugin registry
    debug!("Listing plugin endpoints (adapter)");
    
    let endpoints = state.plugin_manager
        .get_endpoints::<()>()
        .await
        .map_err(|e| e.to_string())?;
    
    let endpoints_info = endpoints
        .into_iter()
        .map(|(plugin_id, endpoint)| {
            LegacyEndpointInfo {
                plugin_id,
                path: endpoint.path,
                method: format!("{:?}", endpoint.method),
                permissions: endpoint.permissions,
            }
        })
        .collect();
    
    Ok(Json(endpoints_info))
}

/// List all available plugin components
///
/// This function will list all available plugin components using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_components(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LegacyComponentInfo>>, String> {
    // For now, use the legacy plugin manager
    // In the future, this will use the unified plugin registry
    debug!("Listing plugin components (adapter)");
    
    let components = state.plugin_manager
        .get_components::<()>()
        .await
        .map_err(|e| e.to_string())?;
    
    let components_info = components
        .into_iter()
        .map(|(plugin_id, component)| {
            LegacyComponentInfo {
                plugin_id,
                name: component.name,
                component_type: component.component_type,
                mount_point: component.mount_point,
            }
        })
        .collect();
    
    Ok(Json(components_info))
}

/// Create plugin endpoint routes
///
/// This function will create routes for plugin endpoints using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn create_plugin_endpoint_routes(
    state: Arc<AppState>,
) -> Result<Router<Arc<AppState>>> {
    // For now, use the legacy plugin manager
    // In the future, this will use the unified plugin registry
    debug!("Creating plugin endpoint routes (adapter)");
    
    // Get all plugin endpoints
    let endpoints = state.plugin_manager
        .get_endpoints::<()>()
        .await?;
    
    // Create a router with all plugin endpoints
    let mut router = Router::new();
    
    // Configure each endpoint route
    for (plugin_id, endpoint) in endpoints {
        let endpoint_path = format!("/api/plugins/{}/endpoints{}", plugin_id, endpoint.path);
        
        // Add the route to the router
        debug!("Adding plugin endpoint: {}", endpoint_path);
        
        // Handle different HTTP methods
        match endpoint.method {
            LegacyHttpMethod::Get => {
                // Add GET route
            },
            LegacyHttpMethod::Post => {
                // Add POST route
            },
            // Handle other methods...
            _ => {
                // Log unsupported method
                error!("Unsupported HTTP method: {:?}", endpoint.method);
            }
        }
    }
    
    Ok(router)
}

/// Convert between legacy and unified HTTP methods
///
/// This function will be used to convert between the legacy HttpMethod enum
/// and the unified HttpMethod enum.
#[allow(dead_code)]
fn convert_http_method(_method: LegacyHttpMethod) -> /* HttpMethod */ LegacyHttpMethod {
    // For now, return the legacy method
    // In the future, this will convert to the unified HttpMethod enum
    _method
}

/// Convert between legacy and unified WebEndpoint structs
///
/// This function will be used to convert between the legacy WebEndpoint struct
/// and the unified WebEndpoint struct.
#[allow(dead_code)]
fn convert_web_endpoint(_endpoint: LegacyWebEndpoint) -> /* WebEndpoint */ LegacyWebEndpoint {
    // For now, return the legacy endpoint
    // In the future, this will convert to the unified WebEndpoint struct
    _endpoint
}

/// Convert between legacy and unified WebComponent structs
///
/// This function will be used to convert between the legacy WebComponent struct
/// and the unified WebComponent struct.
#[allow(dead_code)]
fn convert_web_component(_component: LegacyWebComponent) -> /* WebComponent */ LegacyWebComponent {
    // For now, return the legacy component
    // In the future, this will convert to the unified WebComponent struct
    _component
} 