//! Universal Constants for Squirrel MCP System
//!
//! This crate provides a **single source of truth** for all constants used throughout
//! the Squirrel Universal AI Primal system. It consolidates previously scattered constants
//! from multiple locations into one well-organized, type-safe, and maintainable location.
//!
//! # Organization
//!
//! Constants are organized by domain:
//! - **`timeouts`** - All timeout and duration values
//! - **`limits`** - Size, count, and capacity limits
//! - **`network`** - Network configuration (ports, addresses)
//! - **`protocol`** - Protocol-specific constants
//! - **`env_vars`** - Environment variable names
//! - **`builders`** - Helper functions for building URLs and configurations
//!
//! # Example Usage
//!
//! ```
//! use universal_constants::timeouts;
//! use universal_constants::network;
//! use universal_constants::builders;
//!
//! // Use timeout constants
//! let timeout = timeouts::DEFAULT_CONNECTION_TIMEOUT;
//!
//! // Use network constants
//! let port = network::DEFAULT_WEBSOCKET_PORT;
//!
//! // Use builder helpers
//! let url = builders::localhost_http(port);
//! ```
//!
//! # Design Principles
//!
//! 1. **Single Source of Truth**: All constants defined once
//! 2. **Type Safety**: Use proper types (Duration, not u64)
//! 3. **Domain Organization**: Clear module boundaries
//! 4. **Easy Migration**: Re-exports for common patterns
//! 5. **No Dependencies**: Pure Rust, no external deps
//!
//! # Migration Guide
//!
//! Old code:
//! ```ignore
//! use squirrel_mcp_config::constants::timeouts::DEFAULT_CONNECTION_TIMEOUT;
//! use squirrel_mcp::constants::network::DEFAULT_WEBSOCKET_PORT;
//! ```
//!
//! New code:
//! ```ignore
//! use universal_constants::{timeouts, network};
//! let timeout = timeouts::DEFAULT_CONNECTION_TIMEOUT;
//! let port = network::DEFAULT_WEBSOCKET_PORT;
//! ```

// Module declarations
pub mod timeouts;
pub mod limits;
pub mod network;
pub mod protocol;
pub mod env_vars;
pub mod builders;

// Re-export commonly used items for convenience
pub use timeouts::*;
pub use limits::*;
pub use network::*;
pub use protocol::*;
pub use env_vars::*;

// Version information
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
