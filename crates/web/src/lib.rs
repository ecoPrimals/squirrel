use std::sync::Arc;
use anyhow::Result;
use axum::{Router, http::Method, routing::{get, post}};
use tower_http::cors::{CorsLayer, Any};
#[cfg(feature = "db")]
use sqlx::{migrate::MigrateDatabase, Sqlite};
#[cfg(feature = "mock-db")]
// use sqlx::SqlitePool; // Commented out since it's unused
use serde::{Deserialize, Serialize};
// use chrono; // Commented out since it's unused

pub mod auth;
mod handlers;
pub mod mcp;
pub mod api;
pub mod state;
pub mod websocket;
pub mod config;
pub mod db;
pub mod plugins;

use crate::state::AppState;
use crate::config::Config;
use crate::db::SqlitePool as DbPool;
use auth::{AuthConfig, AuthService};
use mcp::{McpClient, MockMcpClient};
use api::commands::{CommandService, CommandServiceImpl, CommandRepository};
use squirrel_plugins::PluginManager;

// Fix import for repositories
#[cfg(feature = "db")]
use api::commands::repository::SqliteCommandRepository;
#[cfg(feature = "mock-db")]
use api::commands::repository::MockCommandRepository;

pub use api::commands::{
    CommandDefinition,
    CommandExecution,
    CommandStatus,
    CreateCommandRequest,
    CreateCommandResponse,
    CommandStatusResponse,
    CommandSummary,
    AvailableCommand,
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
    
    // Create MCP client
    let mcp_client = Arc::new(MockMcpClient::new(
        "localhost".to_string(), 
        8080
    )) as Arc<dyn McpClient>;

    // Create command repository based on feature
    #[cfg(feature = "db")]
    let command_repository = Arc::new(SqliteCommandRepository::new(db.clone())) as Arc<dyn CommandRepository>;
    
    #[cfg(feature = "mock-db")]
    let command_repository = Arc::new(MockCommandRepository::new()) as Arc<dyn CommandRepository>;
    
    // Create command service
    let command_service = Arc::new(CommandServiceImpl::new(
        command_repository,
        mcp_client.clone(),
    )) as Arc<dyn CommandService>;
    
    // Initialize plugin manager
    let plugin_manager = Arc::new(plugins::init_plugin_system().await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize plugin system: {}", e);
        PluginManager::default()
    }));
    
    // Create app state
    let state = Arc::new(AppState {
        db,
        config,
        mcp: None, // Legacy client, deprecated
        mcp_client,
        ws_manager,
        auth,
        command_service,
        plugin_manager,
    });

    // Setup CORS
    let _cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    // Build the router with all routes
    let app = Router::new()
        .route("/health", get(handlers::health::get_health))
        .route("/api/health", get(handlers::health::get_health))
        .nest("/api", api::router::api_router())
        .route("/api/jobs", post(handlers::jobs::create_job))
        .route("/api/jobs", get(handlers::jobs::list_jobs))
        .route("/api/jobs/:id/status", get(handlers::jobs::get_job_status))
        .route("/api/jobs/:id/result", get(handlers::jobs::get_job_result))
        .route("/api/jobs/:id/cancel", post(handlers::jobs::cancel_job))
        .nest("/api/commands-legacy", handlers::commands::command_routes())
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        .route("/ws", get(websocket::ws_handler))
        .layer(CorsLayer::permissive());
    
    // Add plugin routes
    let app = plugins::create_plugin_routes(app.clone(), state.clone());
    
    app.with_state(state)
}