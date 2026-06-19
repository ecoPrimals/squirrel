// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! BTSP Phase 3 — Encrypted framing primitives.
//!
//! After `btsp.negotiate` agrees on `chacha20-poly1305`, both sides derive
//! directional session keys and switch the connection to length-prefixed
//! encrypted frames.
//!
//! ## Wire format
//!
//! ```text
//! [4B BE u32 len][12B nonce][ciphertext + 16B Poly1305 tag]
//! ```
//!
//! `len` covers the entire payload after the 4-byte header (nonce + ciphertext + tag).
//!
//! ## Key derivation
//!
//! ```text
//! salt = client_nonce || server_nonce
//! c2s_key = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-c2s")
//! s2c_key = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-s2c")
//! ```
//!
//! ## Reference
//!
//! biomeOS v3.38 and rhizoCrypt `serve_after_handshake` define the convergence
//! pattern. This module aligns with the ecosystem wire format (base64 32-byte
//! nonces, per BearDog/sweetGrass).

#[cfg(test)]
mod tests;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit,
    aead::{Aead, Nonce},
};
use hkdf::Hkdf;
use rand::Rng;
use sha2::Sha256;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Maximum encrypted frame size (4 MiB) — prevents amplification attacks.
const MAX_ENCRYPTED_FRAME: usize = 4 * 1024 * 1024;

/// ChaCha20-Poly1305 nonce size in bytes.
const NONCE_SIZE: usize = 12;

/// ChaCha20-Poly1305 Poly1305 tag size in bytes.
const TAG_SIZE: usize = 16;

// ── Error types ─────────────────────────────────────────────────────────

/// Errors from encrypted frame operations.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(not(test), allow(dead_code))]
pub enum FrameError {
    #[error("encryption failed: {0}")]
    Encryption(String),
    #[error("decryption failed: {0}")]
    Decryption(String),
    #[error("frame too large: {size} bytes (max {MAX_ENCRYPTED_FRAME})")]
    FrameTooLarge { size: usize },
    #[error("frame too short: {size} bytes (need at least {NONCE_SIZE} + {TAG_SIZE} + 1)")]
    FrameTooShort { size: usize },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
}

// ── Session keys ────────────────────────────────────────────────────────

/// Directional session keys derived after cipher negotiation.
///
/// Derives `Zeroize`/`ZeroizeOnDrop` so keys are securely erased from memory
/// when the session ends.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    /// Client-to-server key (32 bytes for ChaCha20-Poly1305).
    pub c2s_key: [u8; 32],
    /// Server-to-client key (32 bytes for ChaCha20-Poly1305).
    pub s2c_key: [u8; 32],
}

impl std::fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKeys")
            .field("c2s_key", &"[REDACTED]")
            .field("s2c_key", &"[REDACTED]")
            .finish()
    }
}

impl SessionKeys {
    /// Derive directional session keys from handshake material.
    ///
    /// `handshake_key`: shared secret from Phase 2 (raw bytes).
    /// `client_nonce`:  32-byte client nonce.
    /// `server_nonce`:  32-byte server nonce.
    pub fn derive(
        handshake_key: &[u8],
        client_nonce: &[u8],
        server_nonce: &[u8],
    ) -> Result<Self, FrameError> {
        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut c2s_key = [0u8; 32];
        hk.expand(b"btsp-session-v1-c2s", &mut c2s_key)
            .map_err(|e| FrameError::KeyDerivation(format!("c2s: {e}")))?;

        let mut s2c_key = [0u8; 32];
        hk.expand(b"btsp-session-v1-s2c", &mut s2c_key)
            .map_err(|e| FrameError::KeyDerivation(format!("s2c: {e}")))?;

        Ok(Self { c2s_key, s2c_key })
    }
}

// ── Nonce generation ────────────────────────────────────────────────────

