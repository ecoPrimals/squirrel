// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Adapter Pattern Examples
//!
//! This crate demonstrates the Adapter Pattern in Rust with a command-based architecture.
//! It focuses on three main adapter implementations:
//!
//! 1. Registry Adapter - Basic adapter for command registry operations
//! 2. MCP Adapter - Adapter with authentication and authorization
//! 3. Plugin Adapter - Adapter for plugin system integration
//!
//! Each adapter uses composition to transform one interface into another.

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use thiserror::Error;

// Basic types and errors

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Error type for command operations
#[derive(Error, Debug)]
pub enum CommandError {
    /// Command not found
    #[error("Command not found: {0}")]
    NotFound(String),

    /// Command execution failed
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Authorization failed
    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

// Command interface

/// Command trait representing a command interface
#[async_trait]
pub trait Command: Send + Sync + Debug {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command description
    fn description(&self) -> &str;

    /// Execute the command with the given arguments
    async fn execute(&self, args: Vec<String>) -> CommandResult<String>;
}

/// Test command for examples and testing
#[derive(Debug, Clone)]
pub struct TestCommand {
    name: String,
    description: String,
    result: String,
}

impl TestCommand {
    /// Create a new test command
    #[must_use]
    pub fn new(name: &str, description: &str, result: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            result: result.to_string(),
        }
    }
}

#[async_trait]
impl Command for TestCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    async fn execute(&self, args: Vec<String>) -> CommandResult<String> {
        if args.is_empty() {
            Ok(self.result.clone())
        } else {
            Ok(format!("{} with args: {args:?}", self.result))
        }
    }
}

// Command Registry

/// Command registry to store and execute commands
#[derive(Debug)]
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Create a new command registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command in the registry
    ///
    /// # Errors
    ///
    /// Never returns an error; kept for API consistency.
    pub fn register(&mut self, command: Arc<dyn Command>) -> CommandResult<()> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }

    /// Execute a command by name
    ///
    /// # Errors
    ///
    /// Returns `CommandError::NotFound` if the command is not registered.
    pub async fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        match self.commands.get(name) {
            Some(cmd) => cmd.execute(args).await,
            None => Err(CommandError::NotFound(name.to_string())),
        }
    }

    /// Get help text for a command
    ///
    /// # Errors
    ///
    /// Returns `CommandError::NotFound` if the command is not registered.
    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        self.commands.get(name).map_or_else(
            || Err(CommandError::NotFound(name.to_string())),
            |cmd| Ok(format!("{}: {}", cmd.name(), cmd.description())),
        )
    }

    /// List all available commands
    #[must_use]
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}

// Adapter interface

/// Adapter interface for command operations
#[async_trait]
pub trait CommandAdapter: Send + Sync {
    /// Execute a command with the given arguments
    async fn execute_command(&self, command: &str, args: Vec<String>) -> CommandResult<String>;

    /// Get help text for a command
    async fn get_help(&self, command: &str) -> CommandResult<String>;

    /// List all available commands
    async fn list_commands(&self) -> CommandResult<Vec<String>>;
}

// Registry Adapter

/// Registry adapter implementation
#[derive(Debug, Clone)]
pub struct RegistryAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
}

impl Default for RegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryAdapter {
    /// Create a new registry adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }

    /// Register a command in the registry
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
        registry.register(command)
    }
}

#[async_trait]
impl CommandAdapter for RegistryAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        // Get a clone of the registry to avoid Send issues with MutexGuard across await points
        let registry_clone = {
            let registry = self
                .registry
                .lock()
                .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
            registry
                .list_commands()
                .into_iter()
                .filter_map(|name| registry.commands.get(&name).map(|cmd| (name, cmd.clone())))
                .collect::<HashMap<String, Arc<dyn Command>>>()
        };

        match registry_clone.get(command) {
            Some(cmd) => cmd.execute(args).await,
            None => Err(CommandError::NotFound(command.to_string())),
        }
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        let registry = self
            .registry
            .lock()
            .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
        registry.get_help(command)
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        let registry = self
            .registry
            .lock()
            .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
        Ok(registry.list_commands())
    }
}

// MCP Adapter with authentication

/// Authentication type for MCP adapter
#[derive(Debug, Clone)]
pub enum Auth {
    /// Username/password authentication
    User(String, String),

    /// Token-based authentication
    Token(String),

    /// No authentication
    None,
}

/// User role for permission management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    /// Administrator role
    Admin,

    /// Regular user role
    User,

    /// Guest role with limited access
    Guest,
}

/// Command log entry for audit logging
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    /// Command name
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Username (if authenticated)
    pub user: Option<String>,

    /// Timestamp of execution
    pub timestamp: SystemTime,

    /// Whether the command succeeded
    pub success: bool,

    /// Command output or error message
    pub message: String,
}

