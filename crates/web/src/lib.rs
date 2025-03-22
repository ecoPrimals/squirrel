use std::sync::Arc;
use anyhow::Result;
use axum::{Router, http::Method, routing::get, Extension};
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
use state::{MachineContextClient, DefaultMockMCPClient};

pub use api::{CreateJobRequest, CreateJobResponse, JobStatus, JobState};

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
        
        Self {
            db: mock_db,
            config,
            mcp: None,
            ws_manager,
            auth,
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
    Ok(pool)
}

/// Create the application router
pub async fn create_app(db: DbPool, config: Config) -> Router {
    // Initialize WebSocket manager
    let ws_manager = websocket::init();
    
    // Create auth service
    let auth = AuthService::new(AuthConfig::default(), db.clone());
    
    // Create app state
    let state = Arc::new(AppState {
        db,
        config,
        mcp: None, // Will be initialized based on config if needed
        ws_manager,
        auth,
    });

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    // Create router with all routes
    (Router::new()
        // Auth routes
        .nest("/api/auth", auth::routes::auth_routes())
        // Health routes
        .nest("/api/health", handlers::health::health_routes())
        // Job routes
        .nest("/api/jobs", handlers::jobs::job_routes())
        // WebSocket route
        .route("/api/ws", get(websocket::ws_handler))
        // Add state and middleware
        .layer(Extension(state))
        .layer(cors)) as Router<()>
}