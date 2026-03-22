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
