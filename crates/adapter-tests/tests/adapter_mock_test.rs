//! Mock implementations for adapter testing
//!
//! This module provides additional tests focusing on mocking adapters and
//! testing them with different scenarios. It complements the main integration
//! tests by adding more specific test cases and demonstrating how to create
//! mock adapters for testing.

use adapter_tests::{
    TestCommand,
    MockAdapter,
    AdapterResult,
    Auth,
    create_registry_adapter,
    create_mcp_adapter,
    create_plugin_adapter,
};
use std::sync::Arc;

/// Test the command registry adapter with a basic test command
#[tokio::test]
async fn test_registry_adapter_with_test_command() -> AdapterResult<()> {
    // Create a command registry adapter
    let adapter = create_registry_adapter();
    
    // Register a test command
    let test_cmd = TestCommand::new("test", "Test command", "Test result");
    adapter.register_command(Arc::new(test_cmd))?;
    
    // Execute without arguments
    let result = adapter.execute("test", vec![]).await?;
    assert_eq!(result, "Test result");
    
    // Execute with arguments
    let result = adapter.execute("test", vec!["arg1".to_string(), "arg2".to_string()]).await?;
    assert_eq!(result, "Test result with args: [\"arg1\", \"arg2\"]");
    
    // Get help for command
    let help = adapter.get_help("test").await?;
    assert_eq!(help, "test: Test command");
    
    // List commands
    let commands = adapter.list_commands().await?;
    assert!(commands.contains(&"test".to_string()));
    
    Ok(())
}

/// Test the MCP adapter with authentication and authorization
#[tokio::test]
async fn test_mcp_adapter() -> AdapterResult<()> {
    // Create an MCP adapter
    let adapter = create_mcp_adapter();
    
    // Register two commands - one regular and one admin-only
    let regular_cmd = TestCommand::new("hello", "Regular command", "Hello there!");
    let admin_cmd = TestCommand::new("admin", "Admin command", "Admin access granted");
    
    adapter.register_command(Arc::new(regular_cmd))?;
    adapter.register_command(Arc::new(admin_cmd))?;
    
    // Test command execution without authentication
    let result = adapter.execute("hello", vec![]).await?;
    assert_eq!(result, "Hello there!");
    
    // Test nonexistent command
    let result = adapter.execute("nonexistent", vec![]).await;
    assert!(result.is_err());
    
    Ok(())
}

/// Test the MCP adapter's authentication mechanism
#[tokio::test]
async fn test_mcp_adapter_authentication() -> AdapterResult<()> {
    // Create an MCP adapter
    let adapter = create_mcp_adapter();
    
    // Register an admin-only command
    let admin_cmd = TestCommand::new("admin", "Admin command", "Admin access granted");
    adapter.register_command(Arc::new(admin_cmd))?;
    
    // Try with admin credentials (using the default password "password")
    let result = adapter.execute_with_auth(
        "admin",
        vec![],
        Auth::User("admin".to_string(), "password".to_string())
    ).await?;
    assert_eq!(result, "Admin access granted");
    
    // Try with invalid password (should fail)
    let result = adapter.execute_with_auth(
        "admin",
        vec![],
        Auth::User("admin".to_string(), "wrong_password".to_string())
    ).await;
    assert!(result.is_err());
    
    // Try with non-admin user (should fail for admin command)
    let result = adapter.execute_with_auth(
        "admin",
        vec![],
        Auth::User("user".to_string(), "password".to_string())
    ).await;
    assert!(result.is_err());
    
    // Try with anonymous access (should fail for admin command)
    let result = adapter.execute_with_auth("admin", vec![], Auth::None).await;
    assert!(result.is_err());
    
    // Try accessing a regular command with anonymous access (should succeed)
    let regular_cmd = TestCommand::new("hello", "Regular command", "Hello there!");
    adapter.register_command(Arc::new(regular_cmd))?;
    
    let result = adapter.execute_with_auth("hello", vec![], Auth::None).await?;
    assert_eq!(result, "Hello there!");
    
    Ok(())
}

/// Test the plugin adapter for command integration
#[tokio::test]
async fn test_plugin_adapter() -> AdapterResult<()> {
    // Create a plugin adapter
    let adapter = create_plugin_adapter();
    
    // Register commands through the plugin interface
    let cmd1 = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
    let cmd2 = TestCommand::new("config", "Configuration command", "Config result");
    
    adapter.register_command(Arc::new(cmd1))?;
    adapter.register_command(Arc::new(cmd2))?;
    
    // Execute commands
    let result = adapter.execute_command("plugin-cmd", vec![]).await?;
    assert_eq!(result, "Plugin result");
    
    // Get command help
    let help = adapter.get_command_help("config").await?;
    assert_eq!(help, "config: Configuration command");
    
    // List commands from the plugin
    let commands = adapter.get_commands().await?;
    assert_eq!(commands.len(), 2);
    assert!(commands.contains(&"plugin-cmd".to_string()));
    assert!(commands.contains(&"config".to_string()));
    
    Ok(())
}

/// Test implementing a custom adapter using the MockAdapter trait
#[tokio::test]
async fn test_custom_mock_adapter() -> AdapterResult<()> {
    // Create a simple mock adapter that implements the MockAdapter trait
    struct SimpleAdapter {
        commands: std::collections::HashMap<String, String>,
    }
    
    impl SimpleAdapter {
        fn new() -> Self {
            let mut commands = std::collections::HashMap::new();
            commands.insert("hello".to_string(), "Hello, world!".to_string());
            commands.insert("bye".to_string(), "Goodbye!".to_string());
            Self { commands }
        }
    }
    
    #[async_trait::async_trait]
    impl MockAdapter for SimpleAdapter {
        async fn execute(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
            if let Some(result) = self.commands.get(command) {
                if args.is_empty() {
                    Ok(result.clone())
                } else {
                    Ok(format!("{} (args: {:?})", result, args))
                }
            } else {
                Err(adapter_tests::AdapterError::NotFound(command.to_string()))
            }
        }
        
        async fn get_help(&self, command: &str) -> AdapterResult<String> {
            if self.commands.contains_key(command) {
                Ok(format!("{}: A simple command", command))
            } else {
                Err(adapter_tests::AdapterError::NotFound(command.to_string()))
            }
        }
    }
    
    // Create the adapter
    let adapter = SimpleAdapter::new();
    
    // Execute commands
    let result = adapter.execute("hello", vec![]).await?;
    assert_eq!(result, "Hello, world!");
    
    let result = adapter.execute("bye", vec!["user".to_string()]).await?;
    assert_eq!(result, "Goodbye! (args: [\"user\"])");
    
    // Try a nonexistent command
    let result = adapter.execute("unknown", vec![]).await;
    assert!(result.is_err());
    
    // Get help for a command
    let help = adapter.get_help("hello").await?;
    assert_eq!(help, "hello: A simple command");
    
    Ok(())
} 