fn main() {
    println!("Setting up core integration example");
    
    // Create protocol adapter
    let protocol_adapter = squirrel_mcp::protocol::create_protocol_adapter();
    println!("Created protocol adapter");
    
    // Create a logger with a component name
    let logger = squirrel_mcp::logging::Logger::new("core-integration-example");
    println!("Created logger");
    
    println!("Core integration example setup complete");
} 