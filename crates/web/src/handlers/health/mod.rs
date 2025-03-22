//! Health check handler for the API.

use std::sync::Arc;
use axum::{
    response::IntoResponse, 
    routing::get,
    Router,
    extract::State,
    Json,
};
use serde_json::json;
use serde::Serialize;

use crate::AppState;

/// Health response structure
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Application version
    pub version: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Database status
    pub database: String,
    /// MCP status
    pub mcp: String,
}

/// Health check handler
async fn health_check() -> impl IntoResponse {
    "OK"
}

/// Health check routes
pub fn health_routes() -> Router {
    Router::new().route("/", get(health_check))
}

/// Basic health check endpoint
pub async fn check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": squirrel_core::build_info::version(),
    }))
}

/// Detailed health check endpoint
pub async fn check_detailed(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check database connection
    let db_status = match state.db.acquire().await {
        Ok(_) => "connected",
        Err(_) => "error",
    };

    // Check MCP connection
    let mcp_status = match &state.mcp {
        Some(mcp_client) => match mcp_client.receive_message() {
            Ok(_) => "Operational",
            Err(_) => "Error",
        },
        None => "Not configured",
    };

    // TODO: Get actual uptime
    let uptime = 0;

    let health = HealthResponse {
        status: "ok".to_string(),
        version: squirrel_core::build_info::version().to_string(),
        uptime,
        database: db_status.to_string(),
        mcp: mcp_status.to_string(),
    };

    Json(health)
} 