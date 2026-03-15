// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Modern System Configuration Module
//!
//! This module provides a modernized, testable system configuration with:
//! - Builder pattern for easy construction
//! - Type-safe enums (LogLevel, Environment)
//! - Sensible defaults
//! - Test presets for different scenarios
//!
//! # Example
//!
//! ```rust
//! use squirrel_mcp_config::unified::system::{SystemConfig, LogLevel, Environment};
//!
//! // Simple construction with defaults
//! let config = SystemConfig::default();
//!
//! // Builder pattern
//! let config = SystemConfig::builder()
//!     .environment(Environment::Production)
//!     .log_level(LogLevel::Info)
//!     .build();
//!
//! // For testing - minimal config
//! let config = SystemConfig::testing();
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Log level for the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace-level logging (most verbose)
    Trace,
    /// Debug-level logging
    Debug,
    /// Info-level logging (default)
    Info,
    /// Warning-level logging
    Warn,
    /// Error-level logging (least verbose)
    Error,
}

impl LogLevel {
    /// Parse a log level from a string
    pub fn from_str(s: &str) -> Result<Self, LogLevelError> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(LogLevelError::Invalid(s.to_string())),
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    /// Check if this level is more verbose than another
    pub fn is_more_verbose_than(&self, other: &LogLevel) -> bool {
        self.verbosity() > other.verbosity()
    }

    fn verbosity(&self) -> u8 {
        match self {
            LogLevel::Error => 0,
            LogLevel::Warn => 1,
            LogLevel::Info => 2,
            LogLevel::Debug => 3,
            LogLevel::Trace => 4,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Log level parsing errors
#[derive(Debug, thiserror::Error)]
pub enum LogLevelError {
    #[error("Invalid log level: {0}. Valid values are: trace, debug, info, warn, error")]
    Invalid(String),
}

/// Environment type (re-exported from environment module or defined here)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment
    Development,
    /// Testing environment
    Testing,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
}

impl Environment {
    /// Parse from environment variable or string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prod" | "production" => Environment::Production,
            "staging" | "stage" => Environment::Staging,
            "test" | "testing" => Environment::Testing,
            _ => Environment::Development,
        }
    }

    /// Get from SQUIRREL_ENV environment variable
    pub fn from_env() -> Self {
        std::env::var("SQUIRREL_ENV")
            .map(|s| Self::from_str(&s))
            .unwrap_or(Environment::Development)
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Testing => "testing",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }

    /// Check if this is a production-like environment
    pub fn is_production_like(&self) -> bool {
        matches!(self, Environment::Production | Environment::Staging)
    }

    /// Check if this is a development-like environment
    pub fn is_development_like(&self) -> bool {
        matches!(self, Environment::Development | Environment::Testing)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Modern system configuration with builder pattern
///
/// This replaces the old monolithic SystemConfig with a more testable,
/// ergonomic design using type-safe enums and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Instance identifier (unique per instance)
    #[serde(default = "default_instance_id")]
    instance_id: String,

    /// Environment (development, testing, staging, production)
    #[serde(default)]
    environment: Environment,

    /// Log level
    #[serde(default)]
    log_level: LogLevel,

    /// Working directory
    #[serde(default = "default_work_dir")]
    work_dir: PathBuf,

    /// Data directory
    #[serde(default = "default_data_dir")]
    data_dir: PathBuf,

    /// Plugin directory
    #[serde(default = "default_plugin_dir")]
    plugin_dir: PathBuf,
}

// Default value functions
fn default_instance_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("squirrel-{}", timestamp)
}

fn default_work_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

fn default_plugin_dir() -> PathBuf {
    PathBuf::from("./plugins")
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            instance_id: default_instance_id(),
            environment: Environment::default(),
            log_level: LogLevel::default(),
            work_dir: default_work_dir(),
            data_dir: default_data_dir(),
            plugin_dir: default_plugin_dir(),
        }
    }
}

impl SystemConfig {
    /// Create a new builder for SystemConfig
    pub fn builder() -> SystemConfigBuilder {
        SystemConfigBuilder::default()
    }

    /// Create a minimal config for testing
    ///
    /// Uses predictable paths and settings that won't interfere with real data.
    pub fn testing() -> Self {
        Self {
            instance_id: "test-instance".to_string(),
            environment: Environment::Testing,
            log_level: LogLevel::Debug, // Verbose for debugging tests
            work_dir: PathBuf::from("/tmp/squirrel-test"),
            data_dir: PathBuf::from("/tmp/squirrel-test/data"),
            plugin_dir: PathBuf::from("/tmp/squirrel-test/plugins"),
        }
    }

    /// Create a development config
    pub fn development() -> Self {
        Self {
            instance_id: default_instance_id(),
            environment: Environment::Development,
            log_level: LogLevel::Debug, // Verbose for development
            work_dir: default_work_dir(),
            data_dir: PathBuf::from("./data"),
            plugin_dir: PathBuf::from("./plugins"),
        }
    }

