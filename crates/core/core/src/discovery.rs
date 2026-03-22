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
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_service_new_and_default_are_equivalent() {
        assert_eq!(std::mem::size_of::<DiscoveryService>(), 0);
        let _ = (DiscoveryService::new(), DiscoveryService::default());
    }
}
