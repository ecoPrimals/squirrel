//! Configuration types for the universal service configuration system
//!
//! This module contains all the types, structs, and enums used throughout
//! the universal configuration system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid port: {0}")]
    InvalidPort(u16),

    #[error("Invalid timeout: {0}")]
    InvalidTimeout(String),

    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid service configuration: {0}")]
    InvalidServiceConfig(String),

    #[error("Environment variable error: {0}")]
    Environment(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Universal service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceConfig {
    /// Service discovery endpoints
    pub discovery_endpoints: Vec<String>,
    /// Default service timeout
    pub default_timeout: Duration,
    /// Service-specific configurations
    pub services: HashMap<String, ServiceConfig>,
    /// Service mesh configuration
    pub service_mesh: ServiceMeshConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Load balancing configuration
    pub load_balancing: LoadBalancingConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

/// Individual service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service endpoints (can be multiple for load balancing)
    pub endpoints: Vec<String>,
    /// Service timeout
    pub timeout: Option<Duration>,
    /// Service-specific metadata
    pub metadata: HashMap<String, String>,
    /// Health check URL
    pub health_check_url: Option<String>,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service weight for load balancing
    pub weight: Option<f32>,
    /// Service tags
    pub tags: Vec<String>,
    /// Service priority
    pub priority: Option<u32>,
    /// Whether service is required
    pub required: bool,
}

/// Service mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Enable service mesh
    pub enabled: bool,
    /// Service mesh discovery endpoint
    pub discovery_endpoint: Option<String>,
    /// Service mesh registry type
    pub registry_type: ServiceRegistryType,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Service expiration timeout
    pub service_expiration: Duration,
    /// Maximum number of services to track
    pub max_services: Option<usize>,
    /// Enable service mesh metrics
    pub metrics_enabled: bool,
    /// Service mesh namespace
    pub namespace: Option<String>,
}

/// Service registry type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceRegistryType {
    /// In-memory registry
    InMemory,
    /// File-based registry
    File { path: String },
    /// Network-based registry (e.g., Consul, etcd)
    Network { endpoints: Vec<String> },
    /// Redis-based registry
    Redis { connection_string: String },
    /// Database-based registry
    Database { connection_string: String },
    /// Custom registry
    Custom { config: HashMap<String, String> },
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Number of retries before marking unhealthy
    pub retries: u32,
    /// Health check path
    pub path: String,
    /// Expected HTTP status codes
    pub expected_codes: Vec<u16>,
    /// Health check headers
    pub headers: HashMap<String, String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy
    pub strategy: LoadBalancingStrategy,
    /// Sticky sessions configuration
    pub sticky_sessions: bool,
    /// Session affinity timeout
    pub session_timeout: Duration,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round robin
    RoundRobin,
    /// Random selection
    Random,
    /// Least connections
    LeastConnections,
    /// Weighted round robin
    WeightedRoundRobin,
    /// Health-based selection
    HealthBased,
    /// Response time based
    ResponseTime,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Timeout before attempting to close circuit
    pub timeout: Duration,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    /// TLS key path
    pub tls_key_path: Option<String>,
    /// CA certificate path
    pub ca_cert_path: Option<String>,
    /// Enable mTLS
    pub mtls_enabled: bool,
    /// API key configuration
    pub api_key: Option<String>,
    /// JWT configuration
    pub jwt_secret: Option<String>,
    /// Token expiration time
    pub token_expiration: Duration,
}

// Default implementations
impl Default for UniversalServiceConfig {
    fn default() -> Self {
        Self {
            discovery_endpoints: Self::default_discovery_endpoints(),
            default_timeout: Duration::from_secs(30),
            services: HashMap::new(),
            service_mesh: ServiceMeshConfig::default(),
            health_check: HealthCheckConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

impl Default for ServiceMeshConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_endpoint: None,
            registry_type: ServiceRegistryType::InMemory,
            heartbeat_interval: Duration::from_secs(30),
            service_expiration: Duration::from_secs(120),
            max_services: Some(1000),
            metrics_enabled: true,
            namespace: None,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            path: "/health".to_string(),
            expected_codes: vec![200],
            headers: HashMap::new(),
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            sticky_sessions: false,
            session_timeout: Duration::from_secs(3600),
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            ca_cert_path: None,
            mtls_enabled: false,
            api_key: None,
            jwt_secret: None,
            token_expiration: Duration::from_secs(3600),
        }
    }
}

impl UniversalServiceConfig {
    /// Create new universal service configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Default discovery endpoints
    fn default_discovery_endpoints() -> Vec<String> {
        vec![
            "http://localhost:8500".to_string(), // Consul
            "http://localhost:2379".to_string(), // etcd
            "http://localhost:8080".to_string(), // Local service mesh
        ]
    }

    /// Check if service exists
    pub fn has_service(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// Get service configuration
    pub fn get_service(&self, name: &str) -> Option<&ServiceConfig> {
        self.services.get(name)
    }

    /// Add service configuration
    pub fn add_service(&mut self, name: String, config: ServiceConfig) {
        self.services.insert(name, config);
    }

    /// Remove service configuration
    pub fn remove_service(&mut self, name: &str) -> Option<ServiceConfig> {
        self.services.remove(name)
    }

    /// Get all service names
    pub fn service_names(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

impl ServiceConfig {
    /// Create new service configuration
    pub fn new() -> Self {
        Self {
            endpoints: vec![],
            timeout: None,
            metadata: HashMap::new(),
            health_check_url: None,
            capabilities: vec![],
            weight: None,
            tags: vec![],
            priority: None,
            required: false,
        }
    }

    /// Check if service has capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// Add capability
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Remove capability
    pub fn remove_capability(&mut self, capability: &str) {
        self.capabilities.retain(|c| c != capability);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Remove metadata
    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        self.metadata.remove(key)
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self::new()
    }
}
