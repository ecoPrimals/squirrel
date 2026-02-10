// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Errors related to tool management and execution.

use crate::error::plugin::PluginError;
use thiserror::Error; // Import PluginError

/// Specific errors related to tool operations
#[derive(Error, Debug, Clone)]
pub enum ToolError {
    /// Tool registration failed
    #[error("Tool registration failed: {0}")]
    RegistrationFailed(String),

    /// Tool with the given ID was not found
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    /// Tool configuration is invalid
    #[error("Invalid tool configuration: {0}")]
    InvalidConfiguration(String),

    /// Error during tool lifecycle management
    #[error("Tool lifecycle error: {0}")]
    LifecycleError(String),

    /// Error originating from the underlying plugin system
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    /// Generic internal tool error
    #[error("Internal tool error: {0}")]
    InternalError(String),
}
