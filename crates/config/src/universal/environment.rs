//! Environment variable loading for the universal configuration system
//!
//! This module handles loading configuration from environment variables,
//! providing a centralized way to manage environment-based configuration.

use crate::universal::types::*;
use crate::universal::utils::{
    parse_comma_separated, parse_duration, parse_http_status_codes, parse_key_value_pairs,
};
use crate::universal::validation::ValidationExt;
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// Trait for loading configuration from environment variables
pub trait FromEnv: Sized {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self, ConfigError>;
}

impl FromEnv for UniversalServiceConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load discovery endpoints from environment
        if let Ok(endpoints) = env::var("SERVICE_DISCOVERY_ENDPOINTS") {
            config.discovery_endpoints = parse_comma_separated(&endpoints);
        }

        // Load default timeout
        if let Ok(timeout) = env::var("SERVICE_DEFAULT_TIMEOUT") {
            config.default_timeout =
                parse_duration(&timeout).map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load service mesh configuration
        config.service_mesh = ServiceMeshConfig::from_env()?;
        config.health_check = HealthCheckConfig::from_env()?;
        config.load_balancing = LoadBalancingConfig::from_env()?;
        config.security = SecurityConfig::from_env()?;

        // Load individual service configurations
        config.load_services_from_env()?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }
}

impl UniversalServiceConfig {
    /// Load services from environment variables
    fn load_services_from_env(&mut self) -> Result<(), ConfigError> {
        // Look for service configurations in environment
        for (key, value) in env::vars() {
            if key.starts_with("SERVICE_") && key.ends_with("_ENDPOINT") {
                let service_name = key
                    .strip_prefix("SERVICE_")
                    .ok_or_else(|| {
                        ConfigError::InvalidServiceConfig(format!(
                            "Invalid service key format: {key}"
                        ))
                    })?
                    .strip_suffix("_ENDPOINT")
                    .ok_or_else(|| {
                        ConfigError::InvalidServiceConfig(format!(
                            "Invalid service key format: {key}"
                        ))
                    })?
                    .to_lowercase();

                // Create service configuration
                let mut service_config = ServiceConfig::new();
                service_config.endpoints = parse_comma_separated(&value);

                // Load additional service configuration
                self.load_service_config_from_env(&service_name, &mut service_config)?;

                // Add to services
                self.services.insert(service_name, service_config);
            }
        }

        Ok(())
    }

    /// Load individual service configuration from environment
    fn load_service_config_from_env(
        &self,
        service_name: &str,
        config: &mut ServiceConfig,
    ) -> Result<(), ConfigError> {
        let prefix = format!("SERVICE_{}_", service_name.to_uppercase());

        // Load timeout
        if let Ok(timeout) = env::var(format!("{prefix}TIMEOUT")) {
            config.timeout = Some(
                parse_duration(&timeout).map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?,
            );
        }

        // Load metadata
        if let Ok(metadata) = env::var(format!("{prefix}METADATA")) {
            config.metadata = parse_key_value_pairs(&metadata)?;
        }

        // Load health check URL
        if let Ok(health_url) = env::var(format!("{prefix}HEALTH_CHECK_URL")) {
            config.health_check_url = Some(health_url);
        }

        // Load capabilities
        if let Ok(capabilities) = env::var(format!("{prefix}CAPABILITIES")) {
            config.capabilities = parse_comma_separated(&capabilities);
        }

        // Load weight
        if let Ok(weight) = env::var(format!("{prefix}WEIGHT")) {
            config.weight = Some(weight.parse().map_err(|_| {
                ConfigError::Environment(format!("Invalid weight value: {weight}"))
            })?);
        }

        // Load tags
        if let Ok(tags) = env::var(format!("{prefix}TAGS")) {
            config.tags = parse_comma_separated(&tags);
        }

        // Load priority
        if let Ok(priority) = env::var(format!("{prefix}PRIORITY")) {
            config.priority = Some(priority.parse().map_err(|_| {
                ConfigError::Environment(format!("Invalid priority value: {priority}"))
            })?);
        }

        // Load required flag
        if let Ok(required) = env::var(format!("{prefix}REQUIRED")) {
            config.required = required.parse().map_err(|_| {
                ConfigError::Environment(format!("Invalid required value: {required}"))
            })?;
        }

        Ok(())
    }
}

