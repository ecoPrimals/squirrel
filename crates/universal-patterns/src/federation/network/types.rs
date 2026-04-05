// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation network type re-exports and overlay-only definitions.
//!
//! Shared wire/config types live in [`crate::federation::network_types`].

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub use crate::federation::network_types::{
    DataOperation, NetworkConfig, NetworkMessage, NetworkProtocol, NetworkStats, NodeInfo,
    PeerInfo, PeerStatus, QueuedMessage,
};

/// Connection state for network management
#[derive(Debug, Clone)]
#[expect(
    dead_code,
    reason = "federation Phase 2 — will be wired when connection state tracking is implemented"
)]
pub(super) struct ConnectionState {
    /// Connected peers
    pub peers: HashMap<Uuid, PeerInfo>,
    /// Active connections count
    pub active_connections: usize,
    /// Last heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Network statistics
    pub stats: NetworkStats,
}

impl ConnectionState {
    /// Create new connection state
    #[expect(
        dead_code,
        reason = "federation Phase 2 — will be wired when connection state tracking is implemented"
    )]
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            active_connections: 0,
            last_heartbeat: None,
            stats: NetworkStats::default(),
        }
    }
}
