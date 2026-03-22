// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Unified Transport Layer
//!
//! Hybrid transport system combining WebSocket for external clients
//! and tarpc for internal services with intelligent routing and load balancing.

mod connection;
mod routing;
mod services;
mod types;
mod unified;

pub use connection::ConnectionManager;
pub use routing::{LoadBalancer, MessageRouter};
pub use types::{
    ConnectionHandler, ConnectionInfo, ConnectionState, LoadBalancingStrategy, MessageType,
    RoutingEntry, TransportConfig, TransportMessage, TransportMetrics, TransportService,
    TransportServiceMetrics, TransportType,
};
pub use unified::UnifiedTransport;
