//! Legacy transport implementation for the MCP system.
//!
//! # DEPRECATION NOTICE
//!
//! This module is deprecated and will be removed in a future release.
//! Please migrate to the new `transport` module which provides a more
//! modular and extensible transport layer implementation.
//!
//! Migration guide:
//! 1. Replace `transport_old::Transport` with `transport::Transport` trait implementations
//! 2. For TCP connections, use `transport::TcpTransport`
//! 3. For WebSocket connections, use `transport::WebSocketTransport`
//! 4. For in-memory testing, use `transport::MemoryTransport`
//! 5. For stdio based communication, use `transport::StdioTransport`
//!
//! See the documentation in the `transport` module for more details.
//! For a comprehensive migration guide, see `docs/migration/TRANSPORT_MIGRATION_GUIDE.md`.

/// The main transport implementation
pub mod transport;
pub use transport::Transport;
pub use transport::TransportConfig;
pub use transport::TransportState;

/// Error handling for the transport layer
pub mod error;

/// Compatibility layer to help migrate from old transport to new
pub mod compat;
pub use compat::*;

#[cfg(test)]
pub mod tests; 