impl FromEnv for ServiceMeshConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load enabled flag
        if let Ok(enabled) = env::var("SERVICE_MESH_ENABLED") {
            config.enabled = enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid SERVICE_MESH_ENABLED value".to_string())
            })?;
        }

        // Load discovery endpoint
        if let Ok(endpoint) = env::var("SERVICE_MESH_DISCOVERY_ENDPOINT") {
            config.discovery_endpoint = Some(endpoint);
        }

        // Load registry type
        if let Ok(registry_type) = env::var("SERVICE_MESH_REGISTRY_TYPE") {
            config.registry_type = match registry_type.to_lowercase().as_str() {
                "inmemory" | "in_memory" => ServiceRegistryType::InMemory,
                "file" => {
                    let path = env::var("SERVICE_MESH_REGISTRY_FILE_PATH").map_err(|_| {
                        ConfigError::MissingRequired("SERVICE_MESH_REGISTRY_FILE_PATH".to_string())
                    })?;
                    ServiceRegistryType::File { path }
                }
                "network" => {
                    let endpoints_str = env::var("SERVICE_MESH_REGISTRY_NETWORK_ENDPOINTS")
                        .map_err(|_| {
                            ConfigError::MissingRequired(
                                "SERVICE_MESH_REGISTRY_NETWORK_ENDPOINTS".to_string(),
                            )
                        })?;
                    let endpoints = parse_comma_separated(&endpoints_str);
                    ServiceRegistryType::Network { endpoints }
                }
                "redis" => {
                    let connection_string = env::var("SERVICE_MESH_REGISTRY_REDIS_CONNECTION")
                        .map_err(|_| {
                            ConfigError::MissingRequired(
                                "SERVICE_MESH_REGISTRY_REDIS_CONNECTION".to_string(),
                            )
                        })?;
                    ServiceRegistryType::Redis { connection_string }
                }
                "database" => {
                    let connection_string = env::var("SERVICE_MESH_REGISTRY_DATABASE_CONNECTION")
                        .map_err(|_| {
                        ConfigError::MissingRequired(
                            "SERVICE_MESH_REGISTRY_DATABASE_CONNECTION".to_string(),
                        )
                    })?;
                    ServiceRegistryType::Database { connection_string }
                }
                "custom" => {
                    let config_str =
                        env::var("SERVICE_MESH_REGISTRY_CUSTOM_CONFIG").map_err(|_| {
                            ConfigError::MissingRequired(
                                "SERVICE_MESH_REGISTRY_CUSTOM_CONFIG".to_string(),
                            )
                        })?;
                    let config_map = parse_key_value_pairs(&config_str)?;
                    ServiceRegistryType::Custom { config: config_map }
                }
                _ => {
                    return Err(ConfigError::InvalidServiceConfig(format!(
                        "Unknown registry type: {registry_type}"
                    )))
                }
            };
        }

        // Load heartbeat interval
        if let Ok(interval) = env::var("SERVICE_MESH_HEARTBEAT_INTERVAL") {
            config.heartbeat_interval = parse_duration(&interval)
                .map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load service expiration
        if let Ok(expiration) = env::var("SERVICE_MESH_SERVICE_EXPIRATION") {
            config.service_expiration = parse_duration(&expiration)
                .map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load max services
        if let Ok(max_services) = env::var("SERVICE_MESH_MAX_SERVICES") {
            config.max_services = Some(max_services.parse().map_err(|_| {
                ConfigError::Environment("Invalid SERVICE_MESH_MAX_SERVICES value".to_string())
            })?);
        }

        // Load metrics enabled
        if let Ok(metrics_enabled) = env::var("SERVICE_MESH_METRICS_ENABLED") {
            config.metrics_enabled = metrics_enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid SERVICE_MESH_METRICS_ENABLED value".to_string())
            })?;
        }

        // Load namespace
        if let Ok(namespace) = env::var("SERVICE_MESH_NAMESPACE") {
            config.namespace = Some(namespace);
        }

        Ok(config)
    }
}

impl FromEnv for HealthCheckConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load enabled flag
        if let Ok(enabled) = env::var("HEALTH_CHECK_ENABLED") {
            config.enabled = enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid HEALTH_CHECK_ENABLED value".to_string())
            })?;
        }

        // Load interval
        if let Ok(interval) = env::var("HEALTH_CHECK_INTERVAL") {
            config.interval = parse_duration(&interval)
                .map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load timeout
        if let Ok(timeout) = env::var("HEALTH_CHECK_TIMEOUT") {
            config.timeout =
                parse_duration(&timeout).map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load retries
        if let Ok(retries) = env::var("HEALTH_CHECK_RETRIES") {
            config.retries = retries.parse().map_err(|_| {
                ConfigError::Environment("Invalid HEALTH_CHECK_RETRIES value".to_string())
            })?;
        }

        // Load path
        if let Ok(path) = env::var("HEALTH_CHECK_PATH") {
            config.path = path;
        }

        // Load expected codes
        if let Ok(codes) = env::var("HEALTH_CHECK_EXPECTED_CODES") {
            config.expected_codes = parse_http_status_codes(&codes)?;
        }

        // Load headers
        if let Ok(headers) = env::var("HEALTH_CHECK_HEADERS") {
            config.headers = parse_key_value_pairs(&headers)?;
        }

        Ok(config)
    }
}

