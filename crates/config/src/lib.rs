// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::struct_excessive_bools,
    clippy::needless_pass_by_value,
    reason = "Progressive documentation and style tightening for config crate"
)]

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
//! ```ignore
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

/// Environment-based configuration: variables, environment types, and env-driven config structs.
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
/// Legacy alias for `ConfigLoader`; use `ConfigLoader` directly.
#[deprecated(since = "0.2.0", note = "Use `ConfigLoader` instead")]
pub type DefaultConfigManager = ConfigLoader;

/// Legacy alias for `SquirrelUnifiedConfig`; use `SquirrelUnifiedConfig` directly.
#[deprecated(since = "0.2.0", note = "Use `SquirrelUnifiedConfig` instead")]
pub type Config = SquirrelUnifiedConfig;

// Re-export EcosystemConfig from environment module for convenience
pub use environment::EcosystemConfig;
