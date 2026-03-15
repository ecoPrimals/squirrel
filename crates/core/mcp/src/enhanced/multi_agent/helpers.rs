// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Helper functions for multi-agent coordination

use super::types::CollaborationType;

/// Get human-readable name for collaboration type
pub(crate) fn session_type_name(session_type: &CollaborationType) -> &str {
    match session_type {
        CollaborationType::Sequential => "sequential",
        CollaborationType::Parallel => "parallel",
        CollaborationType::Hierarchical => "hierarchical",
        CollaborationType::PeerToPeer => "peer-to-peer",
        CollaborationType::Consensus => "consensus",
        CollaborationType::Custom(ref s) => s,
    }
}

