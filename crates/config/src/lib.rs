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

pub mod constants;
pub mod environment;

// Unified configuration system - the single source of truth
pub mod unified;

// Re-export environment utilities (still useful)
pub use environment::{Environment, EnvironmentConfig, EnvironmentError};

// ============================================================================
// COMPATIBILITY LAYER REMOVED - November 9, 2025
// ============================================================================
// The legacy compatibility layer has been successfully removed!
// All code now uses the unified config system directly.
//
// Migration complete:
// - compat::Config → unified::SquirrelUnifiedConfig (via ConfigLoader)
// - compat::DefaultConfigManager → unified::ConfigLoader
// - service_endpoints::get_service_endpoints() → std::env::var() directly
// - compat::BiomeOSEndpoints → Direct environment variables
// - compat::ExternalServicesConfig → Direct environment variables
//
// Removed files:
// - crates/config/src/compat.rs (271 LOC removed)
// - crates/config/src/service_endpoints.rs (105 LOC removed)
//
// Total: 376 LOC of legacy code eliminated!
//
// See: docs/sessions/nov-9-2025-evening/ for migration history
// ============================================================================

// Re-export unified configuration types (primary exports)
pub use unified::{
    AiProvidersConfig, CircuitBreakerConfig, ConfigLoader, DatabaseBackend, DatabaseConfig,
    FeatureFlags, HealthCheckConfig, LoadBalancingConfig, LoadBalancingStrategy, LoadedConfig,
    McpConfig, MonitoringConfig, NetworkConfig, SecurityConfig, ServiceMeshConfig,
    SquirrelUnifiedConfig, SystemConfig, TimeoutConfig,
};

// Compatibility aliases for gradual migration (deprecated names → new types)
#[deprecated(since = "0.2.0", note = "Use `ConfigLoader` instead")]
pub type DefaultConfigManager = ConfigLoader;

#[deprecated(since = "0.2.0", note = "Use `SquirrelUnifiedConfig` instead")]
pub type Config = SquirrelUnifiedConfig;

// Re-export EcosystemConfig from environment module for convenience
pub use environment::EcosystemConfig;
