// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP env vars

/// Capability socket for BTSP
pub const CAPABILITY_SOCKET: &str = "BTSP_CAPABILITY_SOCKET";
/// Provider socket for BTSP
pub const PROVIDER_SOCKET: &str = "BTSP_PROVIDER_SOCKET";
/// Handshake timeout (ms)
pub const HANDSHAKE_TIMEOUT_MS: &str = "BTSP_HANDSHAKE_TIMEOUT_MS";
/// Birdsong key label for BTSP trust derivation
pub const BIRDSONG_KEY_LABEL: &str = "BTSP_BIRDSONG_KEY_LABEL";
/// Lineage root prefix for trust chain verification
pub const LINEAGE_ROOT_PREFIX: &str = "BTSP_LINEAGE_ROOT_PREFIX";
/// Max depth for lineage trust chain traversal
pub const LINEAGE_MAX_DEPTH: &str = "BTSP_LINEAGE_MAX_DEPTH";
