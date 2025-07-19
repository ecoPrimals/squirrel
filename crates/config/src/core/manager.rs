//! Configuration manager for Squirrel MCP
//!
//! This module provides the main configuration management functionality
//! including loading, validation, and environment-aware configuration.

// Removed unused imports

use super::ai::AIConfig;
use super::types::{Config, CoreConfigError, DatabaseConfig, NetworkConfig};
// Removed unused ConfigDefaults import
use crate::core::ecosystem::EcosystemConfig;
// Removed unused Environment imports

/// Configuration manager trait
pub trait ConfigManager {
    type Error;

    fn load_config(&self) -> Result<Config, Self::Error>;
    fn validate_config(&self, config: &Config) -> Result<(), Self::Error>;
}

/// Default configuration manager implementation
#[derive(Debug, Clone)]
pub struct DefaultConfigManager {
    config: Config,
}

impl DefaultConfigManager {
    /// Create a new default configuration manager
    pub fn new() -> Self {
        // Load configuration from environment variables with defaults
        let config = Config {
            network: NetworkConfig::default(),
            database: DatabaseConfig::default(),
            ai: AIConfig::default(),
            security: crate::core::security::SecurityConfig::default(),
            observability: crate::core::observability::ObservabilityConfig::default(),
            ecosystem: EcosystemConfig::default(),
        };

        Self { config }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, CoreConfigError> {
        // This is a simplified implementation - in a real system you'd load from env vars
        Ok(Self::new())
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get biomeos endpoints configuration
    pub fn get_biomeos_endpoints(&self) -> std::collections::HashMap<String, String> {
        let mut endpoints = std::collections::HashMap::new();
        endpoints.insert("discovery".to_string(), "http://localhost:8080".to_string());
        endpoints.insert("registry".to_string(), "http://localhost:8081".to_string());
        endpoints.insert("health".to_string(), "http://localhost:8082".to_string());
        endpoints
    }

    /// Get external services configuration
    pub fn get_external_services_config(&self) -> std::collections::HashMap<String, String> {
        let mut services = std::collections::HashMap::new();
        services.insert("nestgate".to_string(), "http://localhost:9000".to_string());
        services.insert("toadstool".to_string(), "http://localhost:9001".to_string());
        services.insert("beardog".to_string(), "http://localhost:9002".to_string());
        services
    }
}

impl Default for DefaultConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager for DefaultConfigManager {
    type Error = CoreConfigError;

    fn load_config(&self) -> Result<Config, Self::Error> {
        Ok(self.config.clone())
    }

    fn validate_config(&self, config: &Config) -> Result<(), Self::Error> {
        // Add validation logic here
        Ok(())
    }
}
