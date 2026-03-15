// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration validation utilities

use crate::error::ConfigError;
use crate::traits::{ResourceConfig, ServiceConfig, SongbirdConfig, UniversalConfig};
use crate::types::SecurityConfig;

/// Configuration validation utilities
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate universal configuration
    pub fn validate_universal_config(config: &UniversalConfig) -> Result<(), ConfigError> {
        Self::validate_service_config(&config.service)?;
        Self::validate_songbird_config(&config.songbird)?;
        Self::validate_security_config(&config.security)?;
        Self::validate_resource_config(&config.resources)?;
        Ok(())
    }

    fn validate_service_config(config: &ServiceConfig) -> Result<(), ConfigError> {
        if config.name.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Service name cannot be empty".to_string(),
            ));
        }
        if config.version.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Service version cannot be empty".to_string(),
            ));
        }
        if config.bind_address.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Bind address cannot be empty".to_string(),
            ));
        }
        if config.instance_id.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Instance ID cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_songbird_config(config: &SongbirdConfig) -> Result<(), ConfigError> {
        if config.discovery_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Discovery endpoint cannot be empty".to_string(),
            ));
        }
        if config.registration_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Registration endpoint cannot be empty".to_string(),
            ));
        }
        if config.health_endpoint.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Health endpoint cannot be empty".to_string(),
            ));
        }
        url::Url::parse(&config.discovery_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid discovery endpoint URL".to_string())
        })?;
        url::Url::parse(&config.registration_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid registration endpoint URL".to_string())
        })?;
        url::Url::parse(&config.health_endpoint).map_err(|_| {
            ConfigError::ValidationFailed("Invalid health endpoint URL".to_string())
        })?;
        if config.retry_config.max_retries == 0 {
            return Err(ConfigError::ValidationFailed(
                "Max retries must be greater than 0".to_string(),
            ));
        }
        if config.retry_config.initial_delay_ms == 0 {
            return Err(ConfigError::ValidationFailed(
                "Initial delay must be greater than 0".to_string(),
            ));
        }
        if config.retry_config.max_delay_ms < config.retry_config.initial_delay_ms {
            return Err(ConfigError::ValidationFailed(
                "Max delay must be greater than initial delay".to_string(),
            ));
        }
        if config.retry_config.backoff_multiplier <= 1.0 {
            return Err(ConfigError::ValidationFailed(
                "Backoff multiplier must be greater than 1.0".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_security_config(config: &SecurityConfig) -> Result<(), ConfigError> {
        if config.auth_method.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Auth method cannot be empty".to_string(),
            ));
        }
        if config.trust_domain.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Trust domain cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn validate_resource_config(config: &ResourceConfig) -> Result<(), ConfigError> {
        if let Some(cpu_cores) = config.cpu_cores {
            if cpu_cores <= 0.0 {
                return Err(ConfigError::ValidationFailed(
                    "CPU cores must be greater than 0".to_string(),
                ));
            }
        }
        if let Some(memory_mb) = config.memory_mb {
            if memory_mb == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Memory must be greater than 0".to_string(),
                ));
            }
        }
        if let Some(disk_mb) = config.disk_mb {
            if disk_mb == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Disk space must be greater than 0".to_string(),
                ));
            }
        }
        if let Some(network_bandwidth_mbps) = config.network_bandwidth_mbps {
            if network_bandwidth_mbps == 0 {
                return Err(ConfigError::ValidationFailed(
                    "Network bandwidth must be greater than 0".to_string(),
                ));
            }
        }
        if let Some(gpu_count) = config.gpu_count {
            if gpu_count == 0 {
                return Err(ConfigError::ValidationFailed(
                    "GPU count must be greater than 0".to_string(),
                ));
            }
        }
        Ok(())
    }
}
