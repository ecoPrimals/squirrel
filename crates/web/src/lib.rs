//! Web interface for the Squirrel system.

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

pub mod api;
pub mod auth;
pub mod handlers;
pub mod state;

/// Mock MCP client trait for the web interface
pub trait MockMCPClient: Send + Sync {
    /// Send a message to the MCP
    fn send_message(&self, message: &str) -> Result<String>;
    
    /// Receive a message from the MCP
    fn receive_message(&self) -> Result<String>;
}

/// Default implementation of MockMCPClient
#[derive(Debug)]
struct DefaultMockMCPClient;

impl MockMCPClient for DefaultMockMCPClient {
    fn send_message(&self, message: &str) -> Result<String> {
        Ok(format!("Sent: {}", message))
    }
    
    fn receive_message(&self) -> Result<String> {
        Ok("Mock response".to_string())
    }
}

/// Mock MCP session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionConfig {
    /// Host address
    pub host: String,
    /// Port
    pub port: u16,
    /// Connection timeout in seconds
    pub timeout: u64,
}

impl Default for MockSessionConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout: 30,
        }
    }
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// MCP client for interacting with services
    pub mcp_client: Arc<Box<dyn MockMCPClient>>,
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
    pub mcp_config: MockSessionConfig,
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
    let mcp_client: Arc<Box<dyn MockMCPClient>> = Arc::new(Box::new(
        DefaultMockMCPClient
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
                .allow_origin(tower_http::cors::AllowOrigin::any())
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