//! Web interface for the Squirrel system.

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

pub mod api;
pub mod auth;
pub mod handlers;
pub mod state;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// MCP client for interacting with services
    pub mcp_client: Arc<Box<dyn squirrel_mcp::McpClient>>,
    /// Database connection pool
    pub db_pool: Arc<sqlx::SqlitePool>,
}

/// Configuration for the web server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Host address to bind to
    pub bind_address: String,
    /// Port to listen on
    pub port: u16,
    /// Database connection URL
    pub database_url: String,
    /// MCP server configuration
    pub mcp_config: squirrel_mcp::SessionConfig,
    /// CORS configuration
    pub cors_config: CorsConfig,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
}

/// Request to create a new job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// Repository URL 
    pub repository_url: String,
    /// Branch or commit
    pub git_ref: String,
    /// Configuration
    pub config: serde_json::Value,
}

/// Response for a created job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobResponse {
    /// Job ID
    pub job_id: Uuid,
    /// Status URL to check job progress
    pub status_url: String,
}

/// Status of a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// Job ID
    pub job_id: Uuid,
    /// Current status
    pub status: JobState,
    /// Progress percentage
    pub progress: f32,
    /// Error message if any
    pub error: Option<String>,
    /// Result URL if completed
    pub result_url: Option<String>,
}

/// State of a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
}

/// Initialize the web application
pub async fn init_app(config: ServerConfig) -> Result<Router> {
    // Initialize database connection
    let db_pool = Arc::new(
        sqlx::SqlitePool::connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?,
    );

    // Initialize MCP client
    let mcp_client: Arc<Box<dyn squirrel_mcp::McpClient>> = Arc::new(Box::new(
        // TODO: Implement actual MCP client
        todo!("Implement MCP client"),
    ));

    // Create application state
    let state = AppState {
        mcp_client,
        db_pool,
    };

    // Create router with routes
    let app = Router::new()
        .route("/api/health", get(handlers::health::check))
        .route("/api/jobs", post(handlers::jobs::create))
        .route("/api/jobs/:id", get(handlers::jobs::status))
        .route("/api/jobs/:id/report", get(handlers::jobs::report))
        .layer(
            CorsLayer::new()
                // Convert Vec<String> to Vec<HeaderValue>
                .allow_origin(config.cors_config.allowed_origins.iter().map(|origin| {
                    origin.parse().unwrap_or_else(|_| panic!("Invalid origin: {}", origin))
                }).collect::<Vec<_>>())
                .allow_methods(config.cors_config.allowed_methods.iter().map(|method| {
                    method.parse().unwrap_or_else(|_| panic!("Invalid method: {}", method))
                }).collect::<Vec<_>>())
                .allow_headers(config.cors_config.allowed_headers.iter().map(|header| {
                    header.parse().unwrap_or_else(|_| panic!("Invalid header: {}", header))
                }).collect::<Vec<_>>()),
        )
        .with_state(state);

    Ok(app)
} 