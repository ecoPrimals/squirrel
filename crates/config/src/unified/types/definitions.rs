// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core configuration structs and enums for the unified config system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::unified::timeouts::TimeoutConfig;

/// Canonical unified configuration for the entire Squirrel ecosystem
///
/// This structure consolidates all configuration across Squirrel, providing
/// a single source of truth with clear hierarchical organization.
///
/// # Configuration Hierarchy
///
/// The configuration is organized into logical domains:
/// - **System**: Core system settings
/// - **Network**: Network and connectivity
/// - **Security**: Security and authentication
/// - **MCP**: MCP protocol configuration
/// - **AI**: AI provider configuration
/// - **Service Mesh**: Service discovery and routing
/// - **Timeouts**: All timeout values (NEW - replaces 2,498 hardcoded values)
/// - **Monitoring**: Observability and metrics
/// - **Features**: Feature flags
///
/// # Example Usage
///
/// ```ignore
/// use squirrel_mcp_config::unified::{SquirrelUnifiedConfig, ConfigLoader};
///
/// // Load complete configuration
/// let config = ConfigLoader::load()?;
///
/// // Access configuration domains
/// let connection_timeout = config.timeouts.connection_timeout();
/// let ai_endpoint = &config.ai.default_endpoint;
/// let security_enabled = config.security.enabled;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquirrelUnifiedConfig {
    /// System-level configuration
    pub system: SystemConfig,

    /// Network and connectivity configuration
    pub network: NetworkConfig,

    /// Security settings
    pub security: SecurityConfig,

    /// MCP protocol configuration
    pub mcp: McpConfig,

    /// AI provider configuration
    pub ai: AiProvidersConfig,

    /// Service mesh configuration
    pub service_mesh: ServiceMeshConfig,

    /// Timeout configuration (NEW - consolidates 2,498 hardcoded values)
    pub timeouts: TimeoutConfig,

    /// Monitoring and observability
    pub monitoring: MonitoringConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Load balancing configuration
    pub load_balancing: LoadBalancingConfig,

    /// Feature flags
    pub features: FeatureFlags,

    /// Custom primal-specific configuration
    ///
    /// Allows primals to store custom configuration without modifying
    /// the core structure. Use sparingly - prefer adding to appropriate domain.
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Instance identifier (unique per instance)
    #[serde(default = "crate::unified::types::defaults::default_instance_id")]
    pub instance_id: String,

    /// Environment (development, staging, production)
    #[serde(default)]
    pub environment: String,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "crate::unified::types::defaults::default_log_level")]
    pub log_level: String,

    /// Working directory
    #[serde(default = "crate::unified::types::defaults::default_work_dir")]
    pub work_dir: PathBuf,

    /// Data directory
    #[serde(default = "crate::unified::types::defaults::default_data_dir")]
    pub data_dir: PathBuf,

    /// Plugin directory
    #[serde(default = "crate::unified::types::defaults::default_plugin_dir")]
    pub plugin_dir: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address
    #[serde(default = "crate::unified::types::defaults::default_bind_address")]
    pub bind_address: String,

    /// HTTP port
    #[serde(default = "crate::unified::types::defaults::default_http_port")]
    pub http_port: u16,

    /// WebSocket port
    #[serde(default = "crate::unified::types::defaults::default_websocket_port")]
    pub websocket_port: u16,

    /// gRPC port
    #[serde(default = "crate::unified::types::defaults::default_grpc_port")]
    pub grpc_port: u16,

    /// Maximum connections
    #[serde(default = "crate::unified::types::defaults::default_max_connections")]
    pub max_connections: u32,

    /// Enable TLS
    #[serde(default)]
    pub enable_tls: bool,

    /// TLS certificate path
    #[serde(default)]
    pub tls_cert_path: Option<PathBuf>,

    /// TLS key path
    #[serde(default)]
    pub tls_key_path: Option<PathBuf>,
}

