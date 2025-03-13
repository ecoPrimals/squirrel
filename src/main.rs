use anyhow::Result;
use squirrel::{initialize, shutdown};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Squirrel...");
    
    // Initialize all systems
    initialize().await?;
    
    // Initialize subsystems
    if let Err(e) = squirrel::security::initialize().await {
        error!("Failed to initialize security system: {}", e);
        return Err(e.into());
    }
    
    if let Err(e) = squirrel::monitoring::initialize().await {
        error!("Failed to initialize monitoring system: {}", e);
        return Err(e.into());
    }
    
    if let Err(e) = squirrel::data::initialize().await {
        error!("Failed to initialize data system: {}", e);
        return Err(e.into());
    }
    
    if let Err(e) = squirrel::deployment::initialize().await {
        error!("Failed to initialize deployment system: {}", e);
        return Err(e.into());
    }
    
    info!("All systems initialized successfully");
    
    // TODO: Add your application logic here
    
    // Shutdown all systems
    shutdown().await?;
    
    info!("Shutdown complete");
    Ok(())
} 