// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP adapter with authentication, authorization, and audit logging.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use crate::command::DynCommand;
use crate::registry::{CommandAdapter, RegistryAdapter};
use crate::{CommandError, CommandResult};

/// Authentication type for MCP adapter.
#[derive(Debug, Clone)]
pub enum Auth {
    /// Username/password authentication.
    User(String, String),
    /// Token-based authentication.
    Token(String),
    /// No authentication.
    None,
}

/// User role for permission management.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRole {
    /// Administrator role.
    Admin,
    /// Regular user role.
    User,
    /// Guest role with limited access.
    Guest,
}

/// Command log entry for audit logging.
#[derive(Debug, Clone)]
pub struct CommandLogEntry {
    /// Command name.
    pub command: String,
    /// Command arguments.
    pub args: Vec<String>,
    /// Username (if authenticated).
    pub user: Option<String>,
    /// Timestamp of execution.
    pub timestamp: SystemTime,
    /// Whether the command succeeded.
    pub success: bool,
    /// Command output or error message.
    pub message: String,
}

/// MCP adapter implementation with authentication.
#[derive(Debug)]
pub struct McpAdapter {
    adapter: RegistryAdapter,
    users: HashMap<String, (String, Vec<UserRole>)>,
    command_permissions: HashMap<String, Vec<UserRole>>,
    tokens: HashMap<String, String>,
    command_log: Vec<CommandLogEntry>,
}

impl Default for McpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

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

impl McpAdapter {
    /// Create a new MCP adapter.
    #[must_use]
    pub fn new() -> Self {
        let mut adapter = Self {
            adapter: RegistryAdapter::new(),
            users: HashMap::new(),
            command_permissions: HashMap::new(),
            tokens: HashMap::new(),
            command_log: Vec::new(),
        };
        adapter.add_user("admin", "password", vec![UserRole::Admin]);
        adapter
    }

    /// Add a user with roles.
    pub fn add_user(&mut self, username: &str, password: &str, roles: Vec<UserRole>) {
        self.users
            .insert(username.to_string(), (password.to_string(), roles));
    }

    /// Register a command.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&mut self, command: Arc<dyn DynCommand>) -> CommandResult<()> {
        let name = command.name();
        if name.starts_with("admin") && !self.command_permissions.contains_key(name) {
            self.add_command_permissions(name, vec![UserRole::Admin]);
        }
        self.adapter.register_command(command)
    }

    /// Add permissions required for a command.
    pub fn add_command_permissions(&mut self, command: &str, roles: Vec<UserRole>) {
        self.command_permissions.insert(command.to_string(), roles);
    }

    /// Generate an authentication token.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` if credentials are invalid.
    pub fn generate_token(&mut self, username: &str, password: &str) -> CommandResult<String> {
        let stored = self
            .users
            .get(username)
            .ok_or_else(|| CommandError::AuthenticationFailed("User not found".to_string()))?;
        if stored.0 != password {
            return Err(CommandError::AuthenticationFailed(
                "Invalid password".to_string(),
            ));
        }
        let token = format!("token-{}-{}", username, uuid::Uuid::new_v4());
        self.tokens.insert(token.clone(), username.to_string());
        Ok(token)
    }

    /// Resolve [`Auth`] into a verified username (or `None` for anonymous).
    fn authenticate(&self, auth: &Auth) -> CommandResult<Option<String>> {
        match auth {
            Auth::User(username, password) => {
                let (stored_pw, _) = self.users.get(username).ok_or_else(|| {
                    CommandError::AuthenticationFailed("User not found".to_string())
                })?;
                if stored_pw != password {
                    return Err(CommandError::AuthenticationFailed(
                        "Invalid password".to_string(),
                    ));
                }
                Ok(Some(username.clone()))
            }
            Auth::Token(token) => {
                let username = self.tokens.get(token).ok_or_else(|| {
                    CommandError::AuthenticationFailed("Invalid token".to_string())
                })?;
                Ok(Some(username.clone()))
            }
            Auth::None => Ok(None),
        }
    }

    /// Check whether `username` may execute `command`.
    fn authorize(&self, command: &str, username: Option<&str>) -> CommandResult<()> {
        let Some(required_roles) = self.command_permissions.get(command) else {
            return Ok(());
        };
        let Some(username) = username else {
            return Err(CommandError::AuthenticationFailed(format!(
                "Authentication required for command '{command}'"
            )));
        };
        let (_, roles) = self
            .users
            .get(username)
            .ok_or_else(|| CommandError::AuthenticationFailed("User not found".to_string()))?;
        let has_role = roles
            .iter()
            .any(|role| required_roles.contains(role) || *role == UserRole::Admin);
        if !has_role {
            return Err(CommandError::AuthorizationFailed(format!(
                "User '{username}' is not authorized to execute command '{command}'"
            )));
        }
        Ok(())
    }

    /// Filter a command list down to what `username` (with its roles) may see.
    fn filter_commands(&self, commands: Vec<String>, username: Option<&str>) -> Vec<String> {
        let roles = username
            .and_then(|u| self.users.get(u))
            .map(|(_, r)| r.as_slice());
        if roles.is_some_and(|r| r.contains(&UserRole::Admin)) {
            return commands;
        }
        commands
            .into_iter()
            .filter(|cmd| {
                self.command_permissions.get(cmd).is_none_or(|required| {
                    roles.is_some_and(|user_roles| user_roles.iter().any(|r| required.contains(r)))
                })
            })
            .collect()
    }

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

    /// Get command logs.
    #[must_use]
    pub fn get_logs(&self) -> &[CommandLogEntry] {
        &self.command_log
    }

    /// Execute a command with authentication.
    ///
    /// # Errors
    ///
    /// Returns auth or execution errors.
    pub async fn execute_with_auth(
        &mut self,
        command: &str,
        args: Vec<String>,
        auth: Auth,
    ) -> CommandResult<String> {
        let username = self.authenticate(&auth)?;
        self.authorize(command, username.as_deref())?;

        let result = self.adapter.execute_command(command, args.clone()).await;

        match &result {
            Ok(output) => {
                self.log_command(command, &args, username.as_deref(), true, output.clone());
            }
            Err(e) => {
                self.log_command(command, &args, username.as_deref(), false, e.to_string());
            }
        }
        result
    }

    /// Get available commands for a user.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` if credentials are invalid.
    pub async fn get_available_commands(&self, auth: Auth) -> CommandResult<Vec<String>> {
        let username = self.authenticate(&auth)?;
        let all = self.adapter.list_commands().await?;
        Ok(self.filter_commands(all, username.as_deref()))
    }
}

impl CommandAdapter for McpAdapter {
    fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let mut cloned = self.clone();
        let command = command.to_string();
        async move { cloned.execute_with_auth(&command, args, Auth::None).await }
    }

    fn get_help(
        &self,
        command: &str,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let adapter = self.adapter.clone();
        let command = command.to_string();
        async move { adapter.get_help(&command).await }
    }

    fn list_commands(
        &self,
    ) -> impl std::future::Future<Output = CommandResult<Vec<String>>> + Send + '_ {
        let this = self.clone();
        async move {
            let all_commands = this.adapter.list_commands().await?;
            Ok(this.filter_commands(all_commands, None))
        }
    }
}
