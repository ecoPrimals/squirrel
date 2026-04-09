// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! JSON-RPC and tarpc Protocol Implementation
//!
//! **MODERN ARCHITECTURE** (Post-HTTP cleanup, Jan 19, 2026):
//! - JSON-RPC 2.0 over Unix sockets (for biomeOS integration) ✅
//! - tarpc for high-performance peer-to-peer RPC ✅
//! - Protocol selection and negotiation ✅
//! - NO HTTP! TRUE PRIMAL uses Unix sockets only! 🎉
//!
//! ## Architecture
//!
//! ```text
//! Request → Universal Transport
//!              ↓
//!      [Protocol Selection]
//!              ↓
//!      ┌───────┴───────┐
//!      ↓               ↓
//!   JSON-RPC        tarpc
//!   (default)       (performance)
//! ```
//!
//! ## Implementation Notes
//!
//! tarpc implementation based on working patterns from ecosystem primals:
//! - Uses tarpc 0.37 with tokio-serde 0.8.0
//! - LengthDelimitedCodec for framing
//! - Bincode for serialization
//! - Feature-gated behind `tarpc-rpc` feature flag

// Core modules (Pure Rust!)
pub mod btsp_handshake;
mod handlers_ai;
mod handlers_capability;
mod handlers_context;
mod handlers_graph;
mod handlers_identity;
mod handlers_system;
mod handlers_tool;
pub mod ipc_client;
mod jsonrpc_handlers;
pub mod jsonrpc_server;
pub mod jsonrpc_types;
pub mod protocol;
pub mod protocol_negotiation;
pub mod spring_tools;
pub mod types;
pub mod unix_socket;

// tarpc binary RPC (feature-gated)
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_client;
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_server;
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_service;
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_transport;

// Integration tests (test-only)
#[cfg(all(test, feature = "tarpc-rpc"))]
mod tarpc_integration_tests;

// Re-exports for convenience
pub use jsonrpc_server::JsonRpcServer;
pub use protocol::{IpcProtocol, ProtocolNegotiation};
pub use protocol_negotiation::{
    ProtocolRequest, ProtocolResponse, negotiate_client, negotiate_server, select_protocol,
};
pub use types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, HealthTier, ListProvidersRequest, ListProvidersResponse, QueryAiRequest,
    QueryAiResponse,
};

// tarpc re-exports (feature-gated)
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_client::{SquirrelClient, SquirrelClientBuilder};
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_server::TarpcRpcServer;
#[cfg(feature = "tarpc-rpc")]
pub use tarpc_service::SquirrelRpc;
