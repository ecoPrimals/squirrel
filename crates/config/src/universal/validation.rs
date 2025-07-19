//! Validation logic for the universal configuration system
//!
//! This module contains validation methods for all configuration types,
//! ensuring data integrity and consistency throughout the system.

use crate::universal::types::*;
use crate::universal::utils::{validate_url, validate_weight};
// Removed unused Duration and Url imports

/// Trait for configuration validation
pub trait ValidationExt {
    /// Validate configuration
    fn validate(&self) -> Result<(), ConfigError>;
}

impl ValidationExt for UniversalServiceConfig {
    /// Validate universal service configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate discovery endpoints
        if self.discovery_endpoints.is_empty() {
            return Err(ConfigError::MissingRequired(
                "At least one discovery endpoint is required".to_string(),
            ));
        }

        for endpoint in &self.discovery_endpoints {
            validate_url(endpoint)?;
        }

        // Validate timeout
        if self.default_timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Default timeout cannot be zero".to_string(),
            ));
        }

        // Validate service configurations
        for (name, config) in &self.services {
            if name.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Service name cannot be empty".to_string(),
                ));
            }
            config.validate()?;
        }

        // Validate sub-configurations
        self.service_mesh.validate()?;
        self.health_check.validate()?;
        self.load_balancing.validate()?;
        self.security.validate()?;

        Ok(())
    }
}

impl ValidationExt for ServiceConfig {
    /// Validate service configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Must have at least one endpoint
        if self.endpoints.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Service must have at least one endpoint".to_string(),
            ));
        }

        // Validate all endpoints
        for endpoint in &self.endpoints {
            validate_url(endpoint)?;
        }

        // Validate timeout
        if let Some(timeout) = &self.timeout {
            if timeout.as_secs() == 0 {
                return Err(ConfigError::InvalidTimeout(
                    "Service timeout cannot be zero".to_string(),
                ));
            }
        }

        // Validate health check URL
        if let Some(health_url) = &self.health_check_url {
            validate_url(health_url)?;
        }

        // Validate weight
        if let Some(weight) = self.weight {
            validate_weight(weight)?;
        }

        // Validate priority
        if let Some(priority) = self.priority {
            if priority == 0 {
                return Err(ConfigError::InvalidServiceConfig(
                    "Service priority cannot be zero".to_string(),
                ));
            }
        }

        // Validate capabilities
        for capability in &self.capabilities {
            if capability.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Capability cannot be empty".to_string(),
                ));
            }
        }

        // Validate tags
        for tag in &self.tags {
            if tag.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Tag cannot be empty".to_string(),
                ));
            }
        }

        // Validate metadata
        for (key, value) in &self.metadata {
            if key.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Metadata key cannot be empty".to_string(),
                ));
            }
            if value.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Metadata value cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl ValidationExt for ServiceMeshConfig {
    /// Validate service mesh configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate discovery endpoint if provided
        if let Some(endpoint) = &self.discovery_endpoint {
            validate_url(endpoint)?;
        }

        // Validate heartbeat interval
        if self.heartbeat_interval.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Heartbeat interval cannot be zero".to_string(),
            ));
        }

        // Validate service expiration
        if self.service_expiration.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Service expiration cannot be zero".to_string(),
            ));
        }

        // Service expiration should be greater than heartbeat interval
        if self.service_expiration <= self.heartbeat_interval {
            return Err(ConfigError::InvalidTimeout(
                "Service expiration must be greater than heartbeat interval".to_string(),
            ));
        }

        // Validate max services
        if let Some(max_services) = self.max_services {
            if max_services == 0 {
                return Err(ConfigError::InvalidServiceConfig(
                    "Maximum services cannot be zero".to_string(),
                ));
            }
        }

        // Validate namespace
        if let Some(namespace) = &self.namespace {
            if namespace.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Namespace cannot be empty".to_string(),
                ));
            }
        }

        // Validate registry type
        self.registry_type.validate()?;

        Ok(())
    }
}

