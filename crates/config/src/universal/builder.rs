//! Builder patterns for the universal configuration system
//!
//! This module provides builder patterns for constructing configuration
//! objects in a fluent and type-safe manner.

use crate::universal::types::*;
use crate::universal::utils::validate_url;
use crate::universal::validation::ValidationExt;
use std::collections::HashMap;
use std::time::Duration;

/// Universal configuration builder
pub struct UniversalConfigBuilder {
    config: UniversalServiceConfig,
}

impl UniversalConfigBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: UniversalServiceConfig::new(),
        }
    }

    /// Add discovery endpoint
    pub fn add_discovery_endpoint(mut self, endpoint: String) -> Result<Self, ConfigError> {
        validate_url(&endpoint)?;
        self.config.discovery_endpoints.push(endpoint);
        Ok(self)
    }

    /// Set default timeout
    pub fn with_default_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Default timeout cannot be zero".to_string(),
            ));
        }
        self.config.default_timeout = timeout;
        Ok(self)
    }

    /// Add service configuration
    pub fn add_service(mut self, name: String, config: ServiceConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        self.config.services.insert(name, config);
        Ok(self)
    }

    /// Configure service mesh
    pub fn with_service_mesh(mut self, config: ServiceMeshConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        self.config.service_mesh = config;
        Ok(self)
    }

    /// Configure health checks
    pub fn with_health_check(mut self, config: HealthCheckConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        self.config.health_check = config;
        Ok(self)
    }

    /// Configure load balancing
    pub fn with_load_balancing(mut self, config: LoadBalancingConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        self.config.load_balancing = config;
        Ok(self)
    }

    /// Configure security
    pub fn with_security(mut self, config: SecurityConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        self.config.security = config;
        Ok(self)
    }

    /// Build configuration
    pub fn build(self) -> Result<UniversalServiceConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for UniversalConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Service configuration builder
pub struct ServiceConfigBuilder {
    config: ServiceConfig,
}

impl ServiceConfigBuilder {
    /// Create new service config builder
    pub fn new() -> Self {
        Self {
            config: ServiceConfig::new(),
        }
    }

    /// Add endpoint
    pub fn add_endpoint(mut self, endpoint: String) -> Result<Self, ConfigError> {
        validate_url(&endpoint)?;
        self.config.endpoints.push(endpoint);
        Ok(self)
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Service timeout cannot be zero".to_string(),
            ));
        }
        self.config.timeout = Some(timeout);
        Ok(self)
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.config.metadata.insert(key, value);
        self
    }

    /// Set metadata from map
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.config.metadata = metadata;
        self
    }

    /// Set health check URL
    pub fn with_health_check_url(mut self, url: String) -> Result<Self, ConfigError> {
        validate_url(&url)?;
        self.config.health_check_url = Some(url);
        Ok(self)
    }

    /// Add capability
    pub fn add_capability(mut self, capability: String) -> Self {
        if !capability.is_empty() {
            self.config.capabilities.push(capability);
        }
        self
    }

    /// Set capabilities from vector
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.config.capabilities = capabilities;
        self
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f32) -> Result<Self, ConfigError> {
        if !(0.0..=1.0).contains(&weight) {
            return Err(ConfigError::InvalidServiceConfig(format!(
                "Weight must be between 0.0 and 1.0, got: {weight}"
            )));
        }
        self.config.weight = Some(weight);
        Ok(self)
    }

    /// Add tag
    pub fn add_tag(mut self, tag: String) -> Self {
        if !tag.is_empty() {
            self.config.tags.push(tag);
        }
        self
    }

    /// Set tags from vector
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.config.tags = tags;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: u32) -> Result<Self, ConfigError> {
        if priority == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Priority cannot be zero".to_string(),
            ));
        }
        self.config.priority = Some(priority);
        Ok(self)
    }

    /// Set required flag
    pub fn with_required(mut self, required: bool) -> Self {
        self.config.required = required;
        self
    }

    /// Build service configuration
    pub fn build(self) -> Result<ServiceConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ServiceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Service mesh configuration builder
pub struct ServiceMeshConfigBuilder {
    config: ServiceMeshConfig,
}

impl ServiceMeshConfigBuilder {
    /// Create new service mesh config builder
    pub fn new() -> Self {
        Self {
            config: ServiceMeshConfig::default(),
        }
    }

