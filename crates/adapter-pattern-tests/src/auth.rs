// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! MCP adapter with authentication and authorization

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::commands::{CommandAdapter, RegistryAdapter};
use crate::types::{
    Auth, AuthUser, Command, CommandError, CommandLogEntry, CommandResult, UserRole,
};

/// MCP adapter implementation with authentication
#[derive(Debug)]
pub struct McpAdapter {
    adapter: Arc<RwLock<RegistryAdapter>>,
    authorized_users: Arc<RwLock<HashMap<String, String>>>, // username -> password
    user_roles: Arc<RwLock<HashMap<String, Vec<UserRole>>>>, // username -> roles
    command_permissions: Arc<RwLock<HashMap<String, Vec<UserRole>>>>, // command -> required roles
    active_tokens: Arc<RwLock<HashMap<String, AuthUser>>>,  // token -> user
    command_log: Arc<RwLock<Vec<CommandLogEntry>>>,         // audit log
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
            authorized_users: self.authorized_users.clone(),
            user_roles: self.user_roles.clone(),
            command_permissions: self.command_permissions.clone(),
            active_tokens: self.active_tokens.clone(),
            command_log: self.command_log.clone(),
        }
    }
}

impl McpAdapter {
    /// Create a new MCP adapter with default admin user
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned (e.g., from a prior panic while holding the lock).
    #[must_use]
    pub fn new() -> Self {
        let instance = Self {
            adapter: Arc::new(RwLock::new(RegistryAdapter::new())),
            authorized_users: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            command_permissions: Arc::new(RwLock::new(HashMap::new())),
            active_tokens: Arc::new(RwLock::new(HashMap::new())),
            command_log: Arc::new(RwLock::new(Vec::new())),
        };

        // Add default admin user synchronously
        {
            let mut users = instance.authorized_users.write().unwrap();
            users.insert("admin".to_string(), "password".to_string());
        }

        {
            let mut user_roles = instance.user_roles.write().unwrap();
            user_roles.insert("admin".to_string(), vec![UserRole::Admin]);
        }

        // Mark admin commands as requiring admin role
        {
            let mut permissions = instance.command_permissions.write().unwrap();
            permissions.insert("admin-cmd".to_string(), vec![UserRole::Admin]);
        }

        instance
    }

    /// Add a new user to the system with specified admin status
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned.
    pub fn add_user(&self, username: &str, password: &str, is_admin: bool) {
        self.authorized_users
            .write()
            .unwrap()
            .insert(username.to_string(), password.to_string());

        let roles = if is_admin {
            vec![UserRole::Admin]
        } else {
            vec![UserRole::RegularUser]
        };

        self.user_roles
            .write()
            .unwrap()
            .insert(username.to_string(), roles);

        if is_admin {
            let mut permissions = self.command_permissions.write().unwrap();
            permissions.insert("admin-cmd".to_string(), vec![UserRole::Admin]);
        }
    }

    /// Add a command with specific role requirements
    ///
    /// # Panics
    ///
    /// Panics if the internal `RwLock` is poisoned.
    pub fn add_command_with_permissions(&mut self, command_name: &str, roles: Vec<UserRole>) {
        self.command_permissions
            .write()
            .unwrap()
            .insert(command_name.to_string(), roles);
    }

    /// Generate an authentication token for a user
    ///
    /// # Errors
    ///
    /// Returns `CommandError::AuthenticationFailed` if credentials are invalid.
    pub fn generate_token(&mut self, username: &str, password: &str) -> CommandResult<String> {
        let stored_password = self
            .authorized_users
            .read()
            .map_err(|_| {
                CommandError::AuthenticationFailed(
                    "Internal error: user storage corrupted".to_string(),
                )
            })?
            .get(username)
            .cloned();

        if let Some(stored_password) = stored_password {
            if password != stored_password {
                return Err(CommandError::AuthenticationFailed(format!(
                    "Invalid password for user '{username}'"
                )));
            }
        } else {
            return Err(CommandError::AuthenticationFailed(format!(
                "User '{username}' not found"
            )));
        }

        let token = format!(
            "token-{}-{}",
            username,
            std::time::SystemTime::now().elapsed().unwrap().as_secs()
        );

        let roles = self
            .user_roles
            .read()
            .map_err(|_| {
                CommandError::AuthenticationFailed(
                    "Internal error: user roles storage corrupted".to_string(),
                )
            })?
            .get(username)
            .cloned()
            .unwrap_or_default();

        self.active_tokens
            .write()
            .map_err(|_| {
                CommandError::AuthenticationFailed(
                    "Internal error: token storage corrupted".to_string(),
                )
            })?
            .insert(
                token.clone(),
                AuthUser {
                    username: username.to_string(),
                    roles,
                },
            );

        Ok(token)
    }

