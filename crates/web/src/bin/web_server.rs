use anyhow::Result;
use squirrel_web::{
    config::Config,
    create_app, ServerConfig, auth::AuthConfig,
    CorsConfig, mcp::McpClientConfig,
};

#[cfg(any(feature = "db", feature = "mock-db"))]
use squirrel_web::setup_database;

#[cfg(not(any(feature = "db", feature = "mock-db")))]
use squirrel_web::db::SqlitePool;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create server configuration
    let server_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 3000,
        database_url: "sqlite::memory:".to_string(),
        mcp_config: McpClientConfig::default(),
        cors_config: CorsConfig {
            allowed_origins: vec![],  // We'll use AllowOrigin::any() in the lib.rs
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        },
        auth_config: AuthConfig::default(),
    };
    
    // Create a default config to pass to create_app
    let app_config = Config::default();
    
    // Connect to the database and run migrations conditionally
    #[cfg(any(feature = "db", feature = "mock-db"))]
    let db = setup_database(&server_config.database_url)
        .await
        .expect("Failed to setup database");
    
    // For server-only feature, create a simple in-memory pool without migrations
    #[cfg(not(any(feature = "db", feature = "mock-db")))]
    let db = squirrel_web::db::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to setup in-memory database");
    
    // Pass the config parameter to create_app
    let app = create_app(db, app_config).await;
    
    // Start the server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], server_config.port));
    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
} 