impl FromEnv for LoadBalancingConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load strategy
        if let Ok(strategy) = env::var("LOAD_BALANCING_STRATEGY") {
            config.strategy = match strategy.to_lowercase().as_str() {
                "round_robin" | "roundrobin" => LoadBalancingStrategy::RoundRobin,
                "random" => LoadBalancingStrategy::Random,
                "least_connections" => LoadBalancingStrategy::LeastConnections,
                "weighted_round_robin" => LoadBalancingStrategy::WeightedRoundRobin,
                "health_based" => LoadBalancingStrategy::HealthBased,
                "response_time" => LoadBalancingStrategy::ResponseTime,
                _ => {
                    return Err(ConfigError::InvalidServiceConfig(format!(
                        "Unknown load balancing strategy: {strategy}"
                    )))
                }
            };
        }

        // Load sticky sessions
        if let Ok(sticky) = env::var("LOAD_BALANCING_STICKY_SESSIONS") {
            config.sticky_sessions = sticky.parse().map_err(|_| {
                ConfigError::Environment("Invalid LOAD_BALANCING_STICKY_SESSIONS value".to_string())
            })?;
        }

        // Load session timeout
        if let Ok(timeout) = env::var("LOAD_BALANCING_SESSION_TIMEOUT") {
            config.session_timeout =
                parse_duration(&timeout).map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        // Load circuit breaker config
        config.circuit_breaker = CircuitBreakerConfig::from_env()?;

        Ok(config)
    }
}

impl FromEnv for CircuitBreakerConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load enabled flag
        if let Ok(enabled) = env::var("CIRCUIT_BREAKER_ENABLED") {
            config.enabled = enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid CIRCUIT_BREAKER_ENABLED value".to_string())
            })?;
        }

        // Load failure threshold
        if let Ok(threshold) = env::var("CIRCUIT_BREAKER_FAILURE_THRESHOLD") {
            config.failure_threshold = threshold.parse().map_err(|_| {
                ConfigError::Environment(
                    "Invalid CIRCUIT_BREAKER_FAILURE_THRESHOLD value".to_string(),
                )
            })?;
        }

        // Load success threshold
        if let Ok(threshold) = env::var("CIRCUIT_BREAKER_SUCCESS_THRESHOLD") {
            config.success_threshold = threshold.parse().map_err(|_| {
                ConfigError::Environment(
                    "Invalid CIRCUIT_BREAKER_SUCCESS_THRESHOLD value".to_string(),
                )
            })?;
        }

        // Load timeout
        if let Ok(timeout) = env::var("CIRCUIT_BREAKER_TIMEOUT") {
            config.timeout =
                parse_duration(&timeout).map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        Ok(config)
    }
}

impl FromEnv for SecurityConfig {
    /// Load from environment variables
    fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Load TLS enabled
        if let Ok(tls_enabled) = env::var("SECURITY_TLS_ENABLED") {
            config.tls_enabled = tls_enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid SECURITY_TLS_ENABLED value".to_string())
            })?;
        }

        // Load TLS paths
        if let Ok(cert_path) = env::var("SECURITY_TLS_CERT_PATH") {
            config.tls_cert_path = Some(cert_path);
        }

        if let Ok(key_path) = env::var("SECURITY_TLS_KEY_PATH") {
            config.tls_key_path = Some(key_path);
        }

        if let Ok(ca_path) = env::var("SECURITY_CA_CERT_PATH") {
            config.ca_cert_path = Some(ca_path);
        }

        // Load mTLS enabled
        if let Ok(mtls_enabled) = env::var("SECURITY_MTLS_ENABLED") {
            config.mtls_enabled = mtls_enabled.parse().map_err(|_| {
                ConfigError::Environment("Invalid SECURITY_MTLS_ENABLED value".to_string())
            })?;
        }

        // Load API key
        if let Ok(api_key) = env::var("SECURITY_API_KEY") {
            config.api_key = Some(api_key);
        }

        // Load JWT secret
        if let Ok(jwt_secret) = env::var("SECURITY_JWT_SECRET") {
            config.jwt_secret = Some(jwt_secret);
        }

        // Load token expiration
        if let Ok(expiration) = env::var("SECURITY_TOKEN_EXPIRATION") {
            config.token_expiration = parse_duration(&expiration)
                .map_err(|e| ConfigError::InvalidTimeout(e.to_string()))?;
        }

        Ok(config)
    }
}

