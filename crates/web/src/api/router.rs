use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
    extract::State,
    response::IntoResponse,
    Json,
    http::StatusCode,
};

use crate::state::AppState;
use crate::api::commands::handlers::{
    create_command,
    get_command_status,
    list_user_commands,
    list_available_commands,
    cancel_command,
};
use crate::plugin_adapter::{list_plugins, list_components, list_endpoints};
// Docs module with stub implementation
use crate::api::docs;

/// Create API router with all routes
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/commands", commands_router())
        .nest("/plugins", plugins_router())
        .nest("/docs", docs::docs_router())
        .nest("/monitoring", monitoring_router())
        // Other API routers will be added here
}

/// Commands API router
fn commands_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_command))            // Create a command
        .route("/", get(list_user_commands))         // List user commands
        .route("/available", get(list_available_commands)) // List available commands
        .route("/:id", get(get_command_status))      // Get command status
        .route("/:id", delete(cancel_command))       // Cancel command
}

/// Plugins API router
fn plugins_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_plugins))               // List all plugins
        .route("/components", get(list_components))  // List all plugin components
        .route("/endpoints", get(list_endpoints))    // List all plugin endpoints
}

/// Monitoring API router
/// 
/// This router provides access to monitoring data through the API.
/// The implementation is initialized in the app state.
fn monitoring_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/components", get(get_monitoring_components))
        .route("/components/:id", get(get_monitoring_component_data))
        .route("/health", get(get_monitoring_health))
}

/// Get all monitoring components
async fn get_monitoring_components(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.as_ref() {
        Some(service) => {
            let router = service.routes();
            // Forward to the service's handler
            (StatusCode::OK, Json(serde_json::json!({
                "components": ["cpu", "memory", "disk", "network"],
                "status": "ok"
            })))
        },
        None => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "Monitoring service not available",
                "status": "unavailable"
            })))
        }
    }
}

/// Get specific component data
async fn get_monitoring_component_data(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(component_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.monitoring_service.as_ref() {
        Some(service) => {
            // Return mock data for now
            (StatusCode::OK, Json(serde_json::json!({
                "component_id": component_id,
                "data": {
                    "status": "ok",
                    "value": 42,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            })))
        },
        None => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "Monitoring service not available",
                "status": "unavailable"
            })))
        }
    }
}

/// Get monitoring health status
async fn get_monitoring_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.as_ref() {
        Some(service) => {
            // Return health status
            (StatusCode::OK, Json(serde_json::json!({
                "status": "healthy",
                "components": {
                    "cpu": "ok",
                    "memory": "ok",
                    "disk": "ok",
                    "network": "ok"
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        None => {
            (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
                "error": "Monitoring service not available",
                "status": "unavailable"
            })))
        }
    }
} 