    /// Set enabled flag
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set discovery endpoint
    pub fn with_discovery_endpoint(mut self, endpoint: String) -> Result<Self, ConfigError> {
        validate_url(&endpoint)?;
        self.config.discovery_endpoint = Some(endpoint);
        Ok(self)
    }

    /// Set registry type
    pub fn with_registry_type(
        mut self,
        registry_type: ServiceRegistryType,
    ) -> Result<Self, ConfigError> {
        registry_type.validate()?;
        self.config.registry_type = registry_type;
        Ok(self)
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Result<Self, ConfigError> {
        if interval.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Heartbeat interval cannot be zero".to_string(),
            ));
        }
        self.config.heartbeat_interval = interval;
        Ok(self)
    }

    /// Set service expiration
    pub fn with_service_expiration(mut self, expiration: Duration) -> Result<Self, ConfigError> {
        if expiration.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Service expiration cannot be zero".to_string(),
            ));
        }
        self.config.service_expiration = expiration;
        Ok(self)
    }

    /// Set max services
    pub fn with_max_services(mut self, max_services: usize) -> Result<Self, ConfigError> {
        if max_services == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Max services cannot be zero".to_string(),
            ));
        }
        self.config.max_services = Some(max_services);
        Ok(self)
    }

    /// Set metrics enabled flag
    pub fn with_metrics_enabled(mut self, enabled: bool) -> Self {
        self.config.metrics_enabled = enabled;
        self
    }

    /// Set namespace
    pub fn with_namespace(mut self, namespace: String) -> Result<Self, ConfigError> {
        if namespace.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Namespace cannot be empty".to_string(),
            ));
        }
        self.config.namespace = Some(namespace);
        Ok(self)
    }

    /// Build service mesh configuration
    pub fn build(self) -> Result<ServiceMeshConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for ServiceMeshConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check configuration builder
pub struct HealthCheckConfigBuilder {
    config: HealthCheckConfig,
}

impl HealthCheckConfigBuilder {
    /// Create new health check config builder
    pub fn new() -> Self {
        Self {
            config: HealthCheckConfig::default(),
        }
    }

    /// Set enabled flag
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set interval
    pub fn with_interval(mut self, interval: Duration) -> Result<Self, ConfigError> {
        if interval.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Health check interval cannot be zero".to_string(),
            ));
        }
        self.config.interval = interval;
        Ok(self)
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Health check timeout cannot be zero".to_string(),
            ));
        }
        self.config.timeout = timeout;
        Ok(self)
    }

    /// Set retries
    pub fn with_retries(mut self, retries: u32) -> Result<Self, ConfigError> {
        if retries == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Retries cannot be zero".to_string(),
            ));
        }
        self.config.retries = retries;
        Ok(self)
    }

    /// Set path
    pub fn with_path(mut self, path: String) -> Result<Self, ConfigError> {
        if path.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Path cannot be empty".to_string(),
            ));
        }
        self.config.path = path;
        Ok(self)
    }

    /// Set expected codes
    pub fn with_expected_codes(mut self, codes: Vec<u16>) -> Result<Self, ConfigError> {
        if codes.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "Expected codes cannot be empty".to_string(),
            ));
        }
        for &code in &codes {
            if !(100..600).contains(&code) {
                return Err(ConfigError::InvalidServiceConfig(format!(
                    "Invalid HTTP status code: {code}"
                )));
            }
        }
        self.config.expected_codes = codes;
        Ok(self)
    }

    /// Add expected code
    pub fn add_expected_code(mut self, code: u16) -> Result<Self, ConfigError> {
        if !(100..600).contains(&code) {
            return Err(ConfigError::InvalidServiceConfig(format!(
                "Invalid HTTP status code: {code}"
            )));
        }
        if !self.config.expected_codes.contains(&code) {
            self.config.expected_codes.push(code);
        }
        Ok(self)
    }

    /// Set headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.config.headers = headers;
        self
    }

    /// Add header
    pub fn add_header(mut self, key: String, value: String) -> Self {
        self.config.headers.insert(key, value);
        self
    }

    /// Build health check configuration
    pub fn build(self) -> Result<HealthCheckConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for HealthCheckConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Load balancing configuration builder
