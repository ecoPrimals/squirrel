// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Universal pattern mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin adapter — wraps a [`RegistryAdapter`] with plugin metadata.

use std::sync::Arc;

use crate::CommandResult;
use crate::command::DynCommand;
use crate::registry::{CommandAdapter, RegistryAdapter};

/// Plugin metadata information.
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin ID.
    pub id: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: String,
}

/// Plugin adapter implementation.
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
    /// Create a new plugin adapter.
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

    /// Get plugin ID.
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        &self.metadata.id
    }

    /// Get plugin version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.metadata.version
    }

    /// Get plugin description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    /// Register a command.
    ///
    /// # Errors
    ///
    /// Returns `CommandError::Internal` if the mutex is poisoned.
    pub fn register_command(&self, command: Arc<dyn DynCommand>) -> CommandResult<()> {
        self.adapter.register_command(command)
    }

    /// Get available commands.
    ///
    /// # Errors
    ///
    /// Never returns an error; kept for API consistency.
    pub async fn get_commands(&self) -> CommandResult<Vec<String>> {
        self.adapter.list_commands().await
    }
}

impl CommandAdapter for PluginAdapter {
    fn execute_command(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> impl std::future::Future<Output = CommandResult<String>> + Send + '_ {
        let adapter = self.adapter.clone();
        let command = command.to_string();
        async move { adapter.execute_command(&command, args).await }
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
        let adapter = self.adapter.clone();
        async move { adapter.list_commands().await }
    }
}
