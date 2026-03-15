// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration validation for PrimalConfig
//!
//! This module provides validation logic to ensure that configurations
//! are correct, complete, and compatible with the target environment.

use super::*;
use std::net::IpAddr;

/// Configuration validator for PrimalConfig
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a complete PrimalConfig
    pub fn validate(config: &PrimalConfig) -> Result<(), ConfigError> {
        Self::validate_primal_info(&config.info)?;
        Self::validate_network_config(&config.network)?;
        Self::validate_security_config(&config.security)?;
        Self::validate_orchestration_config(&config.orchestration)?;
        Self::validate_logging_config(&config.logging)?;
        Self::validate_environment_config(&config.environment)?;
        Self::validate_custom_config(&config.custom)?;
        Self::validate_cross_dependencies(config)?;
        Ok(())
    }

    /// Validate primal information
    fn validate_primal_info(info: &PrimalInfo) -> Result<(), ConfigError> {
        // Validate name
        if info.name.is_empty() {
            return Err(ConfigError::Invalid(
                "Primal name cannot be empty".to_string(),
            ));
        }

        if !info
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ConfigError::Invalid(
                "Primal name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate version
        if info.version.is_empty() {
            return Err(ConfigError::Invalid(
                "Primal version cannot be empty".to_string(),
            ));
        }

        // Basic semver validation
        if !Self::is_valid_semver(&info.version) {
            return Err(ConfigError::Invalid(format!(
                "Invalid semantic version: {}",
                info.version
            )));
        }

        // Validate description
        if info.description.is_empty() {
            return Err(ConfigError::Invalid(
                "Primal description cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate network configuration
    fn validate_network_config(network: &NetworkConfig) -> Result<(), ConfigError> {
        // Validate bind address
        if network.bind_address.parse::<IpAddr>().is_err() {
            return Err(ConfigError::Invalid(format!(
                "Invalid bind address: {}",
                network.bind_address
            )));
        }

        // Validate port
        if network.port == 0 {
            return Err(ConfigError::Invalid("Port cannot be 0".to_string()));
        }

        // Validate public address if provided
        if let Some(public_addr) = &network.public_address
            && public_addr.parse::<IpAddr>().is_err()
            && !Self::is_valid_hostname(public_addr)
        {
            return Err(ConfigError::Invalid(format!(
                "Invalid public address: {public_addr}"
            )));
        }

        // Validate TLS config if provided
        if let Some(tls) = &network.tls {
            Self::validate_tls_config(tls)?;
        }

        // Validate timeouts
        Self::validate_timeout_config(&network.timeouts)?;

        // Validate connection limits
        Self::validate_connection_limits(&network.limits)?;

        Ok(())
    }

    /// Validate TLS configuration
    fn validate_tls_config(tls: &TlsConfig) -> Result<(), ConfigError> {
        // Check if certificate files exist
        if !tls.cert_file.exists() {
            return Err(ConfigError::Invalid(format!(
                "Certificate file not found: {}",
                tls.cert_file.display()
            )));
        }

        if !tls.key_file.exists() {
            return Err(ConfigError::Invalid(format!(
                "Private key file not found: {}",
                tls.key_file.display()
            )));
        }

        // Check CA file if provided
        if let Some(ca_file) = &tls.ca_file
            && !ca_file.exists()
        {
            return Err(ConfigError::Invalid(format!(
                "CA certificate file not found: {}",
                ca_file.display()
            )));
        }

        Ok(())
    }

    /// Validate timeout configuration
    fn validate_timeout_config(timeouts: &TimeoutConfig) -> Result<(), ConfigError> {
        if timeouts.connect == 0 {
            return Err(ConfigError::Invalid(
                "Connect timeout cannot be 0".to_string(),
            ));
        }

        if timeouts.request == 0 {
            return Err(ConfigError::Invalid(
                "Request timeout cannot be 0".to_string(),
            ));
        }

        if timeouts.keep_alive == 0 {
            return Err(ConfigError::Invalid(
                "Keep-alive timeout cannot be 0".to_string(),
            ));
        }

        if timeouts.idle == 0 {
            return Err(ConfigError::Invalid("Idle timeout cannot be 0".to_string()));
        }

        // Validate timeout relationships
        if timeouts.connect > timeouts.request {
            return Err(ConfigError::Invalid(
                "Connect timeout cannot be greater than request timeout".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate connection limits
    fn validate_connection_limits(limits: &ConnectionLimits) -> Result<(), ConfigError> {
        if limits.max_connections == 0 {
            return Err(ConfigError::Invalid(
                "Max connections cannot be 0".to_string(),
            ));
        }

        if limits.max_requests_per_connection == 0 {
            return Err(ConfigError::Invalid(
                "Max requests per connection cannot be 0".to_string(),
            ));
        }

        if let Some(rate_limit) = limits.rate_limit
            && rate_limit <= 0.0
        {
            return Err(ConfigError::Invalid(
                "Rate limit must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate security configuration
    fn validate_security_config(security: &SecurityConfig) -> Result<(), ConfigError> {
        // Validate Beardog endpoint if provided
        if let Some(endpoint) = &security.beardog_endpoint
            && endpoint.scheme() != "https"
            && endpoint.scheme() != "http"
        {
            return Err(ConfigError::Invalid(
                "Beardog endpoint must use HTTP or HTTPS".to_string(),
            ));
        }

        // Validate auth method
        Self::validate_auth_method(&security.auth_method)?;

        // Validate credential storage
        Self::validate_credential_storage(&security.credential_storage)?;

        // Validate encryption config
        Self::validate_encryption_config(&security.encryption)?;

        Ok(())
    }

    /// Validate authentication method
    fn validate_auth_method(auth_method: &AuthMethod) -> Result<(), ConfigError> {
        match auth_method {
            AuthMethod::None => {
                // No validation needed for None
            }
            AuthMethod::Token { token_file } => {
                if !token_file.exists() {
                    return Err(ConfigError::Invalid(format!(
                        "Token file not found: {}",
                        token_file.display()
                    )));
                }
            }
            AuthMethod::Certificate {
                cert_file,
                key_file,
            } => {
                if !cert_file.exists() {
                    return Err(ConfigError::Invalid(format!(
                        "Certificate file not found: {}",
                        cert_file.display()
                    )));
                }
                if !key_file.exists() {
                    return Err(ConfigError::Invalid(format!(
                        "Private key file not found: {}",
                        key_file.display()
                    )));
                }
            }
            AuthMethod::Beardog { service_id } => {
                if service_id.is_empty() {
                    return Err(ConfigError::Invalid(
                        "Service ID cannot be empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate credential storage
    fn validate_credential_storage(storage: &CredentialStorage) -> Result<(), ConfigError> {
        match storage {
            CredentialStorage::Memory => {
                // No validation needed for memory storage
            }
            CredentialStorage::File { path } => {
                if let Some(parent) = path.parent()
                    && !parent.exists()
                {
                    return Err(ConfigError::Invalid(format!(
                        "Credential storage directory not found: {}",
                        parent.display()
                    )));
                }
            }
            CredentialStorage::Beardog => {
                // No validation needed for Beardog storage
            }
        }
        Ok(())
    }

    /// Validate encryption configuration
    fn validate_encryption_config(encryption: &EncryptionConfig) -> Result<(), ConfigError> {
        // Validate key management
        Self::validate_key_management(&encryption.key_management)?;
        Ok(())
    }

    /// Validate key management
    fn validate_key_management(key_mgmt: &KeyManagement) -> Result<(), ConfigError> {
        match key_mgmt {
            KeyManagement::File { path } => {
                if let Some(parent) = path.parent()
                    && !parent.exists()
                {
                    return Err(ConfigError::Invalid(format!(
                        "Key storage directory not found: {}",
                        parent.display()
                    )));
                }
            }
            KeyManagement::Beardog => {
                // No validation needed for Beardog key management
            }
            KeyManagement::Environment { var_name } => {
                if var_name.is_empty() {
                    return Err(ConfigError::Invalid(
                        "Environment variable name cannot be empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate orchestration configuration
    fn validate_orchestration_config(
        orchestration: &OrchestrationConfig,
    ) -> Result<(), ConfigError> {
        // Validate Songbird endpoint if provided
        if let Some(endpoint) = &orchestration.songbird_endpoint
            && endpoint.scheme() != "https"
            && endpoint.scheme() != "http"
        {
            return Err(ConfigError::Invalid(
                "Songbird endpoint must use HTTP or HTTPS".to_string(),
            ));
        }

        // Validate health check config
        Self::validate_health_check_config(&orchestration.health_check)?;

        // Validate service discovery config
        Self::validate_service_discovery_config(&orchestration.service_discovery)?;

        Ok(())
    }

    /// Validate health check configuration
    fn validate_health_check_config(health_check: &HealthCheckConfig) -> Result<(), ConfigError> {
        if health_check.enabled {
            if health_check.interval == 0 {
                return Err(ConfigError::Invalid(
                    "Health check interval cannot be 0".to_string(),
                ));
            }

            if health_check.timeout == 0 {
                return Err(ConfigError::Invalid(
                    "Health check timeout cannot be 0".to_string(),
                ));
            }

            if health_check.timeout >= health_check.interval {
                return Err(ConfigError::Invalid(
                    "Health check timeout must be less than interval".to_string(),
                ));
            }

            if health_check.endpoint.is_empty() {
                return Err(ConfigError::Invalid(
                    "Health check endpoint cannot be empty".to_string(),
                ));
            }

            if !health_check.endpoint.starts_with('/') {
                return Err(ConfigError::Invalid(
                    "Health check endpoint must start with '/'".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate service discovery configuration
    fn validate_service_discovery_config(
        service_discovery: &ServiceDiscoveryConfig,
    ) -> Result<(), ConfigError> {
        if service_discovery.enabled {
            if service_discovery.ttl == 0 {
                return Err(ConfigError::Invalid(
                    "Service discovery TTL cannot be 0".to_string(),
                ));
            }

            // Validate service discovery method
            match &service_discovery.method {
                ServiceDiscoveryMethod::Dns { domain } => {
                    if domain.is_empty() {
                        return Err(ConfigError::Invalid(
                            "DNS domain cannot be empty".to_string(),
                        ));
                    }
                    if !Self::is_valid_hostname(domain) {
                        return Err(ConfigError::Invalid(format!(
                            "Invalid DNS domain: {domain}"
                        )));
                    }
                }
                ServiceDiscoveryMethod::File { path } => {
                    if let Some(parent) = path.parent()
                        && !parent.exists()
                    {
                        return Err(ConfigError::Invalid(format!(
                            "Service discovery file directory not found: {}",
                            parent.display()
                        )));
                    }
                }
                ServiceDiscoveryMethod::Songbird => {
                    // No validation needed for Songbird service discovery
                }
            }
        }

        Ok(())
    }

    /// Validate logging configuration
    fn validate_logging_config(logging: &LoggingConfig) -> Result<(), ConfigError> {
        // Validate log outputs
        for output in &logging.outputs {
            Self::validate_log_output(output)?;
        }

        if logging.outputs.is_empty() {
            return Err(ConfigError::Invalid(
                "At least one log output must be configured".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate log output
    fn validate_log_output(output: &LogOutput) -> Result<(), ConfigError> {
        match output {
            LogOutput::File { path } => {
                if let Some(parent) = path.parent()
                    && !parent.exists()
                {
                    return Err(ConfigError::Invalid(format!(
                        "Log file directory not found: {}",
                        parent.display()
                    )));
                }
            }
            _ => {
                // No validation needed for other output types
            }
        }
        Ok(())
    }

    /// Validate environment configuration
    fn validate_environment_config(environment: &EnvironmentConfig) -> Result<(), ConfigError> {
        if environment.name.is_empty() {
            return Err(ConfigError::Invalid(
                "Environment name cannot be empty".to_string(),
            ));
        }

        // Validate resource limits
        Self::validate_resource_limits(&environment.resources)?;

        Ok(())
    }

    /// Validate resource limits
    fn validate_resource_limits(resources: &ResourceLimits) -> Result<(), ConfigError> {
        if let Some(max_memory) = resources.max_memory_mb
            && max_memory == 0
        {
            return Err(ConfigError::Invalid("Max memory cannot be 0".to_string()));
        }

        if let Some(max_cpu) = resources.max_cpu_percent
            && (max_cpu <= 0.0 || max_cpu > 100.0)
        {
            return Err(ConfigError::Invalid(
                "Max CPU percent must be between 0 and 100".to_string(),
            ));
        }

        if let Some(max_disk) = resources.max_disk_mb
            && max_disk == 0
        {
            return Err(ConfigError::Invalid("Max disk cannot be 0".to_string()));
        }

        if let Some(max_fds) = resources.max_file_descriptors
            && max_fds == 0
        {
            return Err(ConfigError::Invalid(
                "Max file descriptors cannot be 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate custom configuration
    fn validate_custom_config(
        custom: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), ConfigError> {
        // Check for reserved keys
        let reserved_keys = [
            "info",
            "network",
            "security",
            "orchestration",
            "logging",
            "environment",
        ];

        for key in custom.keys() {
            if reserved_keys.contains(&key.as_str()) {
                return Err(ConfigError::Invalid(format!(
                    "Custom configuration key '{key}' is reserved"
                )));
            }
        }

        Ok(())
    }

    /// Validate cross-dependencies between configuration sections
    fn validate_cross_dependencies(config: &PrimalConfig) -> Result<(), ConfigError> {
        // If orchestration is enabled, validate that required endpoints are configured
        if config.orchestration.enabled && config.orchestration.songbird_endpoint.is_none() {
            return Err(ConfigError::Invalid(
                "Songbird endpoint must be configured when orchestration is enabled".to_string(),
            ));
        }

        // If Beardog authentication is used, validate that Beardog endpoint is configured
        if matches!(config.security.auth_method, AuthMethod::Beardog { .. })
            && config.security.beardog_endpoint.is_none()
        {
            return Err(ConfigError::Invalid(
                "Beardog endpoint must be configured when using Beardog authentication".to_string(),
            ));
        }

        // If encryption is enabled, validate that appropriate key management is configured
        if config.security.encryption.enable_inter_primal
            || config.security.encryption.enable_at_rest
        {
            // Validate that key management is properly configured
            match &config.security.encryption.key_management {
                KeyManagement::Environment { var_name } => {
                    if std::env::var(var_name).is_err() {
                        return Err(ConfigError::Invalid(format!(
                            "Environment variable '{var_name}' is not set for encryption key"
                        )));
                    }
                }
                _ => {
                    // Other key management types are validated elsewhere
                }
            }
        }

        Ok(())
    }

    /// Validate semantic version string
    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return false;
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return false;
            }
        }

        true
    }

    /// Validate hostname
    fn is_valid_hostname(hostname: &str) -> bool {
        if hostname.is_empty() || hostname.len() > 253 {
            return false;
        }

        hostname.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label.chars().all(|c| c.is_alphanumeric() || c == '-')
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_validate_empty_name() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.info.name = String::new();

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_invalid_version() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.info.version = "invalid-version".to_string();

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_invalid_bind_address() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.network.bind_address = "invalid-address".to_string();

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_zero_port() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.network.port = 0;

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_missing_tls_files() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.network.tls = Some(TlsConfig {
            cert_file: PathBuf::from("/nonexistent/cert.pem"),
            key_file: PathBuf::from("/nonexistent/key.pem"),
            ca_file: None,
            require_client_cert: false,
        });

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_cross_dependencies() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        // Enable orchestration without Songbird endpoint
        config.orchestration.enabled = true;
        config.orchestration.songbird_endpoint = None;

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_semver() {
        assert!(ConfigValidator::is_valid_semver("1.0.0"));
        assert!(ConfigValidator::is_valid_semver("1.0"));
        assert!(ConfigValidator::is_valid_semver("1.2.3"));
        assert!(!ConfigValidator::is_valid_semver("1"));
        assert!(!ConfigValidator::is_valid_semver("1.0.0.0"));
        assert!(!ConfigValidator::is_valid_semver("1.0.0-alpha"));
        assert!(!ConfigValidator::is_valid_semver("invalid"));
    }

    #[test]
    fn test_validate_hostname() {
        assert!(ConfigValidator::is_valid_hostname("example.com"));
        assert!(ConfigValidator::is_valid_hostname("test-host.example.com"));
        assert!(ConfigValidator::is_valid_hostname("host123.example.com"));
        assert!(!ConfigValidator::is_valid_hostname(""));
        assert!(!ConfigValidator::is_valid_hostname("-example.com"));
        assert!(!ConfigValidator::is_valid_hostname("example-.com"));
        assert!(!ConfigValidator::is_valid_hostname("example..com"));
    }

    #[test]
    fn test_validate_resource_limits() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.environment.resources.max_cpu_percent = Some(150.0);

        assert!(ConfigValidator::validate(&config).is_err());
    }

    #[test]
    fn test_validate_reserved_custom_keys() {
        let mut config = ConfigBuilder::new()
            .name("test-primal")
            .version("1.0.0")
            .build_unchecked();

        config.custom.insert(
            "info".to_string(),
            serde_json::Value::String("test".to_string()),
        );

        assert!(ConfigValidator::validate(&config).is_err());
    }
}
