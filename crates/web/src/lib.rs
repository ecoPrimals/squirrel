use std::sync::Arc;
use anyhow::Result;
use axum::{Router, http::Method, routing::{get, post}};
use tower_http::cors::{CorsLayer, Any};
#[cfg(feature = "db")]
use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};
#[cfg(feature = "mock-db")]
use sqlx::{SqlitePool};
use serde::{Deserialize, Serialize};

pub mod auth;
mod handlers;
mod mcp;
pub mod api;
pub mod state;
pub mod websocket;
pub mod config;
pub mod db;

use crate::state::AppState;
use crate::config::Config;
use crate::db::SqlitePool as DbPool;
use auth::{AuthConfig, AuthService};
use mcp::{McpCommandClient, MockMcpClient};

pub use api::{CreateJobRequest, CreateJobResponse, JobStatus, JobState};
pub use api::commands::{
    CommandDefinition,
    CommandExecution,
    CommandStatus,
    CreateCommandRequest,
    CreateCommandResponse,
    CommandStatusResponse,
    CommandListResponse,
    CommandHistoryResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub database_url: String,
    pub mcp_config: MockSessionConfig,
    pub cors_config: CorsConfig,
    pub auth_config: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSessionConfig {
    pub host: String,
    pub port: u16,
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

#[cfg(feature = "mock-db")]
impl Default for AppState {
    fn default() -> Self {
        // Create a mock database pool
        let mock_db = SqlitePool::connect_lazy("sqlite::memory:")
            .expect("Failed to create mock database pool");
        
        let auth_config = AuthConfig::default();
        let config = Config::default();
        
        // Initialize WebSocket manager
        let ws_manager = websocket::init();
        
        // Create auth service
        let auth = AuthService::new(auth_config, mock_db.clone());
        
        // Create MCP clients
        let mcp_command = Arc::new(MockMcpClient::new(
            "localhost".to_string(), 
            8080
        )) as Arc<dyn McpCommandClient>;
        
        // Create command service
        let command_service = Arc::new(handlers::commands::MockCommandService::new(
            mcp_command.clone()
        )) as Arc<dyn handlers::commands::CommandService>;
        
        Self {
            db: mock_db,
            config,
            mcp: None,
            mcp_command: Some(mcp_command),
            ws_manager,
            auth,
            command_service: Some(command_service),
        }
    }
}

/// Initialize the database with migrations
#[cfg(feature = "db")]
pub async fn setup_database(database_url: &str) -> Result<DbPool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        Sqlite::create_database(database_url).await?;
    }

    // Connect to the database
    let pool = DbPool::connect(database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

#[cfg(feature = "mock-db")]
pub async fn setup_database(_database_url: &str) -> Result<DbPool> {
    // For mock-db, we just create an in-memory database
    let pool = DbPool::connect("sqlite::memory:").await?;
    
    // Run migrations even for in-memory database
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    Ok(pool)
}

/// Create the application router
pub async fn create_app(db: DbPool, config: Config) -> Router {
    // Initialize WebSocket manager
    let ws_manager = websocket::init();
    
    // Create auth service
    let auth = AuthService::new(AuthConfig::default(), db.clone());
    
    // Create MCP clients
    let mcp_command = Arc::new(MockMcpClient::new(
        "localhost".to_string(), 
        8080
    )) as Arc<dyn McpCommandClient>;

    // Create command service based on feature
    #[cfg(feature = "mock-db")]
    let command_service = Arc::new(handlers::commands::MockCommandService::new(
        mcp_command.clone()
    )) as Arc<dyn handlers::commands::CommandService>;
    
    #[cfg(feature = "db")]
    let command_service = Arc::new(handlers::commands::DbCommandService::new(
        db.clone(),
        mcp_command.clone(),
    )) as Arc<dyn handlers::commands::CommandService>;
    
    // Create app state
    let state = Arc::new(AppState {
        db,
        config,
        mcp: None, // Legacy client, deprecated
        mcp_command: Some(mcp_command),
        ws_manager,
        auth,
        command_service: Some(command_service),
    });

    // Create WebSocket handler for commands
    let _command_ws_handler = Arc::new(websocket::CommandWebSocketHandler::new(state.clone()));
    
    // Register the command WebSocket handler with the WebSocket manager
    // This would need a proper registration mechanism in a real implementation

    // Setup CORS
    let _cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    // Create the router with all routes
    
    
    Router::new()
        .route("/health", get(handlers::health::get_health))
        .route("/api/health", get(handlers::health::get_health))
        .nest("/api/commands", handlers::commands::command_routes())
        .route("/api/jobs", post(handlers::jobs::create_job))
        .route("/api/jobs", get(handlers::jobs::list_jobs))
        .route("/api/jobs/:id/status", get(handlers::jobs::get_job_status))
        .route("/api/jobs/:id/result", get(handlers::jobs::get_job_result))
        .route("/api/jobs/:id/cancel", post(handlers::jobs::cancel_job))
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        .route("/ws", get(websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}