pub struct LoadBalancingConfigBuilder {
    config: LoadBalancingConfig,
}

impl LoadBalancingConfigBuilder {
    /// Create new load balancing config builder
    pub fn new() -> Self {
        Self {
            config: LoadBalancingConfig::default(),
        }
    }

    /// Set strategy
    pub fn with_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// Set sticky sessions
    pub fn with_sticky_sessions(mut self, sticky: bool) -> Self {
        self.config.sticky_sessions = sticky;
        self
    }

    /// Set session timeout
    pub fn with_session_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Session timeout cannot be zero".to_string(),
            ));
        }
        self.config.session_timeout = timeout;
        Ok(self)
    }

    /// Set circuit breaker configuration
    pub fn with_circuit_breaker(
        mut self,
        circuit_breaker: CircuitBreakerConfig,
    ) -> Result<Self, ConfigError> {
        circuit_breaker.validate()?;
        self.config.circuit_breaker = circuit_breaker;
        Ok(self)
    }

    /// Build load balancing configuration
    pub fn build(self) -> Result<LoadBalancingConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for LoadBalancingConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker configuration builder
pub struct CircuitBreakerConfigBuilder {
    config: CircuitBreakerConfig,
}

impl CircuitBreakerConfigBuilder {
    /// Create new circuit breaker config builder
    pub fn new() -> Self {
        Self {
            config: CircuitBreakerConfig::default(),
        }
    }

    /// Set enabled flag
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set failure threshold
    pub fn with_failure_threshold(mut self, threshold: u32) -> Result<Self, ConfigError> {
        if threshold == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Failure threshold cannot be zero".to_string(),
            ));
        }
        self.config.failure_threshold = threshold;
        Ok(self)
    }

    /// Set success threshold
    pub fn with_success_threshold(mut self, threshold: u32) -> Result<Self, ConfigError> {
        if threshold == 0 {
            return Err(ConfigError::InvalidServiceConfig(
                "Success threshold cannot be zero".to_string(),
            ));
        }
        self.config.success_threshold = threshold;
        Ok(self)
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Result<Self, ConfigError> {
        if timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Circuit breaker timeout cannot be zero".to_string(),
            ));
        }
        self.config.timeout = timeout;
        Ok(self)
    }

    /// Build circuit breaker configuration
    pub fn build(self) -> Result<CircuitBreakerConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for CircuitBreakerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Security configuration builder
pub struct SecurityConfigBuilder {
    config: SecurityConfig,
}

