// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Adapter Pattern Demonstration
//!
//! This example demonstrates the adapter pattern by showing three different adapters:
//! 1. Registry Adapter - Provides async access to command registry
//! 2. MCP Adapter - Adds authentication and authorization
//! 3. Plugin Adapter - Integrates with a plugin architecture

use adapter_pattern_tests::{
    Auth, Command, CommandAdapter, CommandResult, McpAdapter, PluginAdapter, RegistryAdapter,
    TestCommand,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> CommandResult<()> {
    // Demo the Registry Adapter
    demo_registry_adapter().await?;

    // Demo the MCP Adapter
    demo_mcp_adapter().await?;

    // Demo the Plugin Adapter
    demo_plugin_adapter().await?;

    // Demo using polymorphism with the adapter trait
    demo_adapter_trait().await?;

    println!("\nAdapter Pattern demo completed successfully!");

    Ok(())
}

async fn demo_registry_adapter() -> CommandResult<()> {
    print_section("Registry Adapter Demo");

    // Create the adapter
    let mut adapter = RegistryAdapter::new();

    // Register some commands
    let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");

    let hello_clone = hello_cmd.clone();
    let echo_clone = echo_cmd.clone();
    adapter.register(hello_clone.name(), Arc::new(hello_cmd))?;
    adapter.register(echo_clone.name(), Arc::new(echo_cmd))?;

    // List available commands
    let commands = adapter.list_commands()?;
    println!("Available commands: {commands:?}");

    // Execute commands
    let result = adapter.execute("hello", vec![])?;
    println!("Hello command result: {result}");

    let result = adapter.execute("echo", vec!["Hello".to_string(), "there!".to_string()])?;
    println!("Echo command result: {result}");

    // Get help for a command
    let help = adapter.get_help("hello")?;
    println!("Help for hello command: {help}");

    Ok(())
}

async fn demo_mcp_adapter() -> CommandResult<()> {
    print_section("MCP Adapter Demo");

    // Create the adapter
    let adapter = McpAdapter::new();

    // Register some commands
    let regular_cmd = TestCommand::new("regular", "Regular command", "Regular command result");
    let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin command result");

    adapter.register_command(Arc::new(regular_cmd)).await?;
    adapter.register_command(Arc::new(admin_cmd)).await?;

    // Add users
    adapter.add_user("user", "password", false);

    // Test command visibility
    println!("Command visibility demonstration:");

    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let user_auth = Auth::User("user".to_string(), "password".to_string());
    let anon_auth = Auth::None;

    let admin_cmds = adapter.get_available_commands(admin_auth.clone()).await?;
    let user_cmds = adapter.get_available_commands(user_auth.clone()).await?;
    let anon_cmds = adapter.get_available_commands(anon_auth.clone()).await?;

    println!("Commands available to admin: {admin_cmds:?}");
    println!("Commands available to regular user: {user_cmds:?}");
    println!("Commands available to anonymous user: {anon_cmds:?}");

    // Execute commands with different auth levels
    println!("\nCommand execution with authentication:");

    let result = adapter
        .execute_with_auth("regular", vec![], admin_auth.clone())
        .await?;
    println!("Regular command with admin auth: {result}");

    let result = adapter
        .execute_with_auth("admin-cmd", vec![], admin_auth.clone())
        .await?;
    println!("Admin command with admin auth: {result}");

    // Try to execute admin command with regular user
    match adapter
        .execute_with_auth("admin-cmd", vec![], user_auth)
        .await
    {
        Ok(result) => println!("Unexpected success: {result}"),
        Err(e) => println!("Expected error: {e}"),
    }

    // Try to execute admin command anonymously
    match adapter
        .execute_with_auth("admin-cmd", vec![], anon_auth)
        .await
    {
        Ok(result) => println!("Unexpected success: {result}"),
        Err(e) => println!("Expected error: {e}"),
    }

    Ok(())
}

async fn demo_plugin_adapter() -> CommandResult<()> {
    print_section("Plugin Adapter Demo");

    // Create the adapter
    let adapter = PluginAdapter::new();

    // Register some commands
    let plugin_cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
    let advanced_cmd = TestCommand::new("advanced", "Advanced command", "Advanced result");

    adapter.register_command(Arc::new(plugin_cmd)).await?;
    adapter.register_command(Arc::new(advanced_cmd)).await?;

    // Display plugin metadata
    println!("Plugin ID: {}", adapter.plugin_id());
    println!("Plugin Version: {}", adapter.version());

    // List commands
    let commands = adapter.get_commands().await?;
    println!("Plugin commands: {commands:?}");

    // Execute a command
    let result = adapter
        .execute("plugin-cmd", vec!["plugin".to_string(), "arg".to_string()])
        .await?;
    println!("Plugin command result: {result}");

    // Get help
    let help = adapter.get_help("advanced").await?;
    println!("Help for advanced command: {help}");

    Ok(())
}

async fn demo_adapter_trait() -> CommandResult<()> {
    print_section("Adapter Trait Demo");

    // Create all adapters
    let mut registry = RegistryAdapter::new();
    let mcp = McpAdapter::new();
    let plugin = PluginAdapter::new();

    // Register a test command in all adapters
    let test_cmd = TestCommand::new("test", "Test command", "Test result");
    let test_clone = test_cmd.clone();
    registry.register(test_clone.name(), Arc::new(test_cmd.clone()))?;
    mcp.register_command(Arc::new(test_cmd.clone())).await?;
    plugin.register_command(Arc::new(test_cmd)).await?;

    // Test execution through a common function
    test_adapter(&registry, "Registry").await?;
    test_adapter(&mcp, "MCP").await?;
    test_adapter(&plugin, "Plugin").await?;

    Ok(())
}

/// Tests an adapter through the common trait interface
async fn test_adapter(adapter: &dyn CommandAdapter, name: &str) -> CommandResult<()> {
    println!("Testing {name} adapter:");

    let result = adapter.execute("test", vec![]).await?;
    println!("  Execution result: {result}");

    let help = adapter.get_help("test").await?;
    println!("  Help result: {help}");

    let commands = adapter.list_commands().await?;
    println!("  Commands: {commands:?}");

    Ok(())
}

/// Helper function to print section headers
fn print_section(title: &str) {
    println!("\n{}", "=".repeat(50));
    println!("=== {title} ===");
    println!("{}\n", "=".repeat(50));
}
