//! Compatibility Layer - Legacy type aliases and stubs
//!
//! This module provides backward compatibility for code that relies on
//! old configuration types. New code should use the unified config system.

use crate::unified::SquirrelUnifiedConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Legacy Config type - wrapper around SquirrelUnifiedConfig with legacy fields
#[derive(Debug, Clone)]
pub struct Config {
    /// Unified configuration
    pub unified: SquirrelUnifiedConfig,
    /// Legacy discovery field
    pub discovery: DiscoveryConfig,
    /// Legacy ecosystem field
    pub ecosystem: EcosystemConfigLegacy,
}

impl Config {
    pub fn default() -> Self {
        Self {
            unified: SquirrelUnifiedConfig::default(),
            discovery: DiscoveryConfig::default(),
            ecosystem: EcosystemConfigLegacy::default(),
        }
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self::default())
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

/// Legacy Discovery Config
#[derive(Debug, Clone, Default)]
pub struct DiscoveryConfig {
    pub songbird_endpoint: Option<String>,
}

/// Legacy Ecosystem Config
#[derive(Debug, Clone, Default)]
pub struct EcosystemConfigLegacy {
    pub discovery: DiscoveryConfig,
}

/// Legacy EcosystemConfig - now an alias for Config
pub type EcosystemConfig = Config;

/// Config Manager trait (legacy compatibility)
pub trait ConfigManager {
    fn load(&self) -> Result<SquirrelUnifiedConfig, ConfigError>;
}

/// Default Config Manager (legacy compatibility)
#[derive(Debug, Clone, Default)]
pub struct DefaultConfigManager;

impl DefaultConfigManager {
    pub fn new() -> Self {
        Self
    }

    pub fn load_from_env(&self) -> Result<SquirrelUnifiedConfig, ConfigError> {
        Ok(SquirrelUnifiedConfig::default())
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, _path: P) -> Result<SquirrelUnifiedConfig, ConfigError> {
        Ok(SquirrelUnifiedConfig::default())
    }

    pub fn get_config(&self) -> Result<SquirrelUnifiedConfig, ConfigError> {
        self.load_from_env()
    }

    pub fn validate_config(&self, _config: &SquirrelUnifiedConfig) -> Result<(), ConfigError> {
        // Legacy method - validation happens in unified system
        Ok(())
    }

    pub fn get_biomeos_endpoints(&self) -> BiomeOSEndpoints {
        BiomeOSEndpoints::default()
    }

    pub fn get_external_services_config(&self) -> ExternalServicesConfig {
        ExternalServicesConfig::default()
    }
}

/// BiomeOS Endpoints (legacy compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSEndpoints {
    pub registration_url: String,
    pub health_url: String,
    pub metrics_url: String,
}

impl BiomeOSEndpoints {
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "registration" => Some(self.registration_url.clone()),
            "health" => Some(self.health_url.clone()),
            "metrics" => Some(self.metrics_url.clone()),
            _ => None,
        }
    }
}

impl Default for BiomeOSEndpoints {
    fn default() -> Self {
        Self {
            registration_url: std::env::var("BIOMEOS_REGISTRATION_URL")
                .unwrap_or_else(|_| "http://localhost:5000/register".to_string()),
            health_url: std::env::var("BIOMEOS_HEALTH_URL")
                .unwrap_or_else(|_| "http://localhost:5000/health".to_string()),
            metrics_url: std::env::var("BIOMEOS_METRICS_URL")
                .unwrap_or_else(|_| "http://localhost:5000/metrics".to_string()),
        }
    }
}

/// External Services Config (legacy compatibility)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalServicesConfig {
    // Legacy stub - services are now configured in unified system
}

impl ExternalServicesConfig {
    pub fn get(&self, _key: &str) -> Option<String> {
        // Legacy method - returns None for all keys
        None
    }
}

impl ConfigManager for DefaultConfigManager {
    fn load(&self) -> Result<SquirrelUnifiedConfig, ConfigError> {
        self.load_from_env()
    }
}

/// Config Error (legacy compatibility)
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Generic(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::Io(error.to_string())
    }
}

