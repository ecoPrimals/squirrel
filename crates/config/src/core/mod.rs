//! Core configuration system for Squirrel MCP
//!
//! This module provides a comprehensive configuration system supporting:
//! - Network and database configuration
//! - AI service provider configuration
//! - Security and authentication settings
//! - Observability and monitoring configuration
//! - Ecosystem coordination settings
//! - Environment-aware defaults and overrides
//!
//! # Example Usage
//!
//! ```
//! use squirrel_mcp_config::core::Config;
//!
//! // Load configuration from environment variables
//! let config = Config::from_env()?;
//!
//! // Validate configuration
//! config.validate()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod ai;
pub mod defaults;
pub mod ecosystem;
pub mod manager;
pub mod observability;
pub mod security;
pub mod types;

// Re-export main configuration types
pub use types::*;

// Re-export AI configuration
pub use ai::*;

// Re-export security configuration
pub use security::*;

// Re-export observability configuration
pub use observability::*;

// Re-export ecosystem configuration
pub use ecosystem::*;

// Re-export defaults
pub use defaults::*;

// Re-export manager
pub use manager::*;
