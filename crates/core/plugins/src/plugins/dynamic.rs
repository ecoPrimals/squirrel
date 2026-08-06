// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Dynamic plugin utilities and types
//!
//! This module contains types for dynamic plugin loading and metadata handling.

use serde::{Deserialize, Serialize};

/// Metadata for a plugin dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Optional ID of the dependency (can be null for optional dependencies)
    pub id: Option<String>,

    /// Name of the dependency
    pub name: String,

    /// Version requirement (semver string)
    pub version: String,
}

/// Metadata for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: String,

    /// Name of the plugin
    pub name: String,

    /// Version of the plugin
    pub version: String,

    /// API version this plugin is compatible with
    pub api_version: String,

    /// Description of the plugin functionality
    pub description: String,

    /// Author of the plugin
    pub author: String,

    /// Dependencies required by this plugin
    pub dependencies: Vec<PluginDependency>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        api_version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            api_version: api_version.into(),
            description: description.into(),
            author: author.into(),
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to the plugin metadata
    pub fn with_dependency(
        mut self,
        id: Option<impl Into<String>>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.dependencies.push(PluginDependency {
            id: id.map(Into::into),
            name: name.into(),
            version: version.into(),
        });
        self
    }
}
