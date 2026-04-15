// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Cryptographic provider for MCP security
//!
//! This module provides cryptographic functionality for the MCP system.
//! Hashing uses BLAKE3; signatures use Ed25519 ([`ed25519_dalek`]). Symmetric protection uses a
//! BLAKE3-derived key and keyed XOF keystream (not a full AEAD; enable at-rest encryption only with
//! awareness of the threat model). Deeper HSM or policy integration remains the responsibility of
//! BearDog or another security primal when deployed.

use crate::error::{MCPError, Result};
use blake3::derive_key;
use ed25519_dalek::{Signature, Signer, SigningKey};
use std::fmt;
use std::sync::Arc;

const SYMMETRIC_CONTEXT: &str = "ecoPrimals squirrel-mcp DefaultCryptoProvider v1";

/// Cryptographic provider using BLAKE3 and Ed25519.
#[derive(Clone)]
pub struct DefaultCryptoProvider {
    state: Arc<CryptoState>,
}

impl fmt::Debug for DefaultCryptoProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultCryptoProvider")
            .finish_non_exhaustive()
    }
}

struct CryptoState {
    signing_key: SigningKey,
}

impl DefaultCryptoProvider {
    /// Create a new crypto provider with a random Ed25519 signing key.
    #[must_use]
    pub fn new() -> Self {
        let mut secret = [0u8; 32];
        rand::fill(&mut secret);
        let signing_key = SigningKey::from_bytes(&secret);
        Self {
            state: Arc::new(CryptoState { signing_key }),
        }
    }

    /// Encrypt `data` using a keyed BLAKE3 XOF stream (prepends a random 32-byte nonce).
    ///
    /// When `SecurityConfig.enable_encryption` is off, [`crate::security::manager::SecurityManagerImpl`]
    /// bypasses this path and returns plaintext.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let sym_key = derive_key(
            SYMMETRIC_CONTEXT,
            self.state.signing_key.to_bytes().as_slice(),
        );

        let mut nonce = [0_u8; 32];
        rand::fill(&mut nonce);

        let mut keystream = vec![0_u8; data.len()];
        let mut hasher = blake3::Hasher::new_keyed(&sym_key);
        hasher.update(&nonce);
        hasher.finalize_xof().fill(&mut keystream);

        let mut out = Vec::with_capacity(32 + data.len());
        out.extend_from_slice(&nonce);
        for (i, b) in data.iter().enumerate() {
            out.push(b ^ keystream[i]);
        }
        Ok(out)
    }

    /// Decrypt a payload produced by [`Self::encrypt`].
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 32 {
            return Err(MCPError::InvalidArgument(
                "ciphertext too short for nonce".to_string(),
            ));
        }
        let sym_key = derive_key(
            SYMMETRIC_CONTEXT,
            self.state.signing_key.to_bytes().as_slice(),
        );

        let (nonce, body) = data.split_at(32);
        let mut keystream = vec![0_u8; body.len()];
        let mut hasher = blake3::Hasher::new_keyed(&sym_key);
        hasher.update(nonce);
        hasher.finalize_xof().fill(&mut keystream);

        let mut plain = Vec::with_capacity(body.len());
        for (i, b) in body.iter().enumerate() {
            plain.push(b ^ keystream[i]);
        }
        Ok(plain)
    }

    /// Cryptographic hash (BLAKE3, 32 bytes).
    pub fn hash(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(blake3::hash(data).as_bytes().to_vec())
    }

    /// Sign `data` with Ed25519 using this provider's secret key (64-byte canonical signature).
    #[must_use]
    pub fn sign(&self, data: &[u8]) -> [u8; 64] {
        self.state.signing_key.sign(data).to_bytes()
    }

    /// CSPRNG-backed random bytes (OS RNG).
    pub fn generate_random(&self, size: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0_u8; size];
        rand::fill(&mut buf[..]);
        Ok(buf)
    }

    /// Verify an Ed25519 signature over `data` using this provider's public key.
    ///
    /// `signature` must be a 64-byte canonical Ed25519 signature.
    pub fn verify_signature(&self, data: &[u8], signature: &[u8]) -> Result<bool> {
        let sig = Signature::from_slice(signature).map_err(|_| {
            MCPError::Security("invalid Ed25519 signature length or encoding".to_string())
        })?;
        let vk = self.state.signing_key.verifying_key();
        Ok(vk.verify_strict(data, &sig).is_ok())
    }
}

impl Default for DefaultCryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "Crypto tests use expect on infallible encrypt/decrypt paths"
    )]

    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let p = DefaultCryptoProvider::new();
        let plain = b"hello mcp";
        let ct = p.encrypt(plain).expect("encrypt");
        let out = p.decrypt(&ct).expect("decrypt");
        assert_eq!(out, plain);
    }

    #[test]
    fn hash_length() {
        let p = DefaultCryptoProvider::new();
        let h = p.hash(b"x").expect("hash");
        assert_eq!(h.len(), 32);
    }

    #[test]
    fn sign_and_verify() {
        let p = DefaultCryptoProvider::new();
        let msg = b"message";
        let sig = p.sign(msg);
        assert!(p.verify_signature(msg, sig.as_slice()).expect("verify"));
    }
}
