// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin system for the Squirrel CLI
//!
//! This module implements a plugin system that allows extending the
//! CLI functionality with custom commands and features.

/// Plugin discovery implementations.
pub mod discovery;
/// Plugin error types.
pub mod error;
/// Example built-in plugin for testing and reference.
pub mod example_plugin;
/// Plugin lifecycle and registration.
pub mod manager;
/// TOML manifest parsing (`plugin.toml`).
pub(crate) mod manifest;
#[cfg(test)]
mod mod_tests;
/// Plugin trait and metadata types.
pub mod plugin;
/// Secure plugin loading and validation.
pub mod security;
/// Plugin manager singleton and shared state.
pub mod state;
#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::plugins::plugin::{PluginMetadata, PluginStatus};

pub use discovery::{DefaultPluginDiscovery, PluginDiscovery};
pub use error::PluginError;
pub use example_plugin::ExamplePlugin;
pub use manager::PluginManager;
pub use plugin::Plugin;
pub use security::{PluginSecurityError, SecurePluginLoader, ValidationResult};
use tracing::{debug, error, info, warn}; // Export security types

/// Default plugin directory relative to user's home directory
const DEFAULT_PLUGIN_DIR: &str = ".squirrel/plugins";

/// Get the plugin directories to search for plugins
///
/// This function returns a list of directories to search for plugins:
/// 1. The directory specified by the SQUIRREL_PLUGIN_PATH environment variable, if set
/// 2. The user's $HOME/.squirrel/plugins directory
/// 3. The current directory's plugins subdirectory, if it exists
///
/// # Returns
///
/// A vector of paths to search for plugins
fn get_plugin_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Check for SQUIRREL_PLUGIN_PATH environment variable
    if let Ok(plugin_path) = env::var("SQUIRREL_PLUGIN_PATH") {
        for path in plugin_path.split(':') {
            dirs.push(PathBuf::from(path));
        }
    }

    // Add user's home directory plugin path
    if let Ok(home_dir) = env::var("HOME") {
        let user_plugin_dir = Path::new(&home_dir).join(DEFAULT_PLUGIN_DIR);
        dirs.push(user_plugin_dir);
    }

    // Add current directory's plugins subdirectory if it exists
    let current_plugin_dir = PathBuf::from("plugins");
    if current_plugin_dir.exists() && current_plugin_dir.is_dir() {
        dirs.push(current_plugin_dir);
    }

    dirs
}

/// Discover plugins in the given directory
///
/// This function searches for plugin metadata files in the given directory
/// and loads them into the plugin manager.
///
/// # Arguments
///
/// * `dir` - The directory to search for plugins
/// * `plugin_manager` - The plugin manager to add discovered plugins to
///
/// # Returns
///
/// The number of plugins discovered and loaded
fn discover_plugins_in_directory(
    dir: &Path,
    plugin_manager: &mut PluginManager,
) -> Result<usize, PluginError> {
    debug!("Discovering plugins in directory: {:?}", dir);

    if !dir.exists() || !dir.is_dir() {
        debug!(
            "Plugin directory does not exist or is not a directory: {:?}",
            dir
        );
        return Ok(0);
    }

    let mut count = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip non-directories
        if !path.is_dir() {
            continue;
        }

        // Check for plugin metadata file
        let metadata_path = path.join("plugin.toml");
        if !metadata_path.exists() {
            continue;
        }

        // Read metadata file
        match fs::read_to_string(&metadata_path) {
            Ok(content) => {
                match parse_plugin_metadata(&content, &path) {
                    Ok(metadata) => {
                        // Add plugin to manager
                        match plugin_manager.add_plugin(
                            metadata,
                            path.clone(),
                            PluginStatus::Installed,
                        ) {
                            Ok(plugin) => {
                                info!(
                                    "Discovered plugin: {} v{}",
                                    plugin.metadata().name,
                                    plugin.metadata().version
                                );
                                count += 1;
                            }
                            Err(err) => {
                                warn!("Failed to add plugin from {:?}: {}", path, err);
                            }
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Failed to parse plugin metadata from {:?}: {}",
                            metadata_path, err
                        );
                    }
                }
            }
            Err(err) => {
                warn!(
                    "Failed to read plugin metadata from {:?}: {}",
                    metadata_path, err
                );
            }
        }
    }

    Ok(count)
}

/// Parse plugin metadata from TOML content
///
/// # Arguments
///
/// * `content` - The TOML content to parse
/// * `plugin_path` - The path to the plugin directory
///
/// # Returns
///
/// The parsed plugin metadata
fn parse_plugin_metadata(
    content: &str,
    _plugin_path: &Path,
) -> Result<PluginMetadata, PluginError> {
    let merged = manifest::parse_plugin_manifest(content)?;

    let name = merged
        .name
        .ok_or_else(|| PluginError::ValidationError("Plugin name is required".to_string()))?;
    let version = merged
        .version
        .ok_or_else(|| PluginError::ValidationError("Plugin version is required".to_string()))?;

    Ok(PluginMetadata {
        name,
        version,
        description: merged.description,
        author: merged.author,
        homepage: merged.homepage,
        capabilities: merged.capabilities,
    })
}

/// Initialize the plugin system
///
/// This function is called at application startup to load installed plugins.
/// It discovers plugins in the configured directories and registers them with
/// the plugin manager.
///
/// # Returns
///
/// Ok(()) if successful, or an error if the plugin system could not be initialized
pub async fn initialize_plugins() -> Result<(), error::PluginError> {
    info!("Initializing plugin system");

    // Get the plugin manager singleton
    let plugin_manager_arc = state::get_plugin_manager();
    let mut plugin_manager = plugin_manager_arc.lock().await;

    // Get plugin directories
    let plugin_dirs = get_plugin_directories();

    // Discover plugins in each directory
    let mut total_count = 0;
    for dir in plugin_dirs {
        match discover_plugins_in_directory(&dir, &mut plugin_manager) {
            Ok(count) => {
                total_count += count;
            }
            Err(err) => {
                error!("Error discovering plugins in {:?}: {}", dir, err);
            }
        }
    }

    // Register built-in plugins
    match register_builtin_plugins(&mut plugin_manager) {
        Ok(count) => {
            info!("Registered {} built-in plugins", count);
            total_count += count;
        }
        Err(err) => {
            error!("Error registering built-in plugins: {}", err);
        }
    }

    info!("Plugin system initialized with {} plugins", total_count);
    Ok(())
}

/// Register built-in plugins that come with the CLI
///
/// # Arguments
///
/// * `plugin_manager` - The plugin manager to register built-in plugins with
///
/// # Returns
///
/// The number of built-in plugins registered
fn register_builtin_plugins(plugin_manager: &mut PluginManager) -> Result<usize, PluginError> {
    info!("Registering built-in plugins");

    let mut count = 0;

    // Register example plugin
    let example_factory = example_plugin::ExamplePluginFactory;
    match plugin_manager.register_plugin_factory("example", Arc::new(example_factory)) {
        Ok(_) => {
            info!("Registered built-in example plugin");
            count += 1;
        }
        Err(err) => {
            warn!("Failed to register built-in example plugin: {}", err);
        }
    }

    // Add additional built-in plugins here

    Ok(count)
}
