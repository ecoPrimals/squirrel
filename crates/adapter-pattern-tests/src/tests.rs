// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Adapter pattern tests

use crate::{
    Auth, Command, CommandAdapter, CommandError, CommandResult, McpAdapter, PluginAdapter,
    RegistryAdapter, TestCommand,
};
use std::sync::Arc;

#[tokio::test]
async fn test_registry_adapter() -> CommandResult<()> {
    let mut adapter = RegistryAdapter::new();

    let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
    let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");

    adapter.register(hello_cmd.clone().name(), Arc::new(hello_cmd))?;
    adapter.register(echo_cmd.clone().name(), Arc::new(echo_cmd))?;

    let result = <RegistryAdapter as CommandAdapter>::execute(&adapter, "hello", vec![]).await?;
    assert_eq!(result, "Hello, world!");

    let result = <RegistryAdapter as CommandAdapter>::execute(
        &adapter,
        "echo",
        vec!["Hello".to_string(), "there!".to_string()],
    )
    .await?;
    assert_eq!(result, "Echo with args: [\"Hello\", \"there!\"]");

    let help = <RegistryAdapter as CommandAdapter>::get_help(&adapter, "hello").await?;
    assert_eq!(help, "hello: Says hello");

    let commands = <RegistryAdapter as CommandAdapter>::list_commands(&adapter).await?;
    assert_eq!(commands.len(), 2);
    assert!(commands.contains(&"hello".to_string()));
    assert!(commands.contains(&"echo".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_mcp_adapter_authentication() -> CommandResult<()> {
    let adapter = McpAdapter::new();

    let cmd = TestCommand::new("secure", "Secure command", "Secret data");
    adapter.register_command(Arc::new(cmd)).await?;

    let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin data");
    adapter.register_command(Arc::new(admin_cmd)).await?;

    let admin_auth = Auth::User("admin".to_string(), "password".to_string());
    let result = adapter
        .execute_with_auth("admin-cmd", vec![], admin_auth.clone())
        .await?;
    assert_eq!(result, "Admin data");

    let result = adapter
        .execute_with_auth("secure", vec![], Auth::None)
        .await?;
    assert_eq!(result, "Secret data");

    let result = adapter
        .execute_with_auth("admin-cmd", vec![], Auth::None)
        .await;
    assert!(result.is_err());
    match result {
        Err(CommandError::AuthorizationFailed(_)) => (),
        _ => panic!("Expected authorization failure"),
    }

    let invalid_auth = Auth::User("admin".to_string(), "wrong_password".to_string());
    let result = adapter
        .execute_with_auth("secure", vec![], invalid_auth)
        .await;
    assert!(result.is_err());
    match result {
        Err(CommandError::AuthenticationFailed(_)) => (),
        _ => panic!("Expected authentication failure"),
    }

    Ok(())
}

#[tokio::test]
async fn test_plugin_adapter() -> CommandResult<()> {
    let adapter = PluginAdapter::new();

    assert_eq!(adapter.plugin_id(), "commands");
    assert_eq!(adapter.version(), "1.0.0");

    let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
    adapter.register_command(Arc::new(cmd)).await?;

    let commands = adapter.get_commands().await?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0], "plugin-cmd");

    let result = CommandAdapter::execute(
        &adapter,
        "plugin-cmd",
        vec!["arg1".to_string(), "arg2".to_string()],
    )
    .await?;
    assert_eq!(result, "Plugin result with args: [\"arg1\", \"arg2\"]");

    Ok(())
}

#[tokio::test]
async fn test_adapter_trait() -> CommandResult<()> {
    async fn test_adapter(adapter: &dyn CommandAdapter, cmd_name: &str) -> CommandResult<String> {
        adapter.execute(cmd_name, vec![]).await
    }

    let mut registry_adapter = RegistryAdapter::new();
    let mcp_adapter = McpAdapter::new();
    let plugin_adapter = PluginAdapter::new();

    let test_cmd = TestCommand::new("test", "Test command", "Test result");
    registry_adapter.register(test_cmd.clone().name(), Arc::new(test_cmd.clone()))?;
    mcp_adapter
        .register_command(Arc::new(test_cmd.clone()))
        .await?;
    plugin_adapter.register_command(Arc::new(test_cmd)).await?;

    let result1 = test_adapter(&registry_adapter, "test").await?;
    let result2 = test_adapter(&mcp_adapter, "test").await?;
    let result3 = test_adapter(&plugin_adapter, "test").await?;

    assert_eq!(result1, "Test result");
    assert_eq!(result2, "Test result");
    assert_eq!(result3, "Test result");

    Ok(())
}
