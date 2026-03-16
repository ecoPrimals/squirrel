// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
//! ```ignore
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
#[derive(Debug)]
pub struct ConfigLoader {
    config: SquirrelUnifiedConfig,
    sources_loaded: Vec<String>,
}

impl ConfigLoader {
    /// Create a new `ConfigLoader` with secure defaults
    #[must_use]
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
    /// # Errors
    ///
    /// Returns `ConfigError` if configuration loading or validation fails.
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// Sets reasonable defaults based on the current platform using universal
    /// Rust abstractions (the `dirs` crate).
    ///
    /// ## Philosophy: Universal & Agnostic
    ///
    /// Instead of hardcoding platform-specific paths with #[cfg], we use:
    /// - `dirs` crate for standard directories (data, config)
    /// - Runtime detection, not compile-time cfg branching
    /// - Pure Rust, platform-agnostic patterns
    ///
    /// This creates 1 unified codebase that works everywhere:
    /// - Linux: ~/.local/share/squirrel
    /// - macOS: ~/Library/Application Support/squirrel
    /// - Windows: %APPDATA%/squirrel
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if platform detection fails.
    pub fn with_platform_detection(mut self) -> Result<Self, ConfigError> {
        // Use dirs crate for universal, platform-appropriate data directory
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| {
                // Graceful fallback to current directory
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("./data"))
            })
            .join("squirrel");

        self.config.system.data_dir.clone_from(&data_dir);
        self.config.system.plugin_dir = data_dir.join("plugins");

        // Detect platform for logging (runtime detection)
        let platform_name = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "other"
        };

        self.sources_loaded
            .push(format!("platform_defaults_{platform_name}"));

        tracing::debug!(
            "Applied platform defaults: platform={}, data_dir={:?}, plugin_dir={:?}",
            platform_name,
            self.config.system.data_dir,
            self.config.system.plugin_dir
        );

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
    /// ```ignore
    /// use squirrel_mcp_config::unified::ConfigLoader;
    ///
    /// let config = ConfigLoader::new()
    ///     .with_file_if_exists("squirrel.toml")?
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the file exists but cannot be read or parsed.
    pub fn with_file_if_exists<P: AsRef<Path>>(mut self, path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(self);
        }

        let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
            path: path.to_path_buf(),
            error: e.to_string(),
        })?;

        // Determine format from extension
        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("toml");

        match format {
            "toml" => {
                let file_config: SquirrelUnifiedConfig =
                    toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
                        format: "toml".to_string(),
                        error: e.to_string(),
                    })?;
                self.merge_config(file_config);
                self.sources_loaded.push(format!("file:{}", path.display()));
            }
            "json" => {
                let file_config: SquirrelUnifiedConfig =
                    serde_json::from_str(&contents).map_err(|e| ConfigError::ParseError {
                        format: "json".to_string(),
                        error: e.to_string(),
                    })?;
                self.merge_config(file_config);
                self.sources_loaded.push(format!("file:{}", path.display()));
            }
            "yaml" | "yml" => {
                let file_config: SquirrelUnifiedConfig =
                    serde_yml::from_str(&contents).map_err(|e| ConfigError::ParseError {
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
    /// ```ignore
    /// use squirrel_mcp_config::unified::ConfigLoader;
    ///
    /// let config = ConfigLoader::new()
    ///     .with_env_prefix("SQUIRREL_")?
    ///     .build()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if environment variable parsing fails.
    pub fn with_env_prefix(mut self, prefix: &str) -> Result<Self, ConfigError> {
        // Environment variables are already loaded via default() functions
        // in each config struct. This method is here for explicitness and
        // future expansion.

        // Reload timeouts from environment to ensure latest values
        self.config.timeouts = TimeoutConfig::from_env();

        self.sources_loaded.push(format!("env:{prefix}"));
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
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if validation fails.
    pub fn validate(self) -> Result<Self, ConfigError> {
        if let Err(errors) = self.config.validate() {
            return Err(ConfigError::ValidationFailed { errors });
        }
        Ok(self)
    }

    /// Build the final configuration
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if validation failed in a previous step.
    pub fn build(self) -> Result<SquirrelUnifiedConfig, ConfigError> {
        Ok(self.config)
    }

    /// Build the final configuration with source tracking
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if validation failed in a previous step.
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
    #[must_use]
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
    /// Config file could not be read from disk.
    #[error("Failed to read config file {path}: {error}")]
    FileRead {
        /// Path to the config file that failed to read.
        path: PathBuf,
        /// I/O or permission error message.
        error: String,
    },

    /// Config file content failed to parse (invalid TOML/JSON/YAML).
    #[error("Failed to parse {format} config: {error}")]
    ParseError {
        /// Detected format (e.g. "TOML", "JSON").
        format: String,
        /// Parse error message.
        error: String,
    },

    /// Config format is not supported by the loader.
    #[error("Unsupported config format: {format}")]
    UnsupportedFormat {
        /// The unsupported format identifier.
        format: String,
    },

    /// Config loaded but failed validation.
    #[error("Configuration validation failed:\n{}", .errors.join("\n"))]
    ValidationFailed {
        /// List of validation error messages.
        errors: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_loader_default() {
        let loader = ConfigLoader::new();
        let config = loader.build().unwrap();
        assert!(!config.system.instance_id.is_empty());
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

    #[test]
    fn test_unsupported_format_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.xyz");
        fs::write(&config_path, "invalid").unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        let err = result.expect_err("expected UnsupportedFormat error");
        assert!(matches!(err, ConfigError::UnsupportedFormat { .. }));
        assert!(err.to_string().contains("xyz"));
    }

    #[test]
    fn test_invalid_toml_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(&config_path, "invalid toml [[[[").unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        let err = result.expect_err("expected ParseError");
        assert!(matches!(err, ConfigError::ParseError { .. }));
    }

    #[test]
    fn test_invalid_json_parse_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, "{ invalid json }").unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        let err = result.expect_err("expected ParseError");
        assert!(matches!(err, ConfigError::ParseError { .. }));
    }

    #[test]
    fn test_loaded_config_into_config() {
        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loaded = loader.validate().unwrap().build_with_sources().unwrap();
        let config = loaded.into_config();
        assert!(!config.system.instance_id.is_empty());
    }

    #[test]
    fn test_config_loader_default_impl() {
        let loader = ConfigLoader::default();
        let config = loader.build().unwrap();
        assert!(!config.system.instance_id.is_empty());
    }

    #[test]
    fn test_with_platform_detection() {
        let result = ConfigLoader::new().with_platform_detection();
        assert!(result.is_ok());
        let loader = result.unwrap();
        let config = loader.build().unwrap();
        assert!(
            !config
                .system
                .data_dir
                .as_os_str()
                .to_string_lossy()
                .is_empty()
        );
        assert!(
            !config
                .system
                .plugin_dir
                .as_os_str()
                .to_string_lossy()
                .is_empty()
        );
    }

    #[test]
    fn test_with_env_prefix() {
        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let result = loader.with_env_prefix("SQUIRREL_");
        assert!(result.is_ok());
        let loaded = result.unwrap().build_with_sources().unwrap();
        assert!(loaded.has_source("env:"));
    }

    #[test]
    fn test_valid_toml_file_loading() {
        // Use round-trip: default config -> TOML -> load, to ensure format is valid
        let default_config = SquirrelUnifiedConfig::default();
        let toml_content = toml::to_string(&default_config).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("squirrel.toml");
        fs::write(&config_path, &toml_content).unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
        let loader = result.unwrap();
        let config = loader.build().unwrap();
        assert_eq!(config.system.environment, default_config.system.environment);
        assert_eq!(config.network.http_port, default_config.network.http_port);
    }

    #[test]
    fn test_valid_json_file_loading() {
        // Use round-trip: default config -> JSON -> load
        let default_config = SquirrelUnifiedConfig::default();
        let json_content = serde_json::to_string(&default_config).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, &json_content).unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
        let config = result.unwrap().build().unwrap();
        assert_eq!(config.system.environment, default_config.system.environment);
        assert_eq!(config.network.http_port, default_config.network.http_port);
    }

    #[test]
    fn test_valid_yaml_file_loading() {
        // Use round-trip: default config -> YAML -> load
        let default_config = SquirrelUnifiedConfig::default();
        let yaml_content = serde_yml::to_string(&default_config).unwrap();

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        fs::write(&config_path, &yaml_content).unwrap();

        let result = ConfigLoader::new().with_file_if_exists(&config_path);
        assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
        let config = result.unwrap().build().unwrap();
        assert_eq!(config.system.environment, default_config.system.environment);
        assert_eq!(config.network.http_port, default_config.network.http_port);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::UnsupportedFormat {
            format: "xml".to_string(),
        };
        assert!(err.to_string().contains("xml"));
        assert!(err.to_string().contains("Unsupported"));

        let err = ConfigError::FileRead {
            path: PathBuf::from("/nonexistent"),
            error: "Permission denied".to_string(),
        };
        assert!(err.to_string().contains("Permission denied"));
    }

    fn write_valid_config(path: &std::path::Path, overrides: &[(&str, &str)]) {
        let mut config = SquirrelUnifiedConfig::default();
        config.security.enabled = false;
        for (key, value) in overrides {
            match *key {
                "system.instance_id" => config.system.instance_id = value.to_string(),
                "system.environment" => config.system.environment = value.to_string(),
                "system.log_level" => config.system.log_level = value.to_string(),
                "network.http_port" => config.network.http_port = value.parse().unwrap(),
                "network.websocket_port" => config.network.websocket_port = value.parse().unwrap(),
                _ => {}
            }
        }
        let toml = toml::to_string(&config).unwrap();
        fs::write(path, toml).unwrap();
    }

    #[test]
    fn test_merge_config_non_overlapping_fields() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("a.toml");
        write_valid_config(&file1, &[("system.instance_id", "instance-from-a"), ("system.environment", "staging")]);
        let file2 = temp_dir.path().join("b.toml");
        write_valid_config(
            &file2,
            &[
                ("system.instance_id", ""),
                ("system.environment", ""),
                ("system.log_level", ""),
                ("network.http_port", "9090"),
                ("network.websocket_port", "9091"),
            ],
        );

        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loader = loader
            .with_file_if_exists(&file1)
            .unwrap()
            .with_file_if_exists(&file2)
            .unwrap();
        let config = loader.build().unwrap();

        assert_eq!(config.system.instance_id, "instance-from-a");
        assert_eq!(config.system.environment, "staging");
        assert_eq!(config.network.http_port, 9090);
        assert_eq!(config.network.websocket_port, 9091);
    }

    #[test]
    fn test_merge_config_precedence_later_wins() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("first.toml");
        write_valid_config(&file1, &[("system.instance_id", "first-instance"), ("system.environment", "development"), ("network.http_port", "8080")]);
        let file2 = temp_dir.path().join("second.toml");
        write_valid_config(&file2, &[("system.instance_id", "second-instance"), ("system.environment", "production"), ("network.http_port", "9999")]);

        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loader = loader
            .with_file_if_exists(&file1)
            .unwrap()
            .with_file_if_exists(&file2)
            .unwrap();
        let config = loader.build().unwrap();

        assert_eq!(config.system.instance_id, "second-instance");
        assert_eq!(config.system.environment, "production");
        assert_eq!(config.network.http_port, 9999);
    }

    #[test]
    fn test_merge_config_partial_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("partial.toml");
        write_valid_config(&config_path, &[("network.http_port", "3000")]);

        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loader = loader.with_file_if_exists(&config_path).unwrap();
        let config = loader.build().unwrap();

        assert_eq!(config.network.http_port, 3000);
        assert!(!config.system.instance_id.is_empty());
        assert!(config.network.websocket_port > 0);
    }

    #[test]
    fn test_merge_config_with_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("override.toml");
        write_valid_config(&config_path, &[("system.log_level", "debug")]);

        let mut loader = ConfigLoader::new();
        loader.config.security.enabled = false;
        let loader = loader.with_file_if_exists(&config_path).unwrap();
        let config = loader.build().unwrap();

        assert_eq!(config.system.log_level, "debug");
        assert!(!config.security.enabled);
    }

    #[test]
    fn test_config_loader_load_integration() {
        let temp_dir = TempDir::new().unwrap();
        write_valid_config(
            temp_dir.path().join("squirrel.toml").as_path(),
            &[
                ("system.instance_id", "load-test-instance"),
                ("system.environment", "staging"),
                ("network.http_port", "7777"),
            ],
        );

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = temp_env::with_vars(
            [
                ("SQUIRREL_HTTP_PORT", Some("8888")),
                ("JWT_SECRET", Some("test-jwt-secret-at-least-32-characters-long")),
            ],
            ConfigLoader::load,
        );

        std::env::set_current_dir(&original_cwd).unwrap();

        let loaded = result.expect("load should succeed");
        assert!(loaded.has_source("file:"));
        assert!(loaded.has_source("secure_defaults"));

        let config = loaded.config();
        assert_eq!(config.system.instance_id, "load-test-instance");
        assert_eq!(config.system.environment, "staging");
        assert_eq!(config.network.http_port, 7777);
    }
}
