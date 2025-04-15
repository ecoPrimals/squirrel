use std::sync::Arc;
// use std::path::Path;
// use std::sync::RwLock;
// use std::collections::HashMap;
// use std::fmt;
// Remove the duplicate Mutex import
// use tokio::sync::Mutex;
use axum::{
    Router,
    routing::get,
    routing::post,
    // routing::IntoMakeService,
    // Server,
};
use anyhow::Result;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use http::Method;
#[cfg(feature = "db")]
use sqlx::{migrate::MigrateDatabase, Sqlite};
// use tokio::sync::Mutex;

// Add conditional imports for API documentation
#[cfg(feature = "api-docs")]
use utoipa::OpenApi;
#[cfg(feature = "api-docs")]
use utoipa_swagger_ui::SwaggerUi;

// Add hyper crate dependency if needed

pub mod auth;
mod handlers;
pub mod mcp; // MCP module
pub mod api;
pub mod state;
pub mod websocket;
pub mod config;
pub mod db;
pub mod plugins;
pub mod plugins_legacy; // Renamed legacy plugins module
pub mod plugin_adapter; // New adapter module for plugin migration
pub mod utils;
pub mod mcp_grpc_client; // Add the new module declaration

use crate::state::AppState;
use crate::config::Config;
use crate::db::SqlitePool as DbPool;
use auth::{AuthConfig, AuthService};
// Import from mcp.rs instead of mcp/ module
use crate::mcp::{
    McpClient, McpCommandClient, McpClientConfig, 
    ContextManager
};
use crate::mcp::{
    MockMcpClient, McpEventBridge
};
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

