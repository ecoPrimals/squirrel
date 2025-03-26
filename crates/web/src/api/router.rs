use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, delete},
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