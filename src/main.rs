use anyhow::Result;
use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
use squirrel::monitoring::MetricsCollector;
use squirrel::shutdown::ShutdownManager;
use squirrel::api::ApiServer;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter("squirrel=info,debug")
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    info!("Starting Squirrel Universal AI Primal");

    // Create configuration
    let config = EcosystemConfig::default();
    
    // Initialize metrics collector
    let metrics_collector = Arc::new(MetricsCollector::new());
    
    // Initialize shutdown manager
    let shutdown_manager = Arc::new(ShutdownManager::new());
    
    // Create ecosystem manager
    let ecosystem_manager = EcosystemManager::new(
        config,
        metrics_collector.clone(),
        shutdown_manager.clone(),
    ).await?;

    // Start ecosystem services
    info!("Initializing ecosystem services...");
    ecosystem_manager.start().await?;
    
    // Start API server
    let api_server = ApiServer::new(
        8080,
        Arc::new(ecosystem_manager),
        metrics_collector.clone(),
        shutdown_manager.clone(),
    );
    
    info!("Starting API server on port 8080...");
    api_server.start().await?;
    
    info!("Squirrel primal ecosystem started successfully");
    info!("Service endpoints available at configured ports");
    info!("API server available at http://localhost:8080");
    info!("Press Ctrl+C to shutdown gracefully");

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received shutdown signal, initiating graceful shutdown...");
        }
        _ = shutdown_manager.wait_for_shutdown() => {
            info!("Shutdown signal received from shutdown manager");
        }
    }

    // Perform graceful shutdown
    info!("Shutting down ecosystem services...");
    if let Err(e) = ecosystem_manager.shutdown().await {
        error!("Error during shutdown: {}", e);
    }

    info!("Squirrel primal ecosystem shutdown complete");
    Ok(())
} 