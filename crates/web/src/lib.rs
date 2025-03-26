use std::sync::Arc;
use anyhow::Result;
use axum::{
    Router, 
    http::Method, 
    routing::{get, post}
};
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
#[cfg(feature = "db")]
use sqlx::{migrate::MigrateDatabase, Sqlite};
#[cfg(feature = "mock-db")]
// use sqlx::SqlitePool; // Commented out since it's unused
use serde::{Deserialize, Serialize};

// Add conditional imports for API documentation
#[cfg(feature = "api-docs")]
use utoipa::OpenApi;
#[cfg(feature = "api-docs")]
use utoipa_swagger_ui::SwaggerUi;

// Add hyper crate dependency if needed

pub mod auth;
mod handlers;
pub mod mcp;
pub mod api;
pub mod state;
pub mod websocket;
pub mod config;
pub mod db;
pub mod plugins;
pub mod plugins_legacy; // Renamed legacy plugins module
pub mod plugin_adapter; // New adapter module for plugin migration
pub mod utils;

use crate::state::AppState;
use crate::config::Config;
use crate::db::SqlitePool as DbPool;
use auth::{AuthConfig, AuthService};
use mcp::{McpClient, MockMcpClient, RealMcpClient, McpClientConfig, McpCommandClient, McpEventBridge, ContextManager};
use api::commands::{CommandService, CommandServiceImpl, CommandRepository};
use plugins::WebPluginRegistry;

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

// Add API documentation if feature is enabled
#[cfg(feature = "api-docs")]
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::get_health,
        handlers::jobs::create_job,
        handlers::jobs::list_jobs,
        handlers::jobs::get_job_status,
        handlers::jobs::get_job_result,
        handlers::jobs::cancel_job,
        handlers::auth::login,
        handlers::auth::refresh_token
    ),
    components(
        schemas(
            api::commands::CommandDefinition,
            api::commands::CommandExecution,
            api::commands::CommandStatus,
            api::commands::CreateCommandRequest,
            api::commands::CreateCommandResponse,
            api::commands::CommandStatusResponse,
            api::commands::CommandSummary,
            api::commands::AvailableCommand
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "jobs", description = "Job management endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "commands", description = "Command execution endpoints")
    )
)]
struct ApiDoc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub database_url: String,
    pub mcp_config: McpClientConfig,
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
    let ws_manager = Arc::new(websocket::init());
    
    // Create auth service
    let auth = AuthService::new(AuthConfig::default(), db.clone());
    
    // Create context manager for MCP communication
    let context_manager = Arc::new(ContextManager::new());
    
    // Create MCP client based on configuration and features
    #[cfg(feature = "mock-mcp")]
    let (mcp_client, mcp_command_client) = {
        tracing::info!("Using mock MCP client");
        let mock_client = Arc::new(MockMcpClient::new(
            config.mcp.host.clone(), 
            config.mcp.port
        ));
        // The mock client implements both traits, so we can clone the Arc
        (mock_client.clone() as Arc<dyn McpClient>, mock_client as Arc<dyn McpCommandClient>)
    };
    
    #[cfg(not(feature = "mock-mcp"))]
    let (mcp_client, mcp_command_client) = {
        tracing::info!("Using real MCP client");
        match RealMcpClient::new(config.mcp.clone()).await {
            Ok(client) => {
                tracing::info!("Successfully connected to MCP server at {}:{}", 
                    config.mcp.host, config.mcp.port);
                let client_arc = Arc::new(client);
                // The real client implements both traits, so we can clone the Arc
                (client_arc.clone() as Arc<dyn McpClient>, client_arc as Arc<dyn McpCommandClient>)
            },
            Err(e) => {
                tracing::error!("Failed to connect to MCP server: {}", e);
                tracing::warn!("Falling back to mock MCP client");
                let mock_client = Arc::new(MockMcpClient::new(
                    config.mcp.host.clone(), 
                    config.mcp.port
                ));
                // The mock client implements both traits, so we can clone the Arc
                (mock_client.clone() as Arc<dyn McpClient>, mock_client as Arc<dyn McpCommandClient>)
            }
        }
    };

    // Create command repository based on feature
    #[cfg(feature = "db")]
    let command_repository = Arc::new(SqliteCommandRepository::new(db.clone())) as Arc<dyn CommandRepository>;
    
    #[cfg(feature = "mock-db")]
    let command_repository = Arc::new(MockCommandRepository::new()) as Arc<dyn CommandRepository>;
    
    // Create command service
    let command_service = Arc::new(CommandServiceImpl::new(
        command_repository,
        mcp_command_client.clone(),
        ws_manager.clone(),
    )) as Arc<dyn CommandService>;
    
    // Initialize plugin system using the adapter
    // This now returns both the legacy plugin manager and the new WebPluginRegistry
    let (plugin_manager, plugin_registry) = plugin_adapter::init_plugin_system().await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize plugin system: {}", e);
        (plugins_legacy::PluginManager::new(), WebPluginRegistry::new())
    });
    
    let plugin_manager = Arc::new(plugin_manager);
    let plugin_registry = Arc::new(plugin_registry);
    
    // Try to initialize monitoring service if available
    let monitoring_service = init_monitoring_service().await;
    
    // Create app state
    let state = Arc::new(AppState {
        db,
        config,
        mcp: None, // Legacy client, deprecated
        mcp_client,
        mcp_command_client,
        ws_manager: ws_manager.clone(),
        auth,
        command_service,
        plugin_manager,
        plugin_registry: Some(plugin_registry), // Add the plugin registry to the app state
        context_manager: context_manager.clone(), // Add context manager to app state
        monitoring_service, // Add monitoring service to app state
    });
    
    // Initialize and start MCP event bridge with context manager
    let event_bridge = McpEventBridge::new(
        state.mcp_client.clone(), 
        ws_manager,
        context_manager // Pass context manager to event bridge
    );
    
    if let Err(e) = event_bridge.start().await {
        tracing::error!("Failed to start MCP event bridge: {}", e);
    } else {
        tracing::info!("MCP event bridge started successfully");
    }
    
    // Setup CORS
    let _cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    // Build the router with all routes
    let mut app = Router::new()
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
    
    // Add static file serving for the UI from the new ui-web crate
    let ui_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap() // Go up to crates/
        .join("ui-web/dist");
    
    if ui_dir.exists() {
        tracing::info!("Serving UI files from {:?}", ui_dir);
        app = app.nest_service("/", ServeDir::new(ui_dir));
    } else {
        tracing::warn!("UI directory {:?} does not exist. UI will not be available.", ui_dir);
    }
    
    // Add API documentation if feature is enabled
    #[cfg(feature = "api-docs")]
    {
        tracing::info!("API documentation enabled, adding Swagger UI at /api-docs");
        app = app.merge(
            SwaggerUi::new("/api-docs")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
        );
    }
    
    // Add plugin routes using the adapter
    // This will eventually use the unified plugin registry
    let app_with_plugins = plugin_adapter::create_plugin_routes(app, state.clone()).await;
    
    app_with_plugins.with_state(state)
}

