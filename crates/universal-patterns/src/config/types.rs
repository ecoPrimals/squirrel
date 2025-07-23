//! Core configuration types and structures
//!
//! This module defines all the configuration types used throughout the universal
//! patterns framework, providing a unified type system for primal configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;
use uuid::Uuid;

/// Core configuration structure for all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Primal identification
    pub info: PrimalInfo,

    /// Network configuration
    pub network: NetworkConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Orchestration configuration
    pub orchestration: OrchestrationConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Environment-specific settings
    pub environment: EnvironmentConfig,

    /// Custom primal-specific configuration
    pub custom: HashMap<String, serde_json::Value>,
}

/// Primal identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name (e.g., "squirrel", "beardog", "songbird")
    pub name: String,

    /// Primal version
    pub version: String,

    /// Unique instance identifier
    pub instance_id: Uuid,

    /// Primal type/category
    pub primal_type: PrimalType,

    /// Human-readable description
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Types of primals in the ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Security and authentication management
    Security,
    /// Orchestration and task management
    Orchestration,
    /// Data storage and retrieval
    Storage,
    /// Compute and processing
    Compute,
    /// Custom/Other primal types
    Custom(String),
}

/// Network configuration for primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Primary listening address
    pub bind_address: String,

    /// Primary listening port
    pub port: u16,

    /// External/public address (for service discovery)
    pub public_address: Option<String>,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// Timeout settings
    pub timeouts: TimeoutConfig,

    /// Connection limits
    pub limits: ConnectionLimits,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: PathBuf,

    /// Private key file path
    pub key_file: PathBuf,

    /// CA certificate file path (for mutual TLS)
    pub ca_file: Option<PathBuf>,

    /// Require client certificates
    pub require_client_cert: bool,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection timeout (seconds)
    pub connect: u64,

    /// Request timeout (seconds)
    pub request: u64,

    /// Keep-alive timeout (seconds)
    pub keep_alive: u64,

    /// Idle timeout (seconds)
    pub idle: u64,
}

/// Connection limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Maximum requests per connection
    pub max_requests_per_connection: usize,

    /// Rate limiting (requests per second)
    pub rate_limit: Option<f64>,
}

/// Security configuration for Beardog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Beardog service endpoint
    pub beardog_endpoint: Option<Url>,

    /// Authentication method
    pub auth_method: AuthMethod,

    /// Token/credential storage
    pub credential_storage: CredentialStorage,

    /// Encryption settings
    pub encryption: EncryptionConfig,

    /// Enable audit logging
    pub audit_logging: bool,

    /// Security fallback settings
    pub fallback: SecurityFallback,
}

/// Security fallback configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFallback {
    /// Enable local fallback when Beardog unavailable
    pub enable_local_fallback: bool,

    /// Local authentication method for fallback
    pub local_auth_method: AuthMethod,

    /// Fallback timeout (seconds)
    pub fallback_timeout: u64,
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AuthMethod {
    /// No authentication (development only)
    None,
    /// Token-based authentication
    Token {
        /// Path to the token file
        token_file: PathBuf,
    },
    /// Certificate-based authentication
    Certificate {
        /// Path to the certificate file
        cert_file: PathBuf,
        /// Path to the private key file
        key_file: PathBuf,
    },
    /// Beardog-managed authentication
    Beardog {
        /// Service ID for Beardog authentication
        service_id: String,
    },
}

/// Credential storage options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialStorage {
    /// In-memory storage (not persistent)
    Memory,
    /// File-based storage
    File {
        /// Path to the credential storage file
        path: PathBuf,
    },
    /// Beardog-managed storage
    Beardog,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption for inter-primal communication
    pub enable_inter_primal: bool,

    /// Enable encryption for data at rest
    pub enable_at_rest: bool,

    /// Encryption algorithm
    pub algorithm: EncryptionAlgorithm,

    /// Key management
    pub key_management: KeyManagement,
}

/// Encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

/// Key management options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyManagement {
    /// File-based key storage
    File {
        /// Path to the key file
        path: PathBuf,
    },
    /// Beardog-managed keys
    Beardog,
    /// Environment variable
    Environment {
        /// Name of the environment variable containing the key
        var_name: String,
    },
}

/// Orchestration configuration for Songbird integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Songbird service endpoint
    pub songbird_endpoint: Option<Url>,

    /// Enable orchestration
    pub enabled: bool,

    /// Orchestration mode
    pub mode: OrchestrationMode,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,
}

/// Orchestration modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationMode {
    /// Standalone mode (no orchestration)
    Standalone,
    /// Managed mode (orchestrated by Songbird)
    Managed,
    /// Hybrid mode (partial orchestration)
    Hybrid,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check interval (seconds)
    pub interval: u64,

    /// Health check timeout (seconds)
    pub timeout: u64,

    /// Health check endpoint path
    pub endpoint: String,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Enable service discovery
    pub enabled: bool,

    /// Service discovery method
    pub method: ServiceDiscoveryMethod,

    /// Service registration TTL (seconds)
    pub ttl: u64,
}

