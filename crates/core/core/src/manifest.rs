// SPDX-License-Identifier: AGPL-3.0-only
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
    pub fn new() -> Self {
        Self
    }
}
