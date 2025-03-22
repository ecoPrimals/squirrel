use anyhow::Result;
use squirrel_web::{
    config::Config,
    create_app, ServerConfig, auth::AuthConfig,
    CorsConfig, MockSessionConfig,
};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create server configuration
    let server_config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 3000,
        database_url: "sqlite::memory:".to_string(),
        mcp_config: MockSessionConfig::default(),
        cors_config: CorsConfig {
            allowed_origins: vec![],  // We'll use AllowOrigin::any() in the lib.rs
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        },
        auth_config: AuthConfig::default(),
    };
    
    // Connect to the database
    let db = SqlitePool::connect(&server_config.database_url)
        .await
        .expect("Failed to connect to database");
    
    // Create a default config to pass to create_app
    let app_config = Config::default();
    
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