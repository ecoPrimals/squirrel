//! Cryptography utilities for the security module
//!
//! This module provides encryption, signing, and hashing functions
//! for secure communication and data protection.

use crate::error::{Result, SecurityError};
use crate::security::types::EncryptionFormat;
use ring::{aead, hmac};
use rand::{RngCore, rngs::OsRng};
use tracing::{debug};
use async_trait::async_trait;
use ring::aead::{BoundKey, NonceSequence};

// AES-256-GCM constants
const AES_256_GCM_KEY_LEN: usize = 32;
const AES_256_GCM_NONCE_LEN: usize = 12;
const AES_256_GCM_TAG_LEN: usize = 16;

// ChaCha20-Poly1305 constants
const CHACHA20_POLY1305_KEY_LEN: usize = 32;
const CHACHA20_POLY1305_NONCE_LEN: usize = 12;
const CHACHA20_POLY1305_TAG_LEN: usize = 16;

// HMAC constants
const HMAC_KEY_LEN: usize = 32;

/// Trait for cryptographic operations
#[async_trait]
pub trait CryptoProvider: Send + Sync {
    /// Encrypt data with the specified format
    async fn encrypt(&self, data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>>;
    
    /// Decrypt data with the specified format
    async fn decrypt(&self, data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>>;
    
    /// Generate a signature for the data
    async fn sign(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;
    
    /// Verify a signature for the data
    async fn verify(&self, data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool>;
    
    /// Generate a random key for the specified encryption format
    fn generate_key(&self, format: EncryptionFormat) -> Vec<u8>;
    
    /// Generate a random HMAC key
    fn generate_hmac_key(&self) -> Vec<u8>;
}

/// Default implementation of the CryptoProvider trait
pub struct DefaultCryptoProvider;

impl DefaultCryptoProvider {
    /// Create a new instance of the default crypto provider
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl CryptoProvider for DefaultCryptoProvider {
    async fn encrypt(&self, data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
        encrypt(data, key, format)
    }
    
    async fn decrypt(&self, data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
        decrypt(data, key, format)
    }
    
    async fn sign(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let signature = hmac::sign(&key, data);
        Ok(signature.as_ref().to_vec())
    }
    
    async fn verify(&self, data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let result = hmac::verify(&key, data, signature);
        Ok(result.is_ok())
    }
    
    fn generate_key(&self, format: EncryptionFormat) -> Vec<u8> {
        generate_key(format)
    }
    
    fn generate_hmac_key(&self) -> Vec<u8> {
        let mut key = vec![0u8; HMAC_KEY_LEN];
        OsRng.fill_bytes(&mut key);
        key
    }
}

/// Generate a random nonce for the specified encryption format
///
/// # Arguments
///
/// * `format` - The encryption format to generate a nonce for
///
/// # Returns
///
/// * `Vec<u8>` - The generated nonce
fn generate_nonce(format: EncryptionFormat) -> Vec<u8> {
    let nonce_len = match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::AesGcm | EncryptionFormat::Aes256Gcm => AES_256_GCM_NONCE_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_NONCE_LEN,
    };

    if nonce_len == 0 {
        return Vec::new();
    }

    let mut nonce = vec![0u8; nonce_len];
    OsRng.fill_bytes(&mut nonce);
    
    nonce
}

/// Generate a random key for the specified encryption format
///
/// # Arguments
///
/// * `format` - The encryption format to generate a key for
///
/// # Returns
///
/// * `Vec<u8>` - The generated key
pub fn generate_key(format: EncryptionFormat) -> Vec<u8> {
    let key_len = match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::AesGcm | EncryptionFormat::Aes256Gcm => AES_256_GCM_KEY_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_KEY_LEN,
    };

    if key_len == 0 {
        return Vec::new();
    }

    let mut key = vec![0u8; key_len];
    OsRng.fill_bytes(&mut key);
    
    debug!("Generated {} byte key for {:?}", key_len, format);
    key
}

/// Get the AEAD algorithm for the specified encryption format
///
/// # Arguments
///
/// * `format` - The encryption format to get the algorithm for
///
/// # Returns
///
/// * `Result<&'static aead::Algorithm>` - The AEAD algorithm
///
/// # Errors
///
/// Returns an error if:
/// * `format` is `EncryptionFormat::None`, as this indicates no encryption algorithm was selected
fn get_aead_algorithm(format: &EncryptionFormat) -> Result<&'static aead::Algorithm> {
    match format {
        EncryptionFormat::None => Err(SecurityError::EncryptionFailed(
            "No encryption algorithm selected".to_string(),
        ).into()),
        EncryptionFormat::AesGcm | EncryptionFormat::Aes256Gcm => Ok(&aead::AES_256_GCM),
        EncryptionFormat::ChaCha20Poly1305 => Ok(&aead::CHACHA20_POLY1305),
    }
}

// Define a struct to implement NonceSequence
struct StaticNonce {
    nonce: [u8; AES_256_GCM_NONCE_LEN],
}

impl StaticNonce {
    fn new(nonce: &[u8]) -> Result<Self> {
        let mut n = [0u8; AES_256_GCM_NONCE_LEN];
        if nonce.len() != AES_256_GCM_NONCE_LEN {
            return Err(SecurityError::EncryptionFailed(format!("Invalid nonce length: got {}, expected {}", nonce.len(), AES_256_GCM_NONCE_LEN)).into());
        }
        n.copy_from_slice(nonce);
        Ok(Self { nonce: n })
    }
}

impl NonceSequence for StaticNonce {
    fn advance(&mut self) -> std::result::Result<aead::Nonce, ring::error::Unspecified> {
        Ok(aead::Nonce::assume_unique_for_key(self.nonce))
    }
}

/// Encrypt data with the specified format
///
/// Returns the encrypted data with the following format:
/// - First N bytes: Nonce (12 bytes for AES-GCM, 12 bytes for ChaCha20-Poly1305)
/// - Remaining bytes: Encrypted data with authentication tag
///
/// # Arguments
///
/// * `data` - Data to encrypt
/// * `key` - Encryption key
/// * `format` - Encryption format to use
///
/// # Returns
///
/// Encrypted data as a byte vector or an error
pub fn encrypt(data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
    let aead_algorithm = get_aead_algorithm(&format)?;
    let nonce = generate_nonce(format);
    
    // Create a vector with space for nonce, data, and tag
    let mut output = Vec::with_capacity(nonce.len() + data.len() + AES_256_GCM_TAG_LEN);
    
    // Add nonce to the beginning of the output
    output.extend_from_slice(&nonce);
    
    // Initialize the AEAD algorithm with key and nonce
    let unbound_key = aead::UnboundKey::new(aead_algorithm, key)
        .map_err(|_| SecurityError::EncryptionFailed("Failed to create key".to_string()))?;
    
    let nonce_sequence = StaticNonce::new(&nonce)?;
    
    let mut sealing_key = aead::SealingKey::new(unbound_key, nonce_sequence);
    
    // Create a mutable buffer for plaintext and tag
    let mut in_out = Vec::with_capacity(data.len() + AES_256_GCM_TAG_LEN);
    in_out.extend_from_slice(data);
    
    // Encrypt the data in place, appending the tag
    let aad = aead::Aad::empty();
    match sealing_key.seal_in_place_append_tag(aad, &mut in_out) {
        Ok(_) => {
            // Add the encrypted data with tag to output after the nonce
            output.extend_from_slice(&in_out);
            Ok(output)
        },
        Err(_) => Err(SecurityError::EncryptionFailed("Failed to encrypt data".to_string()).into()),
    }
}

/// Decrypt data with the specified format
///
/// # Arguments
///
/// * `data` - Encrypted data to decrypt
/// * `key` - Encryption key
/// * `format` - Encryption format to use
///
/// # Returns
///
/// Decrypted data as a byte vector or an error
pub fn decrypt(data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
    let aead_algorithm = get_aead_algorithm(&format)?;
    
    let nonce_len = match format {
        EncryptionFormat::None => return Err(SecurityError::DecryptionFailed("No encryption format selected".to_string()).into()),
        EncryptionFormat::AesGcm | EncryptionFormat::Aes256Gcm => AES_256_GCM_NONCE_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_NONCE_LEN,
    };
    
    if data.len() < nonce_len {
        return Err(SecurityError::DecryptionFailed("Invalid data format: too short for nonce".to_string()).into());
    }
    
    let nonce = &data[..nonce_len];
    
    // Initialize the AEAD algorithm with key and nonce
    let unbound_key = aead::UnboundKey::new(aead_algorithm, key)
        .map_err(|_| SecurityError::DecryptionFailed("Failed to create key".to_string()))?;
    
    let nonce_sequence = StaticNonce::new(nonce)?;
    
    let mut opening_key = aead::OpeningKey::new(unbound_key, nonce_sequence);
    
    // Extract ciphertext and tag
    let mut ciphertext_and_tag = data[nonce_len..].to_vec();
    
    let aad = aead::Aad::empty();
    match opening_key.open_in_place(aad, &mut ciphertext_and_tag) {
        Ok(plaintext) => Ok(plaintext.to_vec()),
        Err(_) => Err(SecurityError::DecryptionFailed("Failed to decrypt data or data is corrupted".to_string()).into()),
    }
}