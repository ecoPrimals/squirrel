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
    http::Method,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::AppState;
use crate::plugins_legacy as legacy;
use crate::plugins_legacy::{
    WebEndpoint as LegacyWebEndpoint,
    HttpMethod as LegacyHttpMethod,
    WebComponent as LegacyWebComponent,
    PluginInfo as LegacyPluginInfo,
    EndpointInfo as LegacyEndpointInfo,
    ComponentInfo as LegacyComponentInfo,
};

// New imports for the unified plugin system
use crate::plugins::{
    WebPluginRegistry, 
    model::{WebRequest, WebResponse, HttpMethod},
    example::ExamplePlugin,
    adapter::LegacyWebPluginAdapter,
};

/// Initialize the plugin system with the unified registry
///
/// This function will initialize the plugin system using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn init_plugin_system() -> Result<(legacy::PluginManager, WebPluginRegistry)> {
    // Initialize both the legacy plugin manager and modern plugin registry
    info!("Initializing web plugin system (adapter)");
    
    // Create plugin directory if it doesn't exist
    if !std::path::Path::new(legacy::PLUGIN_DIR).exists() {
        std::fs::create_dir_all(legacy::PLUGIN_DIR)
            .context("Failed to create plugin directory")?;
    }
    
    // Initialize the legacy plugin manager
    let plugin_manager = legacy::PluginManager::new();
    
    // Initialize the modern plugin registry
    let plugin_registry = WebPluginRegistry::new();
    
    // Register the example plugin with the modern registry
    let example_plugin = ExamplePlugin::new();
    plugin_registry.register_plugin(example_plugin).await?;
    
    // Load plugins from the plugin directory
    plugin_registry.load_plugins_from_directory(legacy::PLUGIN_DIR).await?;
    
    info!("Web plugin system initialized (adapter)");
    Ok((plugin_manager, plugin_registry))
}

/// Create plugin routes with the unified registry
///
/// This function will create routes for plugins using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn create_plugin_routes<S>(mut router: Router<S>, state: Arc<AppState>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    // For now, use the legacy plugin routes and add modern plugin routes
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
    
    // If we have a modern plugin registry, add plugin-specific routes
    if let Some(plugin_registry) = &state.plugin_registry {
        // Add routes for each registered endpoint in the modern registry
        let endpoints = plugin_registry.get_endpoints().await;
        
        for (plugin_id, endpoint) in endpoints {
            debug!("Adding route for endpoint: {} {}", endpoint.method as u8, endpoint.path);
            
            // Add routes dynamically based on HTTP method
            let route_path = format!("/api/plugins/{}/endpoints{}", plugin_id, endpoint.path);
            let plugin_registry_arc = plugin_registry.clone();
            
            // Create handler for this specific route
            let handler = move |request: axum::extract::Json<Option<Value>>| {
                let plugin_registry = plugin_registry_arc.clone();
                let endpoint_path = endpoint.path.clone();
                let method = endpoint.method;
                
                async move {
                    // Convert input to WebRequest
                    let web_request = WebRequest::new(endpoint_path, method)
                        .with_body(request.0.unwrap_or(json!({})));
                    
                    // Handle the request with the plugin registry
                    match plugin_registry.handle_request(web_request).await {
                        Ok(response) => {
                            // Convert WebResponse to axum Response
                            let status_code = response.status as u16;
                            let body = response.body.unwrap_or(json!({}));
                            
                            // Create response with appropriate status code
                            let status = axum::http::StatusCode::from_u16(status_code)
                                .unwrap_or(axum::http::StatusCode::OK);
                            
                            (status, Json(body))
                        },
                        Err(err) => {
                            // Handle error
                            error!("Error handling request: {}", err);
                            let status = axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                            (status, Json(json!({ "error": err.to_string() })))
                        }
                    }
                }
            };
            
            // Add route for the endpoint based on HTTP method
            router = match endpoint.method {
                HttpMethod::Get => router.route(&route_path, get(handler)),
                HttpMethod::Post => router.route(&route_path, post(handler)),
                HttpMethod::Put => router.route(&route_path, axum::routing::put(handler)),
                HttpMethod::Delete => router.route(&route_path, axum::routing::delete(handler)),
                HttpMethod::Patch => router.route(&route_path, axum::routing::patch(handler)),
                HttpMethod::Options => router.route(&route_path, axum::routing::options(handler)),
                HttpMethod::Head => router.route(&route_path, axum::routing::get(handler)),
            };
        }
        
        // Add routes for components
        let state_clone = state.clone();
        router = router.route("/api/plugins/components/:id/markup", post(move |
            state: State<S>,
            axum::extract::Path(component_id): axum::extract::Path<String>,
            axum::extract::Json(props): axum::extract::Json<Value>,
        | async move {
            // We need to convert from the generic state to our specific state
            let app_state = state_clone.clone();
            
            if let Some(plugin_registry) = &app_state.plugin_registry {
                match uuid::Uuid::parse_str(&component_id) {
                    Ok(uuid) => {
                        match plugin_registry.get_component_markup(uuid, props).await {
                            Ok(markup) => (axum::http::StatusCode::OK, markup),
                            Err(err) => (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR, 
                                format!("Error getting component markup: {}", err)
                            ),
                        }
                    },
                    Err(_) => (
                        axum::http::StatusCode::BAD_REQUEST, 
                        "Invalid component ID".to_string()
                    ),
                }
            } else {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR, 
                    "Plugin registry not initialized".to_string()
                )
            }
        }));
    }
    
    router
}

