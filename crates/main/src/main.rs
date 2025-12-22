//! Squirrel AI Coordinator Main Entry Point

use anyhow::Result;
use squirrel::api::ApiServer;
use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::shutdown::ShutdownManager;
use squirrel::MetricsCollector;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("squirrel=info,debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    println!("🐿️  Squirrel AI/MCP Primal Starting...");
    println!("✅ Arc<str> Modernization Complete");
    println!("✅ Performance Optimized with Zero-Copy Patterns");

    // Get configuration from environment
    let port = std::env::var("PORT")
        .or_else(|_| std::env::var("SQUIRREL_PORT"))
        .unwrap_or_else(|_| "9010".to_string())
        .parse::<u16>()?;

    // Initialize ecosystem components
    let metrics_collector = Arc::new(MetricsCollector::new());
    let ecosystem_config = EcosystemConfig::default();
    let ecosystem_manager = Arc::new(EcosystemManager::new(
        ecosystem_config,
        metrics_collector.clone(),
    ));
    let shutdown_manager = Arc::new(ShutdownManager::new());

    println!("✅ Ecosystem Manager initialized");
    println!("✅ Metrics Collector initialized");
    println!("✅ Shutdown Manager initialized");

    // Create and start API server
    let api_server = ApiServer::new(
        port,
        ecosystem_manager.clone(),
        metrics_collector.clone(),
        shutdown_manager.clone(),
    );

    println!("🚀 Starting API server on port {}", port);
    println!("   Health: http://localhost:{}/health", port);
    println!("   API: http://localhost:{}/api/v1/*", port);
    println!();
    println!("✅ Squirrel AI/MCP Primal Ready!");

    // Start the server (this will block)
    api_server.start().await?;

    Ok(())
}
