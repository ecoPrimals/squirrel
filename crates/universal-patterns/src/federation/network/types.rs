// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Federation network type re-exports and overlay-only definitions.
//!
//! Shared wire/config types live in [`crate::federation::network_types`].

pub use crate::federation::network_types::{
    DataOperation, NetworkConfig, NetworkMessage, NetworkProtocol, NetworkStats, NodeInfo,
    PeerInfo, PeerStatus, QueuedMessage,
};