    fn authenticate(&self, auth: &Auth) -> CommandResult<Option<AuthUser>> {
        match auth {
            Auth::User(username, password) => {
                if let Some(stored_password) = self.authorized_users.read().unwrap().get(username) {
                    if password != stored_password {
                        return Err(CommandError::AuthenticationFailed(format!(
                            "Invalid password for user '{username}'"
                        )));
                    }

                    let roles = self
                        .user_roles
                        .read()
                        .unwrap()
                        .get(username)
                        .cloned()
                        .unwrap_or_default();

                    Ok(Some(AuthUser {
                        username: username.clone(),
                        roles,
                    }))
                } else {
                    Err(CommandError::AuthenticationFailed(format!(
                        "User '{username}' not found"
                    )))
                }
            }
            Auth::Token(token) => self.active_tokens.read().unwrap().get(token).map_or_else(
                || {
                    Err(CommandError::AuthenticationFailed(
                        "Invalid or expired token".to_string(),
                    ))
                },
                |user| Ok(Some(user.clone())),
            ),
            Auth::ApiKey(key) => {
                if key == "squirrel-api-key" {
                    Ok(Some(AuthUser {
                        username: "api".to_string(),
                        roles: vec![UserRole::PowerUser],
                    }))
                } else {
                    Err(CommandError::AuthenticationFailed(
                        "Invalid API key".to_string(),
                    ))
                }
            }
            Auth::None => Ok(None),
        }
    }

    fn authorize(&self, command: &str, user: Option<&AuthUser>) -> CommandResult<()> {
        let permissions = self.command_permissions.read().unwrap();

        if let Some(required_roles) = permissions.get(command) {
            match user {
                Some(user) => {
                    if user.roles.contains(&UserRole::Admin) {
                        return Ok(());
                    }

                    if user.roles.iter().any(|role| required_roles.contains(role)) {
                        return Ok(());
                    }

                    Err(CommandError::AuthorizationFailed(format!(
                        "User '{}' does not have required role for command '{command}'",
                        user.username
                    )))
                }
                None => Err(CommandError::AuthorizationFailed(format!(
                    "Authentication required for command '{command}'"
                ))),
            }
        } else {
            Ok(())
        }
    }

    fn log_command(
        &self,
        command: &str,
        args: &[String],
        user: Option<&AuthUser>,
        success: bool,
        message: String,
    ) {
        let mut log = self.command_log.write().unwrap();
        log.push(CommandLogEntry {
            command: command.to_string(),
            args: args.to_vec(),
            user: user.map(|u| u.username.clone()),
            timestamp: std::time::SystemTime::now(),
            success,
            message,
        });
    }

    /// Get command execution logs
    #[must_use]
    pub fn get_command_logs(&self) -> Vec<CommandLogEntry> {
        self.command_log.read().unwrap().clone()
    }

    /// Get formatted command logs for display
    #[must_use]
    pub fn get_formatted_command_logs(&self) -> Vec<String> {
        self.command_log
            .read()
            .unwrap()
            .iter()
            .map(|entry| {
                let timestamp = entry
                    .timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let user = entry.user.as_deref().unwrap_or("anonymous");
                let status = if entry.success { "SUCCESS" } else { "FAILED" };
                let args = entry.args.join(" ");
                format!(
                    "[{}] {} {} {} ({}) - {}",
                    timestamp, status, entry.command, args, user, entry.message
                )
            })
            .collect()
    }

    /// Register a command in the adapter
    #[allow(clippy::unused_async)]
    pub async fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let cmd_name = command.name();

        if cmd_name.starts_with("admin")
            && !self
                .command_permissions
                .read()
                .unwrap()
                .contains_key(cmd_name)
        {
            let mut permissions = self.command_permissions.write().unwrap();
            permissions.insert(cmd_name.to_string(), vec![UserRole::Admin]);
        }

        let mut adapter = self.adapter.write().unwrap();
        adapter.register(cmd_name, command.clone())
    }

    /// Execute a command with authentication and authorization
    #[allow(clippy::unused_async)]
    pub async fn execute_with_auth(
        &self,
        command: &str,
        args: Vec<String>,
        auth: Auth,
    ) -> CommandResult<String> {
        let user = self.authenticate(&auth)?;
        self.authorize(command, user.as_ref())?;

        let result = {
            let adapter = self.adapter.read().unwrap();
            adapter.execute(command, args.clone())
        };

        match &result {
            Ok(output) => {
                self.log_command(command, &args, user.as_ref(), true, output.clone());
            }
            Err(e) => {
                self.log_command(command, &args, user.as_ref(), false, e.to_string());
            }
        }

        result
    }

    /// Get list of commands available to the authenticated user
    #[allow(clippy::unused_async)]
    pub async fn get_available_commands(&self, auth: Auth) -> CommandResult<Vec<String>> {
        let user = self.authenticate(&auth).ok().flatten();
        let mut commands = self.adapter.read().unwrap().list_commands()?;

        match user {
            Some(user) => {
                if user.roles.contains(&UserRole::Admin) {
                    return Ok(commands);
                }
                commands.retain(|cmd| {
                    if let Some(required_roles) = self.command_permissions.read().unwrap().get(cmd)
                    {
                        user.roles.iter().any(|role| required_roles.contains(role))
                    } else {
                        true
                    }
                });
            }
            None => {
                commands.retain(|cmd| !self.command_permissions.read().unwrap().contains_key(cmd));
            }
        }

        Ok(commands)
    }
}

#[async_trait]
impl CommandAdapter for McpAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        self.execute_with_auth(command, args, Auth::None).await
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        let adapter = self.adapter.read().unwrap();
        adapter.get_help(command)
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        let adapter = self.adapter.read().unwrap();
        adapter.list_commands()
    }
}
