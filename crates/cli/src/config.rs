//! Configuration management for the Squirrel CLI
//!
//! This module provides functionality for loading, saving, and managing
//! CLI configuration settings.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log::{debug, warn, error};

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
    "text".to_string()
}

fn default_mcp_host() -> String {
    "localhost".to_string()
}

fn default_mcp_port() -> u16 {
    9000
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
            },
            "output_format" => {
                self.output_format = value;
            },
            "mcp_host" => {
                self.mcp_host = value;
            },
            "mcp_port" => {
                self.mcp_port = value.parse::<u16>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid port number: {}", value))
                })?;
            },
            "verbose" => {
                self.verbose = value.parse::<bool>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid boolean value: {}", value))
                })?;
            },
            "quiet" => {
                self.quiet = value.parse::<bool>().map_err(|_| {
                    ConfigError::PathError(format!("Invalid boolean value: {}", value))
                })?;
            },
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
        if found_path.is_none() {
            if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "squirrel") {
                let user_config_path = proj_dirs.config_dir().join("squirrel.toml");
                debug!("Trying user config directory: {:?}", user_config_path);
                if user_config_path.exists() {
                    config = CliConfig::load_from_file(&user_config_path)?;
                    found_path = Some(user_config_path);
                    debug!("Loaded configuration from user config directory");
                }
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
        result.insert("output_format".to_string(), self.config.output_format.clone());
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
                "No config path specified".to_string()
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
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::error::Error;
    
    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.log_level, "info");
        assert_eq!(config.output_format, "text");
        assert_eq!(config.mcp_host, "localhost");
        assert_eq!(config.mcp_port, 9000);
        assert!(!config.verbose);
        assert!(!config.quiet);
        assert!(config.custom.is_empty());
    }
    
    #[test]
    fn test_config_get_set() -> Result<(), Box<dyn Error>> {
        let mut config = CliConfig::default();
        
        // Test standard fields
        config.set("log_level", "debug".to_string())?;
        assert_eq!(config.get("log_level")?, "debug");
        
        config.set("output_format", "json".to_string())?;
        assert_eq!(config.get("output_format")?, "json");
        
        config.set("mcp_port", "9001".to_string())?;
        assert_eq!(config.get("mcp_port")?, "9001");
        
        // Test custom fields
        config.set("custom_key", "custom_value".to_string())?;
        assert_eq!(config.get("custom_key")?, "custom_value");
        
        Ok(())
    }
    
    #[test]
    fn test_config_save_load() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let config_path = dir.path().join("test_config.toml");
        
        // Create and save config
        let mut config = CliConfig::default();
        config.log_level = "debug".to_string();
        config.output_format = "json".to_string();
        config.set("custom_key", "custom_value".to_string())?;
        
        config.save_to_file(&config_path)?;
        
        // Load config
        let loaded_config = CliConfig::load_from_file(&config_path)?;
        
        // Verify loaded config
        assert_eq!(loaded_config.log_level, "debug");
        assert_eq!(loaded_config.output_format, "json");
        assert_eq!(loaded_config.get("custom_key")?, "custom_value");
        
        Ok(())
    }
    
    #[test]
    fn test_config_merge() {
        let mut config1 = CliConfig::default();
        let mut config2 = CliConfig::default();
        
        // Set different values in config1
        config1.log_level = "info".to_string();
        
        // Set different values in config2
        config2.output_format = "json".to_string();
        config2.mcp_port = 9001;
        
        // Add custom values
        config1.custom.insert("key1".to_string(), "value1".to_string());
        config2.custom.insert("key2".to_string(), "value2".to_string());
        
        // Merge configs
        config1.merge(config2);
        
        // Verify merged config
        assert_eq!(config1.log_level, "info");
        assert_eq!(config1.output_format, "json");
        assert_eq!(config1.mcp_port, 9001);
        assert_eq!(config1.custom.get("key1").unwrap(), "value1");
        assert_eq!(config1.custom.get("key2").unwrap(), "value2");
    }
} 