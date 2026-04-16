// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core CLI configuration types and [`CliConfig`] load/save helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, warn};

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Error reading configuration file
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] io::Error),

    /// Error parsing configuration
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Error serializing configuration
    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    /// Error with configuration path
    #[error("Config path error: {0}")]
    PathError(String),

    /// Configuration key not found
    #[error("Config key not found: {0}")]
    KeyNotFound(String),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// CLI Configuration
///
/// Represents the configuration settings for the Squirrel CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Default output format (text, json, yaml)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// MCP server host
    #[serde(default = "default_mcp_host")]
    pub mcp_host: String,

    /// MCP server port
    #[serde(default = "default_mcp_port")]
    pub mcp_port: u16,

    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Enable quiet mode
    #[serde(default)]
    pub quiet: bool,

    /// Additional custom settings
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

// Default value functions for CliConfig
fn default_log_level() -> String {
    "info".to_string()
}

fn default_output_format() -> String {
    std::env::var("CLI_OUTPUT_FORMAT").unwrap_or_else(|_| "text".to_string())
}

fn default_mcp_host() -> String {
    std::env::var("CLI_MCP_HOST")
        .unwrap_or_else(|_| crate::mcp::config::DEFAULT_DEV_HOST.to_string())
}

fn default_mcp_port() -> u16 {
    std::env::var("CLI_MCP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(9000)
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            output_format: default_output_format(),
            mcp_host: default_mcp_host(),
            mcp_port: default_mcp_port(),
            verbose: false,
            quiet: false,
            custom: HashMap::new(),
        }
    }
}

impl CliConfig {
    /// Create a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    ///
    /// A Result containing the loaded configuration or an error
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> ConfigResult<Self> {
        debug!("Loading config from file: {:?}", path.as_ref());

        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Self = toml::from_str(&contents)?;
        debug!("Config loaded successfully");

        Ok(config)
    }

    /// Save configuration to a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the configuration file
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> ConfigResult<()> {
        debug!("Saving config to file: {:?}", path.as_ref());

        // Create parent directories if they don't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        let mut file = File::create(&path)?;
        file.write_all(contents.as_bytes())?;

        debug!("Config saved successfully");
        Ok(())
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
        debug!("Getting config value for key: {}", key);

        match key {
            "log_level" => Ok(self.log_level.clone()),
            "output_format" => Ok(self.output_format.clone()),
            "mcp_host" => Ok(self.mcp_host.clone()),
            "mcp_port" => Ok(self.mcp_port.to_string()),
            "verbose" => Ok(self.verbose.to_string()),
            "quiet" => Ok(self.quiet.to_string()),
            _ => {
                // Check custom settings
                if let Some(value) = self.custom.get(key) {
                    Ok(value.clone())
                } else {
                    Err(ConfigError::KeyNotFound(key.to_string()))
                }
            }
        }
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
        debug!("Setting config value for key: {} = {}", key, value);

        match key {
            "log_level" => {
                self.log_level = value;
            }
            "output_format" => {
                self.output_format = value;
            }
            "mcp_host" => {
                self.mcp_host = value;
            }
            "mcp_port" => {
                self.mcp_port = value.parse::<u16>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid port number: {}", value))
                })?;
            }
            "verbose" => {
                self.verbose = value.parse::<bool>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid boolean value: {}", value))
                })?;
            }
            "quiet" => {
                self.quiet = value.parse::<bool>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid boolean value: {}", value))
                })?;
            }
            _ => {
                // Store in custom settings
                self.custom.insert(key.to_string(), value);
            }
        }

        Ok(())
    }

    /// Merge another configuration into this one
    ///
    /// # Arguments
    ///
    /// * `other` - The other configuration to merge
    pub fn merge(&mut self, other: Self) {
        debug!("Merging configurations");

        if !other.log_level.is_empty() {
            self.log_level = other.log_level;
        }

        if !other.output_format.is_empty() {
            self.output_format = other.output_format;
        }

        if !other.mcp_host.is_empty() {
            self.mcp_host = other.mcp_host;
        }

        if other.mcp_port != 0 {
            self.mcp_port = other.mcp_port;
        }

        self.verbose = other.verbose;
        self.quiet = other.quiet;

        // Merge custom settings
        for (key, value) in other.custom {
            self.custom.insert(key, value);
        }
    }

    /// Load configuration from environment variables
    ///
    /// Environment variables should be prefixed with the specified prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Prefix for environment variables to include
    ///
    /// # Returns
    ///
    /// A Result containing the loaded configuration or an error
    pub fn from_env(prefix: &str) -> ConfigResult<Self> {
        debug!("Loading config from environment with prefix: {}", prefix);

        let mut config = Self::default();

        // Process environment variables
        for (key, value) in std::env::vars() {
            if let Some(suffix) = key.strip_prefix(prefix) {
                let config_key = suffix.to_lowercase();
                debug!("Found env var: {} = {}", config_key, value);

                // Set configuration value based on the key
                if let Err(e) = config.set(&config_key, value) {
                    warn!("Failed to set config value from env: {}", e);
                }
            }
        }

        Ok(config)
    }
}
