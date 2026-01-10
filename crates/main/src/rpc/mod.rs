//! JSON-RPC and tarpc Protocol Implementation
//!
//! This module provides modern inter-primal communication protocols:
//! - JSON-RPC 2.0 over Unix sockets (for biomeOS integration) ✅ COMPLETE
//! - tarpc for high-performance peer-to-peer RPC ⏳ 60% COMPLETE (feature-gated)
//!
//! ## Architecture
//!
//! ```text
//! biomeOS
//!    ↓
//! Unix Socket (/tmp/squirrel-{node_id}.sock)
//!    ↓
//! JSON-RPC 2.0 Server
//!    ↓
//! Squirrel Core APIs
//! ```
//!
//! ## Protocol Selection
//!
//! - **Unix Socket + JSON-RPC**: Local biomeOS coordination (PRIMARY, READY)
//! - **tarpc**: Remote Squirrel-to-Squirrel communication (PHASE 2, IN PROGRESS)
//! - **REST HTTP**: External client APIs (legacy, maintained)

pub mod handlers;
pub mod server;
pub mod types;
pub mod unix_socket;

// Phase 2: tarpc binary RPC (60% complete, needs tarpc 0.34 API research)
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

// Phase 2: tarpc re-exports (feature-gated)
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_client::connect as connect_tarpc;
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_server::SquirrelRpcServer;
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_service::{
    SquirrelRpc, SquirrelRpcClient, TarpcHealthStatus, TarpcProviderInfo, TarpcQueryRequest,
    TarpcQueryResponse,
};