/// Environment variable helper functions
pub mod env_helpers {
    use super::*;

    /// Get environment variable with default value
    pub fn get_env_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get environment variable as boolean with default
    pub fn get_env_bool_or_default(key: &str, default: bool) -> Result<bool, ConfigError> {
        match env::var(key) {
            Ok(value) => value.parse().map_err(|_| {
                ConfigError::Environment(format!("Invalid boolean value for {key}: {value}"))
            }),
            Err(_) => Ok(default),
        }
    }

    /// Get environment variable as u32 with default
    pub fn get_env_u32_or_default(key: &str, default: u32) -> Result<u32, ConfigError> {
        match env::var(key) {
            Ok(value) => value.parse().map_err(|_| {
                ConfigError::Environment(format!("Invalid u32 value for {key}: {value}"))
            }),
            Err(_) => Ok(default),
        }
    }

    /// Get environment variable as Duration with default
    pub fn get_env_duration_or_default(
        key: &str,
        default: Duration,
    ) -> Result<Duration, ConfigError> {
        match env::var(key) {
            Ok(value) => parse_duration(&value).map_err(|e| {
                ConfigError::Environment(format!("Invalid duration value for {key}: {e}"))
            }),
            Err(_) => Ok(default),
        }
    }

    /// Get environment variable as comma-separated list with default
    pub fn get_env_list_or_default(key: &str, default: Vec<String>) -> Vec<String> {
        match env::var(key) {
            Ok(value) => parse_comma_separated(&value),
            Err(_) => default,
        }
    }