use crate::mcp_grpc_client::McpGrpcClient;
use crate::mcp::RealMcpClient;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub database_url: String,
    pub mcp_config: McpClientConfig,
    pub cors_config: CorsConfig,
    pub auth_config: AuthConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        tracing::info!("Using real MCP client with gRPC support");
        
        // Try connecting using the gRPC client first
        match McpGrpcClient::new(config.mcp.clone()).await {
            Ok(client) => {
                tracing::info!("Successfully connected to MCP server via gRPC at {}:{}", 
                    config.mcp.host, config.mcp.port);
                let client_arc: Arc<McpGrpcClient> = Arc::new(client);
                // The gRPC client implements both traits, so we can clone the Arc
                (client_arc.clone() as Arc<dyn McpClient>, client_arc as Arc<dyn McpCommandClient>)
            },
            Err(grpc_err) => {
                tracing::warn!("Failed to connect to MCP server via gRPC: {}", grpc_err);
                tracing::info!("Falling back to standard MCP client");
                
                // Try with the regular client as fallback
                match RealMcpClient::new(config.mcp.clone()).await {
                    Ok(client) => {
                        tracing::info!("Successfully connected to MCP server via standard client at {}:{}", 
                            config.mcp.host, config.mcp.port);
                        // Explicit type annotation needed here
                        let client_arc: Arc<RealMcpClient> = Arc::new(client);
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
            }
        }
    };

    // Create command repository based on feature
    #[cfg(feature = "db")]
    let command_repository = Arc::new(SqliteCommandRepository::new(db.clone())) as Arc<dyn CommandRepository>;
    
    #[cfg(all(feature = "mock-db", not(feature = "db")))]
    let command_repository = Arc::new(MockCommandRepository::new()) as Arc<dyn CommandRepository>;
    
    // Default implementation if neither feature is enabled
    #[cfg(not(any(feature = "db", feature = "mock-db")))]
    let command_repository = {
        // Create a simple in-memory implementation
        use std::collections::HashMap;
        use std::sync::RwLock;
        use uuid::Uuid;
        use chrono::Utc;

        #[derive(Debug)]
        struct DefaultCommandRepository {
            command_definitions: RwLock<HashMap<String, CommandDefinition>>,
            command_executions: RwLock<HashMap<String, CommandExecution>>,
        }

        impl DefaultCommandRepository {
            fn new() -> Self {
                Self {
                    command_definitions: RwLock::new(HashMap::new()),
                    command_executions: RwLock::new(HashMap::new()),
                }
            }
        }

        #[async_trait::async_trait]
        impl CommandRepository for DefaultCommandRepository {
            async fn create_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
                let mut definitions = self.command_definitions.write().unwrap();
                definitions.insert(command.name.clone(), command);
                Ok(())
            }
            
            async fn get_command_definition(&self, name: &str) -> anyhow::Result<Option<CommandDefinition>> {
                let definitions = self.command_definitions.read().unwrap();
                Ok(definitions.get(name).cloned())
            }
            
            async fn list_command_definitions(&self) -> anyhow::Result<Vec<CommandDefinition>> {
                let definitions = self.command_definitions.read().unwrap();
                Ok(definitions.values().cloned().collect())
            }
            
            async fn upsert_command_definition(&self, command: CommandDefinition) -> anyhow::Result<()> {
                let mut definitions = self.command_definitions.write().unwrap();
                definitions.insert(command.name.clone(), command);
                Ok(())
            }
            
            async fn create_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
                let mut executions = self.command_executions.write().unwrap();
                executions.insert(execution.id.clone(), execution);
                Ok(())
            }
            
            async fn get_command_execution(&self, id: &str) -> anyhow::Result<Option<CommandExecution>> {
                let executions = self.command_executions.read().unwrap();
                Ok(executions.get(id).cloned())
            }
            
            async fn update_command_execution(&self, execution: CommandExecution) -> anyhow::Result<()> {
                let mut executions = self.command_executions.write().unwrap();
                executions.insert(execution.id.clone(), execution);
                Ok(())
            }
            
            async fn list_command_executions(&self, user_id: &str, limit: i64, offset: i64) -> anyhow::Result<Vec<CommandSummary>> {
                let executions = self.command_executions.read().unwrap();
                
                let mut filtered: Vec<_> = executions.values()
                    .filter(|e| e.user_id == user_id)
                    .cloned()
                    .collect();
                
                // Sort by created_at descending
                filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                
                // Apply offset and limit
                let start = offset as usize;
                let end = (offset + limit) as usize;
                
                let paginated = filtered.into_iter()
                    .skip(start)
                    .take(end - start)
                    .map(|e| CommandSummary {
                        id: e.id,
                        command: e.command_name,
                        status: e.status,
                        progress: e.progress,
                        created_at: e.created_at,
                        started_at: e.started_at,
                        completed_at: e.completed_at,
                    })
                    .collect();
                
                Ok(paginated)
            }
            
            async fn count_command_executions(&self, user_id: &str) -> anyhow::Result<i64> {
                let executions = self.command_executions.read().unwrap();
                let count = executions.values()
                    .filter(|e| e.user_id == user_id)
                    .count();
                
                Ok(count as i64)
            }
        }
        
        Arc::new(DefaultCommandRepository::new()) as Arc<dyn CommandRepository>
    };
    
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
    
    // Create monitoring service - set to None for now
    let monitoring_service = None;
    
    // Initialize the MCP event bridge
    let _mcp_events = init_mcp_event_bridge(
        mcp_client.clone(), 
        ws_manager.clone(),
        context_manager.clone(),
    ).await;
    
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
    
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    // Build the router with all routes
    let mut app = Router::new()
        .route("/health", get(handlers::health::get_health))
        .route("/api/health", get(handlers::health::get_health))
        .nest("/api", api::router::api_router())
        .nest("/api/commands-legacy", handlers::commands::command_routes())
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        .route("/ws", get(websocket::ws_handler))
        .layer(cors);
        
    // Add job routes conditionally based on features
    #[cfg(any(feature = "db", feature = "mock-db"))]
    {
        app = app.route("/api/jobs", post(handlers::jobs::create_job))
            .route("/api/jobs/:id/status", get(handlers::jobs::get_job_status))
            .route("/api/jobs/:id/result", get(handlers::jobs::get_job_result))
            .route("/api/jobs/:id/cancel", post(handlers::jobs::cancel_job));
            
        #[cfg(feature = "db")]
        {
            app = app.route("/api/jobs", get(handlers::jobs::list_jobs));
        }
        
        #[cfg(all(feature = "mock-db", not(feature = "db")))]
        {
            app = app.route("/api/jobs", get(handlers::jobs::list_jobs));
        }
    }
    
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

