// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! TOML parsing for `plugin.toml` CLI plugin manifests.

use serde::Deserialize;

use super::error::PluginError;

/// Fields merged from a `plugin.toml` after parsing (all optional except where callers validate).
#[derive(Debug, Default)]
pub struct MergedPluginManifest {
    /// Plugin name
    pub name: Option<String>,
    /// Semantic version string
    pub version: Option<String>,
    /// Short description
    pub description: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Project homepage URL
    pub homepage: Option<String>,
    /// Declared capability identifiers
    pub capabilities: Vec<String>,
}

#[derive(Deserialize)]
struct FlatToml {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    homepage: Option<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Deserialize)]
struct NestedToml {
    plugin: FlatToml,
}

/// Parse `plugin.toml` content into a merged manifest (flat table or `[plugin]` section).
///
/// # Errors
///
/// Returns [`PluginError::ValidationError`] if the TOML is syntactically invalid or has no
/// recognizable shape.
pub fn parse_plugin_manifest(content: &str) -> Result<MergedPluginManifest, PluginError> {
    if let Ok(nested) = toml::from_str::<NestedToml>(content) {
        return Ok(flat_to_merged(nested.plugin));
    }
    if let Ok(flat) = toml::from_str::<FlatToml>(content) {
        return Ok(flat_to_merged(flat));
    }
    Err(PluginError::ValidationError(
        "Invalid plugin.toml: could not parse TOML".to_string(),
    ))
}

fn flat_to_merged(f: FlatToml) -> MergedPluginManifest {
    MergedPluginManifest {
        name: f.name,
        version: f.version,
        description: f.description,
        author: f.author,
        homepage: f.homepage,
        capabilities: f.capabilities,
    }
}
