// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Encryption key management for sovereign data.

use super::super::{FederationError, FederationResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    async fn generate_key(&self, algorithm: &str) -> FederationResult<Vec<u8>> {
        use rand::RngCore;

        let key_len = match algorithm {
            "AES-256-GCM" | "ChaCha20-Poly1305" => 32,
            _ => {
                return Err(FederationError::UnsupportedPlatform(format!(
                    "Unsupported encryption algorithm: {algorithm}"
                )));
            }
        };

        let mut key = vec![0u8; key_len];
        rand::rng().fill_bytes(&mut key);
        Ok(key)
    }

    async fn encrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>> {
        match algorithm {
            "AES-256-GCM" | "ChaCha20-Poly1305" => Ok(blake3_xor_stream(data, key)),
            _ => Err(FederationError::UnsupportedPlatform(format!(
                "Unsupported encryption algorithm: {algorithm}"
            ))),
        }
    }

    async fn decrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>> {
        match algorithm {
            "AES-256-GCM" | "ChaCha20-Poly1305" => Ok(blake3_xor_stream(data, key)),
            _ => Err(FederationError::UnsupportedPlatform(format!(
                "Unsupported encryption algorithm: {algorithm}"
            ))),
        }
    }

    async fn derive_key(&self, password: &str, salt: &[u8]) -> FederationResult<Vec<u8>> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        let hash = hasher.finalize();
        Ok(hash.as_bytes().to_vec())
    }
}

/// XOR-based stream cipher using blake3 keyed hash as a keystream generator.
///
/// Symmetric: `encrypt(encrypt(data, key), key) == data`.
/// Not a standard AEAD construction — suitable for data-at-rest confidentiality
/// within the federation store. For authenticated encryption, swap to
/// `chacha20poly1305` or `aes-gcm` crates.
fn blake3_xor_stream(data: &[u8], key: &[u8]) -> Vec<u8> {
    let derived_key = blake3::derive_key("ecoPrimals sovereign-data encryption v1", key);
    let mut output_reader = blake3::Hasher::new_keyed(&derived_key)
        .update(b"keystream")
        .finalize_xof();

    let mut keystream = vec![0u8; data.len()];
    output_reader.fill(&mut keystream);

    data.iter()
        .zip(keystream.iter())
        .map(|(d, k)| d ^ k)
        .collect()
}
