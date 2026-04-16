// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration management for the universal patterns framework
//!
//! This module provides standardized configuration patterns that all primals
//! should use for consistent behavior across the ecosystem. The configuration
//! system has been reorganized into focused modules for better maintainability.
//!
//! ## Architecture
//!
//! The configuration system is organized into several focused areas:
//!
//! * **types**: Core configuration structures and enums
//! * **presets**: Default implementations and environment-specific presets
//! * **methods**: Implementation methods for configuration management
//! * **builder**: Configuration builder pattern for easy setup
//! * **loader**: Configuration loading from files and environment
//! * **validation**: Configuration validation and error checking
//!
//! ## Features
//!
//! * **Environment-specific presets**: Development, staging, production, testing configurations
//! * **Multi-instance management**: Support for managing multiple primal instances
//! * **Security integration**: Beardog authentication and encryption support
//! * **Orchestration support**: service mesh integration for service orchestration
//! * **Flexible networking**: TLS, timeouts, connection limits, service discovery
//! * **Comprehensive logging**: Multiple outputs, formats, and structured logging
//! * **Resource management**: Memory, CPU, disk, and file descriptor limits
//! * **Port management**: Dynamic port allocation with reservation support
//! * **Load balancing**: Multiple strategies including health-based routing
//! * **Failover and scaling**: Circuit breakers and auto-scaling configuration
//!
//! ## Quick Start
//!
//! ```
//! use universal_patterns::config::{ConfigPresets, PrimalConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let preset = ConfigPresets::development();
//! preset.validate()?;
//!
//! let custom = PrimalConfig::builder()
//!     .name("my-primal")
//!     .port(8080)
//!     .environment("production")
//!     .build()?;
//! custom.validate()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Universal Configuration
//!
//! For managing multiple primal instances:
//!
//! ```
//! use universal_patterns::config::{UniversalConfigPresets, UniversalPrimalConfig};
//!
//! let mut config = UniversalConfigPresets::multi_user_prod();
//! config.enable_auto_scaling(2, 10);
//! config.enable_failover(3, 5);
//! config.set_port_range(8000, 9000).expect("valid port range");
//! # let _: UniversalPrimalConfig = config;
//! ```

// Core modules
pub mod builder;
mod builder_presets;
pub mod endpoint_resolver;
pub mod loader;
pub mod methods;
pub mod port_resolver;
pub mod presets;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use builder::ConfigBuilder;
pub use endpoint_resolver::{Endpoint, EndpointResolver, ResolutionStrategy};
pub use loader::ConfigLoader;
pub use port_resolver::PortResolver;
pub use presets::{ConfigPresets, UniversalConfigPresets};
pub use types::*;
pub use validation::ConfigValidator;

