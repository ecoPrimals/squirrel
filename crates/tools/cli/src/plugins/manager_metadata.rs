// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Default metadata loading for plugin directories (`plugin.toml` merge).

use std::path::Path;

use crate::plugins::error::PluginError;
use crate::plugins::manifest;
use crate::plugins::plugin::PluginMetadata;
use tracing::warn;

/// Merge `plugin.toml` from `path` into defaults for `name`, or use defaults if missing.
pub(super) fn plugin_metadata_from_dir(
    name: &str,
    path: &Path,
) -> Result<PluginMetadata, PluginError> {
    let metadata_file = path.join("plugin.toml");

    let mut metadata = PluginMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some(format!("Plugin loaded from {}", path.display())),
        author: Some("Unknown".to_string()),
        homepage: None,
        capabilities: vec![],
    };

    if metadata_file.exists() {
        let content = std::fs::read_to_string(&metadata_file).map_err(PluginError::IoError)?;
        let merged = manifest::parse_plugin_manifest(&content)?;
        if let Some(n) = merged.name {
            metadata.name = n;
        }
        if let Some(v) = merged.version {
            metadata.version = v;
        }
        if merged.description.is_some() {
            metadata.description = merged.description;
        }
        if merged.author.is_some() {
            metadata.author = merged.author;
        }
        if merged.homepage.is_some() {
            metadata.homepage = merged.homepage;
        }
        if !merged.capabilities.is_empty() {
            metadata.capabilities = merged.capabilities;
        }
    } else {
        warn!(
            "⚠️ No metadata file found for plugin {}, using defaults",
            name
        );
    }

    Ok(metadata)
}
