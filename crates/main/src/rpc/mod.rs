//! JSON-RPC and tarpc Protocol Implementation
//!
//! **MODERN ARCHITECTURE** (Post-HTTP cleanup, Jan 19, 2026):
//! - JSON-RPC 2.0 over Unix sockets (for biomeOS integration) ✅
//! - tarpc for high-performance peer-to-peer RPC ✅
//! - NO HTTP! TRUE PRIMAL uses Unix sockets only! 🎉
//!
//! ## Architecture
//!
//! ```text
//! Request → Unix Socket
//!              ↓
//!      [JSON-RPC or tarpc]
//!              ↓
//!      ┌───────┴───────┐
//!      ↓               ↓
//!   JSON-RPC        tarpc
//!   (biomeOS)       (P2P)
//! ```
//!
//! ## Implementation Notes
//!
//! tarpc implementation based on working patterns from Songbird and BearDog primals:
//! - Uses tarpc 0.34 with tokio-serde 0.8.0
//! - LengthDelimitedCodec for framing
//! - Bincode for serialization
//! - Feature-gated behind `tarpc-rpc` feature flag

// Core modules (Pure Rust!)
pub mod types;
pub mod unix_socket;

// tarpc binary RPC (feature-gated)
// tarpc RPC is now CORE functionality (not optional!)
pub mod tarpc_client;
pub mod tarpc_server;
pub mod tarpc_service;

// Re-exports for convenience
pub use types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, ListProvidersRequest, ListProvidersResponse, QueryAiRequest,
    QueryAiResponse,
};

// tarpc re-exports (feature-gated)
// Re-export tarpc types (core functionality)
pub use tarpc_client::connect as connect_tarpc;
pub use tarpc_server::SquirrelRpcServer;
pub use tarpc_service::{
    SquirrelRpc, SquirrelRpcClient, TarpcHealthStatus, TarpcProviderInfo, TarpcQueryRequest,
    TarpcQueryResponse,
};