/// MCP adapter implementation with authentication
#[derive(Debug)]
pub struct McpAdapter {
    adapter: RegistryAdapter,
    users: HashMap<String, (String, Vec<UserRole>)>, // username -> (password, roles)
    command_permissions: HashMap<String, Vec<UserRole>>, // command -> required roles
    tokens: HashMap<String, String>,                 // token -> username
    command_log: Vec<CommandLogEntry>,
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl McpAdapter {
    /// Create a new MCP adapter
    #[must_use]
    pub fn new() -> Self {
        let mut adapter = Self {
            adapter: RegistryAdapter::new(),
            users: HashMap::new(),
            command_permissions: HashMap::new(),
            tokens: HashMap::new(),
            command_log: Vec::new(),
        };

        // Add default admin user
        adapter.add_user("admin", "password", vec![UserRole::Admin]);

        adapter
    }

    /// Add a user with roles
    pub fn add_user(&mut self, username: &str, password: &str, roles: Vec<UserRole>) {
        self.users
            .insert(username.to_string(), (password.to_string(), roles));
    }

    /// Register a command
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&mut self, command: Arc<dyn Command>) -> CommandResult<()> {
        // For admin commands, automatically restrict to admin role
        let name = command.name();
        if name.starts_with("admin") && !self.command_permissions.contains_key(name) {
            self.add_command_permissions(name, vec![UserRole::Admin]);
        }

        self.adapter.register_command(command)
    }

    /// Add permissions required for a command
    pub fn add_command_permissions(&mut self, command: &str, roles: Vec<UserRole>) {
        self.command_permissions.insert(command.to_string(), roles);
    }

    /// Generate an authentication token
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` if credentials are invalid.
    pub fn generate_token(&mut self, username: &str, password: &str) -> CommandResult<String> {
        // Verify credentials
        if let Some((stored_password, _)) = self.users.get(username) {
            if stored_password != password {
                return Err(CommandError::AuthenticationFailed(
                    "Invalid password".to_string(),
                ));
            }
        } else {
            return Err(CommandError::AuthenticationFailed(
                "User not found".to_string(),
            ));
        }

        // Generate a token (in a real system, use a proper JWT)
        let token = format!("token-{}-{}", username, uuid::Uuid::new_v4());
        self.tokens.insert(token.clone(), username.to_string());

        Ok(token)
    }

    /// Log a command execution
    fn log_command(
        &mut self,
        command: &str,
        args: &[String],
        user: Option<&str>,
        success: bool,
        message: String,
    ) {
        self.command_log.push(CommandLogEntry {
            command: command.to_string(),
            args: args.to_vec(),
            user: user.map(std::string::ToString::to_string),
            timestamp: SystemTime::now(),
            success,
            message,
        });
    }

    /// Get command logs
    #[must_use]
    pub fn get_logs(&self) -> &[CommandLogEntry] {
        &self.command_log
    }

    /// Execute a command with authentication
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` or `CommandError::AuthorizationFailed` on auth failure.
    pub async fn execute_with_auth(
        &mut self,
        command: &str,
        args: Vec<String>,
        auth: Auth,
    ) -> CommandResult<String> {
        println!("McpAdapter::execute_with_auth called for {command} with {auth:?}");

        // List commands to debug
        let commands = self.adapter.list_commands().await?;
        println!("Available commands in adapter: {commands:?}");

        // Authenticate and get username
        let username_string = match &auth {
            Auth::User(username, password) => {
                // Verify credentials
                let users = self.users.clone();
                if let Some((stored_password, _)) = users.get(username) {
                    if stored_password != password {
                        return Err(CommandError::AuthenticationFailed(
                            "Invalid password".to_string(),
                        ));
                    }
                    Some(username.clone())
                } else {
                    return Err(CommandError::AuthenticationFailed(
                        "User not found".to_string(),
                    ));
                }
            }
            Auth::Token(token) => {
                // Verify token
                let tokens = self.tokens.clone();
                match tokens.get(token) {
                    Some(username) => Some(username.clone()),
                    None => {
                        return Err(CommandError::AuthenticationFailed(
                            "Invalid token".to_string(),
                        ));
                    }
                }
            }
            Auth::None => None,
        };

        let username = username_string.as_deref();

        // Check authorization
        {
            let command_permissions = self.command_permissions.clone();
            let users = self.users.clone();
            if let Some(required_roles) = command_permissions.get(command) {
                match username {
                    Some(username) => {
                        // Get user roles
                        if let Some((_, roles)) = users.get(username) {
                            // Check if user has any of the required roles
                            let has_role = roles.iter().any(|role| {
                                required_roles.contains(role) || *role == UserRole::Admin
                            });

                            if !has_role {
                                return Err(CommandError::AuthorizationFailed(format!(
                                    "User '{username}' is not authorized to execute command '{command}'"
                                )));
                            }
                        }
                    }
                    None => {
                        return Err(CommandError::AuthenticationFailed(format!(
                            "Authentication required for command '{command}'"
                        )));
                    }
                }
            }
        }

        // Execute the command
        let result = self.adapter.execute_command(command, args.clone()).await;

        // Log the execution
        match &result {
            Ok(output) => {
                self.log_command(command, &args, username, true, output.clone());
            }
            Err(e) => {
                self.log_command(command, &args, username, false, e.to_string());
            }
        }

        result
    }