// Comment out the conditional compilation for the monitoring feature
#[cfg(feature = "monitoring")]
async fn init_monitoring_service() -> Option<Arc<api::monitoring::MonitoringService>> {
    // Temporarily disable due to async trait issues with dyn trait
    /*
    // Create mock monitoring API
    let mock_api = Arc::new(MockMonitoringAPI::new()) as Arc<dyn squirrel_monitoring::api::MonitoringAPI>;
    
    // Create monitoring service with mock API
    let monitoring_service = api::monitoring::MonitoringService::new(mock_api);
    
    Some(Arc::new(monitoring_service))
    */
    None
}

// Comment out the conditional compilation for the monitoring feature
#[cfg(not(feature = "monitoring"))]
async fn init_monitoring_service() -> Option<Arc<api::monitoring::MonitoringService>> {
    None
}

/// Mock implementation of the MonitoringAPI
/*
#[derive(Debug)]
pub struct MockMonitoringAPI {}

impl MockMonitoringAPI {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl squirrel_monitoring::api::MonitoringAPI for MockMonitoringAPI {
    async fn get_component_data(&self, component_id: &str) -> Result<serde_json::Value, squirrel_monitoring::api::Error> {
        // For now, return a simple mock response
        Ok(serde_json::json!({
            "id": component_id,
            "name": format!("Mock Component {}", component_id),
            "status": "running",
            "metrics": {
                "cpu": 0.5,
                "memory": 256.0
            }
        }))
    }
    
    async fn get_available_components(&self) -> Result<Vec<String>, squirrel_monitoring::api::Error> {
        // Return a list of mock components
        Ok(vec!["mock-1".to_string(), "mock-2".to_string(), "mock-3".to_string()])
    }
    
    async fn get_health_status(&self) -> Result<std::collections::HashMap<String, serde_json::Value>, squirrel_monitoring::api::Error> {
        // Return a mock health status
        let mut status = std::collections::HashMap::new();
        status.insert("status".to_string(), serde_json::json!("healthy"));
        status.insert("uptime".to_string(), serde_json::json!(3600));
        status.insert("version".to_string(), serde_json::json!("1.0.0"));
        Ok(status)
    }
    
    async fn subscribe_to_component(&self, component_id: &str) -> Result<String, squirrel_monitoring::api::Error> {
        // Return a mock subscription ID
        Ok(format!("mock-subscription-{}", component_id))
    }
    
    async fn unsubscribe(&self, _subscription_id: &str) -> Result<(), squirrel_monitoring::api::Error> {
        // Pretend to unsubscribe successfully
        Ok(())
    }
}
*/

/// Initialize the MCP Event Bridge
async fn init_mcp_event_bridge(
    mcp_client: Arc<dyn McpClient>, 
    ws_manager: Arc<websocket::ConnectionManager>,
    context_manager: Arc<ContextManager>,
) -> Option<Arc<McpEventBridge>> {
    // Check if client is connected before initializing event bridge
    let status = mcp_client.get_status().await.unwrap_or(crate::mcp::ConnectionStatus::Disconnected);
    
    if status != crate::mcp::ConnectionStatus::Connected {
        tracing::warn!("MCP client is not connected. Event bridge will not be started.");
        return None;
    }
    
    // Create event bridge
    let event_bridge = McpEventBridge::new(
        mcp_client,
        ws_manager,
        context_manager,
    );
    
    // Start event bridge
    match event_bridge.start().await {
        Ok(_) => {
            tracing::info!("MCP event bridge started successfully");
            Some(Arc::new(event_bridge))
        },
        Err(e) => {
            tracing::error!("Failed to start MCP event bridge: {}", e);
            None
        }
    }
}