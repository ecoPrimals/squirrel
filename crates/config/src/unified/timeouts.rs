// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Unified Timeout Configuration
//!
//! This module provides comprehensive timeout configuration for all Squirrel operations.
//! All timeouts are environment-aware with sensible defaults.
//!
//! # Environment Variables
//!
//! All timeout values can be overridden via environment variables:
//!
//! - `SQUIRREL_CONNECTION_TIMEOUT_SECS` - Connection establishment timeout
//! - `SQUIRREL_REQUEST_TIMEOUT_SECS` - Request completion timeout
//! - `SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS` - Health check timeout
//! - `SQUIRREL_OPERATION_TIMEOUT_SECS` - Generic operation timeout
//! - `SQUIRREL_DATABASE_TIMEOUT_SECS` - Database operation timeout
//! - `SQUIRREL_HEARTBEAT_INTERVAL_SECS` - Service heartbeat interval
//! - `SQUIRREL_DISCOVERY_TIMEOUT_SECS` - Service discovery timeout
//! - `SQUIRREL_AI_INFERENCE_TIMEOUT_SECS` - AI inference timeout
//! - `SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS` - Plugin loading timeout
//! - `SQUIRREL_SESSION_TIMEOUT_SECS` - Session expiry timeout
//!
//! # Example Usage
//!
//! ```rust
//! use squirrel_mcp_config::unified::TimeoutConfig;
//! use std::time::Duration;
//!
//! let config = TimeoutConfig::from_env();
//!
//! // Get timeout as Duration
//! let timeout: Duration = config.connection_timeout();
//!
//! // Check if timeout is customized
//! if config.is_custom_timeout("connection") {
//!     println!("Connection timeout is customized");
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Comprehensive timeout configuration for all Squirrel operations
///
/// This structure provides environment-aware timeout configuration with
/// sensible defaults for all system operations. All timeouts can be
/// overridden via environment variables or configuration files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection establishment timeout (default: 30 seconds)
    ///
    /// Controls how long to wait when establishing new network connections.
    /// Environment variable: `SQUIRREL_CONNECTION_TIMEOUT_SECS`
    #[serde(default = "default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,

    /// Request completion timeout (default: 60 seconds)
    ///
    /// Maximum time to wait for a request to complete after connection.
    /// Environment variable: `SQUIRREL_REQUEST_TIMEOUT_SECS`
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,

    /// Health check timeout (default: 5 seconds)
    ///
    /// Timeout for health check probes to remote services.
    /// Environment variable: `SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS`
    #[serde(default = "default_health_check_timeout_secs")]
    pub health_check_timeout_secs: u64,

    /// Generic operation timeout (default: 10 seconds)
    ///
    /// Default timeout for operations without specific timeout configuration.
    /// Environment variable: `SQUIRREL_OPERATION_TIMEOUT_SECS`
    #[serde(default = "default_operation_timeout_secs")]
    pub operation_timeout_secs: u64,

    /// Database operation timeout (default: 30 seconds)
    ///
    /// Timeout for database queries and operations.
    /// Environment variable: `SQUIRREL_DATABASE_TIMEOUT_SECS`
    #[serde(default = "default_database_timeout_secs")]
    pub database_timeout_secs: u64,

    /// Service heartbeat interval (default: 30 seconds)
    ///
    /// Interval between heartbeat signals to service mesh.
    /// Environment variable: `SQUIRREL_HEARTBEAT_INTERVAL_SECS`
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,

    /// Service discovery timeout (default: 10 seconds)
    ///
    /// Timeout for service discovery operations.
    /// Environment variable: `SQUIRREL_DISCOVERY_TIMEOUT_SECS`
    #[serde(default = "default_discovery_timeout_secs")]
    pub discovery_timeout_secs: u64,

    /// AI inference timeout (default: 120 seconds)
    ///
    /// Maximum time to wait for AI model inference to complete.
    /// Environment variable: `SQUIRREL_AI_INFERENCE_TIMEOUT_SECS`
    #[serde(default = "default_ai_inference_timeout_secs")]
    pub ai_inference_timeout_secs: u64,

    /// Plugin loading timeout (default: 15 seconds)
    ///
    /// Maximum time to wait for a plugin to load and initialize.
    /// Environment variable: `SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS`
    #[serde(default = "default_plugin_load_timeout_secs")]
    pub plugin_load_timeout_secs: u64,

    /// Session expiry timeout (default: 3600 seconds / 1 hour)
    ///
    /// How long before an inactive session expires.
    /// Environment variable: `SQUIRREL_SESSION_TIMEOUT_SECS`
    #[serde(default = "default_session_timeout_secs")]
    pub session_timeout_secs: u64,

    /// Custom timeout values for specific operations
    ///
    /// Allows defining custom timeouts for named operations.
    /// Format: `SQUIRREL_CUSTOM_TIMEOUT_<NAME>_SECS=<value>`
    #[serde(default)]
    pub custom_timeouts: HashMap<String, u64>,
}

