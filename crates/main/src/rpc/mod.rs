//! JSON-RPC and tarpc Protocol Implementation
//!
//! This module provides modern inter-primal communication protocols:
//! - JSON-RPC 2.0 over Unix sockets (for biomeOS integration)
//! - tarpc for high-performance peer-to-peer RPC
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
//! - **Unix Socket + JSON-RPC**: Local biomeOS coordination
//! - **tarpc**: Remote Squirrel-to-Squirrel communication
//! - **REST HTTP**: External client APIs (legacy, maintained)

pub mod handlers;
pub mod server;
pub mod tarpc_client;
pub mod tarpc_server;
pub mod tarpc_service;
pub mod types;
pub mod unix_socket;

// Re-exports for convenience
pub use server::RpcServer;
pub use tarpc_client::connect as connect_tarpc;
pub use tarpc_server::SquirrelRpcServer;
pub use tarpc_service::{
    SquirrelRpc, SquirrelRpcClient, TarpcHealthStatus, TarpcProviderInfo, TarpcQueryRequest,
    TarpcQueryResponse,
};
pub use types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, ListProvidersRequest, ListProvidersResponse, QueryAiRequest,
    QueryAiResponse,
};
