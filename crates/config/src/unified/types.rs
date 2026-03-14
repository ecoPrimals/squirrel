// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Unified Configuration Types
//!
//! This module defines the canonical configuration structure for the entire
//! Squirrel ecosystem. All configuration should flow through SquirrelUnifiedConfig.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::TimeoutConfig;

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
    #[serde(default = "default_instance_id")]
    pub instance_id: String,

    /// Environment (development, staging, production)
    #[serde(default)]
    pub environment: String,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Working directory
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,

    /// Data directory
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Plugin directory
    #[serde(default = "default_plugin_dir")]
    pub plugin_dir: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// HTTP port
    #[serde(default = "default_http_port")]
    pub http_port: u16,

    /// WebSocket port
    #[serde(default = "default_websocket_port")]
    pub websocket_port: u16,

    /// gRPC port
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,

    /// Maximum connections
    #[serde(default = "default_max_connections")]
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
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Authentication required
    #[serde(default = "default_true")]
    pub require_authentication: bool,

    /// Authorization enabled
    #[serde(default = "default_true")]
    pub enable_authorization: bool,

    /// JWT secret (from environment)
    #[serde(default)]
    pub jwt_secret: Option<String>,

    /// JWT token expiration in seconds (from universal)
    #[serde(default = "default_token_expiration")]
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
    #[serde(default = "default_encryption_format")]
    pub encryption_default_format: String,

    /// Enable audit logging
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Controls whether security events are logged for audit purposes.
    #[serde(default = "default_true")]
    pub enable_audit: bool,

    /// Enable encryption features
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Master toggle for encryption features in the security subsystem.
    #[serde(default = "default_true")]
    pub enable_encryption: bool,

    /// Enable RBAC (Role-Based Access Control)
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Controls whether RBAC enforcement is enabled.
    #[serde(default = "default_true")]
    pub enable_rbac: bool,

    /// Token expiry in minutes
    ///
    /// **Consolidated from**: `crates/core/mcp/src/security/manager.rs`
    ///
    /// Default expiration time for security tokens.
    /// Default: 60 minutes
    #[serde(default = "default_token_expiry_minutes")]
    pub token_expiry_minutes: u64,
}

/// MCP protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Protocol version
    #[serde(default = "default_mcp_version")]
    pub version: String,

    /// Maximum message size in bytes
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,

    /// Buffer size in bytes
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// Enable compression
    #[serde(default)]
    pub enable_compression: bool,

    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
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
    #[serde(default = "default_true")]
    pub enable_local: bool,

    /// Enable cloud providers (OpenAI, Anthropic, etc.)
    #[serde(default = "default_true")]
    pub enable_cloud: bool,

    /// Maximum concurrent requests
    #[serde(default = "default_max_concurrent_ai_requests")]
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
    #[serde(default = "default_true")]
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
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Discovery endpoints (from unified - supports multiple endpoints)
    #[serde(default)]
    pub discovery_endpoints: Vec<String>,

    /// Service registry type (from universal - rich abstraction)
    #[serde(default = "default_registry_type")]
    pub registry_type: ServiceRegistryType,

    /// Maximum services to track
    #[serde(default = "default_max_services")]
    pub max_services: usize,

    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,

    /// Heartbeat interval in seconds (from universal - for active health checks)
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Service expiration timeout in seconds (from universal - when to remove stale services)
    #[serde(default = "default_service_expiration")]
    pub service_expiration_secs: u64,

    /// Enable automatic failover (from unified)
    #[serde(default = "default_true")]
    pub enable_failover: bool,

    /// Enable service mesh metrics (from universal)
    #[serde(default = "default_true")]
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
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics endpoint
    #[serde(default = "default_metrics_endpoint")]
    pub metrics_endpoint: String,

    /// Tracing endpoint
    #[serde(default)]
    pub tracing_endpoint: Option<String>,

    /// Enable Prometheus export
    #[serde(default = "default_true")]
    pub enable_prometheus: bool,

    /// Prometheus port
    #[serde(default = "default_prometheus_port")]
    pub prometheus_port: u16,
}