    /// Get environment variable as key-value pairs with default
    pub fn get_env_map_or_default(
        key: &str,
        default: HashMap<String, String>,
    ) -> Result<HashMap<String, String>, ConfigError> {
        match env::var(key) {
            Ok(value) => parse_key_value_pairs(&value),
            Err(_) => Ok(default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_service_mesh_config_from_env() {
        // Set up environment variables
        env::set_var("SERVICE_MESH_ENABLED", "true");
        env::set_var("SERVICE_MESH_DISCOVERY_ENDPOINT", "http://localhost:8500");
        env::set_var("SERVICE_MESH_REGISTRY_TYPE", "inmemory");
        env::set_var("SERVICE_MESH_HEARTBEAT_INTERVAL", "30s");
        env::set_var("SERVICE_MESH_SERVICE_EXPIRATION", "120s");
        env::set_var("SERVICE_MESH_MAX_SERVICES", "1000");
        env::set_var("SERVICE_MESH_METRICS_ENABLED", "true");

        let config = ServiceMeshConfig::from_env().unwrap();
        assert!(config.enabled);
        assert_eq!(
            config.discovery_endpoint,
            Some("http://localhost:8500".to_string())
        );
        assert!(matches!(
            config.registry_type,
            ServiceRegistryType::InMemory
        ));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(30));
        assert_eq!(config.service_expiration, Duration::from_secs(120));
        assert_eq!(config.max_services, Some(1000));
        assert!(config.metrics_enabled);

        // Clean up
        env::remove_var("SERVICE_MESH_ENABLED");
        env::remove_var("SERVICE_MESH_DISCOVERY_ENDPOINT");
        env::remove_var("SERVICE_MESH_REGISTRY_TYPE");
        env::remove_var("SERVICE_MESH_HEARTBEAT_INTERVAL");
        env::remove_var("SERVICE_MESH_SERVICE_EXPIRATION");
        env::remove_var("SERVICE_MESH_MAX_SERVICES");
        env::remove_var("SERVICE_MESH_METRICS_ENABLED");
    }

    #[test]
    fn test_health_check_config_from_env() {
        // Set up environment variables
        env::set_var("HEALTH_CHECK_ENABLED", "true");
        env::set_var("HEALTH_CHECK_INTERVAL", "30s");
        env::set_var("HEALTH_CHECK_TIMEOUT", "5s");
        env::set_var("HEALTH_CHECK_RETRIES", "3");
        env::set_var("HEALTH_CHECK_PATH", "/health");
        env::set_var("HEALTH_CHECK_EXPECTED_CODES", "200,201,204");

        let config = HealthCheckConfig::from_env().unwrap();
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.retries, 3);
        assert_eq!(config.path, "/health");
        assert_eq!(config.expected_codes, vec![200, 201, 204]);

        // Clean up
        env::remove_var("HEALTH_CHECK_ENABLED");
        env::remove_var("HEALTH_CHECK_INTERVAL");
        env::remove_var("HEALTH_CHECK_TIMEOUT");
        env::remove_var("HEALTH_CHECK_RETRIES");
        env::remove_var("HEALTH_CHECK_PATH");
        env::remove_var("HEALTH_CHECK_EXPECTED_CODES");
    }

    #[test]
    fn test_security_config_from_env() {
        // Set up environment variables
        env::set_var("SECURITY_TLS_ENABLED", "true");
        env::set_var("SECURITY_TLS_CERT_PATH", "/path/to/cert.pem");
        env::set_var("SECURITY_TLS_KEY_PATH", "/path/to/key.pem");
        env::set_var("SECURITY_MTLS_ENABLED", "false");
        env::set_var("SECURITY_API_KEY", "secret-key");
        env::set_var("SECURITY_JWT_SECRET", "jwt-secret");
        env::set_var("SECURITY_TOKEN_EXPIRATION", "1h");

        let config = SecurityConfig::from_env().unwrap();
        assert!(config.tls_enabled);
        assert_eq!(config.tls_cert_path, Some("/path/to/cert.pem".to_string()));
        assert_eq!(config.tls_key_path, Some("/path/to/key.pem".to_string()));
        assert!(!config.mtls_enabled);
        assert_eq!(config.api_key, Some("secret-key".to_string()));
        assert_eq!(config.jwt_secret, Some("jwt-secret".to_string()));
        assert_eq!(config.token_expiration, Duration::from_secs(3600));

        // Clean up
        env::remove_var("SECURITY_TLS_ENABLED");
        env::remove_var("SECURITY_TLS_CERT_PATH");
        env::remove_var("SECURITY_TLS_KEY_PATH");
        env::remove_var("SECURITY_MTLS_ENABLED");
        env::remove_var("SECURITY_API_KEY");
        env::remove_var("SECURITY_JWT_SECRET");
        env::remove_var("SECURITY_TOKEN_EXPIRATION");
    }

    #[test]
    fn test_env_helpers() {
        env::set_var("TEST_BOOL", "true");
        env::set_var("TEST_U32", "42");
        env::set_var("TEST_DURATION", "30s");
        env::set_var("TEST_LIST", "a,b,c");
        env::set_var("TEST_MAP", "key1=value1,key2=value2");

        assert_eq!(
            env_helpers::get_env_or_default("TEST_BOOL", "false"),
            "true"
        );
        assert_eq!(
            env_helpers::get_env_or_default("NONEXISTENT", "default"),
            "default"
        );

        assert!(env_helpers::get_env_bool_or_default("TEST_BOOL", false).unwrap());
        assert!(!env_helpers::get_env_bool_or_default("NONEXISTENT", false).unwrap());

        assert_eq!(
            env_helpers::get_env_u32_or_default("TEST_U32", 0).unwrap(),
            42
        );
        assert_eq!(
            env_helpers::get_env_u32_or_default("NONEXISTENT", 0).unwrap(),
            0
        );

        assert_eq!(
            env_helpers::get_env_duration_or_default("TEST_DURATION", Duration::from_secs(0))
                .unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            env_helpers::get_env_duration_or_default("NONEXISTENT", Duration::from_secs(0))
                .unwrap(),
            Duration::from_secs(0)
        );

        assert_eq!(
            env_helpers::get_env_list_or_default("TEST_LIST", vec![]),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            env_helpers::get_env_list_or_default("NONEXISTENT", vec![]),
            Vec::<String>::new()
        );

        let map = env_helpers::get_env_map_or_default("TEST_MAP", HashMap::new()).unwrap();
        assert_eq!(map.get("key1"), Some(&"value1".to_string()));
        assert_eq!(map.get("key2"), Some(&"value2".to_string()));

        // Clean up
        env::remove_var("TEST_BOOL");
        env::remove_var("TEST_U32");
        env::remove_var("TEST_DURATION");
        env::remove_var("TEST_LIST");
        env::remove_var("TEST_MAP");
    }
}
