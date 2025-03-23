use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;
use crate::api::commands::handlers::{
    create_command,
    get_command_status,
    list_user_commands,
    list_available_commands,
};

/// Create API router with all routes
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/commands", commands_router())
        // Other API routers will be added here
}

/// Commands API router
fn commands_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_command))            // Create a command
        .route("/", get(list_user_commands))         // List user commands
        .route("/available", get(list_available_commands)) // List available commands
        .route("/:id", get(get_command_status))      // Get command status
} 