impl SecurityConfigBuilder {
    /// Create new security config builder
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
        }
    }

    /// Set TLS enabled flag
    pub fn with_tls_enabled(mut self, enabled: bool) -> Self {
        self.config.tls_enabled = enabled;
        self
    }

    /// Set TLS certificate path
    pub fn with_tls_cert_path(mut self, path: String) -> Result<Self, ConfigError> {
        if path.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "TLS certificate path cannot be empty".to_string(),
            ));
        }
        self.config.tls_cert_path = Some(path);
        Ok(self)
    }

    /// Set TLS key path
    pub fn with_tls_key_path(mut self, path: String) -> Result<Self, ConfigError> {
        if path.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "TLS key path cannot be empty".to_string(),
            ));
        }
        self.config.tls_key_path = Some(path);
        Ok(self)
    }

    /// Set CA certificate path
    pub fn with_ca_cert_path(mut self, path: String) -> Result<Self, ConfigError> {
        if path.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "CA certificate path cannot be empty".to_string(),
            ));
        }
        self.config.ca_cert_path = Some(path);
        Ok(self)
    }

    /// Set mTLS enabled flag
    pub fn with_mtls_enabled(mut self, enabled: bool) -> Self {
        self.config.mtls_enabled = enabled;
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: String) -> Result<Self, ConfigError> {
        if api_key.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "API key cannot be empty".to_string(),
            ));
        }
        self.config.api_key = Some(api_key);
        Ok(self)
    }

    /// Set JWT secret
    pub fn with_jwt_secret(mut self, secret: String) -> Result<Self, ConfigError> {
        if secret.is_empty() {
            return Err(ConfigError::InvalidServiceConfig(
                "JWT secret cannot be empty".to_string(),
            ));
        }
        self.config.jwt_secret = Some(secret);
        Ok(self)
    }

    /// Set token expiration
    pub fn with_token_expiration(mut self, expiration: Duration) -> Result<Self, ConfigError> {
        if expiration.as_secs() == 0 {
            return Err(ConfigError::InvalidTimeout(
                "Token expiration cannot be zero".to_string(),
            ));
        }
        self.config.token_expiration = expiration;
        Ok(self)
    }

    /// Build security configuration
    pub fn build(self) -> Result<SecurityConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for SecurityConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_config_builder() {
        let config = UniversalConfigBuilder::new()
            .add_discovery_endpoint("http://localhost:8500".to_string())
            .expect("Valid endpoint")
            .with_default_timeout(Duration::from_secs(30))
            .expect("Valid timeout")
            .add_service(
                "ai-service".to_string(),
                ServiceConfigBuilder::new()
                    .add_endpoint("http://localhost:8080".to_string())
                    .expect("Valid endpoint")
                    .add_capability("chat".to_string())
                    .build()
                    .expect("Valid service config"),
            )
            .expect("Valid service")
            .build()
            .expect("Valid config");

        assert_eq!(config.discovery_endpoints.len(), 4); // 3 default + 1 added
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert!(config.has_service("ai-service"));
    }

    #[test]
    fn test_service_config_builder() {
        let config = ServiceConfigBuilder::new()
            .add_endpoint("http://localhost:8080".to_string())
            .expect("Valid endpoint")
            .add_endpoint("http://localhost:8081".to_string())
            .expect("Valid endpoint")
            .with_timeout(Duration::from_secs(10))
            .expect("Valid timeout")
            .add_metadata("type".to_string(), "ai".to_string())
            .add_capability("chat".to_string())
            .with_weight(0.8)
            .expect("Valid weight")
            .with_required(true)
            .build()
            .expect("Valid config");

        assert_eq!(config.endpoints.len(), 2);
        assert_eq!(config.timeout, Some(Duration::from_secs(10)));
        assert_eq!(config.metadata.get("type").unwrap(), "ai");
        assert_eq!(config.capabilities.len(), 1);
        assert_eq!(config.weight, Some(0.8));
        assert!(config.required);
    }

    #[test]
    fn test_health_check_config_builder() {
        let config = HealthCheckConfigBuilder::new()
            .with_enabled(true)
            .with_interval(Duration::from_secs(30))
            .expect("Valid interval")
            .with_timeout(Duration::from_secs(5))
            .expect("Valid timeout")
            .with_retries(3)
            .expect("Valid retries")
            .with_path("/health".to_string())
            .expect("Valid path")
            .add_expected_code(200)
            .expect("Valid code")
            .add_expected_code(204)
            .expect("Valid code")
            .build()
            .expect("Valid config");

        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 3);
        assert_eq!(config.path, "/health");
        assert_eq!(config.expected_codes.len(), 2);
    }

    #[test]
    fn test_security_config_builder() {
        let config = SecurityConfigBuilder::new()
            .with_tls_enabled(true)
            .with_tls_cert_path("/path/to/cert.pem".to_string())
            .expect("Valid path")
            .with_tls_key_path("/path/to/key.pem".to_string())
            .expect("Valid path")
            .with_api_key("secret-key".to_string())
            .expect("Valid key")
            .with_token_expiration(Duration::from_secs(3600))
            .expect("Valid expiration")
            .build()
            .expect("Valid config");

        assert!(config.tls_enabled);
        assert_eq!(config.tls_cert_path, Some("/path/to/cert.pem".to_string()));
        assert_eq!(config.tls_key_path, Some("/path/to/key.pem".to_string()));
        assert_eq!(config.api_key, Some("secret-key".to_string()));
        assert_eq!(config.token_expiration, Duration::from_secs(3600));
    }

    #[test]
    fn test_circuit_breaker_config_builder() {
        let config = CircuitBreakerConfigBuilder::new()
            .with_enabled(true)
            .with_failure_threshold(5)
            .expect("Valid threshold")
            .with_success_threshold(3)
            .expect("Valid threshold")
            .with_timeout(Duration::from_secs(60))
            .expect("Valid timeout")
            .build()
            .expect("Valid config");

        assert!(config.enabled);
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }
}
