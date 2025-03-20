//! Test program for dependency injection.

use squirrel_mcp::{MCPConfig, MCPAdapter};

fn main() {
    // Create a simple MCP adapter with our new structure
    let config = MCPConfig::default();
    let adapter = MCPAdapter::new(config);
    
    println!("Di-tests mode active");
    println!("MCP adapter initialized");
} 