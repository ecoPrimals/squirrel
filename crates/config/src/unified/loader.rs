//! Configuration Loader with Hierarchical Precedence
//!
//! This module implements the configuration loading system with clear precedence rules.
//! Configuration is loaded from multiple sources with the following priority (highest first):
//!
//! 1. Command-line arguments (not yet implemented)
//! 2. Environment variables
//! 3. Configuration file (TOML/JSON/YAML)
//! 4. Platform-specific defaults
//! 5. Secure fallback defaults
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use squirrel_mcp_config::unified::ConfigLoader;
//!
//! // Load with full precedence hierarchy
//! let config = ConfigLoader::load()?;
//!
//! // Or use the builder pattern for custom loading
//! let config = ConfigLoader::new()
//!     .with_file_if_exists("custom-config.toml")?
//!     .with_env_prefix("SQUIRREL_")?
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::{SquirrelUnifiedConfig, TimeoutConfig};
use std::path::{Path, PathBuf};

/// Configuration loader with hierarchical precedence
///
/// Implements a builder pattern for loading configuration from multiple sources
/// with clear precedence rules.
pub struct ConfigLoader {
    config: SquirrelUnifiedConfig,
    sources_loaded: Vec<String>,
}

impl ConfigLoader {
    /// Create a new ConfigLoader with secure defaults
    pub fn new() -> Self {
        Self {
            config: SquirrelUnifiedConfig::default(),
            sources_loaded: vec!["secure_defaults".to_string()],
        }
    }