/// Service discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceDiscoveryMethod {
    /// DNS-based discovery
    Dns {
        /// Domain name for DNS-based discovery
        domain: String,
    },
    /// File-based discovery
    File {
        /// Path to the service discovery file
        path: PathBuf,
    },
    /// Songbird-managed discovery
    Songbird,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: LogFormat,

    /// Log output destinations
    pub outputs: Vec<LogOutput>,

    /// Enable structured logging
    pub structured: bool,

    /// Enable tracing
    pub tracing: bool,
}

/// Log levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level logging
    Trace,
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging
    Error,
}

/// Log formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// Human-readable format
    Human,
    /// JSON format
    Json,
    /// Compact format
    Compact,
}

/// Log output destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File output
    File {
        /// Path to the log file
        path: PathBuf,
    },
    /// Syslog output
    Syslog,
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    /// Environment name (development, staging, production)
    pub name: String,

    /// Environment-specific variables
    pub variables: HashMap<String, String>,

    /// Feature flags
    pub features: HashMap<String, bool>,

    /// Resource limits
    pub resources: ResourceLimits,
}

/// Resource limits for the primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (MB)
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU usage (percentage)
    pub max_cpu_percent: Option<f64>,

    /// Maximum disk usage (MB)
    pub max_disk_mb: Option<u64>,

    /// Maximum file descriptors
    pub max_file_descriptors: Option<u64>,
}

/// Universal configuration for managing multiple primal instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPrimalConfig {
    /// Whether to enable auto-discovery of primals
    pub auto_discovery_enabled: bool,

    /// Individual primal instance configurations
    pub primal_instances: HashMap<String, PrimalInstanceConfig>,

    /// Multi-instance management settings
    pub multi_instance: MultiInstanceConfig,

    /// Instance lifecycle management
    pub lifecycle: InstanceLifecycleConfig,

    /// Port management configuration
    pub port_management: PortManagementConfig,

    /// Global timeout settings
    pub timeouts: TimeoutConfig,

    /// Logging and monitoring configuration
    pub monitoring: MonitoringConfig,

    /// Legacy configuration for backward compatibility
    pub squirrel: Option<PrimalConfig>,
    /// Legacy Beardog configuration
    pub beardog: Option<PrimalConfig>,
    /// Legacy Nestgate configuration
    pub nestgate: Option<PrimalConfig>,
    /// Legacy Toadstool configuration
    pub toadstool: Option<PrimalConfig>,
}

/// Configuration for a specific primal instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstanceConfig {
    /// Base URL for the primal service
    pub base_url: String,

    /// Instance identifier
    pub instance_id: String,

    /// User ID this instance serves
    pub user_id: String,

    /// Device ID this instance serves
    pub device_id: String,

    /// Security level for this instance
    pub security_level: String,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Custom headers for requests
    pub headers: HashMap<String, String>,

    /// Maximum request timeout
    pub timeout_seconds: u64,

    /// Connection pool settings
    pub connection_pool: ConnectionPoolConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Multi-instance management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceConfig {
    /// Maximum number of instances per primal type
    pub max_instances_per_type: usize,

    /// Maximum number of instances per user
    pub max_instances_per_user: usize,

    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,

    /// Instance failover configuration
    pub failover: FailoverConfig,

    /// Instance scaling configuration
    pub scaling: ScalingConfig,
}

/// Instance lifecycle management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceLifecycleConfig {
    /// Whether to automatically start instances
    pub auto_start: bool,

    /// Whether to automatically stop unused instances
    pub auto_stop: bool,

    /// Time before stopping unused instances
    pub idle_timeout_minutes: u64,

    /// Health check configuration
    pub health_monitoring: HealthMonitoringConfig,
}

/// Port management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortManagementConfig {
    /// Port range for dynamic allocation
    pub port_range: PortRange,

    /// Port lease duration
    pub lease_duration_minutes: u64,

    /// Port allocation strategy
    pub allocation_strategy: PortAllocationStrategy,

    /// Reserved ports that should not be allocated
    pub reserved_ports: Vec<u16>,
}

/// Connection pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections
    pub max_connections: usize,

    /// Minimum number of connections
    pub min_connections: usize,

    /// Connection timeout
    pub connection_timeout_seconds: u64,

    /// Idle timeout
    pub idle_timeout_seconds: u64,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,

    /// Least connections load balancing
    LeastConnections,

    /// Random load balancing
    Random,

    /// Weighted load balancing
    Weighted,

    /// Health-based load balancing
    HealthBased,
}

/// Failover configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    /// Whether to enable failover
    pub enabled: bool,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry delay
    pub retry_delay_seconds: u64,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Whether to enable auto-scaling
    pub auto_scaling_enabled: bool,

    /// Minimum number of instances
    pub min_instances: usize,

    /// Maximum number of instances
    pub max_instances: usize,

    /// CPU usage threshold for scaling up
    pub scale_up_cpu_threshold: f64,

    /// CPU usage threshold for scaling down
    pub scale_down_cpu_threshold: f64,

    /// Memory usage threshold for scaling up
    pub scale_up_memory_threshold: f64,

    /// Memory usage threshold for scaling down
    pub scale_down_memory_threshold: f64,
}

