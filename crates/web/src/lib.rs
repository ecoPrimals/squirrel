//! Web interface for code analysis and reporting services.

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
    /// MCP client for interacting with analysis services
    pub mcp_client: Arc<Box<dyn mcp::McpClient>>,
    /// Database connection pool
    pub db_pool: Arc<sqlx::PgPool>,
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
    pub mcp_config: mcp::SessionConfig,
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

/// Request to create a new analysis job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnalysisJobRequest {
    /// Repository URL to analyze
    pub repository_url: String,
    /// Branch or commit to analyze
    pub git_ref: String,
    /// Analysis configuration
    pub config: analysis::AnalysisConfig,
}

/// Response for a created analysis job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnalysisJobResponse {
    /// Job ID
    pub job_id: Uuid,
    /// Status URL to check job progress
    pub status_url: String,
}

/// Status of an analysis job
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

/// State of an analysis job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Cloning,
    Analyzing,
    GeneratingReport,
    Completed,
    Failed,
}

/// Initialize the web application
pub async fn init_app(config: ServerConfig) -> Result<Router> {
    // Initialize database connection
    let db_pool = Arc::new(
        sqlx::PgPool::connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?,
    );

    // Initialize MCP client
    let mcp_client: Arc<Box<dyn mcp::McpClient>> = Arc::new(Box::new(
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
                .allow_origin(config.cors_config.allowed_origins)
                .allow_methods(config.cors_config.allowed_methods)
                .allow_headers(config.cors_config.allowed_headers),
        )
        .with_state(state);

    Ok(app)
} 