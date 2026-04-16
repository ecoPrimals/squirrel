// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration management for the Squirrel CLI
//!
//! This module provides functionality for loading, saving, and managing
//! CLI configuration settings.

#[path = "config_types.rs"]
mod config_types;

pub use config_types::{CliConfig, ConfigError, ConfigResult};

use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Configuration manager for the CLI
///
/// Manages configuration loading, saving, and access.
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Current configuration
    config: CliConfig,

    /// Path to the configuration file
    config_path: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new configuration manager with default settings
    pub fn new() -> Self {
        Self {
            config: CliConfig::default(),
            config_path: None,
        }
    }

    /// Create a configuration manager with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use
    pub fn with_config(config: CliConfig) -> Self {
        Self {
            config,
            config_path: None,
        }
    }

    /// Load configuration from standard locations
    ///
    /// Looks for configuration files in the following order:
    /// 1. Path specified in `config_path` if provided
    /// 2. `.squirrel.toml` in the current directory
    /// 3. `squirrel.toml` in the user's configuration directory
    /// 4. `/etc/squirrel/squirrel.toml` on Unix-like systems
    ///
    /// # Arguments
    ///
    /// * `config_path` - Optional path to a specific configuration file
    ///
    /// # Returns
    ///
    /// A Result containing the ConfigManager or an error
    pub fn load(config_path: Option<PathBuf>) -> ConfigResult<Self> {
        debug!("Loading configuration");

        let mut config = CliConfig::default();
        let mut found_path = None;

        // 1. Try specified path
        if let Some(path) = &config_path {
            debug!("Trying specified config path: {:?}", path);
            if path.exists() {
                config = CliConfig::load_from_file(path)?;
                found_path = Some(path.clone());
                debug!("Loaded configuration from specified path");
            } else {
                warn!("Specified config file not found: {:?}", path);
            }
        }

        // 2. Try current directory
        if found_path.is_none() {
            let current_dir_path = PathBuf::from(".squirrel.toml");
            debug!("Trying current directory: {:?}", current_dir_path);
            if current_dir_path.exists() {
                config = CliConfig::load_from_file(&current_dir_path)?;
                found_path = Some(current_dir_path);
                debug!("Loaded configuration from current directory");
            }
        }

        // 3. Try user config directory
        if found_path.is_none()
            && let Some(proj_dirs) = directories::ProjectDirs::from("", "", "squirrel")
        {
            let user_config_path = proj_dirs.config_dir().join("squirrel.toml");
            debug!("Trying user config directory: {:?}", user_config_path);
            if user_config_path.exists() {
                config = CliConfig::load_from_file(&user_config_path)?;
                found_path = Some(user_config_path);
                debug!("Loaded configuration from user config directory");
            }
        }

        // 4. Try system config directory on Unix
        #[cfg(unix)]
        if found_path.is_none() {
            let system_config_path = PathBuf::from("/etc/squirrel/squirrel.toml");
            debug!("Trying system config directory: {:?}", system_config_path);
            if system_config_path.exists() {
                config = CliConfig::load_from_file(&system_config_path)?;
                found_path = Some(system_config_path);
                debug!("Loaded configuration from system config directory");
            }
        }

        // 5. Apply environment variable overrides
        let env_config = CliConfig::from_env("SQUIRREL_")?;
        config.merge(env_config);

        if found_path.is_none() && config_path.is_some() {
            // If a path was specified but not found, use it for future saves
            found_path = config_path;
            debug!("No existing config found, will create new one at specified path");
        }

        Ok(Self {
            config,
            config_path: found_path,
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &CliConfig {
        &self.config
    }

    /// Get a mutable reference to the current configuration
    pub fn config_mut(&mut self) -> &mut CliConfig {
        &mut self.config
    }

    /// Get the path to the configuration file
    pub fn config_path(&self) -> &Option<PathBuf> {
        &self.config_path
    }

    /// Get a configuration value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to get
    ///
    /// # Returns
    ///
    /// A Result containing the configuration value or an error
    pub fn get(&self, key: &str) -> ConfigResult<String> {
        self.config.get(key)
    }

    /// Set a configuration value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key to set
    /// * `value` - The value to set
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn set(&mut self, key: &str, value: String) -> ConfigResult<()> {
        self.config.set(key, value)
    }

    /// List all configuration keys and values
    ///
    /// # Returns
    ///
    /// A HashMap containing all configuration keys and values
    pub fn list(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Add standard fields
        result.insert("log_level".to_string(), self.config.log_level.clone());
        result.insert(
            "output_format".to_string(),
            self.config.output_format.clone(),
        );
        result.insert("mcp_host".to_string(), self.config.mcp_host.clone());
        result.insert("mcp_port".to_string(), self.config.mcp_port.to_string());
        result.insert("verbose".to_string(), self.config.verbose.to_string());
        result.insert("quiet".to_string(), self.config.quiet.to_string());

        // Add custom fields
        for (key, value) in &self.config.custom {
            result.insert(key.clone(), value.clone());
        }

        result
    }

    /// Save the current configuration
    ///
    /// # Arguments
    ///
    /// * `path` - Optional path to save to. If None, uses the path from which the config was loaded.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn save(&self, path: Option<PathBuf>) -> ConfigResult<()> {
        let save_path = if let Some(path) = path {
            path
        } else if let Some(path) = &self.config_path {
            path.clone()
        } else {
            return Err(ConfigError::PathError(
                "No config path specified".to_string(),
            ));
        };

        self.config.save_to_file(save_path)
    }

    /// Import configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file to import
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn import(&mut self, path: PathBuf) -> ConfigResult<()> {
        debug!("Importing configuration from: {:?}", path);
        let imported_config = CliConfig::load_from_file(path)?;
        self.config.merge(imported_config);
        Ok(())
    }

    /// Export configuration to a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to export the configuration to
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn export(&self, path: PathBuf) -> ConfigResult<()> {
        self.config.save_to_file(path)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
