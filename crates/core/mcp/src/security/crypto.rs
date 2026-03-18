// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Cryptographic provider for MCP security
//!
//! This module provides cryptographic functionality for the MCP system.
//! Actual cryptographic operations are delegated to the BearDog framework.

use crate::error::Result;
use std::sync::Arc;

/// Cryptographic provider interface
///
/// This provides basic cryptographic operations that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct DefaultCryptoProvider {
    // Internal state for crypto operations
    _state: Arc<CryptoState>,
}

#[derive(Debug)]
struct CryptoState {
    #[allow(dead_code)] // Will be used when BearDog crypto is integrated
    initialized: bool,
}

impl DefaultCryptoProvider {
    /// Create a new crypto provider
    pub fn new() -> Self {
        Self {
            _state: Arc::new(CryptoState { initialized: true }),
        }
    }

    /// Encrypt data (stub — will delegate to BearDog)
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    /// Decrypt data (stub — will delegate to BearDog)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    /// Hash data (stub — will delegate to BearDog)
    pub fn hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        Ok(hasher.finish().to_be_bytes().to_vec())
    }

    /// Generate random bytes (stub — will delegate to BearDog)
    pub fn generate_random(&self, size: usize) -> Result<Vec<u8>> {
        Ok(vec![0u8; size])
    }

    /// Verify signature (stub — will delegate to BearDog)
    #[expect(
        clippy::missing_const_for_fn,
        reason = "will be non-const when BearDog crypto is integrated"
    )]
    pub fn verify_signature(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        Ok(data.len() == signature.len())
    }
}

impl Default for DefaultCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}
