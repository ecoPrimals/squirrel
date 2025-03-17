//! Main entry point for the Squirrel application

use squirrel_core::{Core, MCP};
use squirrel_core::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize core
    let core = Core::new();
    
    // Initialize MCP
    let mcp = MCP::default();
    
    println!("Squirrel initialized successfully!");
    println!("Core version: {}", core.version());
    println!("MCP version: {}", mcp.get_config().await.unwrap().version);
    
    Ok(())
} 