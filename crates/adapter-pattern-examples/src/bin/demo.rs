// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Adapter Pattern Demo
//!
//! This demo showcases the adapter pattern implementations including registry,
//! MCP (authentication), and plugin adapters.

use adapter_pattern_examples::{
    Auth, CommandAdapter, McpAdapter, PluginAdapter, RegistryAdapter, TestCommand, UserRole,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Adapter Pattern Demo ===\n");

    // === Registry Adapter Demo ===
    println!("--- Registry Adapter Demo ---");
    let registry_adapter = RegistryAdapter::new();

    // Register commands
    let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");

    registry_adapter.register_command(Arc::new(hello_cmd))?;
    registry_adapter.register_command(Arc::new(echo_cmd))?;

    // Execute commands
    let result = registry_adapter.execute_command("hello", vec![]).await?;
    println!("Hello command result: {}", result);

    let result = registry_adapter
        .execute_command(
            "echo",
            vec![
                "Hello".to_string(),
                "from".to_string(),
                "adapter!".to_string(),
            ],
        )
        .await?;
    println!("Echo command result: {}", result);

    let commands = registry_adapter.list_commands().await?;
    println!("Available commands: {:?}\n", commands);

    // === MCP Adapter Demo ===
    println!("--- MCP Adapter Demo ---");
    let mut mcp_adapter = McpAdapter::new();

    // Register commands
    let cmd = TestCommand::new("secure", "Secure command", "Secret data");
    let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin data");

    mcp_adapter.register_command(Arc::new(cmd))?;
    mcp_adapter.register_command(Arc::new(admin_cmd))?;

    // Add a regular user
    mcp_adapter.add_user("user", "userpass", vec![UserRole::User]);

    // Restrict secure command to users
    mcp_adapter.add_command_permissions("secure", vec![UserRole::User]);

    // Try anonymous access (should fail for secure command)
    let result = mcp_adapter.execute_command("secure", vec![]).await;
    println!("Anonymous access to secure command: {:?}", result);

    // Try with user credentials
    let user_auth = Auth::User("user".to_string(), "userpass".to_string());
    let result = mcp_adapter
        .execute_with_auth("secure", vec![], user_auth.clone())
        .await?;
    println!("User access to secure command: {}", result);

    // Try user access to admin command (should fail)
    let result = mcp_adapter
        .execute_with_auth("admin-cmd", vec![], user_auth)
        .await;
    println!("User access to admin command: {:?}", result);

    // Try with admin credentials
    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = mcp_adapter
        .execute_with_auth("admin-cmd", vec![], admin_auth.clone())
        .await?;
    println!("Admin access to admin command: {}", result);

    // Get available commands for different users
    let commands = mcp_adapter.get_available_commands(Auth::None).await?;
    println!(
        "Commands available to unauthenticated users: {:?}",
        commands
    );

    let commands = mcp_adapter.get_available_commands(admin_auth).await?;
    println!("Commands available to admin: {:?}\n", commands);

    // === Plugin Adapter Demo ===
    println!("--- Plugin Adapter Demo ---");
    let plugin_adapter = PluginAdapter::new();

    // Show plugin metadata
    println!("Plugin ID: {}", plugin_adapter.plugin_id());
    println!("Plugin Version: {}", plugin_adapter.version());
    println!("Plugin Description: {}", plugin_adapter.description());

    // Register commands
    let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
    plugin_adapter.register_command(Arc::new(cmd))?;

    // Execute command
    let result = plugin_adapter
        .execute_command("plugin-cmd", vec!["arg1".to_string(), "arg2".to_string()])
        .await?;
    println!("Plugin command result: {}", result);

    // Get help
    let help = plugin_adapter.get_help("plugin-cmd").await?;
    println!("Plugin command help: {}", help);

    Ok(())
}
