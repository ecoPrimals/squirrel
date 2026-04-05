// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin manager trait definitions
//!
//! This module defines the core traits that plugin managers must implement.

use crate::Plugin;
use crate::errors::Result;
use crate::registry::PluginRegistry;
use crate::types::PluginStatus;
use std::sync::Arc;
use uuid::Uuid;

/// Plugin manager trait
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait PluginManagerTrait: PluginRegistry + Send + Sync {
    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;

    /// Initialize a plugin
    async fn initialize_plugin(&self, id: Uuid) -> Result<()>;

    /// Shutdown a plugin
    async fn shutdown_plugin(&self, id: Uuid) -> Result<()>;

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;

    /// Load plugins from a directory
    async fn load_plugins(&self, directory: &str) -> Result<Vec<Uuid>>;

    /// Initialize all registered plugins
    async fn initialize_all_plugins(&self) -> Result<()>;

    /// Shutdown all plugins
    async fn shutdown_all_plugins(&self) -> Result<()>;
}
