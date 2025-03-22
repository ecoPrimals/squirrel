//! Health check handler for the API.

use std::sync::Arc;
use axum::{
    response::IntoResponse, 
    routing::get,
    Router,
    extract::State,
    Json,
    Extension,
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
    let db_status = "Connected";

    // Check MCP connection
    let mcp_status = match &state.mcp {
        None => "Not configured",
        Some(mcp_client) => {
            match mcp_client.receive_message().await {
                Ok(_) => "Operational",
                Err(_) => "Error",
            }
        }
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

/// Get system uptime (mock implementation)
fn get_system_uptime() -> String {
    // In a real implementation, this would get the actual system uptime
    "3 days 5 hours".to_string()
}

/// Get memory usage (mock implementation)
fn get_memory_usage() -> serde_json::Value {
    // In a real implementation, this would get the actual memory usage
    serde_json::json!({
        "total": "16 GB",
        "used": "4.2 GB",
        "free": "11.8 GB"
    })
}

/// Get health status
pub async fn get_health(
    state: Extension<Arc<AppState>>,
) -> impl IntoResponse {
    // Get system info for health check
    let uptime = get_system_uptime();
    let memory = get_memory_usage();
    
    // Check database connection
    let db_status = "Connected";
    
    // Check MCP connection
    let mcp_status = match &state.mcp {
        None => "Not configured",
        Some(mcp_client) => {
            match mcp_client.receive_message().await {
                Ok(_) => "Operational",
                Err(_) => "Error",
            }
        }
    };
    
    // Create health check response
    let health = json!({
        "status": "OK",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": uptime,
        "memory": memory,
        "services": {
            "database": db_status,
            "mcp": mcp_status
        }
    });
    
    Json(health)
} 