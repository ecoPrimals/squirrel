//! Unified Configuration System for Squirrel
//!
//! This module provides the canonical unified configuration structure that consolidates
//! all configuration across the Squirrel ecosystem. It implements a hierarchical loading
//! system with clear precedence rules.
//!
//! # Precedence Hierarchy (highest to lowest)
//!
//! 1. Command-line arguments (highest priority)
//! 2. Environment variables
//! 3. Configuration file (TOML/JSON/YAML)
//! 4. Platform-specific defaults
//! 5. Secure fallback defaults (lowest priority)
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use squirrel_mcp_config::unified::{SquirrelUnifiedConfig, ConfigLoader};
//!
//! // Load with full precedence hierarchy
//! let config = ConfigLoader::load()?;
//!
//! // Access timeout configuration
//! let timeout = config.timeouts.connection_timeout();
//!
//! // Access AI provider configuration
//! let openai_key = config.ai.providers.get("openai");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod environment_utils;
pub mod health_check;
pub mod loader;
pub mod timeouts;
pub mod types;
pub mod validation;

pub use environment_utils::*;
pub use health_check::HealthCheckConfig;
pub use loader::{ConfigLoader, LoadedConfig};
pub use timeouts::TimeoutConfig;
pub use types::{
    AiProvidersConfig, CircuitBreakerConfig, DatabaseBackend, DatabaseConfig, FeatureFlags, 
    LoadBalancingConfig, LoadBalancingStrategy, McpConfig, MonitoringConfig, NetworkConfig, 
    SecurityConfig, ServiceMeshConfig, ServiceRegistryType, SquirrelUnifiedConfig, SystemConfig,
};
pub use validation::{Validator, ValidationError, ValidationResult};

