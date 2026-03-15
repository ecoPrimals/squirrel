// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Environment-specific configuration loader (DEPRECATED)
//!
//! **This module is deprecated. Use `crate::unified` instead.**
//!
//! The old `AppConfig` system is being replaced by the modern unified config system
//! which provides:
//! - Builder pattern
//! - Type safety
//! - Better testing support
//! - Validation
//!
//! See ADR-009 for migration details.
//!
//! This module provides strongly-typed configuration loading from TOML files
//! to eliminate hardcoded values throughout the codebase.

#![deprecated(since = "0.3.0", note = "Use `crate::unified` instead. See ADR-009.")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Environment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    pub fn from_env() -> Self {
        match std::env::var("SQUIRREL_ENV")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase()
            .as_str()
        {
            "prod" | "production" => Self::Production,
            "staging" | "stage" => Self::Staging,
            "test" | "testing" => Self::Testing,
            _ => Self::Development,
        }
    }

    pub fn config_file(&self) -> &'static str {
        match self {
            Self::Development => "config/development.toml",
            Self::Testing => "config/testing.toml",
            Self::Staging => "config/production.toml", // Staging uses production config
            Self::Production => "config/production.toml",
        }
    }
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub service_discovery: ServiceDiscoveryConfig,
    pub database: DatabaseConfig,
    pub mcp: McpConfig,
    pub ai_tools: AIToolsConfig,
    pub observability: ObservabilityConfig,
    pub security: SecurityConfig,
    pub limits: LimitsConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub default_host: String,
    pub default_port: u16,
    pub health_check_port: u16,
    pub metrics_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub registry_url: String,
    pub consul_address: String,
    pub etcd_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub server_host: String,
    pub server_port: u16,
    pub websocket_port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIToolsConfig {
    pub anthropic_url: String,
    pub openai_url: String,
    /// Local AI server URL (agnostic: works with Ollama, llama.cpp, vLLM, etc.)
    pub local_server_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub jaeger_endpoint: String,
    pub prometheus_port: u16,
    pub grafana_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub beardog_url: String,
    pub auth_timeout_secs: u64,
    pub session_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_request_size_bytes: usize,
    pub max_response_size_bytes: usize,
    pub request_timeout_secs: u64,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_chaos_testing: bool,
    pub enable_experimental_features: bool,
    pub enable_telemetry: bool,
    pub enable_metrics: bool,
}

impl AppConfig {
    /// Load configuration from environment-specific file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let env = Environment::from_env();
        Self::load_from_file(env.config_file())
    }

    /// Load configuration from specific file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration for specific environment
    pub fn load_for_env(env: Environment) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from_file(env.config_file())
    }

    /// Get development configuration
    pub fn development() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_for_env(Environment::Development)
    }

    /// Get testing configuration
    pub fn testing() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_for_env(Environment::Testing)
    }

    /// Get production configuration
    pub fn production() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_for_env(Environment::Production)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_from_env() {
        // Save original environment
        let original = std::env::var("SQUIRREL_ENV").ok();

        std::env::set_var("SQUIRREL_ENV", "production");
        assert_eq!(Environment::from_env(), Environment::Production);

        std::env::set_var("SQUIRREL_ENV", "development");
        assert_eq!(Environment::from_env(), Environment::Development);

        std::env::remove_var("SQUIRREL_ENV");
        assert_eq!(Environment::from_env(), Environment::Development); // default

        // Restore original environment
        if let Some(val) = original {
            std::env::set_var("SQUIRREL_ENV", val);
        }
    }

    #[test]
    fn test_environment_config_file() {
        assert_eq!(
            Environment::Development.config_file(),
            "config/development.toml"
        );
        assert_eq!(Environment::Testing.config_file(), "config/testing.toml");
        assert_eq!(
            Environment::Production.config_file(),
            "config/production.toml"
        );
    }

    #[test]
    fn test_all_environment_variants() {
        assert_eq!(Environment::Development, Environment::Development);
        assert_ne!(Environment::Development, Environment::Production);
    }
}

#[cfg(test)]
#[path = "environment_config_tests.rs"]
mod environment_config_tests;