/// Security configuration
///
/// Consolidated from universal and unified modules - contains both transport
/// security (TLS/mTLS) and application security (authentication/authorization).
///
/// **Nov 9, 2025 Update**: Consolidated additional fields from MCP config and security manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security features
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Authentication required
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub require_authentication: bool,

    /// Authorization enabled
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_authorization: bool,

    /// JWT secret (from environment)
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// JWT token expiration in seconds (from universal)
    #[serde(default = "crate::unified::types::defaults::default_token_expiration")]
    pub token_expiration_secs: u64,

    /// API keys (from environment) - supports multiple keys
    #[serde(default)]
    pub api_keys: Vec<String>,

    /// Allowed origins for CORS
    #[serde(default)]
    pub allowed_origins: Vec<String>,

    /// Enable TLS for transport security (from universal)
    #[serde(default)]
    pub tls_enabled: bool,

    /// TLS certificate path (from universal)
    #[serde(default)]
    pub tls_cert_path: Option<String>,

    /// TLS private key path (from universal)
    #[serde(default)]
    pub tls_key_path: Option<String>,

    /// CA certificate path for mTLS (from universal)
    #[serde(default)]
    pub ca_cert_path: Option<String>,

    /// Enable mutual TLS (mTLS) (from universal)
    #[serde(default)]
    pub mtls_enabled: bool,

    // ===== Consolidated from MCP modules (Nov 9, 2025) =====
    /// Default encryption format for MCP protocol
    ///
    /// **Consolidated from**: `crates/core/mcp/src/config/mod.rs`
    ///
    /// Specifies the encryption algorithm for MCP protocol encryption.
    /// Default: "AES256GCM"
    #[serde(default = "crate::unified::types::defaults::default_encryption_format")]
    pub encryption_default_format: String,

    /// Enable audit logging
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Controls whether security events are logged for audit purposes.
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_audit: bool,

    /// Enable encryption features
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Master toggle for encryption features in the security subsystem.
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_encryption: bool,

    /// Enable RBAC (Role-Based Access Control)
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Controls whether RBAC enforcement is enabled.
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_rbac: bool,

    /// Token expiry in minutes
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Default expiration time for security tokens.
    /// Default: 60 minutes
    #[serde(default = "crate::unified::types::defaults::default_token_expiry_minutes")]
    pub token_expiry_minutes: u64,
}

/// MCP protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Protocol version
    #[serde(default = "crate::unified::types::defaults::default_mcp_version")]
    pub version: String,

    /// Maximum message size in bytes
    #[serde(default = "crate::unified::types::defaults::default_max_message_size")]
    pub max_message_size: usize,

    /// Buffer size in bytes
    #[serde(default = "crate::unified::types::defaults::default_buffer_size")]
    pub buffer_size: usize,

    /// Enable compression
    #[serde(default)]
    pub enable_compression: bool,

    /// Compression level (1-9)
    #[serde(default = "crate::unified::types::defaults::default_compression_level")]
    pub compression_level: u32,
}

/// AI providers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvidersConfig {
    /// Default AI endpoint
    #[serde(default)]
    pub default_endpoint: String,

    /// Provider-specific configuration
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Enable local providers (any OpenAI-compatible server: Ollama, llama.cpp, vLLM, etc.)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_local: bool,

    /// Enable cloud providers (OpenAI, Anthropic, etc.)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_cloud: bool,

    /// Maximum concurrent requests
    #[serde(default = "crate::unified::types::defaults::default_max_concurrent_ai_requests")]
    pub max_concurrent_requests: usize,
}

