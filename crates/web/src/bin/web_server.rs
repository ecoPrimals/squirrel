use anyhow::Result;
use squirrel_web::{init_app, ServerConfig, CorsConfig, MockSessionConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Create server configuration
    let config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 3000,
        database_url: "sqlite::memory:".to_string(),
        mcp_config: MockSessionConfig::default(),
        cors_config: CorsConfig {
            allowed_origins: vec![],  // We'll use AllowOrigin::any() in the lib.rs
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
        },
    };
    
    // Build the application
    let app = init_app(config.clone()).await?;
    
    // Start the server
    let addr = format!("{}:{}", config.bind_address, config.port);
    tracing::info!("Starting server on {}", addr);
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
} 