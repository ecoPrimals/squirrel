// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Manifest module.
//!
//! Handles biome.yaml manifest generation and biomeOS integration.

/// Service for manifest generation and validation.
pub struct ManifestService;

impl Default for ManifestService {
    fn default() -> Self {
        Self::new()
    }
}

impl ManifestService {
    /// Creates a new manifest service.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_service_new_and_default() {
        let _ = (ManifestService::new(), ManifestService);
    }
}
