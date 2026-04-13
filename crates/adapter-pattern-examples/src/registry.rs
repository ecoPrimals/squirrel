// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Command registry and the base [`CommandAdapter`] trait + [`RegistryAdapter`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::command::DynCommand;
use crate::{CommandError, CommandResult};

/// Command registry to store and execute commands.
#[derive(Debug)]
pub struct CommandRegistry {
    pub(crate) commands: HashMap<String, Arc<dyn DynCommand>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Create a new command registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command in the registry.
    ///
    /// # Errors
    ///
    /// Never returns an error; kept for API consistency.
    pub fn register(&mut self, command: Arc<dyn DynCommand>) -> CommandResult<()> {
        let name = command.name().to_string();
        self.commands.insert(name, command);
        Ok(())
    }

    /// Execute a command by name.
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

    /// Get help text for a command.
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

    /// List all available commands.
    #[must_use]
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }
}

/// Adapter interface for command operations.
///
/// Uses `impl Future + Send` for the same reasons as [`Command`](crate::command::Command).
pub trait CommandAdapter: Send + Sync {
    /// Execute a command with the given arguments.
    fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_;

    /// Get help text for a command.
    fn get_help(
        &self,
        command: &str,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_;

    /// List all available commands.
    fn list_commands(
        &self,
    ) -> impl std::future::Future<Output = CommandResult<Vec<String>>> + Send + '_;
}

/// Registry adapter implementation.
#[derive(Debug, Clone)]
pub struct RegistryAdapter {
    pub(crate) registry: Arc<Mutex<CommandRegistry>>,
}

impl Default for RegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryAdapter {
    /// Create a new registry adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }

    /// Register a command in the registry.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&self, command: Arc<dyn DynCommand>) -> CommandResult<()> {
        let mut registry = self
            .registry
            .lock()
            .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
        registry.register(command)
    }
}

impl CommandAdapter for RegistryAdapter {
    fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let registry = self.registry.clone();
        let command = command.to_string();
        async move {
            let registry_clone = {
                let reg = registry
                    .lock()
                    .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
                reg.list_commands()
                    .into_iter()
                    .filter_map(|name| reg.commands.get(&name).map(|cmd| (name, cmd.clone())))
                    .collect::<HashMap<String, Arc<dyn DynCommand>>>()
            };

            match registry_clone.get(&command) {
                Some(cmd) => cmd.execute(args).await,
                None => Err(CommandError::NotFound(command)),
            }
        }
    }

    fn get_help(
        &self,
        command: &str,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let registry = self.registry.clone();
        let command = command.to_string();
        async move {
            let reg = registry
                .lock()
                .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
            reg.get_help(&command)
        }
    }

    fn list_commands(
        &self,
    ) -> impl std::future::Future<Output = CommandResult<Vec<String>>> + Send + '_ {
        let registry = self.registry.clone();
        async move {
            let reg = registry
                .lock()
                .map_err(|e| CommandError::Internal(format!("Lock error: {e}")))?;
            Ok(reg.list_commands())
        }
    }
}
