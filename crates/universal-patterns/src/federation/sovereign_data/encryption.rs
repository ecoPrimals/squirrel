// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Encryption key management for sovereign data.

use super::super::{FederationError, FederationResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const ENCRYPTION_DELEGATION_MESSAGE: &str =
    "Encryption requires a security capability provider (IPC not available)";

fn encryption_delegation_error() -> FederationError {
    FederationError::SecurityViolation(ENCRYPTION_DELEGATION_MESSAGE.to_string())
}

/// Encryption key manager trait
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait EncryptionKeyManager: Send + Sync {
    /// Generate a new encryption key
    async fn generate_key(&self, algorithm: &str) -> FederationResult<Vec<u8>>;

    /// Encrypt data with the given key
    async fn encrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>>;

    /// Decrypt data with the given key
    async fn decrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>>;

    /// Derive key from password
    async fn derive_key(&self, password: &str, salt: &[u8]) -> FederationResult<Vec<u8>>;
}

/// Default encryption key manager
pub struct DefaultEncryptionKeyManager {
    /// Key storage (reserved for future key persistence)
    #[expect(dead_code, reason = "Phase 2 placeholder — key persistence")]
    keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl DefaultEncryptionKeyManager {
    /// Create a new key manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DefaultEncryptionKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptionKeyManager for DefaultEncryptionKeyManager {
    async fn generate_key(&self, _algorithm: &str) -> FederationResult<Vec<u8>> {
        Err(encryption_delegation_error())
    }

    async fn encrypt(
        &self,
        _data: &[u8],
        _key: &[u8],
        _algorithm: &str,
    ) -> FederationResult<Vec<u8>> {
        Err(encryption_delegation_error())
    }

    async fn decrypt(
        &self,
        _data: &[u8],
        _key: &[u8],
        _algorithm: &str,
    ) -> FederationResult<Vec<u8>> {
        Err(encryption_delegation_error())
    }

    async fn derive_key(&self, _password: &str, _salt: &[u8]) -> FederationResult<Vec<u8>> {
        Err(encryption_delegation_error())
    }
}
