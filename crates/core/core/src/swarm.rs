// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Placeholder swarm coordination API reserved for future multi-instance orchestration.

/// No-op placeholder for future swarm-level coordination hooks.
pub struct SwarmService;

impl Default for SwarmService {
    fn default() -> Self {
        Self::new()
    }
}

impl SwarmService {
    /// Creates a new placeholder swarm service instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
