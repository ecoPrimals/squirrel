// SPDX-License-Identifier: AGPL-3.0-only
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
        if let Some(cpu_cores) = config.cpu_cores
            && cpu_cores <= 0.0
        {
            return Err(ConfigError::ValidationFailed(
                "CPU cores must be greater than 0".to_string(),
            ));
        }
        if let Some(memory_mb) = config.memory_mb
            && memory_mb == 0
        {
            return Err(ConfigError::ValidationFailed(
                "Memory must be greater than 0".to_string(),
            ));
        }
        if let Some(disk_mb) = config.disk_mb
            && disk_mb == 0
        {
            return Err(ConfigError::ValidationFailed(
                "Disk space must be greater than 0".to_string(),
            ));
        }
        if let Some(network_bandwidth_mbps) = config.network_bandwidth_mbps
            && network_bandwidth_mbps == 0
        {
            return Err(ConfigError::ValidationFailed(
                "Network bandwidth must be greater than 0".to_string(),
            ));
        }
        if let Some(gpu_count) = config.gpu_count
            && gpu_count == 0
        {
            return Err(ConfigError::ValidationFailed(
                "GPU count must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigValidator;
    use crate::traits::{FeatureFlags, ResourceConfig, RetryConfig, ServiceConfig, SongbirdConfig};
    use crate::types::{SecurityConfig, SecurityLevel};

    fn valid_universal_config() -> crate::traits::UniversalConfig {
        crate::traits::UniversalConfig {
            service: ServiceConfig {
                name: "svc".to_string(),
                version: "1.0.0".to_string(),
                description: "d".to_string(),
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                log_level: "info".to_string(),
                instance_id: "i1".to_string(),
            },
            songbird: SongbirdConfig {
                discovery_endpoint: "http://a:1".to_string(),
                registration_endpoint: "http://b:2".to_string(),
                health_endpoint: "http://c:3".to_string(),
                auth_token: None,
                retry_config: RetryConfig {
                    max_retries: 3,
                    initial_delay_ms: 100,
                    max_delay_ms: 1000,
                    backoff_multiplier: 2.0,
                },
                heartbeat_interval_secs: 30,
            },
            security: SecurityConfig {
                auth_method: "bearer".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "local".to_string(),
                security_level: SecurityLevel::Internal,
                crypto_lock_enabled: false,
            },
            resources: ResourceConfig {
                cpu_cores: None,
                memory_mb: None,
                disk_mb: None,
                network_bandwidth_mbps: None,
                gpu_count: None,
            },
            features: FeatureFlags {
                development_mode: false,
                debug_logging: false,
                metrics_enabled: true,
                tracing_enabled: true,
                experimental_features: vec![],
            },
            primal_specific: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn validate_rejects_empty_service_version() {
        let mut c = valid_universal_config();
        c.service.version.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_bind_address() {
        let mut c = valid_universal_config();
        c.service.bind_address.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_instance_id() {
        let mut c = valid_universal_config();
        c.service.instance_id.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_invalid_discovery_url() {
        let mut c = valid_universal_config();
        c.songbird.discovery_endpoint = "not a url".to_string();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_zero_max_retries() {
        let mut c = valid_universal_config();
        c.songbird.retry_config.max_retries = 0;
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_zero_initial_delay() {
        let mut c = valid_universal_config();
        c.songbird.retry_config.initial_delay_ms = 0;
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_registration_endpoint() {
        let mut c = valid_universal_config();
        c.songbird.registration_endpoint.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_retry_max_delay_less_than_initial() {
        let mut c = valid_universal_config();
        c.songbird.retry_config.initial_delay_ms = 500;
        c.songbird.retry_config.max_delay_ms = 100;
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_backoff_multiplier_too_low() {
        let mut c = valid_universal_config();
        c.songbird.retry_config.backoff_multiplier = 1.0;
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_auth_method() {
        let mut c = valid_universal_config();
        c.security.auth_method.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_empty_trust_domain() {
        let mut c = valid_universal_config();
        c.security.trust_domain.clear();
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_non_positive_cpu_cores() {
        let mut c = valid_universal_config();
        c.resources.cpu_cores = Some(0.0);
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_zero_memory_mb() {
        let mut c = valid_universal_config();
        c.resources.memory_mb = Some(0);
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }

    #[test]
    fn validate_rejects_zero_gpu_count() {
        let mut c = valid_universal_config();
        c.resources.gpu_count = Some(0);
        assert!(ConfigValidator::validate_universal_config(&c).is_err());
    }
}