impl ValidationExt for ServiceRegistryType {
    /// Validate service registry type
    fn validate(&self) -> Result<(), ConfigError> {
        match self {
            ServiceRegistryType::InMemory => {
                // No validation needed for in-memory registry
                Ok(())
            }
            ServiceRegistryType::File { path } => {
                if path.is_empty() {
                    return Err(ConfigError::InvalidServiceConfig(
                        "File registry path cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            ServiceRegistryType::Network { endpoints } => {
                if endpoints.is_empty() {
                    return Err(ConfigError::InvalidServiceConfig(
                        "Network registry must have at least one endpoint".to_string(),
                    ));
                }
                for endpoint in endpoints {
                    validate_url(endpoint)?;
                }
                Ok(())
            }
            ServiceRegistryType::Redis { connection_string } => {
                if connection_string.is_empty() {
                    return Err(ConfigError::InvalidServiceConfig(
                        "Redis connection string cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            ServiceRegistryType::Database { connection_string } => {
                if connection_string.is_empty() {
                    return Err(ConfigError::InvalidServiceConfig(
                        "Database connection string cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            ServiceRegistryType::Custom { config } => {
                if config.is_empty() {
                    return Err(ConfigError::InvalidServiceConfig(
                        "Custom registry configuration cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
        }
    }
}

impl ValidationExt for HealthCheckConfig {
    /// Validate health check configuration
    fn validate(&self) -> Result<(), ConfigError> {
        if self.interval.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Health check interval cannot be zero".to_string(),
            ));
        }

        if self.timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Health check timeout cannot be zero".to_string(),
            ));
        }

        if self.timeout >= self.interval {
            return Err(ConfigError::InvalidTimeout(
                "Health check timeout must be less than interval".to_string(),
            ));
        }

        if self.retries == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Health check retries cannot be zero".to_string(),
            ));
        }

        if self.path.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Health check path cannot be empty".to_string(),
            ));
        }

        if self.expected_codes.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Health check must have at least one expected code".to_string(),
            ));
        }

        // Validate expected codes
        for &code in &self.expected_codes {
            if !(100..600).contains(&code) {
                return Err(ConfigError::InvalidServiceConfig(format!(
                    "Invalid HTTP status code: {code}"
                )));
            }
        }

        // Validate headers
        for (key, value) in &self.headers {
            if key.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Header key cannot be empty".to_string(),
                ));
            }
            if value.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "Header value cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl ValidationExt for LoadBalancingConfig {
    /// Validate load balancing configuration
    fn validate(&self) -> Result<(), ConfigError> {
        if self.session_timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Session timeout cannot be zero".to_string(),
            ));
        }

        self.circuit_breaker.validate()?;

        Ok(())
    }
}

impl ValidationExt for CircuitBreakerConfig {
    /// Validate circuit breaker configuration
    fn validate(&self) -> Result<(), ConfigError> {
        if self.failure_threshold == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Circuit breaker failure threshold cannot be zero".to_string(),
            ));
        }

        if self.success_threshold == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Circuit breaker success threshold cannot be zero".to_string(),
            ));
        }

        if self.timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Circuit breaker timeout cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

impl ValidationExt for SecurityConfig {
    /// Validate security configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // If TLS is enabled, cert and key paths must be provided
        if self.tls_enabled {
            if self.tls_cert_path.is_none() {
                return Err(ConfigError::MissingRequired(
                    "TLS certificate path".to_string(),
                ));
            }
            if self.tls_key_path.is_none() {
                return Err(ConfigError::MissingRequired("TLS key path".to_string()));
            }
        }

        // If mTLS is enabled, TLS must be enabled and CA cert must be provided
        if self.mtls_enabled {
            if !self.tls_enabled {
                return Err(ConfigError::InvalidServiceConfig(
                    "mTLS requires TLS to be enabled".to_string(),
                ));
            }
            if self.ca_cert_path.is_none() {
                return Err(ConfigError::MissingRequired(
                    "CA certificate path for mTLS".to_string(),
                ));
            }
        }

        // Validate token expiration
        if self.token_expiration.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Token expiration cannot be zero".to_string(),
            ));
        }

        // Validate certificate paths if provided
        if let Some(cert_path) = &self.tls_cert_path {
            if cert_path.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "TLS certificate path cannot be empty".to_string(),
                ));
            }
        }

        if let Some(key_path) = &self.tls_key_path {
            if key_path.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "TLS key path cannot be empty".to_string(),
                ));
            }
        }

        if let Some(ca_path) = &self.ca_cert_path {
            if ca_path.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "CA certificate path cannot be empty".to_string(),
                ));
            }
        }

        // Validate API key
        if let Some(api_key) = &self.api_key {
            if api_key.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "API key cannot be empty".to_string(),
                ));
            }
        }

        // Validate JWT secret
        if let Some(jwt_secret) = &self.jwt_secret {
            if jwt_secret.is_empty() {
                return Err(ConfigError::InvalidServiceConfig(
                    "JWT secret cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_universal_config_validation() {
        let config = UniversalServiceConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.default_timeout = Duration::from_secs(0);
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.discovery_endpoints.clear();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_service_config_validation() {
        let mut config = ServiceConfig::new();
        config.endpoints.push("http://localhost:8080".to_string());
        assert!(config.validate().is_ok());

        let empty_config = ServiceConfig::new();
        assert!(empty_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.endpoints.push("invalid-url".to_string());
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_service_mesh_config_validation() {
        let config = ServiceMeshConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.heartbeat_interval = Duration::from_secs(0);
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.service_expiration = Duration::from_secs(15);
        invalid_config.heartbeat_interval = Duration::from_secs(30);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_health_check_config_validation() {
        let config = HealthCheckConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.interval = Duration::from_secs(0);
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.timeout = Duration::from_secs(60);
        invalid_config.interval = Duration::from_secs(30);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_security_config_validation() {
        let config = SecurityConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.tls_enabled = true;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.mtls_enabled = true;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_circuit_breaker_config_validation() {
        let config = CircuitBreakerConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config.clone();
        invalid_config.failure_threshold = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.success_threshold = 0;
        assert!(invalid_config.validate().is_err());
    }
}
