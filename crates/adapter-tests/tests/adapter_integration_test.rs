//! Integration tests for the Adapter Pattern implementation
//!
//! This test suite validates the functionality of the three main adapter types:
//! - CommandRegistryAdapter: Basic command registry operations
//! - McpCommandAdapter: Command execution with authentication
//! - CommandsPluginAdapter: Plugin system integration
//!
//! It also tests the MockAdapter trait to ensure all adapters implement it correctly.
//! These tests verify that adapters maintain the expected behavior as defined in the
//! architectural contract, ensuring that the Adapter Pattern is properly implemented.

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

#[tokio::test]
async fn test_command_registry_adapter() -> AdapterResult<()> {
    // Create a registry adapter
    let adapter = CommandRegistryAdapter::new();
    
    // Create and register test commands
    let cmd = TestCommand::new("test", "Test command", "Test result");
    adapter.register_command(Arc::new(cmd))?;
    
    // Test execution
    let result = adapter.execute("test", vec![]).await?;
    assert_eq!(result, "Test result");
    
    // Test with arguments
    let result = adapter.execute("test", vec!["arg1".to_string(), "arg2".to_string()]).await?;
    assert_eq!(result, "Test result with args: [\"arg1\", \"arg2\"]");
    
    // Test help
    let help = adapter.get_help("test").await?;
    assert_eq!(help, "test: Test command");
    
    Ok(())
}

#[tokio::test]
async fn test_mcp_adapter_authentication() -> AdapterResult<()> {
    // Create an MCP adapter
    let adapter = McpCommandAdapter::new();
    
    // Register a secure command
    let cmd = TestCommand::new("secure", "Secure command", "Secret data");
    adapter.register_command(Arc::new(cmd))?;
    
    // Test with admin authentication
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = adapter.execute_with_auth("secure", vec![], admin_auth).await?;
    assert_eq!(result, "Secret data");
    
    // Try with anonymous auth (should fail for admin commands, but work for regular commands)
    let result = adapter.execute_with_auth("secure", vec![], Auth::None).await?;
    assert_eq!(result, "Secret data"); // Regular command should work with any auth
    
    // Test with an admin command (should fail without admin auth)
    let admin_cmd = TestCommand::new("admin-test", "Admin command", "Admin data");
    adapter.register_command(Arc::new(admin_cmd))?;
    
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = adapter.execute_with_auth("admin-test", vec![], admin_auth).await?;
    assert_eq!(result, "Admin data");
    
    // Anonymous access to admin command should fail
    let result = adapter.execute_with_auth("admin-test", vec![], Auth::None).await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_plugin_adapter() -> AdapterResult<()> {
    // Create a plugin adapter
    let adapter = CommandsPluginAdapter::new();
    
    // Register a plugin command
    let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
    adapter.register_command(Arc::new(cmd))?;
    
    // Test execution
    let result = adapter.execute_command("plugin-cmd", vec![]).await?;
    assert_eq!(result, "Plugin result");
    
    // Test with arguments
    let result = adapter.execute_command(
        "plugin-cmd", 
        vec!["arg1".to_string(), "arg2".to_string()]
    ).await?;
    assert_eq!(result, "Plugin result with args: [\"arg1\", \"arg2\"]");
    
    // Check plugin metadata
    assert_eq!(adapter.plugin_id(), "commands");
    assert_eq!(adapter.version(), "1.0.0");
    
    // Test help
    let help = adapter.get_command_help("plugin-cmd").await?;
    assert_eq!(help, "plugin-cmd: Plugin command");
    
    Ok(())
}

#[tokio::test]
async fn test_adapter_trait() -> AdapterResult<()> {
    // Create implementations of the MockAdapter trait
    let registry_adapter = CommandRegistryAdapter::new();
    let mcp_adapter = McpCommandAdapter::new();
    let plugin_adapter = CommandsPluginAdapter::new();
    
    // Register the same test command with all adapters
    let cmd = TestCommand::new("common", "Common command", "Common result");
    registry_adapter.register_command(Arc::new(cmd.clone()))?;
    mcp_adapter.register_command(Arc::new(cmd.clone()))?;
    plugin_adapter.register_command(Arc::new(cmd.clone()))?;
    
    // Test the adapters through the trait interface
    async fn test_through_trait(adapter: &impl MockAdapter) -> AdapterResult<()> {
        let result = adapter.execute("common", vec![]).await?;
        assert_eq!(result, "Common result");
        
        let help = adapter.get_help("common").await?;
        assert_eq!(help, "common: Common command");
        
        Ok(())
    }
    
    test_through_trait(&registry_adapter).await?;
    test_through_trait(&mcp_adapter).await?;
    test_through_trait(&plugin_adapter).await?;
    
    Ok(())
} 