/// Feature flags for optional functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable experimental features
    #[serde(default)]
    pub experimental: bool,

    /// Enable plugin system
    #[serde(default = "default_true")]
    pub enable_plugins: bool,

    /// Enable federation
    #[serde(default)]
    pub enable_federation: bool,

    /// Enable advanced routing
    #[serde(default = "default_true")]
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
    #[serde(default = "default_database_url")]
    pub connection_string: String,

    /// Maximum number of connections (env: DB_MAX_CONNECTIONS)
    #[serde(default = "default_max_db_connections")]
    pub max_connections: u32,

    /// Connection timeout in seconds (env: DB_TIMEOUT)
    #[serde(default = "default_db_timeout")]
    pub timeout_seconds: u64,

    /// Database backend type
    #[serde(default)]
    pub backend: DatabaseBackend,

    /// Enable connection pooling
    #[serde(default = "default_true")]
    pub enable_pooling: bool,

    /// Pool size
    #[serde(default = "default_pool_size")]
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
    #[serde(default = "default_session_timeout")]
    pub session_timeout_secs: u64,

    /// Circuit breaker configuration
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,

    /// Health-based routing
    #[serde(default = "default_true")]
    pub health_based_routing: bool,

    /// Retry failed requests
    #[serde(default = "default_true")]
    pub retry_failed: bool,

    /// Maximum retries
    #[serde(default = "default_max_retries")]
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
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Failure threshold before opening circuit
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// Success threshold to close circuit
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,

    /// Timeout before attempting to close circuit (seconds)
    #[serde(default = "default_circuit_timeout")]
    pub timeout_secs: u64,

    /// Half-open state max requests
    #[serde(default = "default_half_open_requests")]
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: default_failure_threshold(),
            success_threshold: default_success_threshold(),
            timeout_secs: default_circuit_timeout(),
            half_open_max_requests: default_half_open_requests(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_string: default_database_url(),
            max_connections: default_max_db_connections(),
            timeout_seconds: default_db_timeout(),
            backend: DatabaseBackend::default(),
            enable_pooling: true,
            pool_size: default_pool_size(),
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::default(),
            sticky_sessions: false,
            session_timeout_secs: default_session_timeout(),
            circuit_breaker: CircuitBreakerConfig::default(),
            health_based_routing: true,
            retry_failed: true,
            max_retries: default_max_retries(),
        }
    }
}

// Default value functions
fn default_instance_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_log_level() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
}

// Database defaults
fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string())
}

fn default_max_db_connections() -> u32 {
    std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10)
}

