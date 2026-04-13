// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![warn(missing_docs)]

//! Adapter Pattern Examples
//!
//! This crate demonstrates the Adapter Pattern in Rust with a command-based architecture.
//! It focuses on three main adapter implementations:
//!
//! 1. **Registry Adapter** — basic adapter for command registry operations
//! 2. **MCP Adapter** — adapter with authentication and authorization
//! 3. **Plugin Adapter** — adapter for plugin system integration
//!
//! Each adapter uses composition to transform one interface into another.
//!
//! Implement [`Command`](command::Command) and [`CommandAdapter`](registry::CommandAdapter)
//! with `impl Future<Output = _> + Send` (no `async_trait` on these traits).
//! For heterogeneous command storage, use the object-safe [`DynCommand`](command::DynCommand)
//! bridge (`async_trait`, same idea as `DynPlugin` in the interfaces crate).

use thiserror::Error;

pub mod command;
pub mod mcp;
pub mod plugin;
pub mod registry;

pub use command::{Command, DynCommand, TestCommand};
pub use mcp::{Auth, CommandLogEntry, McpAdapter, UserRole};
pub use plugin::{PluginAdapter, PluginMetadata};
pub use registry::{CommandAdapter, CommandRegistry, RegistryAdapter};

/// Result type for command operations.
pub type CommandResult<T> = Result<T, CommandError>;

/// Error type for command operations.
#[derive(Error, Debug)]
pub enum CommandError {
    /// Command not found.
    #[error("Command not found: {0}")]
    NotFound(String),

    /// Command execution failed.
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization failed.
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_registry_adapter() -> CommandResult<()> {
        let adapter = RegistryAdapter::new();
        let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
        let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");

        adapter.register_command(Arc::new(hello_cmd))?;
        adapter.register_command(Arc::new(echo_cmd))?;

        let result = adapter.execute_command("hello", vec![]).await?;
        assert_eq!(result, "Hello, world!");

        let result = adapter
            .execute_command("echo", vec!["Hello".to_string(), "there!".to_string()])
            .await?;
        assert_eq!(result, "Echo with args: [\"Hello\", \"there!\"]");

        let help = adapter.get_help("hello").await?;
        assert_eq!(help, "hello: Says hello");

        let commands = adapter.list_commands().await?;
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"hello".to_string()));
        assert!(commands.contains(&"echo".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_mcp_adapter_authentication() -> CommandResult<()> {
        let mut adapter = McpAdapter::new();

        let cmd = TestCommand::new("secure", "Secure command", "Secret data");
        let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin data");

        adapter.register_command(Arc::new(cmd))?;
        adapter.register_command(Arc::new(admin_cmd))?;

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

        let invalid_auth = Auth::User("admin".to_string(), "wrong-password".to_string());
        let result = adapter
            .execute_with_auth("secure", vec![], invalid_auth)
            .await;
        assert!(result.is_err());

        let token = adapter.generate_token("admin", "password")?;
        let token_auth = Auth::Token(token);
        let result = adapter
            .execute_with_auth("admin-cmd", vec![], token_auth)
            .await?;
        assert_eq!(result, "Admin data");

        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_adapter() -> CommandResult<()> {
        let adapter = PluginAdapter::new();

        assert_eq!(adapter.plugin_id(), "commands");
        assert_eq!(adapter.version(), "1.0.0");

        let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
        adapter.register_command(Arc::new(cmd))?;

        let commands = adapter.get_commands().await?;
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "plugin-cmd");

        let result = adapter
            .execute_command("plugin-cmd", vec!["arg1".to_string(), "arg2".to_string()])
            .await?;
        assert_eq!(result, "Plugin result with args: [\"arg1\", \"arg2\"]");

        Ok(())
    }

    #[tokio::test]
    async fn test_polymorphic_adapter_usage() -> CommandResult<()> {
        async fn execute_with_adapter<A: CommandAdapter>(
            adapter: &A,
            command: &str,
        ) -> CommandResult<String> {
            adapter.execute_command(command, vec![]).await
        }

        let registry_adapter = RegistryAdapter::new();
        let mut mcp_adapter = McpAdapter::new();
        let plugin_adapter = PluginAdapter::new();

        let test_cmd = TestCommand::new("test", "Test command", "Test result");
        registry_adapter.register_command(Arc::new(test_cmd.clone()))?;
        mcp_adapter.register_command(Arc::new(test_cmd.clone()))?;
        plugin_adapter.register_command(Arc::new(test_cmd.clone()))?;

        let result1 = execute_with_adapter(&registry_adapter, "test").await?;
        let result2 = execute_with_adapter(&mcp_adapter, "test").await?;
        let result3 = execute_with_adapter(&plugin_adapter, "test").await?;

        assert_eq!(result1, "Test result");
        assert_eq!(result2, "Test result");
        assert_eq!(result3, "Test result");

        Ok(())
    }
}