/// Generate a cryptographically random 32-byte nonce, returned base64-encoded.
///
/// Matches the ecosystem convergence (BearDog/sweetGrass/biomeOS): 32-byte
/// nonces encoded as base64.
#[must_use]
pub fn generate_server_nonce() -> String {
    let nonce: [u8; 32] = rand::rng().random();
    BASE64.encode(nonce)
}

/// Generate a random 12-byte ChaCha20-Poly1305 frame nonce.
#[cfg_attr(not(test), allow(dead_code))]
fn generate_frame_nonce() -> [u8; NONCE_SIZE] {
    rand::rng().random()
}

// ── Frame encrypt / decrypt ─────────────────────────────────────────────

/// Encrypt `plaintext` into a length-prefixed encrypted frame.
///
/// Returns: `[4B BE u32 len][12B nonce][ciphertext + 16B Poly1305 tag]`
#[cfg_attr(not(test), allow(dead_code))]
pub fn encrypt_frame(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, FrameError> {
    let cipher =
        ChaCha20Poly1305::new_from_slice(key).map_err(|e| FrameError::Encryption(e.to_string()))?;

    let nonce_bytes = generate_frame_nonce();
    let nonce = Nonce::<ChaCha20Poly1305>::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| FrameError::Encryption(e.to_string()))?;

    let payload_len = NONCE_SIZE + ciphertext.len();
    if payload_len > MAX_ENCRYPTED_FRAME {
        return Err(FrameError::FrameTooLarge { size: payload_len });
    }

    let mut frame = Vec::with_capacity(4 + payload_len);
    frame.extend_from_slice(&(payload_len as u32).to_be_bytes());
    frame.extend_from_slice(&nonce_bytes);
    frame.extend_from_slice(&ciphertext);
    Ok(frame)
}

/// Decrypt an encrypted payload (nonce + ciphertext + tag, without the length header).
///
/// `payload` must be at least `NONCE_SIZE + TAG_SIZE + 1` bytes.
#[cfg_attr(not(test), allow(dead_code))]
pub fn decrypt_frame(key: &[u8; 32], payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    let min_len = NONCE_SIZE + TAG_SIZE + 1;
    if payload.len() < min_len {
        return Err(FrameError::FrameTooShort {
            size: payload.len(),
        });
    }

    let (nonce_bytes, ciphertext) = payload.split_at(NONCE_SIZE);
    let cipher =
        ChaCha20Poly1305::new_from_slice(key).map_err(|e| FrameError::Decryption(e.to_string()))?;
    let nonce = Nonce::<ChaCha20Poly1305>::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| FrameError::Decryption(e.to_string()))
}

// ── Async frame I/O ─────────────────────────────────────────────────────

/// Read one encrypted frame from the stream and decrypt it.
///
/// Wire format: `[4B BE u32 len][payload]` where payload = `[12B nonce][ciphertext+tag]`.
#[cfg_attr(not(test), allow(dead_code))]
pub async fn read_encrypted_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    key: &[u8; 32],
) -> Result<Vec<u8>, FrameError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > MAX_ENCRYPTED_FRAME {
        return Err(FrameError::FrameTooLarge { size: len });
    }
    if len < NONCE_SIZE + TAG_SIZE + 1 {
        return Err(FrameError::FrameTooShort { size: len });
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;

    decrypt_frame(key, &payload)
}

/// Encrypt plaintext and write as a length-prefixed frame to the stream.
#[cfg_attr(not(test), allow(dead_code))]
pub async fn write_encrypted_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    key: &[u8; 32],
    plaintext: &[u8],
) -> Result<(), FrameError> {
    let frame = encrypt_frame(key, plaintext)?;
    writer.write_all(&frame).await?;
    writer.flush().await?;
    Ok(())
}

/// Decode a base64-encoded nonce, returning the raw bytes.
///
/// Accepts both standard and URL-safe base64 for interoperability.
pub fn decode_nonce(encoded: &str) -> Result<Vec<u8>, FrameError> {
    BASE64
        .decode(encoded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(encoded))
        .map_err(|e| FrameError::KeyDerivation(format!("nonce decode: {e}")))
}