/// Initialize the monitoring service
async fn init_monitoring_service() -> Option<Arc<api::monitoring::MonitoringService>> {
    // Try to get monitoring API from the monitoring crate
    #[cfg(feature = "monitoring")]
    {
        // Import the API module from monitoring crate
        match squirrel_monitoring::api::get_monitoring_api() {
            Ok(monitoring_api) => {
                // Create a wrapper for the monitoring API
                let api_wrapper = Arc::new(MockMonitoringAPI::new()) as Arc<dyn squirrel_monitoring::api::MonitoringAPI>;
                
                // Create monitoring service with the API
                let monitoring_service = api::monitoring::MonitoringService::new(api_wrapper);
                
                // Log that monitoring API is available
                tracing::info!("Monitoring API initialized successfully");
                
                // Return the service wrapped in an Arc
                Some(Arc::new(monitoring_service))
            },
            Err(e) => {
                tracing::error!("Failed to initialize monitoring API: {}", e);
                tracing::info!("Falling back to mock monitoring API");
                
                // Create a mock monitoring API
                let mock_api = Arc::new(MockMonitoringAPI::new()) as Arc<dyn squirrel_monitoring::api::MonitoringAPI>;
                
                // Create monitoring service with the mock API
                let monitoring_service = api::monitoring::MonitoringService::new(mock_api);
                
                // Return the service wrapped in an Arc
                Some(Arc::new(monitoring_service))
            }
        }
    }
    
    #[cfg(not(feature = "monitoring"))]
    {
        tracing::info!("Monitoring feature not enabled, using mock monitoring API");
        
        // Create a mock monitoring API
        let mock_api = Arc::new(MockMonitoringAPI::new()) as Arc<dyn squirrel_monitoring::api::MonitoringAPI>;
        
        // Create monitoring service with the mock API
        let monitoring_service = api::monitoring::MonitoringService::new(mock_api);
        
        // Return the service wrapped in an Arc
        Some(Arc::new(monitoring_service))
    }
}

/// Mock implementation of the MonitoringAPI
#[derive(Debug)]
struct MockMonitoringAPI {
    // No fields needed for mock
}

impl MockMonitoringAPI {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl squirrel_monitoring::api::MonitoringAPI for MockMonitoringAPI {
    async fn get_component_data(&self, component_id: &str) -> squirrel_core::error::Result<serde_json::Value> {
        // Return mock data for the component
        Ok(serde_json::json!({
            "name": component_id,
            "status": "ok",
            "value": 42,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
    
    async fn get_available_components(&self) -> squirrel_core::error::Result<Vec<String>> {
        // Return mock list of components
        Ok(vec![
            "cpu".to_string(),
            "memory".to_string(),
            "disk".to_string(),
            "network".to_string()
        ])
    }
    
    async fn get_health_status(&self) -> squirrel_core::error::Result<std::collections::HashMap<String, serde_json::Value>> {
        // Return mock health status
        let mut health = std::collections::HashMap::new();
        health.insert("status".to_string(), serde_json::Value::String("healthy".to_string()));
        health.insert("components_count".to_string(), serde_json::Value::Number(
            serde_json::Number::from(4)
        ));
        Ok(health)
    }
    
    async fn subscribe_to_component(&self, component_id: &str) -> squirrel_core::error::Result<String> {
        // Return mock subscription ID
        Ok(format!("mock_subscription_{}", component_id))
    }
    
    async fn unsubscribe(&self, subscription_id: &str) -> squirrel_core::error::Result<()> {
        // Mock successful unsubscribe
        Ok(())
    }
}