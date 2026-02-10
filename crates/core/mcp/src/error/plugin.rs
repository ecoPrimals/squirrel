// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Errors related to plugin management and execution.

use thiserror::Error;
use uuid::Uuid;

/// Errors related to MCP plugin operations
#[derive(Error, Debug, Clone)]
pub enum PluginError {
    /// Plugin discovery failed
    #[error("Plugin discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Plugin loading failed
    #[error("Plugin loading failed for {plugin_id}: {reason}")]
    LoadingFailed { plugin_id: Uuid, reason: String },

    /// Plugin initialization failed
    #[error("Plugin initialization failed for {plugin_id}: {reason}")]
    InitializationFailed { plugin_id: Uuid, reason: String },

    /// Plugin execution failed
    #[error("Plugin execution failed for {plugin_id}: {reason}")]
    ExecutionFailed { plugin_id: Uuid, reason: String },

    /// Plugin with the given ID was not found
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Plugin is already registered
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    /// Feature is not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Plugin configuration is invalid
    #[error("Invalid plugin configuration for {plugin_id}: {reason}")]
    InvalidConfiguration { plugin_id: Uuid, reason: String },

    /// Error during plugin lifecycle management
    #[error("Plugin lifecycle error for {plugin_id}: {reason}")]
    LifecycleError { plugin_id: Uuid, reason: String },

    /// Dependency resolution failed for a plugin
    #[error("Plugin dependency resolution failed for {plugin_id}: {reason}")]
    DependencyError { plugin_id: Uuid, reason: String },

    /// Generic internal plugin error
    #[error("Internal plugin error: {0}")]
    InternalError(String),
}