/// Port range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    /// Start port
    pub start: u16,

    /// End port
    pub end: u16,
}

/// Port allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortAllocationStrategy {
    /// Sequential allocation
    Sequential,

    /// Random allocation
    Random,

    /// Least recently used allocation
    LeastRecentlyUsed,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    /// Whether to enable health monitoring
    pub enabled: bool,

    /// Health check interval
    pub check_interval_seconds: u64,

    /// Health check timeout
    pub check_timeout_seconds: u64,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy
    pub recovery_threshold: u32,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether to enable circuit breaker
    pub enabled: bool,

    /// Failure threshold
    pub failure_threshold: u32,

    /// Success threshold
    pub success_threshold: u32,

    /// Timeout duration
    pub timeout_seconds: u64,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether to enable metrics collection
    pub metrics_enabled: bool,

    /// Metrics endpoint
    pub metrics_endpoint: String,

    /// Metrics port
    pub metrics_port: u16,

    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Whether to enable tracing
    pub enabled: bool,

    /// Tracing level
    pub level: String,

    /// Tracing format
    pub format: String,

    /// Whether to include file and line information
    pub include_location: bool,
}

impl PrimalInstanceConfig {
    /// Create a new primal instance configuration
    pub fn new(base_url: String, instance_id: String, user_id: String, device_id: String) -> Self {
        Self {
            base_url,
            instance_id,
            user_id,
            device_id,
            security_level: "standard".to_string(),
            api_key: None,
            headers: HashMap::new(),
            timeout_seconds: 30,
            connection_pool: ConnectionPoolConfig {
                max_connections: 10,
                min_connections: 1,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 300,
            },
            health_check: HealthCheckConfig {
                enabled: true,
                interval: 30,
                timeout: 10,
                endpoint: "/health".to_string(),
            },
        }
    }

    /// Set the API key for this instance
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the security level for this instance
    pub fn with_security_level(mut self, level: String) -> Self {
        self.security_level = level;
        self
    }

    /// Add a custom header
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
}

impl PortRange {
    /// Create a new port range
    pub fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// Check if a port is within this range
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }

    /// Get the size of this port range
    pub fn size(&self) -> usize {
        (self.end - self.start + 1) as usize
    }

    /// Get an iterator over all ports in this range
    pub fn ports(&self) -> impl Iterator<Item = u16> {
        self.start..=self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_instance_config_builder() {
        let config = PrimalInstanceConfig::new(
            "http://localhost:8080".to_string(),
            "instance-1".to_string(),
            "user-123".to_string(),
            "device-456".to_string(),
        )
        .with_api_key("test-key".to_string())
        .with_security_level("high".to_string())
        .with_header("X-Custom".to_string(), "value".to_string());

        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.instance_id, "instance-1");
        assert_eq!(config.user_id, "user-123");
        assert_eq!(config.device_id, "device-456");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.security_level, "high");
        assert_eq!(config.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_port_range() {
        let range = PortRange::new(8000, 8010);

        assert!(range.contains(8005));
        assert!(!range.contains(7999));
        assert!(!range.contains(8011));
        assert_eq!(range.size(), 11);

        let ports: Vec<u16> = range.ports().collect();
        assert_eq!(ports.len(), 11);
        assert_eq!(ports[0], 8000);
        assert_eq!(ports[10], 8010);
    }

    #[test]
    fn test_primal_type_serialization() {
        let coordinator = PrimalType::Coordinator;
        let custom = PrimalType::Custom("my-primal".to_string());

        let coordinator_json = serde_json::to_string(&coordinator).unwrap();
        let custom_json = serde_json::to_string(&custom).unwrap();

        assert_eq!(coordinator_json, "\"Coordinator\"");
        assert!(custom_json.contains("my-primal"));
    }

    #[test]
    fn test_auth_method_variants() {
        let none = AuthMethod::None;
        let token = AuthMethod::Token {
            token_file: PathBuf::from("/path/to/token"),
        };
        let cert = AuthMethod::Certificate {
            cert_file: PathBuf::from("/cert"),
            key_file: PathBuf::from("/key"),
        };
        let beardog = AuthMethod::Beardog {
            service_id: "service-123".to_string(),
        };

        match none {
            AuthMethod::None => {}
            _ => panic!("Expected None variant"),
        }

        match token {
            AuthMethod::Token { .. } => {}
            _ => panic!("Expected Token variant"),
        }

        match cert {
            AuthMethod::Certificate { .. } => {}
            _ => panic!("Expected Certificate variant"),
        }

        match beardog {
            AuthMethod::Beardog { .. } => {}
            _ => panic!("Expected Beardog variant"),
        }
    }
}