/// Individual provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider endpoint URL
    pub endpoint: String,

    /// API key (if required)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Enabled flag
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Provider-specific settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Service mesh configuration
///
/// Consolidated from universal and unified modules - contains best features from both.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Enable service mesh
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Discovery endpoints (from unified - supports multiple endpoints)
    #[serde(default)]
    pub discovery_endpoints: Vec<String>,

    /// Service registry type (from universal - rich abstraction)
    #[serde(default = "crate::unified::types::defaults::default_registry_type")]
    pub registry_type: ServiceRegistryType,

    /// Maximum services to track
    #[serde(default = "crate::unified::types::defaults::default_max_services")]
    pub max_services: usize,

    /// Health check interval in seconds
    #[serde(default = "crate::unified::types::defaults::default_health_check_interval")]
    pub health_check_interval_secs: u64,

    /// Heartbeat interval in seconds (from universal - for active health checks)
    #[serde(default = "crate::unified::types::defaults::default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Service expiration timeout in seconds (from universal - when to remove stale services)
    #[serde(default = "crate::unified::types::defaults::default_service_expiration")]
    pub service_expiration_secs: u64,

    /// Enable automatic failover (from unified)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_failover: bool,

    /// Enable service mesh metrics (from universal)
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub metrics_enabled: bool,

    /// Service mesh namespace (from universal - for multi-tenancy)
    #[serde(default)]
    pub namespace: Option<String>,
}

/// Service registry type
///
/// Defines how services are discovered and tracked. Moved from universal module.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServiceRegistryType {
    /// In-memory registry (default for development)
    InMemory,

    /// File-based registry
    File {
        /// Path to registry file
        path: String,
    },

    /// Network-based registry (e.g., Consul, etcd)
    Network {
        /// Registry endpoints
        endpoints: Vec<String>,
    },

    /// Redis-based registry
    Redis {
        /// Redis connection string
        connection_string: String,
    },
    /// Database-based registry
    Database {
        /// Database connection string
        connection_string: String,
    },

    /// Custom registry with flexible configuration
    Custom {
        /// Custom configuration key-value pairs
        config: HashMap<String, String>,
    },
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Metrics endpoint
    #[serde(default = "crate::unified::types::defaults::default_metrics_endpoint")]
    pub metrics_endpoint: String,

    /// Tracing endpoint
    #[serde(default)]
    pub tracing_endpoint: Option<String>,

    /// Enable Prometheus export
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_prometheus: bool,

    /// Prometheus port
    #[serde(default = "crate::unified::types::defaults::default_prometheus_port")]
    pub prometheus_port: u16,
}

/// Feature flags for optional functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable experimental features
    #[serde(default)]
    pub experimental: bool,

    /// Enable plugin system
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_plugins: bool,

    /// Enable federation
    #[serde(default)]
    pub enable_federation: bool,

    /// Enable advanced routing
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_advanced_routing: bool,

    /// Custom feature flags
    #[serde(default)]
    pub custom: HashMap<String, bool>,
}

/// Database configuration
///
/// Consolidated from core/ and environment.rs modules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection string (env: DATABASE_URL)
    #[serde(default = "crate::unified::types::defaults::default_database_url")]
    pub connection_string: String,

    /// Maximum number of connections (env: DB_MAX_CONNECTIONS)
    #[serde(default = "crate::unified::types::defaults::default_max_db_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds (env: DB_TIMEOUT)
    #[serde(default = "crate::unified::types::defaults::default_db_timeout")]
    pub timeout_seconds: u64,

    /// Database backend type
    #[serde(default)]
    pub backend: DatabaseBackend,

    /// Enable connection pooling
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enable_pooling: bool,

    /// Pool size
    #[serde(default = "crate::unified::types::defaults::default_pool_size")]
    pub pool_size: u32,
}

/// Database backend options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DatabaseBackend {
    /// NestGate distributed storage
    #[serde(rename = "nestgate")]
    NestGate,

    /// PostgreSQL database
    #[serde(rename = "postgres")]
    PostgreSQL,

    /// SQLite database
    #[serde(rename = "sqlite")]
    #[default]
    SQLite,

    /// In-memory database (for testing)
    #[serde(rename = "memory")]
    Memory,
}