// Default timeout functions that check environment variables
fn default_connection_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_CONNECTION_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn default_request_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60)
}

fn default_health_check_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_HEALTH_CHECK_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

fn default_operation_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_OPERATION_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn default_database_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_DATABASE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn default_heartbeat_interval_secs() -> u64 {
    std::env::var("SQUIRREL_HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn default_discovery_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_DISCOVERY_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn default_ai_inference_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_AI_INFERENCE_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120)
}

fn default_plugin_load_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_PLUGIN_LOAD_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(15)
}

fn default_session_timeout_secs() -> u64 {
    std::env::var("SQUIRREL_SESSION_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600)
}

impl TimeoutConfig {
    /// Create a new TimeoutConfig with all environment variables loaded
    pub fn from_env() -> Self {
        Self {
            connection_timeout_secs: default_connection_timeout_secs(),
            request_timeout_secs: default_request_timeout_secs(),
            health_check_timeout_secs: default_health_check_timeout_secs(),
            operation_timeout_secs: default_operation_timeout_secs(),
            database_timeout_secs: default_database_timeout_secs(),
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
            discovery_timeout_secs: default_discovery_timeout_secs(),
            ai_inference_timeout_secs: default_ai_inference_timeout_secs(),
            plugin_load_timeout_secs: default_plugin_load_timeout_secs(),
            session_timeout_secs: default_session_timeout_secs(),
            custom_timeouts: Self::load_custom_timeouts(),
        }
    }

    /// Load custom timeout values from environment
    ///
    /// Scans for environment variables matching the pattern:
    /// `SQUIRREL_CUSTOM_TIMEOUT_<NAME>_SECS=<value>`
    fn load_custom_timeouts() -> HashMap<String, u64> {
        let mut custom = HashMap::new();

        for (key, value) in std::env::vars() {
            if let Some(name) = key.strip_prefix("SQUIRREL_CUSTOM_TIMEOUT_") {
                if let Some(name) = name.strip_suffix("_SECS") {
                    if let Ok(timeout) = value.parse::<u64>() {
                        custom.insert(name.to_lowercase(), timeout);
                    }
                }
            }
        }

        custom
    }

    /// Get connection timeout as Duration
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }

    /// Get request timeout as Duration
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    /// Get health check timeout as Duration
    pub fn health_check_timeout(&self) -> Duration {
        Duration::from_secs(self.health_check_timeout_secs)
    }

    /// Get operation timeout as Duration
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_secs(self.operation_timeout_secs)
    }

    /// Get database timeout as Duration
    pub fn database_timeout(&self) -> Duration {
        Duration::from_secs(self.database_timeout_secs)
    }

    /// Get heartbeat interval as Duration
    pub fn heartbeat_interval(&self) -> Duration {
        Duration::from_secs(self.heartbeat_interval_secs)
    }

    /// Get discovery timeout as Duration
    pub fn discovery_timeout(&self) -> Duration {
        Duration::from_secs(self.discovery_timeout_secs)
    }

    /// Get AI inference timeout as Duration
    pub fn ai_inference_timeout(&self) -> Duration {
        Duration::from_secs(self.ai_inference_timeout_secs)
    }

    /// Get plugin load timeout as Duration
    pub fn plugin_load_timeout(&self) -> Duration {
        Duration::from_secs(self.plugin_load_timeout_secs)
    }

    /// Get session timeout as Duration
    pub fn session_timeout(&self) -> Duration {
        Duration::from_secs(self.session_timeout_secs)
    }

    /// Get a custom timeout by name
    ///
    /// Returns the custom timeout if configured, otherwise returns the
    /// default operation timeout.
    pub fn get_custom_timeout(&self, name: &str) -> Duration {
        let secs = self
            .custom_timeouts
            .get(name)
            .copied()
            .unwrap_or(self.operation_timeout_secs);
        Duration::from_secs(secs)
    }

    /// Check if a custom timeout is configured
    pub fn is_custom_timeout(&self, name: &str) -> bool {
        self.custom_timeouts.contains_key(name)
    }

    /// Set a custom timeout (useful for testing)
    pub fn set_custom_timeout(&mut self, name: impl Into<String>, timeout_secs: u64) {
        self.custom_timeouts.insert(name.into(), timeout_secs);
    }

    /// Validate all timeout values
    ///
    /// Ensures timeouts are within reasonable ranges:
    /// - All timeouts must be > 0
    /// - Health checks should be < 30 seconds
    /// - Sessions should be < 24 hours
    ///
    /// Now uses unified validation module for consistency.
    pub fn validate(&self) -> Result<(), String> {
        use super::validation::Validator;

        // Validate basic timeouts
        Validator::validate_timeout_secs(self.connection_timeout_secs, "connection_timeout")
            .map_err(|e| e.to_string())?;
        Validator::validate_timeout_secs(self.request_timeout_secs, "request_timeout")
            .map_err(|e| e.to_string())?;

        // Validate health check timeout with max
        Validator::validate_timeout_with_max(
            self.health_check_timeout_secs,
            30,
            "health_check_timeout",
        )
        .map_err(|e| e.to_string())?;

        // Validate session timeout with max (24 hours)
        Validator::validate_timeout_with_max(self.session_timeout_secs, 86400, "session_timeout")
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert!(config.connection_timeout_secs > 0);
        assert!(config.request_timeout_secs > 0);
    }

    #[test]
    fn test_timeout_as_duration() {
        let config = TimeoutConfig::default();
        assert_eq!(
            config.connection_timeout(),
            Duration::from_secs(config.connection_timeout_secs)
        );
    }

    #[test]
    fn test_custom_timeout() {
        let mut config = TimeoutConfig::default();
        config.set_custom_timeout("test_operation", 42);

        assert!(config.is_custom_timeout("test_operation"));
        assert_eq!(
            config.get_custom_timeout("test_operation"),
            Duration::from_secs(42)
        );
    }

    #[test]
    fn test_validation() {
        let config = TimeoutConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_variable_loading() {
        // Set environment variable
        std::env::set_var("SQUIRREL_CONNECTION_TIMEOUT_SECS", "45");

        let config = TimeoutConfig::from_env();
        assert_eq!(config.connection_timeout(), Duration::from_secs(45));

        // Clean up
        std::env::remove_var("SQUIRREL_CONNECTION_TIMEOUT_SECS");
    }

    #[test]
    fn test_custom_timeout_from_env() {
        // Set custom timeout via environment
        std::env::set_var("SQUIRREL_CUSTOM_TIMEOUT_MY_OPERATION_SECS", "99");

        let config = TimeoutConfig::from_env();
        assert!(config.is_custom_timeout("my_operation"));
        assert_eq!(
            config.get_custom_timeout("my_operation"),
            Duration::from_secs(99)
        );

        // Clean up
        std::env::remove_var("SQUIRREL_CUSTOM_TIMEOUT_MY_OPERATION_SECS");
    }
}