    /// Get available commands for a user
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` if credentials are invalid.
    pub async fn get_available_commands(&self, auth: Auth) -> CommandResult<Vec<String>> {
        println!("McpAdapter::get_available_commands called with {auth:?}");

        // Get all commands
        let all_commands = self.adapter.list_commands().await?;
        println!("All commands from registry: {all_commands:?}");

        // Filter commands based on permissions
        match auth {
            Auth::User(username, password) => {
                // Verify credentials
                if let Some((stored_password, roles)) = self.users.get(&username) {
                    if stored_password != &password {
                        return Err(CommandError::AuthenticationFailed(
                            "Invalid password".to_string(),
                        ));
                    }

                    // Admin can access all commands
                    if roles.contains(&UserRole::Admin) {
                        return Ok(all_commands);
                    }

                    // Filter commands based on roles
                    let filtered = all_commands
                        .into_iter()
                        .filter(|cmd| {
                            self.command_permissions
                                .get(cmd)
                                .is_none_or(|required_roles| {
                                    roles.iter().any(|role| required_roles.contains(role))
                                })
                        })
                        .collect();

                    Ok(filtered)
                } else {
                    Err(CommandError::AuthenticationFailed(
                        "User not found".to_string(),
                    ))
                }
            }
            Auth::Token(token) => {
                // Verify token
                if let Some(username) = self.tokens.get(&token) {
                    // Get user roles
                    if let Some((_, roles)) = self.users.get(username) {
                        // Admin can access all commands
                        if roles.contains(&UserRole::Admin) {
                            return Ok(all_commands);
                        }

                        // Filter commands based on roles
                        let filtered = all_commands
                            .into_iter()
                            .filter(|cmd| {
                                self.command_permissions
                                    .get(cmd)
                                    .is_none_or(|required_roles| {
                                        roles.iter().any(|role| required_roles.contains(role))
                                    })
                            })
                            .collect();

                        Ok(filtered)
                    } else {
                        Err(CommandError::AuthenticationFailed(
                            "User not found".to_string(),
                        ))
                    }
                } else {
                    Err(CommandError::AuthenticationFailed(
                        "Invalid token".to_string(),
                    ))
                }
            }
            Auth::None => {
                // Filter out commands that require authentication
                let filtered = all_commands
                    .into_iter()
                    .filter(|cmd| !self.command_permissions.contains_key(cmd))
                    .collect();

                Ok(filtered)
            }
        }
    }
}

// Allow cloning McpAdapter for tests
impl Clone for McpAdapter {
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            users: self.users.clone(),
            command_permissions: self.command_permissions.clone(),
            tokens: self.tokens.clone(),
            command_log: self.command_log.clone(),
        }
    }
}

#[async_trait]
impl CommandAdapter for McpAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        // Clone self to avoid borrow checker issues with async fn & mut self
        let mut cloned = self.clone();
        println!("McpAdapter::execute_command called for {command}");
        cloned.execute_with_auth(command, args, Auth::None).await
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        println!("McpAdapter::get_help called for {command}");
        self.adapter.get_help(command).await
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        // Get all commands
        println!("McpAdapter::list_commands called");
        let all_commands = self.adapter.list_commands().await?;

        // Filter out commands that require authentication
        let filtered = all_commands
            .into_iter()
            .filter(|cmd| !self.command_permissions.contains_key(cmd))
            .collect();

        Ok(filtered)
    }
}

// Plugin Adapter

/// Plugin metadata information
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: String,

    /// Plugin version
    pub version: String,

    /// Plugin description
    pub description: String,
}

/// Plugin adapter implementation
#[derive(Debug)]
pub struct PluginAdapter {
    adapter: RegistryAdapter,
    metadata: PluginMetadata,
}

impl Default for PluginAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginAdapter {
    /// Create a new plugin adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: RegistryAdapter::new(),
            metadata: PluginMetadata {
                id: "commands".to_string(),
                version: "1.0.0".to_string(),
                description: "Command plugin for the Squirrel CLI".to_string(),
            },
        }
    }

    /// Get plugin ID
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        &self.metadata.id
    }

    /// Get plugin version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.metadata.version
    }

    /// Get plugin description
    #[must_use]
    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    /// Register a command
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        self.adapter.register_command(command)
    }

    /// Get available commands
    ///
    /// # Errors
    ///
    /// Never returns an error; kept for API consistency.
    pub async fn get_commands(&self) -> CommandResult<Vec<String>> {
        self.adapter.list_commands().await
    }
}

#[async_trait]
impl CommandAdapter for PluginAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        self.adapter.execute_command(command, args).await
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        self.adapter.get_help(command).await
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        self.adapter.list_commands().await
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_adapter() -> CommandResult<()> {
        // Create a registry adapter
        let adapter = RegistryAdapter::new();

        // Create and register test commands
        let hello_cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
        let echo_cmd = TestCommand::new("echo", "Echoes arguments", "Echo");

        adapter.register_command(Arc::new(hello_cmd))?;
        adapter.register_command(Arc::new(echo_cmd))?;

        // Test command execution without arguments
        let result = adapter.execute_command("hello", vec![]).await?;
        assert_eq!(result, "Hello, world!");

        // Test command execution with arguments
        let result = adapter
            .execute_command("echo", vec!["Hello".to_string(), "there!".to_string()])
            .await?;
        assert_eq!(result, "Echo with args: [\"Hello\", \"there!\"]");

        // Test help information
        let help = adapter.get_help("hello").await?;
        assert_eq!(help, "hello: Says hello");

        // Test listing commands
        let commands = adapter.list_commands().await?;
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&"hello".to_string()));
        assert!(commands.contains(&"echo".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_mcp_adapter_authentication() -> CommandResult<()> {
        // Create an MCP adapter
        let mut adapter = McpAdapter::new();

        // Register commands
        let cmd = TestCommand::new("secure", "Secure command", "Secret data");
        let admin_cmd = TestCommand::new("admin-cmd", "Admin command", "Admin data");

        adapter.register_command(Arc::new(cmd))?;
        adapter.register_command(Arc::new(admin_cmd))?;

        // Test admin authentication and admin command execution
        let admin_auth = Auth::User("admin".to_string(), "password".to_string());
        let result = adapter
            .execute_with_auth("admin-cmd", vec![], admin_auth.clone())
            .await?;
        assert_eq!(result, "Admin data");

        // Test anonymous access to regular command (should succeed)
        let result = adapter
            .execute_with_auth("secure", vec![], Auth::None)
            .await?;
        assert_eq!(result, "Secret data");

        // Test anonymous access to admin command (should fail)
        let result = adapter
            .execute_with_auth("admin-cmd", vec![], Auth::None)
            .await;
        assert!(result.is_err());

        // Test access with invalid credentials
        let invalid_auth = Auth::User("admin".to_string(), "wrong-password".to_string());
        let result = adapter
            .execute_with_auth("secure", vec![], invalid_auth)
            .await;
        assert!(result.is_err());

        // Test token authentication
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
        // Create a plugin adapter
        let adapter = PluginAdapter::new();

        // Verify plugin metadata
        assert_eq!(adapter.plugin_id(), "commands");
        assert_eq!(adapter.version(), "1.0.0");

        // Register commands
        let cmd = TestCommand::new("plugin-cmd", "Plugin command", "Plugin result");
        adapter.register_command(Arc::new(cmd))?;

        // List commands
        let commands = adapter.get_commands().await?;
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "plugin-cmd");

        // Execute command
        let result = adapter
            .execute_command("plugin-cmd", vec!["arg1".to_string(), "arg2".to_string()])
            .await?;
        assert_eq!(result, "Plugin result with args: [\"arg1\", \"arg2\"]");

        Ok(())
    }

    #[tokio::test]
    async fn test_polymorphic_adapter_usage() -> CommandResult<()> {
        // Function that works with any CommandAdapter implementation
        async fn execute_with_adapter(
            adapter: &dyn CommandAdapter,
            command: &str,
        ) -> CommandResult<String> {
            adapter.execute_command(command, vec![]).await
        }

        // Create different adapters
        let registry_adapter = RegistryAdapter::new();
        let mut mcp_adapter = McpAdapter::new();
        let plugin_adapter = PluginAdapter::new();

        // Register the same command in all adapters
        let test_cmd = TestCommand::new("test", "Test command", "Test result");
        registry_adapter.register_command(Arc::new(test_cmd.clone()))?;
        mcp_adapter.register_command(Arc::new(test_cmd.clone()))?;
        plugin_adapter.register_command(Arc::new(test_cmd.clone()))?;

        // Execute command with different adapters
        let result1 = execute_with_adapter(&registry_adapter, "test").await?;
        let result2 = execute_with_adapter(&mcp_adapter, "test").await?;
        let result3 = execute_with_adapter(&plugin_adapter, "test").await?;

        // All should return the same result
        assert_eq!(result1, "Test result");
        assert_eq!(result2, "Test result");
        assert_eq!(result3, "Test result");

        Ok(())
    }
}
