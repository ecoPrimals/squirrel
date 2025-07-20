//! Plugin registry traits and implementations
//!
//! This module provides the core registry functionality for plugin management.

use crate::errors::{PluginError, Result};
use crate::types::PluginStatus;
use crate::Plugin;
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Plugin registry trait
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()>;

    /// Unregister a plugin
    async fn unregister_plugin(&self, id: Uuid) -> Result<()>;

    /// Get a plugin by ID
    async fn get_plugin(&self, id: Uuid) -> Result<Arc<dyn Plugin>>;

    /// Get a plugin by name
    async fn get_plugin_by_name(&self, name: &str) -> Result<Arc<dyn Plugin>>;

    /// List all plugins
    async fn list_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;

    /// Get plugin status
    async fn get_plugin_status(&self, id: Uuid) -> Result<PluginStatus>;

    /// Set plugin status
    async fn set_plugin_status(&self, id: Uuid, status: PluginStatus) -> Result<()>;

    /// Get all registered plugins
    async fn get_all_plugins(&self) -> Result<Vec<Arc<dyn Plugin>>>;
}
