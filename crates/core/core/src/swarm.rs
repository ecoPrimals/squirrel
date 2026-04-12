// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Swarm coordination API reserved for multi-instance orchestration.

/// Reserved for multi-instance coordination. Wiring depends on `capability.call` discovery for peer detection.
pub struct SwarmService;

impl Default for SwarmService {
    fn default() -> Self {
        Self::new()
    }
}

impl SwarmService {
    /// Creates a new swarm service handle (coordination hooks are deferred to Phase 2).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
