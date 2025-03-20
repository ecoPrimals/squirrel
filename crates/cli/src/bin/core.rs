//! Entry point for the Squirrel Core binary.

use squirrel_core::Core;
use squirrel_core::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the core
    let _core = Core::new();
    
    println!("Core initialized successfully");
    
    // Note: MCP initialization has been moved to the separate squirrel-mcp crate
    
    Ok(())
} 