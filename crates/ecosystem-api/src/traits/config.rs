// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Factory and configuration traits plus [`UniversalConfig`] graph.

use crate::error::UniversalResult;
use crate::types::SecurityConfig;

use super::primal::UniversalPrimalProvider;

/// Primal factory trait for creating primal instances
pub trait PrimalFactory: Send + Sync {
    /// Create a new primal instance
    ///
    /// # Errors
    ///
    /// Returns an error when the primal cannot be constructed from the given configuration.
    fn create_primal(
        &self,
        config: UniversalConfig,
    ) -> UniversalResult<Box<dyn UniversalPrimalProvider>>;
}

/// Configuration trait for universal configuration management
pub trait ConfigProvider: Send + Sync {
    /// Load configuration from environment
    ///
    /// # Errors
    ///
    /// Returns an error when environment values are missing or invalid.
    fn load_from_environment(&self) -> UniversalResult<UniversalConfig>;

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns an error when the configuration fails validation checks.
    fn validate(&self, config: &UniversalConfig) -> UniversalResult<()>;

    /// Get configuration value
    fn get_value(&self, key: &str) -> Option<String>;

    /// Set configuration value
    ///
    /// # Errors
    ///
    /// Returns an error when the value cannot be persisted or applied.
    fn set_value(&self, key: &str, value: String) -> UniversalResult<()>;
}

/// Universal configuration structure
#[derive(Debug, Clone)]
pub struct UniversalConfig {
    /// Service configuration
    pub service: ServiceConfig,

    /// Service mesh integration settings
    pub service_mesh: ServiceMeshConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Resource limits and requirements
    pub resources: ResourceConfig,

    /// Feature flags
    pub features: FeatureFlags,

    /// Primal-specific configuration
    pub primal_specific: std::collections::HashMap<String, serde_json::Value>,
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    pub description: String,

    /// Bind address
    pub bind_address: String,

    /// Port number
    pub port: u16,

    /// Log level
    pub log_level: String,

    /// Instance ID
    pub instance_id: String,
}

/// Service mesh configuration
#[derive(Debug, Clone)]
pub struct ServiceMeshConfig {
    /// Discovery endpoint
    pub discovery_endpoint: String,

    /// Registration endpoint
    pub registration_endpoint: String,

    /// Health endpoint
    pub health_endpoint: String,

    /// Authentication token
    pub auth_token: Option<String>,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
}

/// Retry configuration for resilient operations
///
/// Simple `RetryConfig` for ecosystem-api (standalone crate)
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay in milliseconds before first retry
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds between retries
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles each retry)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Resource configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// CPU cores
    pub cpu_cores: Option<f64>,

    /// Memory in MB
    pub memory_mb: Option<u64>,

    /// Disk space in MB
    pub disk_mb: Option<u64>,

    /// Network bandwidth in Mbps
    pub network_bandwidth_mbps: Option<u64>,

    /// GPU count
    pub gpu_count: Option<u32>,
}

/// Feature flags
#[derive(Debug, Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Independent toggles for runtime features"
)]
pub struct FeatureFlags {
    /// Development mode
    pub development_mode: bool,

    /// Debug logging
    pub debug_logging: bool,

    /// Metrics enabled
    pub metrics_enabled: bool,

    /// Tracing enabled
    pub tracing_enabled: bool,

    /// Experimental features
    pub experimental_features: Vec<String>,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network port
    pub port: u16,

    /// Max connections
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Read timeout in seconds
    pub read_timeout_secs: u64,

    /// Write timeout in seconds
    pub write_timeout_secs: u64,
}