/// List all available plugins
///
/// This function will list all available plugins using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Value>>, String> {
    // Combine plugins from both legacy and modern systems
    debug!("Listing plugins (adapter)");
    
    let mut plugins = Vec::new();
    
    // Add plugins from legacy system
    let legacy_plugins = state.plugin_manager
        .list_plugins()
        .await
        .into_iter()
        .map(|p| json!({
            "id": p.id,
            "name": p.name,
            "version": p.version,
            "system": "legacy"
        }))
        .collect::<Vec<_>>();
    
    plugins.extend(legacy_plugins);
    
    // Add plugins from modern system if available
    if let Some(plugin_registry) = &state.plugin_registry {
        let modern_plugins = plugin_registry
            .get_plugins()
            .await
            .into_iter()
            .map(|p| json!({
                "id": p.metadata().id,
                "name": p.metadata().name,
                "version": p.metadata().version,
                "description": p.metadata().description,
                "author": p.metadata().author,
                "system": "modern"
            }))
            .collect::<Vec<_>>();
        
        plugins.extend(modern_plugins);
    }
    
    Ok(Json(plugins))
}

/// List all available plugin endpoints
///
/// This function will list all available plugin endpoints using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_endpoints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Value>>, String> {
    // Combine endpoints from both legacy and modern systems
    debug!("Listing plugin endpoints (adapter)");
    
    let mut endpoints = Vec::new();
    
    // Add endpoints from legacy system
    let legacy_endpoints = state.plugin_manager
        .get_endpoints::<()>()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(plugin_id, endpoint)| {
            json!({
                "plugin_id": plugin_id,
                "path": endpoint.path,
                "method": format!("{:?}", endpoint.method),
                "permissions": endpoint.permissions,
                "system": "legacy"
            })
        })
        .collect::<Vec<_>>();
    
    endpoints.extend(legacy_endpoints);
    
    // Add endpoints from modern system if available
    if let Some(plugin_registry) = &state.plugin_registry {
        let modern_endpoints = plugin_registry
            .get_endpoints()
            .await
            .into_iter()
            .map(|(plugin_id, endpoint)| {
                json!({
                    "plugin_id": plugin_id,
                    "id": endpoint.id,
                    "path": endpoint.path,
                    "method": format!("{:?}", endpoint.method),
                    "description": endpoint.description,
                    "permissions": endpoint.permissions,
                    "is_public": endpoint.is_public,
                    "is_admin": endpoint.is_admin,
                    "tags": endpoint.tags,
                    "system": "modern"
                })
            })
            .collect::<Vec<_>>();
        
        endpoints.extend(modern_endpoints);
    }
    
    Ok(Json(endpoints))
}

/// List all available plugin components
///
/// This function will list all available plugin components using the unified plugin registry.
/// During migration, it will maintain compatibility with both systems.
pub async fn list_components(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Value>>, String> {
    // Combine components from both legacy and modern systems
    debug!("Listing plugin components (adapter)");
    
    let mut components = Vec::new();
    
    // Add components from legacy system
    let legacy_components = state.plugin_manager
        .get_components::<()>()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(plugin_id, component)| {
            json!({
                "plugin_id": plugin_id,
                "name": component.name,
                "component_type": component.component_type,
                "mount_point": component.mount_point,
                "system": "legacy"
            })
        })
        .collect::<Vec<_>>();
    
    components.extend(legacy_components);
    
    // Add components from modern system if available
    if let Some(plugin_registry) = &state.plugin_registry {
        let modern_components = plugin_registry
            .get_components()
            .await
            .into_iter()
            .map(|(plugin_id, component)| {
                json!({
                    "plugin_id": plugin_id,
                    "id": component.id,
                    "name": component.name,
                    "description": component.description,
                    "component_type": format!("{:?}", component.component_type),
                    "properties": component.properties,
                    "route": component.route,
                    "priority": component.priority,
                    "permissions": component.permissions,
                    "parent": component.parent,
                    "icon": component.icon,
                    "system": "modern"
                })
            })
            .collect::<Vec<_>>();
        
        components.extend(modern_components);
    }
    
    Ok(Json(components))
}

/// Convert between legacy and unified HTTP methods
///
/// This function will be used to convert between the legacy HttpMethod enum
/// and the unified HttpMethod enum.
pub fn convert_http_method(method: LegacyHttpMethod) -> HttpMethod {
    match method {
        LegacyHttpMethod::Get => HttpMethod::Get,
        LegacyHttpMethod::Post => HttpMethod::Post,
        LegacyHttpMethod::Put => HttpMethod::Put,
        LegacyHttpMethod::Delete => HttpMethod::Delete,
        LegacyHttpMethod::Patch => HttpMethod::Patch,
        LegacyHttpMethod::Options => HttpMethod::Options,
        LegacyHttpMethod::Head => HttpMethod::Head,
    }
}

/// Convert between legacy and unified WebEndpoint structs
///
/// This function will be used to convert between the legacy WebEndpoint struct
/// and the unified WebEndpoint struct.
pub fn convert_legacy_endpoint(endpoint: &LegacyWebEndpoint) -> crate::plugins::model::WebEndpoint {
    let mut new_endpoint = crate::plugins::model::WebEndpoint::new(
        endpoint.path.clone(),
        convert_http_method(endpoint.method),
        format!("Legacy endpoint: {}", endpoint.path),
    );
    
    for permission in &endpoint.permissions {
        new_endpoint = new_endpoint.with_permission(permission.clone());
    }
    
    new_endpoint
}

/// Register a legacy plugin with the modern registry
///
/// This function registers a legacy plugin with the modern registry using the adapter.
pub async fn register_legacy_plugin(
    registry: &WebPluginRegistry,
    plugin: Box<dyn legacy::WebPlugin>,
) -> Result<()> {
    let adapter = LegacyWebPluginAdapter::new(plugin);
    registry.register_plugin(adapter).await
} 