//! Command Adapter Pattern Showcase
//!
//! This example demonstrates three different adapter implementations for command handling:
//! 1. **Command Registry Adapter** - Basic adapter for registry operations
//! 2. **MCP Adapter** - Adapter with authentication/authorization
//! 3. **Plugin Adapter** - Adapter for plugin integration
//!
//! Each adapter transforms the underlying command registry interface in a different way
//! to accommodate specific use cases.

use adapter_tests::{
    TestCommand,
    CommandRegistryAdapter,
    McpCommandAdapter,
    CommandsPluginAdapter,
    MockAdapter,
    Auth,
    AdapterResult,
};
use std::sync::Arc;

/// Main function demonstrating various adapter implementations
#[tokio::main]
async fn main() -> AdapterResult<()> {
    // Demonstrate the Command Registry Adapter
    demonstrate_registry_adapter().await?;
    
    // Demonstrate the MCP Adapter with authentication
    demonstrate_mcp_adapter().await?;
    
    // Demonstrate the Plugin Adapter for plugin integration
    demonstrate_plugin_adapter().await?;
    
    // Demonstrate the MockAdapter trait
    demonstrate_mock_adapter_trait().await?;
    
    println!("\nAdapted showcase completed successfully!");
    
    Ok(())
}

/// Demonstrates the basic Command Registry Adapter pattern
///
/// This adapter provides thread-safe access to the command registry with
/// async operation support.
async fn demonstrate_registry_adapter() -> AdapterResult<()> {
    println!("\n==================================================");
    println!("=== Command Registry Adapter Example ===");
    println!("==================================================\n");
    
    // Create and initialize the adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Register commands
    let hello_cmd = TestCommand::new("hello", "Says hello to the user", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echoes the given arguments", "Echo");
    
    adapter.register_command(Arc::new(hello_cmd))?;
    adapter.register_command(Arc::new(echo_cmd))?;
    
    // List available commands
    let commands = adapter.list_commands().await?;
    println!("Available commands: {:?}", commands);
    
    // Execute commands
    let result = adapter.execute("hello", vec![]).await?;
    println!("Hello command result: {}", result);
    
    let result = adapter.execute("echo", vec!["Hello".to_string(), "there!".to_string()]).await?;
    println!("Echo command result: {}", result);
    
    // Show help information
    let help = adapter.get_help("hello").await?;
    println!("Help for hello command: {}", help);
    
    Ok(())
}

/// Demonstrates the MCP Adapter pattern with authentication
///
/// This adapter extends the basic registry adapter with authentication
/// and authorization capabilities.
async fn demonstrate_mcp_adapter() -> AdapterResult<()> {
    println!("\n==================================================");
    println!("=== MCP Adapter Example ===");
    println!("==================================================\n");
    
    // Create and initialize the adapter
    let mut adapter = McpCommandAdapter::new();
    
    // Create some test commands with different permissions
    let regular_cmd = TestCommand::new("regular", "Regular user command", "Regular command result");
    let admin_cmd = TestCommand::new("admin-cmd", "Admin only command", "Admin command result");
    
    // Register commands
    adapter.register_command(Arc::new(regular_cmd))?;
    adapter.register_command(Arc::new(admin_cmd))?;
    
    // Add users with different roles
    adapter.add_user("admin", "password", true);
    adapter.add_user("user", "password", false);
    
    // Demonstrate command visibility for different users
    println!("Command visibility demonstration:");
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let user_auth = Auth::User("user".to_string(), "password".to_string());
    let anon_auth = Auth::None;
    
    let admin_cmds = adapter.get_available_commands(admin_auth.clone()).await?;
    let user_cmds = adapter.get_available_commands(user_auth.clone()).await?;
    let anon_cmds = adapter.get_available_commands(anon_auth.clone()).await?;
    
    println!("Commands available to admin: {:?}", admin_cmds);
    println!("Commands available to regular user: {:?}", user_cmds);
    println!("Commands available to public: {:?}", anon_cmds);
    
    // Demonstrate command execution with different auth levels
    println!("\nCommand execution with authentication:");
    
    // Admin user can execute regular commands
    let result = adapter.execute_with_auth("regular", vec![], admin_auth.clone()).await?;
    println!("Regular command with admin auth: {}", result);
    
    // Admin user can execute admin commands
    let result = adapter.execute_with_auth("admin-cmd", vec![], admin_auth.clone()).await?;
    println!("Admin command with admin auth: {}", result);
    
    // Regular user cannot execute admin commands
    let result = adapter.execute_with_auth("admin-cmd", vec![], user_auth.clone()).await;
    match result {
        Ok(_) => println!("This should not happen!"),
        Err(e) => println!("Admin command with user auth failed (expected): {}", e),
    }
    
    // Anonymous users cannot execute admin commands
    let result = adapter.execute_with_auth("admin-cmd", vec![], anon_auth.clone()).await;
    match result {
        Ok(_) => println!("This should not happen!"),
        Err(e) => println!("Admin command without auth failed (expected): {}", e),
    }
    
    Ok(())
}

/// Demonstrates the Plugin Adapter pattern
///
/// This adapter extends the basic registry adapter for plugin integration,
/// allowing commands to be managed as part of a plugin system.
async fn demonstrate_plugin_adapter() -> AdapterResult<()> {
    println!("==================================================");
    println!("=== Plugin Adapter Example ===");
    println!("==================================================\n");
    
    // Create and initialize the adapter
    let adapter = CommandsPluginAdapter::new();
    
    // Register some plugin commands
    let plugin_cmd = TestCommand::new("plugin-cmd", "A plugin command", "Plugin command result");
    let advanced_cmd = TestCommand::new("advanced", "An advanced plugin command", "Advanced command result");
    
    adapter.register_command(Arc::new(plugin_cmd))?;
    adapter.register_command(Arc::new(advanced_cmd))?;
    
    // Show plugin metadata
    println!("Plugin ID: {}", adapter.plugin_id());
    println!("Plugin Version: {}", adapter.version());
    
    // List plugin commands
    let commands = adapter.get_commands().await?;
    println!("Plugin commands: {:?}", commands);
    
    // Execute a plugin command
    let result = adapter.execute("plugin-cmd", vec!["plugin".to_string(), "arg".to_string()]).await?;
    println!("Plugin command result: {}", result);
    
    // Get help for a plugin command
    let help = adapter.get_help("advanced").await?;
    println!("Help for advanced command: {}", help);
    
    Ok(())
}

/// Demonstrates the use of the MockAdapter trait
///
/// This shows how all adapters implement a common interface,
/// allowing them to be used interchangeably.
async fn demonstrate_mock_adapter_trait() -> AdapterResult<()> {
    println!("\n==================================================");
    println!("=== Using the MockAdapter Trait ===");
    println!("==================================================\n");
    
    // Create test command
    let test_cmd = TestCommand::new("test", "A test command", "Test command result");
    
    // Test with registry adapter
    let registry_adapter = CommandRegistryAdapter::new();
    registry_adapter.register_command(Arc::new(test_cmd.clone()))?;
    test_adapter(&registry_adapter, "registry").await?;
    
    // Test with MCP adapter
    let mcp_adapter = McpCommandAdapter::new();
    mcp_adapter.register_command(Arc::new(test_cmd))?;
    test_adapter(&mcp_adapter, "mcp").await?;
    
    Ok(())
}

/// Helper function to test any adapter implementing the MockAdapter trait
async fn test_adapter(adapter: &dyn MockAdapter, name: &str) -> AdapterResult<()> {
    println!("Testing {} adapter:", name);
    
    let result = adapter.execute("test", vec![]).await?;
    println!("  Execution result: {}", result);
    
    let help = adapter.get_help("test").await?;
    println!("  Help result: {}", help);
    
    Ok(())
} 