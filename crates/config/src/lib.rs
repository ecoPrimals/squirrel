//! Configuration system for Squirrel MCP
//!
//! This crate provides a unified configuration system supporting:
//! - Environment variable overrides
//! - TOML/JSON/YAML file configuration
//! - Comprehensive defaults and validation
//! - Modern service mesh and load balancing configuration
//!
//! # Example Usage
//!
//! ```
//! use squirrel_mcp_config::{SquirrelUnifiedConfig, ConfigLoader};
//!
//! // Load configuration from environment variables
//! let config = ConfigLoader::load_from_env()?;
//!
//! // Or load from a file
//! let config = ConfigLoader::load_from_file("config.toml")?;
//!
//! // Validate configuration
//! config.validate()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod compat;
pub mod constants;
pub mod environment;
pub mod service_endpoints;

// Unified configuration system - the single source of truth
pub mod unified;

// Re-export environment utilities (still useful)
pub use environment::{Environment, EnvironmentConfig, EnvironmentError};

// Re-export service endpoints (legacy compatibility)
pub use service_endpoints::{get_service_endpoints, GlobalServiceEndpoints};

// Re-export compatibility layer (legacy types)
pub use compat::{BiomeOSEndpoints, Config, ConfigManager, DefaultConfigManager, EcosystemConfig, ExternalServicesConfig};

// Re-export unified configuration types (primary exports)
pub use unified::{
    AiProvidersConfig, CircuitBreakerConfig, ConfigLoader, DatabaseBackend, DatabaseConfig,
    FeatureFlags, HealthCheckConfig, LoadBalancingConfig, LoadBalancingStrategy, LoadedConfig,
    McpConfig, MonitoringConfig, NetworkConfig, SecurityConfig, ServiceMeshConfig,
    SquirrelUnifiedConfig, SystemConfig, TimeoutConfig,
};
