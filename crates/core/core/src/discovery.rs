// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service discovery module.
//!
//! Handles discovery of other primals in the ecosystem.

/// Service for discovering primals in the ecosystem.
pub struct DiscoveryService;

impl Default for DiscoveryService {
    fn default() -> Self {
        Self::new()
    }
}

impl DiscoveryService {
    /// Creates a new discovery service.
    pub const fn new() -> Self {
        Self
    }
}