    /// Create a production config
    pub fn production() -> Self {
        Self {
            instance_id: default_instance_id(),
            environment: Environment::Production,
            log_level: LogLevel::Info, // Less verbose in production
            work_dir: PathBuf::from("/var/lib/squirrel"),
            data_dir: PathBuf::from("/var/lib/squirrel/data"),
            plugin_dir: PathBuf::from("/usr/lib/squirrel/plugins"),
        }
    }

    // Getters with clear ownership
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn environment(&self) -> Environment {
        self.environment
    }

    pub fn log_level(&self) -> LogLevel {
        self.log_level
    }

    pub fn work_dir(&self) -> &PathBuf {
        &self.work_dir
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn plugin_dir(&self) -> &PathBuf {
        &self.plugin_dir
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), SystemConfigError> {
        // Check instance_id is not empty
        if self.instance_id.is_empty() {
            return Err(SystemConfigError::EmptyInstanceId);
        }

        // Check directories are absolute in production
        if self.environment.is_production_like() {
            if !self.work_dir.is_absolute() {
                return Err(SystemConfigError::RelativePathInProduction {
                    path: self.work_dir.clone(),
                    field: "work_dir".to_string(),
                });
            }
            if !self.data_dir.is_absolute() {
                return Err(SystemConfigError::RelativePathInProduction {
                    path: self.data_dir.clone(),
                    field: "data_dir".to_string(),
                });
            }
            if !self.plugin_dir.is_absolute() {
                return Err(SystemConfigError::RelativePathInProduction {
                    path: self.plugin_dir.clone(),
                    field: "plugin_dir".to_string(),
                });
            }
        }

        // Warn if using trace in production (via validation success)
        if self.environment.is_production_like() && self.log_level == LogLevel::Trace {
            // This is allowed but not recommended
            // Could add a warning system here
        }

        Ok(())
    }
}

/// Builder for SystemConfig
#[derive(Debug, Default)]
pub struct SystemConfigBuilder {
    instance_id: Option<String>,
    environment: Option<Environment>,
    log_level: Option<LogLevel>,
    work_dir: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    plugin_dir: Option<PathBuf>,
}

impl SystemConfigBuilder {
    /// Set the instance ID
    pub fn instance_id(mut self, id: impl Into<String>) -> Self {
        self.instance_id = Some(id.into());
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.environment = Some(env);
        self
    }

    /// Set the log level
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Set the working directory
    pub fn work_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.work_dir = Some(path.into());
        self
    }

    /// Set the data directory
    pub fn data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Set the plugin directory
    pub fn plugin_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.plugin_dir = Some(path.into());
        self
    }

    /// Build the SystemConfig
    pub fn build(self) -> SystemConfig {
        SystemConfig {
            instance_id: self.instance_id.unwrap_or_else(default_instance_id),
            environment: self.environment.unwrap_or_default(),
            log_level: self.log_level.unwrap_or_default(),
            work_dir: self.work_dir.unwrap_or_else(default_work_dir),
            data_dir: self.data_dir.unwrap_or_else(default_data_dir),
            plugin_dir: self.plugin_dir.unwrap_or_else(default_plugin_dir),
        }
    }
}

/// System configuration errors
#[derive(Debug, thiserror::Error)]
pub enum SystemConfigError {
    #[error("Instance ID cannot be empty")]
    EmptyInstanceId,

