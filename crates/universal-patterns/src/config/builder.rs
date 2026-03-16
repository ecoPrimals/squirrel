// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration builder for creating PrimalConfig instances
//!
//! This module provides a fluent API for building configuration objects
//! with validation and sensible defaults.

use super::*;
use url::Url;

/// Builder for creating PrimalConfig instances
#[derive(Debug, Clone)]
#[must_use = "call `.build()` to construct the configuration"]
pub struct ConfigBuilder {
    config: PrimalConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder with default values
    pub fn new() -> Self {
        Self {
            config: PrimalConfig::default(),
        }
    }

    /// Set the primal name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.config.info.name = name.into();
        self
    }

    /// Set the primal version
    pub fn version<S: Into<String>>(mut self, version: S) -> Self {
        self.config.info.version = version.into();
        self
    }

    /// Set the primal type
    pub fn primal_type(mut self, primal_type: PrimalType) -> Self {
        self.config.info.primal_type = primal_type;
        self
    }

    /// Set the description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.config.info.description = description.into();
        self
    }

    /// Set the bind address
    pub fn bind_address<S: Into<String>>(mut self, address: S) -> Self {
        self.config.network.bind_address = address.into();
        self
    }

    /// Set the port
    pub fn port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Set the public address
    pub fn public_address<S: Into<String>>(mut self, address: S) -> Self {
        self.config.network.public_address = Some(address.into());
        self
    }

    /// Enable TLS with certificate files
    pub fn tls(mut self, cert_file: PathBuf, key_file: PathBuf) -> Self {
        self.config.network.tls = Some(TlsConfig {
            cert_file,
            key_file,
            ca_file: None,
            require_client_cert: false,
        });
        self
    }

    /// Enable mutual TLS with CA certificate
    pub fn mutual_tls(mut self, cert_file: PathBuf, key_file: PathBuf, ca_file: PathBuf) -> Self {
        self.config.network.tls = Some(TlsConfig {
            cert_file,
            key_file,
            ca_file: Some(ca_file),
            require_client_cert: true,
        });
        self
    }

    /// Set connection timeout
    pub fn connect_timeout(mut self, seconds: u64) -> Self {
        self.config.network.timeouts.connect = seconds;
        self
    }

    /// Set request timeout
    pub fn request_timeout(mut self, seconds: u64) -> Self {
        self.config.network.timeouts.request = seconds;
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.network.limits.max_connections = max;
        self
    }

    /// Set rate limiting
    pub fn rate_limit(mut self, requests_per_second: f64) -> Self {
        self.config.network.limits.rate_limit = Some(requests_per_second);
        self
    }

    /// Set Beardog endpoint
    pub fn beardog_endpoint<S: Into<String>>(mut self, endpoint: S) -> Result<Self, ConfigError> {
        let url = Url::parse(&endpoint.into())?;
        self.config.security.beardog_endpoint = Some(url);
        Ok(self)
    }

    /// Enable Beardog authentication
    pub fn beardog_auth<S: Into<String>>(mut self, service_id: S) -> Self {
        self.config.security.auth_method = AuthMethod::Beardog {
            service_id: service_id.into(),
        };
        self.config.security.credential_storage = CredentialStorage::Beardog;
        self
    }

    /// Enable token authentication
    pub fn token_auth(mut self, token_file: PathBuf) -> Self {
        self.config.security.auth_method = AuthMethod::Token { token_file };
        self
    }

    /// Enable certificate authentication
    pub fn cert_auth(mut self, cert_file: PathBuf, key_file: PathBuf) -> Self {
        self.config.security.auth_method = AuthMethod::Certificate {
            cert_file,
            key_file,
        };
        self
    }

    /// Enable encryption for inter-primal communication
    pub fn enable_inter_primal_encryption(mut self) -> Self {
        self.config.security.encryption.enable_inter_primal = true;
        self
    }

    /// Enable encryption for data at rest
    pub fn enable_at_rest_encryption(mut self) -> Self {
        self.config.security.encryption.enable_at_rest = true;
        self
    }

    /// Set encryption algorithm
    pub fn encryption_algorithm(mut self, algorithm: EncryptionAlgorithm) -> Self {
        self.config.security.encryption.algorithm = algorithm;
        self
    }

    /// Enable audit logging
    pub fn enable_audit_logging(mut self) -> Self {
        self.config.security.audit_logging = true;
        self
    }

    /// Enable local security fallback
    pub fn enable_local_fallback(mut self) -> Self {
        self.config.security.fallback.enable_local_fallback = true;
        self
    }

    /// Disable local security fallback
    pub fn disable_fallback(mut self) -> Self {
        self.config.security.fallback.enable_local_fallback = false;
        self
    }

    /// Set fallback authentication method
    pub fn fallback_auth(mut self, auth_method: AuthMethod) -> Self {
        self.config.security.fallback.local_auth_method = auth_method;
        self
    }

    /// Set fallback timeout
    pub fn fallback_timeout(mut self, timeout_seconds: u64) -> Self {
        self.config.security.fallback.fallback_timeout = timeout_seconds;
        self
    }

    /// Set Beardog endpoint (optional)
    pub fn beardog_endpoint_optional(mut self, endpoint: Option<String>) -> Self {
        if let Some(endpoint_str) = endpoint
            && let Ok(url) = Url::parse(&endpoint_str)
        {
            self.config.security.beardog_endpoint = Some(url);
        }
        self
    }

    /// Set Songbird endpoint
    pub fn songbird_endpoint<S: Into<String>>(mut self, endpoint: S) -> Result<Self, ConfigError> {
        let url = Url::parse(&endpoint.into())?;
        self.config.orchestration.songbird_endpoint = Some(url);
        Ok(self)
    }

    /// Enable orchestration
    pub fn enable_orchestration(mut self) -> Self {
        self.config.orchestration.enabled = true;
        self
    }

    /// Set orchestration mode
    pub fn orchestration_mode(mut self, mode: OrchestrationMode) -> Self {
        self.config.orchestration.mode = mode;
        self
    }

    /// Configure health checks
    pub fn health_check(
        mut self,
        enabled: bool,
        interval: u64,
        timeout: u64,
        endpoint: String,
    ) -> Self {
        self.config.orchestration.health_check = HealthCheckConfig {
            enabled,
            interval,
            timeout,
            endpoint,
        };
        self
    }

    /// Enable service discovery
    pub fn enable_service_discovery(mut self, method: ServiceDiscoveryMethod, ttl: u64) -> Self {
        self.config.orchestration.service_discovery = ServiceDiscoveryConfig {
            enabled: true,
            method,
            ttl,
        };
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.config.logging.level = level;
        self
    }

    /// Set log format
    pub fn log_format(mut self, format: LogFormat) -> Self {
        self.config.logging.format = format;
        self
    }

    /// Add log output
    pub fn add_log_output(mut self, output: LogOutput) -> Self {
        self.config.logging.outputs.push(output);
        self
    }

    /// Enable structured logging
    pub fn enable_structured_logging(mut self) -> Self {
        self.config.logging.structured = true;
        self
    }

    /// Enable tracing
    pub fn enable_tracing(mut self) -> Self {
        self.config.logging.tracing = true;
        self
    }

    /// Set environment name
    pub fn environment<S: Into<String>>(mut self, name: S) -> Self {
        self.config.environment.name = name.into();
        self
    }

    /// Add environment variable
    pub fn add_env_var<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.config
            .environment
            .variables
            .insert(key.into(), value.into());
        self
    }

    /// Add feature flag
    pub fn add_feature<S: Into<String>>(mut self, name: S, enabled: bool) -> Self {
        self.config
            .environment
            .features
            .insert(name.into(), enabled);
        self
    }

    /// Set resource limits
    pub fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.environment.resources = limits;
        self
    }

    /// Set maximum memory usage
    pub fn max_memory_mb(mut self, mb: u64) -> Self {
        self.config.environment.resources.max_memory_mb = Some(mb);
        self
    }

    /// Set maximum CPU usage
    pub fn max_cpu_percent(mut self, percent: f64) -> Self {
        self.config.environment.resources.max_cpu_percent = Some(percent);
        self
    }

    /// Add custom configuration
    pub fn add_custom<K, V>(mut self, key: K, value: V) -> Result<Self, ConfigError>
    where
        K: Into<String>,
        V: serde::Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.config.custom.insert(key.into(), json_value);
        Ok(self)
    }

    /// Build the configuration
    #[must_use = "call this to finish building and validate the configuration"]
    pub fn build(self) -> Result<PrimalConfig, ConfigError> {
        // Validate the configuration before building
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build without validation (for testing)
    pub fn build_unchecked(self) -> PrimalConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating specific primal configurations
impl ConfigBuilder {
    /// Create a configuration builder for Squirrel (AI coordinator)
    pub fn squirrel() -> Self {
        Self::new()
            .name("squirrel")
            .primal_type(PrimalType::Coordinator)
            .description("AI coordination and MCP protocol management")
            .port(8080)
    }

    /// Create a configuration builder for Beardog (Security)
    pub fn beardog() -> Self {
        Self::new()
            .name("beardog")
            .primal_type(PrimalType::Security)
            .description("Security and authentication management")
            .port(8081)
    }

    /// Create a configuration builder for Songbird (Orchestration)
    pub fn songbird() -> Self {
        Self::new()
            .name("songbird")
            .primal_type(PrimalType::Orchestration)
            .description("Orchestration and task management")
            .port(8082)
    }

    /// Create a configuration builder for development environment
    pub fn development() -> Self {
        Self::new()
            .environment("development")
            .log_level(LogLevel::Debug)
            .enable_structured_logging()
            .enable_tracing()
            .add_feature("debug_mode", true)
            .add_feature("hot_reload", true)
    }

    /// Create a configuration builder for production environment
    pub fn production() -> Self {
        Self::new()
            .environment("production")
            .log_level(LogLevel::Info)
            .log_format(LogFormat::Json)
            .enable_structured_logging()
            .enable_audit_logging()
            .enable_inter_primal_encryption()
            .enable_at_rest_encryption()
            .add_feature("debug_mode", false)
            .add_feature("hot_reload", false)
            .max_memory_mb(2048)
            .max_cpu_percent(80.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_builder_basic() {
        let config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .port(9000)
            .build()
            .unwrap();

        assert_eq!(config.info.name, "test-primal");
        assert_eq!(config.info.version, "1.0.0");
        assert_eq!(config.network.port, 9000);
    }

    #[test]
    fn test_builder_squirrel() {
        let config = ConfigBuilder::squirrel().version("1.0.0").build().unwrap();

        assert_eq!(config.info.name, "squirrel");
        assert_eq!(config.info.primal_type, PrimalType::Coordinator);
        assert_eq!(config.network.port, 8080);
    }

    #[test]
    fn test_builder_beardog() {
        let config = ConfigBuilder::beardog().version("1.0.0").build().unwrap();

        assert_eq!(config.info.name, "beardog");
        assert_eq!(config.info.primal_type, PrimalType::Security);
        assert_eq!(config.network.port, 8081);
    }

    #[test]
    fn test_builder_songbird() {
        let config = ConfigBuilder::songbird().version("1.0.0").build().unwrap();

        assert_eq!(config.info.name, "songbird");
        assert_eq!(config.info.primal_type, PrimalType::Orchestration);
        assert_eq!(config.network.port, 8082);
    }

    #[test]
    fn test_builder_development() {
        let config = ConfigBuilder::development()
            .name("test-primal")
            .version("1.0.0")
            .build()
            .unwrap();

        assert_eq!(config.environment.name, "development");
        assert_eq!(config.logging.level, LogLevel::Debug);
        assert!(config.logging.structured);
        assert!(config.logging.tracing);
        assert_eq!(config.environment.features.get("debug_mode"), Some(&true));
    }

    #[test]
    fn test_builder_production() {
        temp_env::with_var(
            "PRIMAL_ENCRYPTION_KEY",
            Some("test_key_for_testing_12345678901234567890"),
            || {
                let config = ConfigBuilder::production()
                    .name("test-primal")
                    .version("1.0.0")
                    .build()
                    .unwrap();

                assert_eq!(config.environment.name, "production");
                assert_eq!(config.logging.level, LogLevel::Info);
                assert_eq!(config.logging.format, LogFormat::Json);
                assert!(config.security.audit_logging);
                assert!(config.security.encryption.enable_inter_primal);
                assert!(config.security.encryption.enable_at_rest);
                assert_eq!(config.environment.features.get("debug_mode"), Some(&false));
            },
        );
    }

    #[test]
    fn test_builder_tls() {
        let cert_file = NamedTempFile::new().unwrap();
        let key_file = NamedTempFile::new().unwrap();

        let config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .tls(
                cert_file.path().to_path_buf(),
                key_file.path().to_path_buf(),
            )
            .build()
            .unwrap();

        assert!(config.network.tls.is_some());
        let tls = config.network.tls.unwrap();
        assert_eq!(tls.cert_file, cert_file.path());
        assert_eq!(tls.key_file, key_file.path());
        assert!(!tls.require_client_cert);
    }

    #[test]
    fn test_builder_custom_config() {
        let config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .add_custom("test_key", "test_value")
            .unwrap()
            .build()
            .unwrap();

        let custom_value: Option<String> = config.get_custom("test_key").unwrap();
        assert_eq!(custom_value, Some("test_value".to_string()));
    }

    #[test]
    fn test_builder_resource_limits() {
        let config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .max_memory_mb(1024)
            .max_cpu_percent(50.0)
            .build()
            .unwrap();

        assert_eq!(config.environment.resources.max_memory_mb, Some(1024));
        assert_eq!(config.environment.resources.max_cpu_percent, Some(50.0));
    }
}
