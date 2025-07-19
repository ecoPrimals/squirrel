//! Universal Configuration System
//!
//! This module provides a universal configuration system that replaces hardcoded
//! primal endpoints with dynamic service discovery and configuration.
//!
//! ## Features
//!
//! - Environment variable-based configuration
//! - Dynamic service discovery configuration
//! - Service mesh integration
//! - Health check configuration
//! - Comprehensive validation
//! - Builder pattern for easy configuration
//!
//! ## Usage
//!
//! ```rust
//! use squirrel_mcp_config::universal::{UniversalServiceConfig, ServiceConfigBuilder, UniversalConfigBuilder, FromEnv};
//! use std::time::Duration;
//!
//! // Load from environment variables
//! let config = UniversalServiceConfig::from_env();
//!
//! // Or build programmatically
//! let config = UniversalConfigBuilder::new()
//!     .add_discovery_endpoint("http://localhost:8500".to_string())
//!     .unwrap()
//!     .add_service(
//!         "ai-service".to_string(),
//!         ServiceConfigBuilder::new()
//!             .add_endpoint("http://localhost:8080".to_string())
//!             .unwrap()
//!             .add_capability("chat".to_string())
//!             .build()
//!             .unwrap()
//!     )
//!     .unwrap()
//!     .build();
//! ```

mod builder;
pub mod environment;
mod types;
mod utils;
mod validation;

// Re-export public types
pub use builder::{ServiceConfigBuilder, UniversalConfigBuilder};
pub use types::*;
pub use utils::{parse_duration, validate_url};

// Re-export validation functionality
pub use validation::ValidationExt;

// Re-export environment functionality
pub use environment::FromEnv;