use std::path::{Path, PathBuf};
use url::Url;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    /// Missing required configuration field
    #[error("Missing required field: {0}")]
    MissingField(String),
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    /// URL parsing error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),
    /// Configuration loading error
    #[error("Config loading error: {0}")]
    Loading(#[from] config::ConfigError),
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Factory functions for common configuration scenarios
pub struct ConfigFactory;

impl ConfigFactory {
    /// Create a development configuration for the specified primal
    pub fn development(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::development();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a production configuration for the specified primal
    pub fn production(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::production();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a testing configuration for the specified primal
    pub fn testing(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::testing();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a Docker configuration for the specified primal
    pub fn docker(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::docker();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a Kubernetes configuration for the specified primal
    pub fn kubernetes(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::kubernetes();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a high-performance configuration for the specified primal
    pub fn high_performance(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::high_performance();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a secure configuration for the specified primal
    pub fn secure(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::secure();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Create a minimal configuration for the specified primal
    pub fn minimal(primal_name: &str) -> PrimalConfig {
        let mut config = ConfigPresets::minimal();
        config.info.name = primal_name.to_string();
        config.info.primal_type = Self::infer_primal_type(primal_name);
        config
    }

    /// Infer primal type from name using capability constants.
    ///
    /// TRUE PRIMAL: Discovery is capability-based. Use capability names (security,
    /// service-mesh, storage, compute) or self-knowledge (squirrel). Legacy primal
    /// names map to Custom for backward compat when deserializing.
    fn infer_primal_type(name: &str) -> PrimalType {
        use universal_constants::capabilities;
        match name.to_lowercase().as_str() {
            n if n == capabilities::SELF_PRIMAL_NAME => PrimalType::Coordinator,
            n if n == capabilities::SECURITY_CAPABILITY => PrimalType::Security,
            n if n == capabilities::SERVICE_MESH_CAPABILITY || n == "network" => {
                PrimalType::Orchestration
            }
            n if n == capabilities::STORAGE_CAPABILITY => PrimalType::Storage,
            n if n == capabilities::COMPUTE_CAPABILITY => PrimalType::Compute,
            _ => PrimalType::Custom(name.to_string()),
        }
    }

    /// Create a universal configuration for a specific environment
    pub fn universal_for_environment(environment: &str) -> UniversalPrimalConfig {
        match environment {
            "development" => UniversalConfigPresets::single_user_dev(),
            "production" => UniversalConfigPresets::multi_user_prod(),
            "testing" => UniversalConfigPresets::testing(),
            "high-availability" => UniversalConfigPresets::high_availability(),
            _ => UniversalPrimalConfig::default(),
        }
    }
}

/// Utility functions for configuration management
pub struct ConfigUtils;

impl ConfigUtils {
    /// Merge multiple configurations with the last one taking precedence
    pub fn merge_configs(configs: Vec<PrimalConfig>) -> ConfigResult<PrimalConfig> {
        if configs.is_empty() {
            return Err(ConfigError::Invalid(
                "No configurations to merge".to_string(),
            ));
        }

        let mut merged = configs[0].clone();
        for config in configs.into_iter().skip(1) {
            merged.merge(config)?;
        }

        Ok(merged)
    }

    /// Validate a configuration and return detailed error information
    pub fn validate_detailed(config: &PrimalConfig) -> Result<Vec<String>, Vec<String>> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check for common issues
        if config.network.port == 0 {
            warnings.push("Port is set to 0, will use random port allocation".to_string());
        }

        if config.security.auth_method == AuthMethod::None
            && config.environment.name == "production"
        {
            errors.push("No authentication configured for production environment".to_string());
        }

        if config.network.tls.is_none() && config.environment.name == "production" {
            warnings.push("TLS not configured for production environment".to_string());
        }

        if !config.security.encryption.enable_inter_primal
            && config.environment.name == "production"
        {
            warnings.push("Inter-primal encryption not enabled for production".to_string());
        }

        if config.orchestration.enabled && config.orchestration.discovery_endpoint.is_none() {
            errors
                .push("Orchestration enabled but no service-mesh endpoint configured".to_string());
        }

        if config.security.auth_method != AuthMethod::None
            && config.security.security_endpoint.is_none()
        {
            errors.push("Authentication configured but no security endpoint specified".to_string());
        }

        if errors.is_empty() {
            Ok(warnings)
        } else {
            Err(errors)
        }
    }

    /// Convert configuration to different formats
    pub fn to_json(config: &PrimalConfig) -> ConfigResult<String> {
        Ok(serde_json::to_string_pretty(config)?)
    }

    /// Convert configuration to YAML
    pub fn to_yaml(config: &PrimalConfig) -> ConfigResult<String> {
        Ok(serde_yaml_ng::to_string(config)?)
    }

    /// Load configuration from JSON string
    pub fn from_json(json: &str) -> ConfigResult<PrimalConfig> {
        Ok(serde_json::from_str(json)?)
    }

    /// Load configuration from YAML string
    pub fn from_yaml(yaml: &str) -> ConfigResult<PrimalConfig> {
        Ok(serde_yaml_ng::from_str(yaml)?)
    }

    /// Generate a configuration template
    pub fn generate_template(primal_name: &str, environment: &str) -> ConfigResult<String> {
        let config = match environment {
            "development" => ConfigFactory::development(primal_name),
            "production" => ConfigFactory::production(primal_name),
            "testing" => ConfigFactory::testing(primal_name),
            "docker" => ConfigFactory::docker(primal_name),
            "kubernetes" => ConfigFactory::kubernetes(primal_name),
            _ => ConfigFactory::development(primal_name),
        };

        Self::to_yaml(&config)
    }

    /// Get configuration file locations in order of precedence
    pub fn config_file_locations() -> Vec<PathBuf> {
        vec![
            PathBuf::from("./primal.yaml"),
            PathBuf::from("./primal.yml"),
            PathBuf::from("./config/primal.yaml"),
            PathBuf::from("./config/primal.yml"),
            PathBuf::from("~/.config/primal/primal.yaml"),
            PathBuf::from("/etc/primal/primal.yaml"),
        ]
    }

    /// Check if a configuration file exists at standard locations
    pub fn find_config_file() -> Option<PathBuf> {
        Self::config_file_locations()
            .into_iter()
            .find(|path| path.exists())
    }
}

/// Configuration validation helpers
pub mod validation_helpers {
    #![cfg_attr(
        not(test),
        expect(
            clippy::wildcard_imports,
            reason = "Subset of `config` types for small helpers"
        )
    )]

    use super::*;

    /// Check if a port is valid and available
    pub fn validate_port(port: u16) -> Result<(), String> {
        if port == 0 {
            return Ok(()); // 0 means random port allocation
        }

        if port < 1024 {
            return Err(format!("Port {port} is in the privileged range (< 1024)"));
        }

        // Try to bind to the port to check availability
        match std::net::TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Port {port} is not available: {e}")),
        }
    }

    /// Validate URL format
    pub fn validate_url(url: &str) -> Result<Url, String> {
        url.parse().map_err(|e| format!("Invalid URL '{url}': {e}"))
    }

    /// Validate file path exists
    pub fn validate_file_exists(path: &Path) -> Result<(), String> {
        if path.exists() {
            Ok(())
        } else {
            Err(format!("File does not exist: {}", path.display()))
        }
    }

    /// Validate directory exists
    pub fn validate_dir_exists(path: &Path) -> Result<(), String> {
        if path.is_dir() {
            Ok(())
        } else {
            Err(format!("Directory does not exist: {}", path.display()))
        }
    }

    /// Validate environment name
    pub fn validate_environment(env: &str) -> Result<(), String> {
        match env {
            "development" | "staging" | "production" | "testing" | "docker" | "kubernetes" => {
                Ok(())
            }
            _ => Err(format!("Unknown environment: '{env}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_factory() {
        let dev_config = ConfigFactory::development("test-primal");
        assert_eq!(dev_config.info.name, "test-primal");
        assert_eq!(dev_config.environment.name, "development");
        assert_eq!(dev_config.logging.level, LogLevel::Debug);

        let prod_config = ConfigFactory::production("test-primal");
        assert_eq!(prod_config.info.name, "test-primal");
        assert_eq!(prod_config.environment.name, "production");
        assert_eq!(prod_config.logging.level, LogLevel::Warn);
    }

    #[test]
    fn test_primal_type_inference() {
        use universal_constants::capabilities;
        assert_eq!(
            ConfigFactory::infer_primal_type(capabilities::SELF_PRIMAL_NAME),
            PrimalType::Coordinator
        );
        assert_eq!(
            ConfigFactory::infer_primal_type(capabilities::SECURITY_CAPABILITY),
            PrimalType::Security
        );
        assert_eq!(
            ConfigFactory::infer_primal_type("custom-primal"),
            PrimalType::Custom("custom-primal".to_string())
        );
    }

    #[test]
    fn test_config_utils_format_conversion() {
        let config = ConfigFactory::development("test");

        let json = ConfigUtils::to_json(&config).expect("should succeed");
        assert!(json.contains("test"));

        let yaml = ConfigUtils::to_yaml(&config).expect("should succeed");
        assert!(yaml.contains("test"));

        let parsed_json = ConfigUtils::from_json(&json).expect("should succeed");
        assert_eq!(parsed_json.info.name, "test");

        let parsed_yaml = ConfigUtils::from_yaml(&yaml).expect("should succeed");
        assert_eq!(parsed_yaml.info.name, "test");
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = ConfigFactory::development("test1");
        config1.network.port = 8080;

        let mut config2 = ConfigFactory::development("test2");
        config2.network.port = 9090;
        config2
            .set_custom("custom_key", "custom_value")
            .expect("should succeed");

        let merged = ConfigUtils::merge_configs(vec![config1, config2]).expect("should succeed");
        assert_eq!(merged.network.port, 9090); // Second config takes precedence
        assert_eq!(
            merged
                .get_custom::<String>("custom_key")
                .expect("should succeed"),
            Some("custom_value".to_string())
        );
    }

    #[test]
    fn test_validation_detailed() {
        let mut config = ConfigFactory::production("test");

        // Should have warnings but no errors for default production config
        let result = ConfigUtils::validate_detailed(&config);
        assert!(result.is_ok());

        // Make it invalid
        config.orchestration.enabled = true;
        config.orchestration.discovery_endpoint = None;

        let result = ConfigUtils::validate_detailed(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_universal_config_factory() {
        let dev_config = ConfigFactory::universal_for_environment("development");
        assert_eq!(dev_config.multi_instance.max_instances_per_user, 1);

        let prod_config = ConfigFactory::universal_for_environment("production");
        assert_eq!(prod_config.multi_instance.max_instances_per_user, 10);
        assert!(prod_config.multi_instance.scaling.auto_scaling_enabled);
    }

    #[test]
    fn test_config_file_locations() {
        let locations = ConfigUtils::config_file_locations();
        assert!(!locations.is_empty());
        assert!(locations[0].to_string_lossy().contains("primal.yaml"));
    }

    #[test]
    fn test_validation_helpers() {
        use validation_helpers::*;

        // Port validation
        assert!(validate_port(0).is_ok()); // Random port
        // Note: Can't reliably test specific ports as they may be in use
        assert!(validate_port(80).is_err()); // Privileged port
        assert!(validate_port(1023).is_err()); // Privileged port

        // URL validation
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("invalid-url").is_err());

        // Environment validation
        assert!(validate_environment("production").is_ok());
        assert!(validate_environment("invalid-env").is_err());
    }

    #[test]
    fn merge_configs_empty_is_error() {
        let err = ConfigUtils::merge_configs(vec![]).expect_err("empty merge");
        assert!(matches!(err, ConfigError::Invalid(msg) if msg.contains("No configurations")));
    }

    #[test]
    fn config_factory_variants_set_name_and_type() {
        let d = ConfigFactory::docker("dock");
        assert_eq!(d.info.name, "dock");
        assert_eq!(d.network.bind_address, "0.0.0.0");

        let k = ConfigFactory::kubernetes("kube");
        assert_eq!(k.info.name, "kube");
        assert_eq!(k.environment.name, "kubernetes");

        let hp = ConfigFactory::high_performance("hp");
        assert_eq!(hp.info.name, "hp");

        let sec = ConfigFactory::secure("sec");
        assert_eq!(sec.info.name, "sec");

        let min = ConfigFactory::minimal("min");
        assert_eq!(min.info.name, "min");
    }

    #[test]
    fn universal_for_environment_unknown_matches_default_universal() {
        let cfg = ConfigFactory::universal_for_environment("staging");
        let def = UniversalPrimalConfig::default();
        assert_eq!(
            cfg.multi_instance.max_instances_per_user,
            def.multi_instance.max_instances_per_user
        );
        assert_eq!(cfg.auto_discovery_enabled, def.auto_discovery_enabled);
    }

    #[test]
    fn validate_detailed_port_zero_warns() {
        let mut c = ConfigFactory::development("p0");
        c.network.port = 0;
        let warnings = ConfigUtils::validate_detailed(&c).expect("ok");
        assert!(warnings.iter().any(|w| w.contains("random port")));
    }

    #[test]
    fn validate_detailed_production_auth_and_tls_warnings() {
        let mut c = ConfigFactory::production("prod");
        c.security.auth_method = AuthMethod::None;
        let err = ConfigUtils::validate_detailed(&c).expect_err("prod no auth");
        assert!(err.iter().any(|e| e.contains("No authentication")));

        let mut c2 = ConfigFactory::production("prod2");
        c2.network.tls = None;
        c2.security.encryption.enable_inter_primal = false;
        let w = ConfigUtils::validate_detailed(&c2).expect("warnings");
        assert!(w.iter().any(|x| x.contains("TLS")));
        assert!(w.iter().any(|x| x.contains("encryption")));
    }

    #[test]
    fn validate_detailed_auth_without_endpoint_errors() {
        let mut c = ConfigFactory::development("dev");
        c.security.auth_method = AuthMethod::Beardog {
            service_id: "svc".to_string(),
        };
        c.security.security_endpoint = None;
        let err = ConfigUtils::validate_detailed(&c).expect_err("endpoint");
        assert!(err.iter().any(|e| e.contains("security endpoint")));
    }

    #[test]
    fn generate_template_covers_environments() {
        for env in [
            "development",
            "production",
            "testing",
            "docker",
            "kubernetes",
            "custom",
        ] {
            let yaml = ConfigUtils::generate_template("tpl-name", env).expect("tpl");
            assert!(yaml.contains("tpl-name"));
        }
    }

    #[test]
    fn find_config_file_sees_primal_yaml_in_cwd() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("primal.yaml");
        std::fs::write(&path, "x: 1").expect("write");
        let prev = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(dir.path()).expect("chdir");
        let found = ConfigUtils::find_config_file();
        std::env::set_current_dir(&prev).expect("restore");
        let ok = found
            .as_ref()
            .is_some_and(|p| p.file_name().and_then(|n| n.to_str()) == Some("primal.yaml"));
        assert!(ok, "expected ./primal.yaml when present: {found:?}");
    }

    #[test]
    fn validation_helpers_file_and_dir_checks() {
        use validation_helpers::*;
        let dir = tempfile::tempdir().expect("d");
        let file = dir.path().join("f.txt");
        std::fs::write(&file, "hi").expect("w");
        assert!(validate_file_exists(&file).is_ok());
        assert!(validate_file_exists(&dir.path().join("nope.txt")).is_err());
        assert!(validate_dir_exists(dir.path()).is_ok());
        assert!(validate_dir_exists(&file).is_err());
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn primal_config_round_trip_serde(name in "[a-zA-Z0-9_-]{1,80}") {
            let original = ConfigFactory::development(&name);
            let json = ConfigUtils::to_json(&original).expect("should succeed");
            let deserialized: PrimalConfig = ConfigUtils::from_json(&json).expect("should succeed");
            prop_assert_eq!(original.info.name, deserialized.info.name);
            prop_assert_eq!(original.info.version, deserialized.info.version);
            prop_assert_eq!(original.network.port, deserialized.network.port);
        }

        #[test]
        fn primal_config_yaml_round_trip(name in "[a-zA-Z0-9_-]{1,80}") {
            let original = ConfigFactory::development(&name);
            let yaml = ConfigUtils::to_yaml(&original).expect("should succeed");
            let deserialized: PrimalConfig = ConfigUtils::from_yaml(&yaml).expect("should succeed");
            prop_assert_eq!(original.info.name, deserialized.info.name);
            prop_assert_eq!(original.network.port, deserialized.network.port);
        }
    }
}
