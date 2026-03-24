// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration management for ecosystem integration
//!
//! This module provides environment-driven configuration management
//! that eliminates hardcoded values and supports dynamic configuration
//! updates through the ecosystem.

mod defaults;
mod loader;
mod validation;

pub use defaults::ConfigDefaults;
pub use loader::ConfigLoader;
pub use validation::ConfigValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{RetryConfig, UniversalConfig};
    use crate::types::SecurityLevel;

    fn create_valid_config() -> UniversalConfig {
        UniversalConfig {
            service: crate::traits::ServiceConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: "A test service".to_string(),
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                log_level: "info".to_string(),
                instance_id: "inst-1".to_string(),
            },
            songbird: crate::traits::SongbirdConfig {
                discovery_endpoint: "http://discovery:8001".to_string(),
                registration_endpoint: "http://registration:8001".to_string(),
                health_endpoint: "http://health:8001".to_string(),
                auth_token: None,
                retry_config: RetryConfig {
                    max_retries: 3,
                    initial_delay_ms: 1000,
                    max_delay_ms: 30000,
                    backoff_multiplier: 2.0,
                },
                heartbeat_interval_secs: 30,
            },
            security: crate::types::SecurityConfig {
                auth_method: "bearer".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "ecosystem.local".to_string(),
                security_level: SecurityLevel::Internal,
                crypto_lock_enabled: false,
            },
            resources: crate::traits::ResourceConfig {
                cpu_cores: None,
                memory_mb: None,
                disk_mb: None,
                network_bandwidth_mbps: None,
                gpu_count: None,
            },
            features: crate::traits::FeatureFlags {
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
    fn test_config_loader_new() {
        let loader = ConfigLoader::new("TEST_PREFIX");
        let result = loader.load_universal_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loader_require_var() {
        let mut loader = ConfigLoader::new("TEST");
        loader.require_var("VAR1").require_var("VAR2");
        let result = loader.load_universal_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_env_or_default_with_default() {
        let loader = ConfigLoader::new("NONEXISTENT_PREFIX_TEST");
        let value = loader.get_env_or_default("SOME_VAR", "default_value");
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_validate_valid_config() {
        let config = create_valid_config();
        assert!(ConfigValidator::validate_universal_config(&config).is_ok());
    }

    #[test]
    fn test_validate_empty_service_name() {
        let mut config = create_valid_config();
        config.service.name = String::new();
        assert!(ConfigValidator::validate_universal_config(&config).is_err());
    }

    #[test]
    fn test_production_config() {
        let config = ConfigDefaults::production();
        assert_eq!(config.service.name, "squirrel");
        assert!(config.security.tls_enabled);
        assert!(config.security.mtls_required);
    }

    #[test]
    fn test_development_config() {
        let config = ConfigDefaults::development();
        assert_eq!(config.service.name, "squirrel-dev");
        assert!(config.features.development_mode);
        assert!(config.features.debug_logging);
    }

    #[test]
    fn test_production_config_requires_songbird_endpoints() {
        let config = ConfigDefaults::production();
        let result = ConfigValidator::validate_universal_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_development_config_validates() {
        let config = ConfigDefaults::development();
        assert!(ConfigValidator::validate_universal_config(&config).is_ok());
    }

    #[test]
    fn test_load_universal_config_missing_required() {
        let loader = ConfigLoader::new("FULLY_MISSING_UNIVERSALCFG");
        let result = loader.load_universal_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_universal_config_with_env() {
        let prefix = "UCFG_FULL_TEST";
        temp_env::with_vars(
            [
                (format!("{prefix}_SERVICE_NAME"), Some("test-svc")),
                (
                    format!("{prefix}_SERVICE_DESCRIPTION"),
                    Some("A test service"),
                ),
                (
                    format!("{prefix}_SONGBIRD_DISCOVERY_ENDPOINT"),
                    Some("http://disc:8001"),
                ),
                (
                    format!("{prefix}_SONGBIRD_REGISTRATION_ENDPOINT"),
                    Some("http://reg:8001"),
                ),
                (
                    format!("{prefix}_SONGBIRD_HEALTH_ENDPOINT"),
                    Some("http://health:8001"),
                ),
            ],
            || {
                let loader = ConfigLoader::new(prefix);
                let config = loader.load_universal_config();
                assert!(config.is_ok());
                let config = config.expect("should succeed");
                assert_eq!(config.service.name, "test-svc");
                assert_eq!(config.songbird.discovery_endpoint, "http://disc:8001");
            },
        );
    }

    #[test]
    fn test_load_universal_config_invalid_port() {
        let prefix = "UCFG_BAD_PORT";
        temp_env::with_vars(
            [
                (format!("{prefix}_SERVICE_NAME"), Some("x")),
                (format!("{prefix}_SERVICE_DESCRIPTION"), Some("d")),
                (format!("{prefix}_PORT"), Some("not-a-port")),
                (
                    format!("{prefix}_SONGBIRD_DISCOVERY_ENDPOINT"),
                    Some("http://a:1"),
                ),
                (
                    format!("{prefix}_SONGBIRD_REGISTRATION_ENDPOINT"),
                    Some("http://b:2"),
                ),
                (
                    format!("{prefix}_SONGBIRD_HEALTH_ENDPOINT"),
                    Some("http://c:3"),
                ),
            ],
            || {
                let loader = ConfigLoader::new(prefix);
                let err = loader.load_universal_config().unwrap_err();
                assert!(
                    matches!(&err, crate::error::ConfigError::InvalidValue { key, .. } if key == "PORT")
                );
            },
        );
    }

    #[test]
    fn test_load_universal_config_invalid_security_level() {
        let prefix = "UCFG_BAD_SEC";
        temp_env::with_vars(
            [
                (format!("{prefix}_SERVICE_NAME"), Some("x")),
                (format!("{prefix}_SERVICE_DESCRIPTION"), Some("d")),
                (format!("{prefix}_SECURITY_LEVEL"), Some("ultra-secret")),
                (
                    format!("{prefix}_SONGBIRD_DISCOVERY_ENDPOINT"),
                    Some("http://a:1"),
                ),
                (
                    format!("{prefix}_SONGBIRD_REGISTRATION_ENDPOINT"),
                    Some("http://b:2"),
                ),
                (
                    format!("{prefix}_SONGBIRD_HEALTH_ENDPOINT"),
                    Some("http://c:3"),
                ),
            ],
            || {
                let loader = ConfigLoader::new(prefix);
                let err = loader.load_universal_config().unwrap_err();
                assert!(
                    matches!(&err, crate::error::ConfigError::InvalidValue { key, .. } if key == "SECURITY_LEVEL")
                );
            },
        );
    }

    #[test]
    fn test_config_loader_require_var_missing() {
        let prefix = "LOADER_REQ_UNSET_9134";
        let full_var = format!("{prefix}_MYVAR");
        temp_env::with_var(&full_var, None::<&str>, || {
            let mut loader = ConfigLoader::new(prefix);
            loader.require_var("MYVAR");
            let err = loader.load_universal_config().unwrap_err();
            assert!(matches!(
                &err,
                crate::error::ConfigError::MissingEnvVar(s) if s == &full_var
            ));
        });
    }
}