    #[error("Relative path not allowed in production: {field} = {}", path.display())]
    RelativePathInProduction { path: PathBuf, field: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== LogLevel Tests ==========

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("trace").expect("test: should succeed"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("debug").expect("test: should succeed"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("info").expect("test: should succeed"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("warn").expect("test: should succeed"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("warning").expect("test: should succeed"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("error").expect("test: should succeed"), LogLevel::Error);

        // Case insensitive
        assert_eq!(LogLevel::from_str("INFO").expect("test: should succeed"), LogLevel::Info);

        // Invalid
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "trace");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn test_log_level_verbosity() {
        assert!(LogLevel::Trace.is_more_verbose_than(&LogLevel::Debug));
        assert!(LogLevel::Debug.is_more_verbose_than(&LogLevel::Info));
        assert!(LogLevel::Info.is_more_verbose_than(&LogLevel::Warn));
        assert!(LogLevel::Warn.is_more_verbose_than(&LogLevel::Error));

        assert!(!LogLevel::Error.is_more_verbose_than(&LogLevel::Warn));
    }

    #[test]
    fn test_log_level_default() {
        assert_eq!(LogLevel::default(), LogLevel::Info);
    }

    // ========== Environment Tests ==========

    #[test]
    fn test_environment_from_str() {
        assert_eq!(
            Environment::from_str("development"),
            Environment::Development
        );
        assert_eq!(Environment::from_str("testing"), Environment::Testing);
        assert_eq!(Environment::from_str("test"), Environment::Testing);
        assert_eq!(Environment::from_str("staging"), Environment::Staging);
        assert_eq!(Environment::from_str("stage"), Environment::Staging);
        assert_eq!(Environment::from_str("production"), Environment::Production);
        assert_eq!(Environment::from_str("prod"), Environment::Production);

        // Unknown defaults to development
        assert_eq!(Environment::from_str("unknown"), Environment::Development);
    }

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Development.as_str(), "development");
        assert_eq!(Environment::Testing.as_str(), "testing");
        assert_eq!(Environment::Staging.as_str(), "staging");
        assert_eq!(Environment::Production.as_str(), "production");
    }

    #[test]
    fn test_environment_production_like() {
        assert!(Environment::Production.is_production_like());
        assert!(Environment::Staging.is_production_like());
        assert!(!Environment::Development.is_production_like());
        assert!(!Environment::Testing.is_production_like());
    }

    #[test]
    fn test_environment_development_like() {
        assert!(Environment::Development.is_development_like());
        assert!(Environment::Testing.is_development_like());
        assert!(!Environment::Production.is_development_like());
        assert!(!Environment::Staging.is_development_like());
    }

    // ========== SystemConfig Tests ==========

    #[test]
    fn test_default_config() {
        let config = SystemConfig::default();
        assert!(!config.instance_id().is_empty());
        assert_eq!(config.environment(), Environment::Development);
        assert_eq!(config.log_level(), LogLevel::Info);
    }

    #[test]
    fn test_testing_config() {
        let config = SystemConfig::testing();
        assert_eq!(config.instance_id(), "test-instance");
        assert_eq!(config.environment(), Environment::Testing);
        assert_eq!(config.log_level(), LogLevel::Debug);
        assert_eq!(config.work_dir(), &PathBuf::from("/tmp/squirrel-test"));
    }

    #[test]
    fn test_development_config() {
        let config = SystemConfig::development();
        assert!(!config.instance_id().is_empty());
        assert_eq!(config.environment(), Environment::Development);
        assert_eq!(config.log_level(), LogLevel::Debug);
    }

    #[test]
    fn test_production_config() {
        let config = SystemConfig::production();
        assert!(!config.instance_id().is_empty());
        assert_eq!(config.environment(), Environment::Production);
        assert_eq!(config.log_level(), LogLevel::Info);
        assert_eq!(config.work_dir(), &PathBuf::from("/var/lib/squirrel"));
    }

    #[test]
    fn test_builder() {
        let config = SystemConfig::builder()
            .instance_id("custom-instance")
            .environment(Environment::Staging)
            .log_level(LogLevel::Warn)
            .work_dir("/custom/work")
            .build();

        assert_eq!(config.instance_id(), "custom-instance");
        assert_eq!(config.environment(), Environment::Staging);
        assert_eq!(config.log_level(), LogLevel::Warn);
        assert_eq!(config.work_dir(), &PathBuf::from("/custom/work"));
    }

    #[test]
    fn test_builder_with_defaults() {
        let config = SystemConfig::builder()
            .environment(Environment::Testing)
            .build();

        // Should use defaults for unspecified fields
        assert!(!config.instance_id().is_empty());
        assert_eq!(config.environment(), Environment::Testing);
        assert_eq!(config.log_level(), LogLevel::Info); // default
    }

    #[test]
    fn test_validation_empty_instance_id() {
        let mut config = SystemConfig::testing();
        config.instance_id = String::new();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SystemConfigError::EmptyInstanceId
        ));
    }

    #[test]
    fn test_validation_relative_paths_in_production() {
        let mut config = SystemConfig::testing();
        config.environment = Environment::Production;
        config.work_dir = PathBuf::from("./relative"); // Relative path!

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            SystemConfigError::RelativePathInProduction { field, .. } => {
                assert_eq!(field, "work_dir");
            }
            _ => panic!("Expected RelativePathInProduction error"),
        }
    }

    #[test]
    fn test_validation_success() {
        let config = SystemConfig::testing();
        assert!(config.validate().is_ok());

        let config = SystemConfig::development();
        assert!(config.validate().is_ok());

        let config = SystemConfig::production();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = SystemConfig::builder()
            .instance_id("test-123")
            .environment(Environment::Production)
            .log_level(LogLevel::Warn)
            .build();

        let toml = toml::to_string(&config).expect("test: should succeed");
        let parsed: SystemConfig = toml::from_str(&toml).expect("test: should succeed");

        assert_eq!(parsed.instance_id(), "test-123");
        assert_eq!(parsed.environment(), Environment::Production);
        assert_eq!(parsed.log_level(), LogLevel::Warn);
    }
}
