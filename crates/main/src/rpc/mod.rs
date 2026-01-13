//! JSON-RPC and tarpc Protocol Implementation
//!
//! This module provides modern inter-primal communication protocols:
//! - JSON-RPC 2.0 over Unix sockets (for biomeOS integration) ✅ COMPLETE
//! - tarpc for high-performance peer-to-peer RPC ✅ COMPLETE
//! - HTTPS fallback for compatibility ✅ NEW
//!
//! ## Architecture
//!
//! ```text
//! Request → Protocol Router
//!              ↓
//!      [Priority Selection]
//!              ↓
//!      ┌───────┴───────┐
//!      ↓       ↓       ↓
//!    tarpc  JSON-RPC HTTPS
//!      ↓       ↓       ↓
//!    [Fast] [Local] [Compat]
//! ```
//!
//! ## Protocol Selection (Following Songbird)
//!
//! 1. **tarpc**: High-performance binary RPC (primary for federation)
//! 2. **JSON-RPC 2.0**: Unix socket IPC (primary for biomeOS)
//! 3. **HTTPS**: RESTful fallback (compatibility layer)
//!
//! ## Implementation Notes
//!
//! tarpc implementation based on working patterns from Songbird and BearDog primals:
//! - Uses tarpc 0.34 with tokio-serde 0.8.0
//! - LengthDelimitedCodec for framing
//! - Bincode for serialization
//! - Feature-gated behind `tarpc-rpc` feature flag

pub mod handlers;
pub mod server;
pub mod types;
pub mod unix_socket;

// Protocol router - intelligent protocol selection (NEW - Songbird pattern)
pub mod protocol_router;

// HTTPS fallback server (NEW - Songbird pattern)
pub mod https_fallback;

// Internal handlers (protocol-agnostic implementations)
pub mod handlers_internal;

// Handler wiring (connects protocol router to implementations)
mod handler_stubs;

// tarpc binary RPC (feature-gated)
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_client;
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_server;
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_service;

// Re-exports for convenience
pub use server::RpcServer;
pub use types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, ListProvidersRequest, ListProvidersResponse, QueryAiRequest,
    QueryAiResponse,
};

// Protocol router re-exports (NEW)
pub use protocol_router::{
    ProtocolRequest, ProtocolResponse, ProtocolRouter, ProtocolRouterConfig,
};

// HTTPS fallback re-exports (NEW)
pub use https_fallback::{HttpsFallbackConfig, HttpsFallbackServer};

// tarpc re-exports (feature-gated)
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_client::connect as connect_tarpc;
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_server::SquirrelRpcServer;
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_service::{
    SquirrelRpc, SquirrelRpcClient, TarpcHealthStatus, TarpcProviderInfo, TarpcQueryRequest,
    TarpcQueryResponse,
};
