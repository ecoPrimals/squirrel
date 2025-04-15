use squirrel_integration::context_mcp::{
    ContextMcpAdapter,
    types::ContextMcpAdapterConfig
};
use tokio::time::Duration;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

/// Example demonstrating Context-MCP integration
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .init();
    
    // Create adapter with default configuration
    println!("Creating Context-MCP adapter...");
    let adapter = ContextMcpAdapter::with_config(ContextMcpAdapterConfig::default()).await?;
    
    // Initialize the adapter
    println!("Initializing adapter...");
    adapter.initialize().await?;
    
    // Create a shared reference for the async tasks
    let adapter = Arc::new(adapter);
    
    // Clone for the status task
    let status_adapter = adapter.clone();
    
    // Start a task to periodically print adapter status
    tokio::spawn(async move {
        loop {
            let status = status_adapter.get_status().await;
            println!("Adapter status: {:#?}", status);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    
    println!("Context-MCP integration running!");
    println!("Press Ctrl+C to exit");
    
    // Keep the main task running
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        
        // Perform a manual sync
        println!("Performing manual sync...");
        if let Err(err) = adapter.sync_all().await {
            eprintln!("Error during manual sync: {}", err);
        } else {
            println!("Manual sync completed successfully");
        }
    }
} 