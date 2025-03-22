//! Test program for the MCPAdapter

use squirrel_mcp::{adapter::MCPInterface, MCPAdapter, MCPConfig};

fn main() {
    println!("Testing MCPAdapter...");

    // Create a config
    let config = MCPConfig::default();

    // Create an adapter
    let adapter = MCPAdapter::new(config);

    // Check initial state
    println!("Adapter initialized: {}", adapter.is_initialized());

    // Initialize the adapter
    match adapter.initialize() {
        Ok(_) => println!("Adapter initialized successfully"),
        Err(e) => println!("Failed to initialize adapter: {:?}", e),
    }

    // Check state after initialization
    println!("Adapter initialized: {}", adapter.is_initialized());

    // Get the config
    match adapter.get_config() {
        Ok(config) => println!("Got config: {:?}", config),
        Err(e) => println!("Failed to get config: {:?}", e),
    }

    // Send a message
    match adapter.send_message("Hello, MCP!") {
        Ok(response) => println!("Got response: {}", response),
        Err(e) => println!("Failed to send message: {:?}", e),
    }

    println!("MCPAdapter test completed successfully!");
}