    /// Load configuration with full hierarchical precedence
    ///
    /// This is the primary entry point for loading configuration.
    /// It applies the following precedence (highest to lowest):
    ///
    /// 1. Environment variables
    /// 2. Configuration file (if exists)
    /// 3. Platform-specific defaults
    /// 4. Secure fallback defaults
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel_mcp_config::unified::ConfigLoader;
    ///
    /// let config = ConfigLoader::load()?;
    /// println!("Loaded from: {:?}", config.sources_loaded());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load() -> Result<LoadedConfig, ConfigError> {
        Self::new()
            .with_platform_detection()?
            .with_file_if_exists("squirrel.toml")?
            .with_file_if_exists("config/squirrel.toml")?
            .with_env_prefix("SQUIRREL_")?
            .validate()?
            .build_with_sources()
    }

    /// Apply platform-specific defaults
    ///
    /// Detects the current platform and applies appropriate defaults.
    pub fn with_platform_detection(mut self) -> Result<Self, ConfigError> {
        // Platform-specific adjustments
        #[cfg(target_os = "linux")]
        {
            self.config.system.data_dir = PathBuf::from("/var/lib/squirrel");
            self.config.system.plugin_dir = PathBuf::from("/usr/lib/squirrel/plugins");
            self.sources_loaded.push("platform_defaults_linux".to_string());
        }

        #[cfg(target_os = "macos")]
        {
            self.config.system.data_dir = PathBuf::from("/usr/local/var/squirrel");
            self.config.system.plugin_dir = PathBuf::from("/usr/local/lib/squirrel/plugins");
            self.sources_loaded.push("platform_defaults_macos".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            use std::env;
            let program_data = env::var("PROGRAMDATA")
                .unwrap_or_else(|_| "C:\\ProgramData".to_string());
            self.config.system.data_dir = PathBuf::from(format!("{}\\Squirrel\\data", program_data));
            self.config.system.plugin_dir = PathBuf::from(format!("{}\\Squirrel\\plugins", program_data));
            self.sources_loaded.push("platform_defaults_windows".to_string());
        }

        Ok(self)
    }

    /// Load configuration from a file if it exists
    ///
    /// Supports TOML, JSON, and YAML formats based on file extension.
    /// If the file doesn't exist, this is not an error - it simply continues.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel_mcp_config::unified::ConfigLoader;
    ///
    /// let config = ConfigLoader::new()
    ///     .with_file_if_exists("squirrel.toml")?
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_file_if_exists<P: AsRef<Path>>(mut self, path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(self);
        }

        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead {
                path: path.to_path_buf(),
                error: e.to_string(),
            })?;

        // Determine format from extension
        let format = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("toml");

        match format {
            "toml" => {
                let file_config: SquirrelUnifiedConfig = toml::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError {
                        format: "toml".to_string(),
                        error: e.to_string(),
                    })?;
                self.merge_config(file_config);
                self.sources_loaded.push(format!("file:{}", path.display()));
            }
            "json" => {
                let file_config: SquirrelUnifiedConfig = serde_json::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError {
                        format: "json".to_string(),
                        error: e.to_string(),
                    })?;
                self.merge_config(file_config);
                self.sources_loaded.push(format!("file:{}", path.display()));
            }
            "yaml" | "yml" => {
                let file_config: SquirrelUnifiedConfig = serde_yaml::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError {
                        format: "yaml".to_string(),
                        error: e.to_string(),
                    })?;
                self.merge_config(file_config);
                self.sources_loaded.push(format!("file:{}", path.display()));
            }
            _ => {
                return Err(ConfigError::UnsupportedFormat {
                    format: format.to_string(),
                });
            }
        }

        Ok(self)
    }

    /// Load environment variables with a specific prefix
    ///
    /// Environment variables override configuration from files.
    /// Variables should be in the format: `PREFIX_SECTION_KEY=value`
    ///
    /// # Example
    ///
    /// ```bash
    /// export SQUIRREL_NETWORK_HTTP_PORT=8080
    /// export SQUIRREL_TIMEOUTS_CONNECTION_TIMEOUT_SECS=45
    /// ```
    ///
    /// ```rust,no_run
    /// use squirrel_mcp_config::unified::ConfigLoader;
    ///
    /// let config = ConfigLoader::new()
    ///     .with_env_prefix("SQUIRREL_")?
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_env_prefix(mut self, prefix: &str) -> Result<Self, ConfigError> {
        // Environment variables are already loaded via default() functions
        // in each config struct. This method is here for explicitness and
        // future expansion.
        
        // Reload timeouts from environment to ensure latest values
        self.config.timeouts = TimeoutConfig::from_env();
        
        self.sources_loaded.push(format!("env:{}", prefix));
        Ok(self)
    }

    /// Merge another configuration into this one
    ///
    /// Non-default values from `other` override values in `self`.
    fn merge_config(&mut self, other: SquirrelUnifiedConfig) {
        // For now, we do a simple overlay. In the future, we could do
        // smarter merging that only overrides non-default values.
        
        // Merge system config
        if !other.system.instance_id.is_empty() {
            self.config.system.instance_id = other.system.instance_id;
        }
        if !other.system.environment.is_empty() {
            self.config.system.environment = other.system.environment;
        }
        if !other.system.log_level.is_empty() {
            self.config.system.log_level = other.system.log_level;
        }

        // Merge network config
        if other.network.http_port != 0 {
            self.config.network.http_port = other.network.http_port;
        }
        if other.network.websocket_port != 0 {
            self.config.network.websocket_port = other.network.websocket_port;
        }
        if other.network.grpc_port != 0 {
            self.config.network.grpc_port = other.network.grpc_port;
        }

        // Merge security config
        self.config.security.enabled = other.security.enabled;
        self.config.security.require_authentication = other.security.require_authentication;
        if other.security.jwt_secret.is_some() {
            self.config.security.jwt_secret = other.security.jwt_secret;
        }

        // Merge AI providers
        for (key, value) in other.ai.providers {
            self.config.ai.providers.insert(key, value);
        }

        // Merge custom values
        for (key, value) in other.custom {
            self.config.custom.insert(key, value);
        }

        // Note: Timeouts are handled via environment variables in the default() impl
    }

    /// Validate the configuration
    ///
    /// Performs comprehensive validation across all configuration domains.
    pub fn validate(self) -> Result<Self, ConfigError> {
        if let Err(errors) = self.config.validate() {
            return Err(ConfigError::ValidationFailed { errors });
        }
        Ok(self)
    }

    /// Build the final configuration
    pub fn build(self) -> Result<SquirrelUnifiedConfig, ConfigError> {
        Ok(self.config)
    }

    /// Build the final configuration with source tracking
    pub fn build_with_sources(self) -> Result<LoadedConfig, ConfigError> {
        Ok(LoadedConfig {
            config: self.config,
            sources: self.sources_loaded,
        })
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Loaded configuration with source tracking
///
/// Tracks which sources were used to load the configuration,
/// useful for debugging and auditing.
pub struct LoadedConfig {
    config: SquirrelUnifiedConfig,
    sources: Vec<String>,
}

impl LoadedConfig {
    /// Get the configuration
    pub fn config(&self) -> &SquirrelUnifiedConfig {
        &self.config
    }

    /// Get the configuration (consuming)
    pub fn into_config(self) -> SquirrelUnifiedConfig {
        self.config
    }

    /// Get the list of sources that were loaded
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Check if a specific source was loaded
    pub fn has_source(&self, source: &str) -> bool {
        self.sources.iter().any(|s| s.contains(source))
    }
}

/// Configuration loading errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {error}")]
    FileRead {
        path: PathBuf,
        error: String,
    },

    #[error("Failed to parse {format} config: {error}")]
    ParseError {
        format: String,
        error: String,
    },

    #[error("Unsupported config format: {format}")]
    UnsupportedFormat {
        format: String,
    },

    #[error("Configuration validation failed:\n{}", .errors.join("\n"))]
    ValidationFailed {
        errors: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::new();
        let config = loader.build().unwrap();
        assert!(config.system.instance_id.len() > 0);
    }

    #[test]
    fn test_config_validation() {
        // Create a config with security disabled to avoid JWT/API key requirements
        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let result = loader.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let loader = ConfigLoader::new();
        let result = loader.with_file_if_exists("nonexistent.toml");
        assert!(result.is_ok()); // Should not error if file doesn't exist
    }

    #[test]
    fn test_sources_tracking() {
        // Create a config with security disabled to avoid JWT/API key requirements
        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loaded = loader.validate().unwrap().build_with_sources().unwrap();
        assert!(!loaded.sources().is_empty());
        assert!(loaded.has_source("secure_defaults"));
    }
}

