// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

/// Represents the source of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginSource {
    /// Local filesystem path
    Local(PathBuf),
    /// Remote URL
    Remote(String),
    /// Embedded in the application
    Embedded(String),
}

/// Represents a step in the plugin lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginLifecycleStep {
    /// Discovery stage - plugin is found
    Discovered,
    /// Registration stage - plugin is registered with the system
    Registered,
    /// Initialization stage - plugin is initialized
    Initialized,
    /// Configuration stage - plugin is configured
    Configured,
    /// Activation stage - plugin is activated
    Activated,
    /// Deactivation stage - plugin is deactivated
    Deactivated,
    /// Unregistration stage - plugin is unregistered
    Unregistered,
}

/// Plugin version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginVersion {
    /// Major version
    pub major: u32,
    /// Minor version
    pub minor: u32,
    /// Patch version
    pub patch: u32,
}

/// Plugin ID type
pub type PluginId = String;

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: PluginId,
    /// Plugin name
    pub name: String,
    /// Plugin description
    pub description: String,
    /// Plugin version
    pub version: PluginVersion,
    /// Plugin author
    pub author: String,
    /// Plugin source
    pub source: PluginSource,
    /// Plugin dependencies
    pub dependencies: Vec<PluginId>,
    /// Additional metadata as key-value pairs
    pub additional_metadata: HashMap<String, String>,
} 