//! Basic example for the Adapter Pattern implementation
//!
//! This example demonstrates the fundamental usage of the Adapter Pattern
//! in a simplified context. It shows how to create adapters, register commands,
//! and execute them both with and without authentication.
//!
//! Key concepts demonstrated:
//! - Creating registry adapters
//! - Registering commands
//! - Executing commands with arguments
//! - Getting help information
//! - Authentication with the MCP adapter
//! - Authorization checks

use adapter_tests::{
    TestCommand,
    CommandRegistryAdapter,
    McpCommandAdapter,
    Auth,
    AdapterResult,
};
use std::sync::Arc;

/// This example demonstrates basic usage of the adapter pattern with commands
#[tokio::main]
async fn main() -> AdapterResult<()> {
    println!("Basic Adapter Pattern Example");
    println!("=============================\n");

    // Step 1: Create a command registry adapter
    println!("Step 1: Creating command registry adapter");
    let adapter = CommandRegistryAdapter::new();
    
    // Step 2: Create and register commands
    println!("Step 2: Creating and registering commands");
    let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo:");
    
    adapter.register_command(Arc::new(hello_cmd))?;
    adapter.register_command(Arc::new(echo_cmd))?;
    
    // Step 3: List available commands
    println!("Step 3: Listing available commands");
    let commands = adapter.list_commands().await?;
    println!("  Available commands: {:?}", commands);
    
    // Step 4: Execute commands
    println!("\nStep 4: Executing commands");
    
    // Execute a command without arguments
    let result = adapter.execute("hello", vec![]).await?;
    println!("  Hello command result: {}", result);
    
    // Execute a command with arguments
    let result = adapter.execute("echo", vec!["Hello".to_string(), "there!".to_string()]).await?;
    println!("  Echo command result: {}", result);
    
    // Step 5: Show how to get help for a command
    println!("\nStep 5: Getting help for commands");
    let help = adapter.get_help("hello").await?;
    println!("  Help for hello command: {}", help);
    
    // Step 6: Demonstrate the MCP adapter with authentication
    println!("\nStep 6: Using MCP adapter with authentication");
    let mcp_adapter = McpCommandAdapter::new();
    
    let secure_cmd = TestCommand::new("secure", "A secure command", "Secure data");
    mcp_adapter.register_command(Arc::new(secure_cmd))?;
    
    // Execute with admin authentication
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = mcp_adapter.execute_with_auth("secure", vec![], admin_auth).await?;
    println!("  Secure command with admin auth: {}", result);
    
    // Try anonymous access
    let result = mcp_adapter.execute_with_auth("secure", vec![], Auth::None).await?;
    println!("  Secure command with anonymous auth: {}", result);
    
    // Create an admin command
    let admin_cmd = TestCommand::new("admin-cmd", "Admin only command", "Admin data");
    mcp_adapter.register_command(Arc::new(admin_cmd))?;
    
    // Execute admin command with admin auth
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = mcp_adapter.execute_with_auth("admin-cmd", vec![], admin_auth).await?;
    println!("  Admin command with admin auth: {}", result);
    
    // Try to execute admin command without authentication (should fail)
    println!("\nStep 7: Testing authorization failure");
    match mcp_adapter.execute_with_auth("admin-cmd", vec![], Auth::None).await {
        Ok(result) => println!("  This should not happen: {}", result),
        Err(e) => println!("  Expected error (anonymous cannot access admin command): {}", e),
    }
    
    println!("\nExample completed successfully!");
    
    Ok(())
} 