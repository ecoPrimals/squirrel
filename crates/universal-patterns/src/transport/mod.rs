// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Transport Abstraction
//!
//! This module provides a universal transport abstraction that works across
//! all platforms without hardcoded platform-specific code paths.
//!
//! ## Philosophy: Universal & Agnostic
//!
//! Instead of platform `cfg` branches in every caller, use one API:
//!
//! ```text
//! #[cfg(unix)]   → Unix socket
//! #[cfg(windows)] → Named pipe
//! …
//! ```
//!
//! We use:
//! ```rust,no_run
//! use universal_patterns::transport::{UniversalListener, UniversalTransport};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let _transport = UniversalTransport::connect("service_name", None).await?;
//!     let listener = UniversalListener::bind("service_name", None).await?;
//!     let (_stream, _addr) = listener.accept().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Transport Hierarchy
//!
//! The universal transport automatically selects the best transport for the
//! current platform with automatic fallback:
//!
//! 1. **Unix Domain Sockets** (Linux, macOS, BSD)
//!    - Abstract namespace sockets (Linux)
//!    - Filesystem sockets (all Unix)
//!
//! 2. **Named Pipes** (Windows)
//!    - `\\.\pipe\name` format
//!
//! 3. **XPC** (macOS system services)
//!    - Only for system-level services
//!
//! 4. **TCP** (Universal fallback)
//!    - localhost:port
//!    - Works everywhere
//!
//! 5. **In-Process** (Testing, embedded)
//!    - Direct function calls
//!    - Zero overhead

mod client;
mod discovery;
mod listener;
mod types;

// Re-export all public types and functions
pub use client::UniversalTransport;
pub use listener::UniversalListener;
pub use types::{
    InProcessTransport, IpcEndpoint, ListenerConfig, RemoteAddr, TransportConfig, TransportType,
};

// Re-export discovery functions for advanced usage
pub use discovery::{
    discover_ipc_endpoint, discover_tcp_endpoint, get_socket_paths,
    get_tcp_discovery_file_candidates, write_tcp_discovery_file,
};