/// Load balancing configuration
///
/// Migrated from universal/ system - provides sophisticated load balancing strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy
    #[serde(default)]
    pub strategy: LoadBalancingStrategy,

    /// Enable sticky sessions
    #[serde(default)]
    pub sticky_sessions: bool,

    /// Session affinity timeout (seconds)
    #[serde(default = "crate::unified::types::defaults::default_session_timeout")]
    pub session_timeout_secs: u64,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Health-based routing
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub health_based_routing: bool,

    /// Retry failed requests
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub retry_failed: bool,

    /// Maximum retries
    #[serde(default = "crate::unified::types::defaults::default_max_retries")]
    pub max_retries: u32,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoadBalancingStrategy {
    /// Round robin distribution
    #[default]
    RoundRobin,

    /// Random selection
    Random,

    /// Least connections first
    LeastConnections,

    /// Weighted round robin
    WeightedRoundRobin,

    /// Health-based selection
    HealthBased,

    /// Response time based
    ResponseTime,

    /// Consistent hashing
    ConsistentHash,
}

/// Circuit breaker configuration (already in unified/, ensuring completeness)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    #[serde(default = "crate::unified::types::defaults::default_true")]
    pub enabled: bool,

    /// Failure threshold before opening circuit
    #[serde(default = "crate::unified::types::defaults::default_failure_threshold")]
    pub failure_threshold: u32,

    /// Success threshold to close circuit
    #[serde(default = "crate::unified::types::defaults::default_success_threshold")]
    pub success_threshold: u32,

    /// Timeout before attempting to close circuit (seconds)
    #[serde(default = "crate::unified::types::defaults::default_circuit_timeout")]
    pub timeout_secs: u64,

    /// Half-open state max requests
    #[serde(default = "crate::unified::types::defaults::default_half_open_requests")]
    pub half_open_max_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified::timeouts::TimeoutConfig;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn assert_serde_json_roundtrip<T>(value: &T)
    where
        T: Serialize + DeserializeOwned + core::fmt::Debug,
    {
        let json = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_value(value).unwrap(),
            serde_json::to_value(&back).unwrap(),
            "JSON round-trip mismatch for type {}",
            core::any::type_name::<T>()
        );
    }

    fn sample_timeouts() -> TimeoutConfig {
        TimeoutConfig {
            connection_timeout_secs: 30,
            request_timeout_secs: 60,
            health_check_timeout_secs: 5,
            operation_timeout_secs: 10,
            database_timeout_secs: 30,
            heartbeat_interval_secs: 30,
            discovery_timeout_secs: 10,
            ai_inference_timeout_secs: 120,
            plugin_load_timeout_secs: 15,
            session_timeout_secs: 3600,
            custom_timeouts: HashMap::from([("edge_op".to_string(), u64::MAX)]),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn sample_squirrel_unified_config() -> SquirrelUnifiedConfig {
        let mut providers = HashMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                endpoint: "https://api.example/v1".to_string(),
                api_key: Some("secret".to_string()),
                enabled: true,
                settings: HashMap::from([("temperature".to_string(), serde_json::json!(0.7_f64))]),
            },
        );

        SquirrelUnifiedConfig {
            system: SystemConfig {
                instance_id: "test-instance".to_string(),
                environment: "staging".to_string(),
                log_level: "info".to_string(),
                work_dir: PathBuf::from("/var/work"),
                data_dir: PathBuf::from("/var/data"),
                plugin_dir: PathBuf::from("/var/plugins"),
            },
            network: NetworkConfig {
                bind_address: "0.0.0.0".to_string(),
                http_port: 8080,
                websocket_port: 8081,
                grpc_port: 50051,
                max_connections: 500,
                enable_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            security: SecurityConfig {
                enabled: true,
                require_authentication: true,
                enable_authorization: true,
                jwt_secret: None,
                token_expiration_secs: 3600,
                api_keys: vec!["k1".to_string()],
                allowed_origins: vec!["https://app.example".to_string()],
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
                ca_cert_path: None,
                mtls_enabled: false,
                encryption_default_format: "AES256GCM".to_string(),
                enable_audit: true,
                enable_encryption: true,
                enable_rbac: true,
                token_expiry_minutes: 60,
            },
            mcp: McpConfig {
                version: "1.0".to_string(),
                max_message_size: 16 * 1024 * 1024,
                buffer_size: 8192,
                enable_compression: false,
                compression_level: 6,
            },
            ai: AiProvidersConfig {
                default_endpoint: "https://ai.default".to_string(),
                providers,
                enable_local: true,
                enable_cloud: true,
                max_concurrent_requests: 10,
            },
            service_mesh: ServiceMeshConfig {
                enabled: true,
                discovery_endpoints: vec!["http://disc:9000".to_string()],
                registry_type: ServiceRegistryType::InMemory,
                max_services: 1000,
                health_check_interval_secs: 30,
                heartbeat_interval_secs: 15,
                service_expiration_secs: 90,
                enable_failover: true,
                metrics_enabled: true,
                namespace: Some("ns1".to_string()),
            },
            timeouts: sample_timeouts(),
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_endpoint: "/metrics".to_string(),
                tracing_endpoint: Some("http://trace:4317".to_string()),
                enable_prometheus: true,
                prometheus_port: 9090,
            },
            database: DatabaseConfig {
                connection_string: "sqlite::memory:".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
                backend: DatabaseBackend::SQLite,
                enable_pooling: true,
                pool_size: 5,
            },
            load_balancing: LoadBalancingConfig {
                strategy: LoadBalancingStrategy::RoundRobin,
                sticky_sessions: false,
                session_timeout_secs: 3600,
                circuit_breaker: CircuitBreakerConfig {
                    enabled: true,
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout_secs: 60,
                    half_open_max_requests: 3,
                },
                health_based_routing: true,
                retry_failed: true,
                max_retries: 3,
            },
            features: FeatureFlags {
                experimental: false,
                enable_plugins: true,
                enable_federation: false,
                enable_advanced_routing: true,
                custom: HashMap::from([("beta_ui".to_string(), true)]),
            },
            custom: HashMap::from([(
                "primal".to_string(),
                serde_json::json!({ "mode": "strict" }),
            )]),
        }
    }

    #[test]
    fn database_backend_default_is_sqlite() {
        assert!(matches!(
            DatabaseBackend::default(),
            DatabaseBackend::SQLite
        ));
    }

    #[test]
    fn database_backend_serializes_roundtrip() {
        for backend in [
            DatabaseBackend::NestGate,
            DatabaseBackend::PostgreSQL,
            DatabaseBackend::SQLite,
            DatabaseBackend::Memory,
        ] {
            assert_serde_json_roundtrip(&backend);
        }
    }

    #[test]
    fn load_balancing_strategy_default_is_round_robin() {
        assert!(matches!(
            LoadBalancingStrategy::default(),
            LoadBalancingStrategy::RoundRobin
        ));
    }

    #[test]
    fn load_balancing_strategy_serializes_roundtrip() {
        for strategy in [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::WeightedRoundRobin,
            LoadBalancingStrategy::HealthBased,
            LoadBalancingStrategy::ResponseTime,
            LoadBalancingStrategy::ConsistentHash,
        ] {
            assert_serde_json_roundtrip(&strategy);
        }
    }

    #[test]
    fn system_config_constructed_and_serializes_roundtrip() {
        let cfg = SystemConfig {
            instance_id: "id-1".to_string(),
            environment: "dev".to_string(),
            log_level: "trace".to_string(),
            work_dir: PathBuf::from("/w"),
            data_dir: PathBuf::from("/d"),
            plugin_dir: PathBuf::from("/p"),
        };
        assert_serde_json_roundtrip(&cfg);
        let debug = format!("{cfg:?}");
        assert!(debug.contains("id-1"));
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn system_config_deserializes_minimal_json_with_serde_defaults() {
        let cfg: SystemConfig = serde_json::from_str("{}").unwrap();
        assert!(!cfg.instance_id.is_empty());
        let debug = format!("{cfg:?}");
        assert!(debug.contains("instance_id"));
    }

    #[test]
    fn network_config_serializes_roundtrip() {
        let cfg = NetworkConfig {
            bind_address: "127.0.0.1".to_string(),
            http_port: 3000,
            websocket_port: 3001,
            grpc_port: 3002,
            max_connections: 42,
            enable_tls: true,
            tls_cert_path: Some(PathBuf::from("/tmp/cert.pem")),
            tls_key_path: Some(PathBuf::from("/tmp/key.pem")),
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn network_config_accepts_port_boundaries() {
        let cfg = NetworkConfig {
            bind_address: "::1".to_string(),
            http_port: u16::MAX,
            websocket_port: 0,
            grpc_port: 1,
            max_connections: u32::MAX,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        };
        assert_serde_json_roundtrip(&cfg);
        assert_eq!(cfg.http_port, u16::MAX);
        assert_eq!(cfg.max_connections, u32::MAX);
    }

    #[test]
    fn security_config_serializes_roundtrip() {
        let cfg = SecurityConfig {
            enabled: false,
            require_authentication: false,
            enable_authorization: false,
            jwt_secret: Some("jwt".to_string()),
            token_expiration_secs: 7200,
            api_keys: vec![],
            allowed_origins: vec![],
            tls_enabled: true,
            tls_cert_path: Some("/c".to_string()),
            tls_key_path: Some("/k".to_string()),
            ca_cert_path: Some("/ca".to_string()),
            mtls_enabled: true,
            encryption_default_format: "CHACHA20".to_string(),
            enable_audit: false,
            enable_encryption: false,
            enable_rbac: false,
            token_expiry_minutes: 120,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn security_config_numeric_fields_support_large_values() {
        let cfg = SecurityConfig {
            enabled: true,
            require_authentication: true,
            enable_authorization: true,
            jwt_secret: None,
            token_expiration_secs: u64::MAX,
            api_keys: vec![],
            allowed_origins: vec![],
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            ca_cert_path: None,
            mtls_enabled: false,
            encryption_default_format: "AES256GCM".to_string(),
            enable_audit: true,
            enable_encryption: true,
            enable_rbac: true,
            token_expiry_minutes: u64::MAX,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn mcp_config_serializes_roundtrip() {
        let cfg = McpConfig {
            version: "2.0".to_string(),
            max_message_size: 1024,
            buffer_size: 512,
            enable_compression: true,
            compression_level: 9,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn mcp_config_message_and_buffer_size_boundaries() {
        let cfg = McpConfig {
            version: "1.0".to_string(),
            max_message_size: 0,
            buffer_size: usize::MAX,
            enable_compression: false,
            compression_level: 0,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn ai_providers_config_serializes_roundtrip() {
        let mut providers = HashMap::new();
        providers.insert(
            "local".to_string(),
            ProviderConfig {
                endpoint: "http://127.0.0.1:11434".to_string(),
                api_key: None,
                enabled: true,
                settings: HashMap::new(),
            },
        );
        let cfg = AiProvidersConfig {
            default_endpoint: "".to_string(),
            providers,
            enable_local: false,
            enable_cloud: false,
            max_concurrent_requests: usize::MAX,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn provider_config_preserves_float_setting_within_epsilon() {
        let mut settings = HashMap::new();
        let expected = 1.5_f64;
        settings.insert("ratio".to_string(), serde_json::json!(expected));
        let cfg = ProviderConfig {
            endpoint: "http://x".to_string(),
            api_key: None,
            enabled: true,
            settings,
        };
        assert_serde_json_roundtrip(&cfg);
        let back: ProviderConfig =
            serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        let ratio = back
            .settings
            .get("ratio")
            .and_then(serde_json::Value::as_f64)
            .unwrap();
        assert!((ratio - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn service_mesh_config_serializes_roundtrip() {
        let cfg = ServiceMeshConfig {
            enabled: false,
            discovery_endpoints: vec![],
            registry_type: ServiceRegistryType::InMemory,
            max_services: 0,
            health_check_interval_secs: u64::MAX,
            heartbeat_interval_secs: 1,
            service_expiration_secs: 0,
            enable_failover: false,
            metrics_enabled: false,
            namespace: None,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn service_registry_type_in_memory_serializes_roundtrip() {
        let t = ServiceRegistryType::InMemory;
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn service_registry_type_file_serializes_roundtrip() {
        let t = ServiceRegistryType::File {
            path: "/reg/services.json".to_string(),
        };
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn service_registry_type_network_serializes_roundtrip() {
        let t = ServiceRegistryType::Network {
            endpoints: vec!["consul:8500".to_string(), "etcd:2379".to_string()],
        };
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn service_registry_type_redis_serializes_roundtrip() {
        let t = ServiceRegistryType::Redis {
            connection_string: "redis://localhost:6379/0".to_string(),
        };
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn service_registry_type_database_serializes_roundtrip() {
        let t = ServiceRegistryType::Database {
            connection_string: "postgres://localhost/db".to_string(),
        };
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn service_registry_type_custom_serializes_roundtrip() {
        let t = ServiceRegistryType::Custom {
            config: HashMap::from([("a".to_string(), "b".to_string())]),
        };
        assert_serde_json_roundtrip(&t);
    }

    #[test]
    fn monitoring_config_serializes_roundtrip() {
        let cfg = MonitoringConfig {
            enabled: true,
            metrics_endpoint: "/m".to_string(),
            tracing_endpoint: None,
            enable_prometheus: false,
            prometheus_port: u16::MAX,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn feature_flags_serializes_roundtrip() {
        let cfg = FeatureFlags {
            experimental: true,
            enable_plugins: false,
            enable_federation: true,
            enable_advanced_routing: false,
            custom: HashMap::from([("x".to_string(), false)]),
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn database_config_serializes_roundtrip() {
        let cfg = DatabaseConfig {
            connection_string: "sqlite::memory:".to_string(),
            max_connections: u32::MAX,
            timeout_seconds: 1,
            backend: DatabaseBackend::PostgreSQL,
            enable_pooling: false,
            pool_size: 0,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn load_balancing_config_serializes_roundtrip() {
        let cfg = LoadBalancingConfig {
            strategy: LoadBalancingStrategy::LeastConnections,
            sticky_sessions: true,
            session_timeout_secs: 0,
            circuit_breaker: CircuitBreakerConfig {
                enabled: false,
                failure_threshold: u32::MAX,
                success_threshold: 0,
                timeout_secs: u64::MAX,
                half_open_max_requests: u32::MAX,
            },
            health_based_routing: false,
            retry_failed: false,
            max_retries: u32::MAX,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn circuit_breaker_config_serializes_roundtrip() {
        let cfg = CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 1,
            success_threshold: 2,
            timeout_secs: 3,
            half_open_max_requests: 4,
        };
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn squirrel_unified_config_serializes_roundtrip() {
        let cfg = sample_squirrel_unified_config();
        assert_serde_json_roundtrip(&cfg);
    }

    #[test]
    fn squirrel_unified_config_clone_matches_serde_value() {
        let cfg = sample_squirrel_unified_config();
        let expected = serde_json::to_value(&cfg).unwrap();
        let cfg2 = sample_squirrel_unified_config();
        assert_eq!(expected, serde_json::to_value(&cfg2).unwrap());
    }

    #[test]
    fn squirrel_unified_config_debug_is_non_empty() {
        let cfg = sample_squirrel_unified_config();
        let debug = format!("{cfg:?}");
        assert!(debug.len() > 50);
        assert!(debug.contains("SquirrelUnifiedConfig"));
    }

    #[test]
    fn timeout_config_serializes_roundtrip() {
        let t = sample_timeouts();
        assert_serde_json_roundtrip(&t);
    }
}
