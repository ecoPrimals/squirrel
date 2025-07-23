//! Configuration system for Squirrel MCP
//!
//! This crate provides a comprehensive configuration system supporting:
//! - Environment variable overrides
//! - TOML file configuration
//! - Universal patterns configuration
//! - Core system configuration
//! - Validation and error handling
//!
//! # Example Usage
//!
//! ```
//! use squirrel_mcp_config::Config;
//!
//! // Load configuration from environment variables
//! let config = Config::from_env()?;
//!
//! // Validate configuration
//! config.validate()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod constants;
pub mod core;
pub mod environment;
pub mod universal;

// Re-export main configuration types with explicit names to avoid conflicts
pub use core::{
    AIConfig, AIServiceConfig, BiomeOSEndpoints, Config, ConfigDefaults, ConfigManager,
    DatabaseConfig, DefaultConfigManager, EcosystemConfig, ExtendedObservabilityConfig,
    ExternalServiceConfig, NetworkConfig, ObservabilityConfig,
};
pub use environment::{Environment, EnvironmentConfig, EnvironmentError};
pub use universal::{
    FromEnv, ServiceConfig, ServiceConfigBuilder, UniversalConfigBuilder, UniversalServiceConfig,
};

// Re-export error types with module prefixes to avoid conflicts
pub use core::types::CoreConfigError;
pub use universal::ConfigError as UniversalConfigError;

// Re-export security config types with module prefixes to avoid conflicts
pub use core::security::BeardogConfig;
pub use core::security::SecurityConfig as CoreSecurityConfig;
pub use universal::SecurityConfig as UniversalSecurityConfig;

// CRITICAL FIX: Export the missing service endpoints functionality
pub use core::service_endpoints::{get_service_endpoints, GlobalServiceEndpoints};