fn default_db_timeout() -> u64 {
    std::env::var("DB_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn default_pool_size() -> u32 {
    5
}

// Load balancing defaults
fn default_session_timeout() -> u64 {
    3600 // 1 hour
}

fn default_max_retries() -> u32 {
    3
}

fn default_failure_threshold() -> u32 {
    5
}

fn default_success_threshold() -> u32 {
    3
}

fn default_circuit_timeout() -> u64 {
    60
}

fn default_half_open_requests() -> u32 {
    3
}

fn default_work_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

fn default_plugin_dir() -> PathBuf {
    PathBuf::from("./plugins")
}

fn default_bind_address() -> String {
    std::env::var("SQUIRREL_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
}

fn default_http_port() -> u16 {
    std::env::var("SQUIRREL_HTTP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

fn default_websocket_port() -> u16 {
    std::env::var("SQUIRREL_WEBSOCKET_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8081)
}

fn default_grpc_port() -> u16 {
    std::env::var("SQUIRREL_GRPC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082)
}

fn default_max_connections() -> u32 {
    100
}

fn default_true() -> bool {
    true
}

fn default_mcp_version() -> String {
    "1.0".to_string()
}

fn default_max_message_size() -> usize {
    16 * 1024 * 1024 // 16 MB
}

fn default_buffer_size() -> usize {
    8192
}

fn default_compression_level() -> u32 {
    6
}

fn default_max_concurrent_ai_requests() -> usize {
    10
}

fn default_max_services() -> usize {
    1000
}

fn default_health_check_interval() -> u64 {
    30
}

fn default_heartbeat_interval() -> u64 {
    15
}

fn default_service_expiration() -> u64 {
    90
}

fn default_registry_type() -> ServiceRegistryType {
    ServiceRegistryType::InMemory
}

fn default_token_expiration() -> u64 {
    3600 // 1 hour
}

// Security config defaults (Nov 9, 2025 consolidation)
fn default_encryption_format() -> String {
    "AES256GCM".to_string()
}

fn default_token_expiry_minutes() -> u64 {
    60 // 1 hour in minutes
}

fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}

fn default_prometheus_port() -> u16 {
    9090
}

impl Default for SquirrelUnifiedConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                instance_id: default_instance_id(),
                environment: "development".to_string(),
                log_level: default_log_level(),
                work_dir: default_work_dir(),
                data_dir: default_data_dir(),
                plugin_dir: default_plugin_dir(),
            },
            network: NetworkConfig {
                bind_address: default_bind_address(),
                http_port: default_http_port(),
                websocket_port: default_websocket_port(),
                grpc_port: default_grpc_port(),
                max_connections: default_max_connections(),
                enable_tls: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            security: SecurityConfig {
                enabled: true,
                require_authentication: true,
                enable_authorization: true,
                jwt_secret: std::env::var("JWT_SECRET").ok(),
                token_expiration_secs: default_token_expiration(),
                api_keys: vec![],
                allowed_origins: vec!["*".to_string()],
                tls_enabled: false,
                tls_cert_path: std::env::var("TLS_CERT_PATH").ok(),
                tls_key_path: std::env::var("TLS_KEY_PATH").ok(),
                ca_cert_path: std::env::var("CA_CERT_PATH").ok(),
                mtls_enabled: false,
                // Consolidated fields (Nov 9, 2025)
                encryption_default_format: default_encryption_format(),
                enable_audit: true,
                enable_encryption: true,
                enable_rbac: true,
                token_expiry_minutes: default_token_expiry_minutes(),
            },
            mcp: McpConfig {
                version: default_mcp_version(),
                max_message_size: default_max_message_size(),
                buffer_size: default_buffer_size(),
                enable_compression: false,
                compression_level: default_compression_level(),
            },
            ai: AiProvidersConfig {
                default_endpoint: String::new(),
                providers: HashMap::new(),
                enable_local: true,
                enable_cloud: true,
                max_concurrent_requests: default_max_concurrent_ai_requests(),
            },
            service_mesh: ServiceMeshConfig {
                enabled: true,
                discovery_endpoints: vec![],
                registry_type: default_registry_type(),
                max_services: default_max_services(),
                health_check_interval_secs: default_health_check_interval(),
                heartbeat_interval_secs: default_heartbeat_interval(),
                service_expiration_secs: default_service_expiration(),
                enable_failover: true,
                metrics_enabled: true,
                namespace: None,
            },
            timeouts: TimeoutConfig::default(),
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_endpoint: default_metrics_endpoint(),
                tracing_endpoint: None,
                enable_prometheus: true,
                prometheus_port: default_prometheus_port(),
            },
            database: DatabaseConfig::default(),
            load_balancing: LoadBalancingConfig::default(),
            features: FeatureFlags {
                experimental: false,
                enable_plugins: true,
                enable_federation: false,
                enable_advanced_routing: true,
                custom: HashMap::new(),
            },
            custom: HashMap::new(),
        }
    }
}

impl SquirrelUnifiedConfig {
    /// Validate the entire configuration
    ///
    /// Performs comprehensive validation across all configuration domains.
    /// Now uses the unified validation module for consistent error messages.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        use super::validation::Validator;

        let mut errors = Vec::new();

        // Validate timeouts
        if let Err(e) = self.timeouts.validate() {
            errors.push(format!("Timeout validation failed: {}", e));
        }

        // Validate network ports using unified validators
        if let Err(e) = Validator::validate_port(self.network.http_port) {
            errors.push(format!("HTTP port: {}", e));
        }
        if let Err(e) = Validator::validate_port(self.network.websocket_port) {
            errors.push(format!("WebSocket port: {}", e));
        }
        if let Err(e) = Validator::validate_ports_differ(
            self.network.http_port,
            self.network.websocket_port,
            "HTTP",
            "WebSocket",
        ) {
            errors.push(e.to_string());
        }

        // Validate security
        if self.security.enabled && self.security.require_authentication {
            if self.security.jwt_secret.is_none() && self.security.api_keys.is_empty() {
                errors.push(
                    "Authentication required but no JWT secret or API keys configured".to_string(),
                );
            }

            // Validate JWT secret length if provided
            if let Some(ref secret) = self.security.jwt_secret {
                if let Err(e) = Validator::validate_jwt_secret(secret) {
                    errors.push(format!("JWT secret: {}", e));
                }
            }
        }

        // Validate monitoring ports using unified validators
        if self.monitoring.enabled && self.monitoring.enable_prometheus {
            if let Err(e) = Validator::validate_port(self.monitoring.prometheus_port) {
                errors.push(format!("Prometheus port: {}", e));
            }
            if let Err(e) = Validator::validate_ports_differ(
                self.monitoring.prometheus_port,
                self.network.http_port,
                "Prometheus",
                "HTTP",
            ) {
                errors.push(e.to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squirrel_unified_config_default() {
        let config = SquirrelUnifiedConfig::default();
        assert!(!config.system.instance_id.is_empty());
        assert_eq!(config.system.environment, "development");
        assert!(!config.system.log_level.is_empty());
        assert!(config.network.http_port > 0);
        assert!(config.network.websocket_port > 0);
        assert!(config.security.enabled);
        assert_eq!(config.mcp.version, "1.0");
        assert!(config.service_mesh.enabled);
        assert!(config.monitoring.enabled);
        assert!(config.features.enable_plugins);
    }

    #[test]
    fn test_network_config_default_values() {
        let config = SquirrelUnifiedConfig::default();
        assert!(!config.network.bind_address.is_empty());
        assert_eq!(config.network.max_connections, 100);
        assert!(!config.network.enable_tls);
    }

    #[test]
    fn test_load_balancing_strategy_default() {
        let strategy = LoadBalancingStrategy::default();
        assert!(matches!(strategy, LoadBalancingStrategy::RoundRobin));
    }

    #[test]
    fn test_load_balancing_strategy_serde() {
        let strategies = vec![
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::Random,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::HealthBased,
        ];
        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            let decoded: LoadBalancingStrategy = serde_json::from_str(&json).unwrap();
            assert!(std::mem::discriminant(&strategy) == std::mem::discriminant(&decoded));
        }
    }

    #[test]
    fn test_database_backend_default() {
        let backend = DatabaseBackend::default();
        assert!(matches!(backend, DatabaseBackend::SQLite));
    }

    #[test]
    fn test_service_registry_type_in_memory_serde() {
        let registry = ServiceRegistryType::InMemory;
        let json = serde_json::to_string(&registry).unwrap();
        assert!(json.contains("in_memory"));
        let decoded: ServiceRegistryType = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, ServiceRegistryType::InMemory));
    }

    #[test]
    fn test_service_registry_type_file_serde() {
        let registry = ServiceRegistryType::File {
            path: "/tmp/registry.json".to_string(),
        };
        let json = serde_json::to_string(&registry).unwrap();
        let decoded: ServiceRegistryType = serde_json::from_str(&json).unwrap();
        if let ServiceRegistryType::File { path } = decoded {
            assert_eq!(path, "/tmp/registry.json");
        } else {
            panic!("Expected File variant");
        }
    }

    #[test]
    fn test_provider_config_serde() {
        let provider = ProviderConfig {
            endpoint: "https://api.openai.com".to_string(),
            api_key: Some("sk-xxx".to_string()),
            enabled: true,
            settings: HashMap::new(),
        };
        let json = serde_json::to_string(&provider).unwrap();
        let decoded: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.endpoint, provider.endpoint);
        assert_eq!(decoded.api_key, provider.api_key);
        assert!(decoded.enabled);
    }

    #[test]
    fn test_circuit_breaker_config_default() {
        let cb = CircuitBreakerConfig::default();
        assert!(cb.enabled);
        assert!(cb.failure_threshold > 0);
        assert!(cb.success_threshold > 0);
        assert!(cb.timeout_secs > 0);
    }

    #[test]
    fn test_config_validate_security_disabled() {
        let mut config = SquirrelUnifiedConfig::default();
        config.security.enabled = false;
        config.security.require_authentication = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_security_enabled_no_auth() {
        let mut config = SquirrelUnifiedConfig::default();
        config.security.enabled = true;
        config.security.require_authentication = false;
        config.security.jwt_secret = None;
        config.security.api_keys = vec![];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serde_roundtrip_minimal() {
        let config = SquirrelUnifiedConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let decoded: SquirrelUnifiedConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.system.environment, decoded.system.environment);
        assert_eq!(config.network.http_port, decoded.network.http_port);
    }

    #[test]
    fn test_feature_flags_default() {
        let config = SquirrelUnifiedConfig::default();
        assert!(!config.features.experimental);
        assert!(config.features.enable_plugins);
        assert!(!config.features.enable_federation);
        assert!(config.features.enable_advanced_routing);
    }

    #[test]
    fn test_database_config_default() {
        let db = DatabaseConfig::default();
        assert!(!db.connection_string.is_empty());
        assert!(db.max_connections > 0);
        assert!(db.enable_pooling);
    }
}
