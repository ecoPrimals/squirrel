//! Cryptography utilities for the security module
//!
//! This module provides encryption, signing, and hashing functions
//! for secure communication and data protection.

use crate::error::{Result, MCPError};
use crate::config::SecurityConfig;
use crate::error::SecurityError;
use crate::protocol::types::EncryptionFormat;
use ring::{aead, hmac};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::{RngCore, rngs::OsRng};
use tracing::{debug, error};
use crate::security::types::{KeyId, SecurityContext, SecurityLevel, Permission, Action};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::security::policies::{PolicyEngine, PolicyRequest, PolicyDecision};

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
        EncryptionFormat::Aes256Gcm => AES_256_GCM_NONCE_LEN,
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
        EncryptionFormat::Aes256Gcm => AES_256_GCM_KEY_LEN,
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
fn get_aead_algorithm(format: EncryptionFormat) -> Result<&'static aead::Algorithm> {
    match format {
        EncryptionFormat::None => Err(MCPError::Security(SecurityError::EncryptionFailed(
            "No encryption algorithm selected".to_string(),
        ))),
        EncryptionFormat::Aes256Gcm => Ok(&aead::AES_256_GCM),
        EncryptionFormat::ChaCha20Poly1305 => Ok(&aead::CHACHA20_POLY1305),
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
/// * `format` - Encryption format
///
/// # Returns
///
/// * `Result<Vec<u8>>` - Encrypted data with nonce prepended
///
/// # Errors
///
/// Returns an error if:
/// * The key length does not match the expected length for the chosen encryption format
/// * Failed to create the encryption key from the provided key bytes
/// * Failed to create a unique nonce
/// * The encryption operation fails
pub fn encrypt(data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
    // If no encryption is specified, return the data as-is
    if format == EncryptionFormat::None {
        return Ok(data.to_vec());
    }

    let algorithm = get_aead_algorithm(format)?;
    
    // Check key length
    let expected_key_len = match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::Aes256Gcm => AES_256_GCM_KEY_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_KEY_LEN,
    };
    
    if key.len() != expected_key_len {
        return Err(MCPError::Security(SecurityError::EncryptionFailed(
            format!("Invalid key length: expected {}, got {}", expected_key_len, key.len()),
        )));
    }

    // Create unbound key
    let unbound_key = aead::UnboundKey::new(algorithm, key).map_err(|err| {
        error!("Failed to create unbound key: {:?}", err);
        MCPError::Security(SecurityError::EncryptionFailed(
            "Failed to create encryption key".to_string(),
        ))
    })?;

    // Generate nonce
    let nonce_vec = generate_nonce(format);
    let nonce = aead::Nonce::try_assume_unique_for_key(&nonce_vec).map_err(|_| {
        MCPError::Security(SecurityError::EncryptionFailed(
            "Failed to create nonce".to_string(),
        ))
    })?;

    // Use LessSafeKey for single message encryption
    let sealing_key = aead::LessSafeKey::new(unbound_key);

    // Prepare buffer for in-place encryption
    let mut in_out = data.to_vec();
    
    // Encrypt in place and append tag
    sealing_key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|err| {
            error!("Encryption failed: {:?}", err);
            MCPError::Security(SecurityError::EncryptionFailed(
                "Encryption operation failed".to_string(),
            ))
        })?;

    // Prepend nonce to the encrypted data
    let mut result = nonce_vec;
    result.extend_from_slice(&in_out);
    
    debug!("Encrypted {} bytes using {:?}", data.len(), format);
    Ok(result)
}

/// Decrypt data with the specified format
///
/// Expects the data in the format returned by `encrypt`:
/// - First N bytes: Nonce (12 bytes for AES-GCM, 12 bytes for ChaCha20-Poly1305)
/// - Remaining bytes: Encrypted data with authentication tag
///
/// # Arguments
///
/// * `encrypted_data` - Encrypted data with nonce prepended
/// * `key` - Decryption key
/// * `format` - Encryption format
///
/// # Returns
///
/// * `Result<Vec<u8>>` - Decrypted data
///
/// # Errors
///
/// Returns an error if:
/// * The key length does not match the expected length for the chosen encryption format
/// * The encrypted data is too short to contain a valid nonce and ciphertext
/// * Failed to create the decryption key from the provided key bytes
/// * Failed to extract the nonce from the encrypted data
/// * The decryption operation fails (e.g., due to data tampering or incorrect key)
pub fn decrypt(encrypted_data: &[u8], key: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
    // If no encryption is specified, return the data as-is
    if format == EncryptionFormat::None {
        return Ok(encrypted_data.to_vec());
    }

    let algorithm = get_aead_algorithm(format)?;
    
    // Check key length
    let expected_key_len = match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::Aes256Gcm => AES_256_GCM_KEY_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_KEY_LEN,
    };
    
    if key.len() != expected_key_len {
        return Err(MCPError::Security(SecurityError::DecryptionFailed(
            format!("Invalid key length: expected {}, got {}", expected_key_len, key.len()),
        )));
    }

    // Determine nonce length
    let nonce_len = match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::Aes256Gcm => AES_256_GCM_NONCE_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_NONCE_LEN,
    };

    // Ensure the encrypted data is long enough to contain the nonce and at least some ciphertext + tag
    let min_len = nonce_len + 1 + match format {
        EncryptionFormat::None => 0,
        EncryptionFormat::Aes256Gcm => AES_256_GCM_TAG_LEN,
        EncryptionFormat::ChaCha20Poly1305 => CHACHA20_POLY1305_TAG_LEN,
    };

    if encrypted_data.len() < min_len {
        return Err(MCPError::Security(SecurityError::DecryptionFailed(
            format!("Encrypted data too short: got {} bytes, need at least {}", encrypted_data.len(), min_len),
        )));
    }

    // Extract nonce
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(nonce_len);
    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| {
        MCPError::Security(SecurityError::DecryptionFailed(
            "Failed to create nonce".to_string(),
        ))
    })?;

    // Create unbound key
    let unbound_key = aead::UnboundKey::new(algorithm, key).map_err(|err| {
        error!("Failed to create unbound key: {:?}", err);
        MCPError::Security(SecurityError::DecryptionFailed(
            "Failed to create decryption key".to_string(),
        ))
    })?;
    
    // Use LessSafeKey for single message decryption
    let opening_key = aead::LessSafeKey::new(unbound_key);

    // Copy ciphertext for in-place decryption
    let mut in_out = ciphertext.to_vec();
    
    // Decrypt in place
    let plaintext = opening_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|err| {
            error!("Decryption failed: {:?}", err);
            MCPError::Security(SecurityError::DecryptionFailed(
                "Decryption operation failed".to_string(),
            ))
        })?;

    debug!("Decrypted {} bytes using {:?}", plaintext.len(), format);
    Ok(plaintext.to_vec())
}

/// Sign data using HMAC-SHA256
///
/// # Arguments
///
/// * `data` - Data to sign
/// * `key` - Signing key
///
/// # Returns
///
/// * `Result<Vec<u8>>` - The HMAC signature
///
/// # Errors
///
/// Returns an error if:
/// * The key length is not equal to the expected HMAC key length (32 bytes)
/// * Failed to create the HMAC key from the provided key bytes
pub fn sign(data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if key.len() != HMAC_KEY_LEN {
        return Err(MCPError::Security(SecurityError::InternalError(
            format!("Invalid HMAC key length: expected {}, got {}", HMAC_KEY_LEN, key.len()),
        )));
    }

    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let signature = hmac::sign(&s_key, data);
    debug!("Signed {} bytes of data", data.len());
    
    Ok(signature.as_ref().to_vec())
}

/// Verify a signature using HMAC-SHA256
///
/// # Arguments
///
/// * `data` - The data that was signed
/// * `signature` - The signature to verify
/// * `key` - The key used for signing
///
/// # Returns
///
/// * `Result<bool>` - True if the signature is valid, false otherwise
///
/// # Errors
///
/// Returns an error if:
/// * The key length is not equal to the expected HMAC key length (32 bytes)
/// * Failed to create the HMAC key from the provided key bytes
/// * The signature verification process fails
pub fn verify(data: &[u8], signature: &[u8], key: &[u8]) -> Result<bool> {
    if key.len() != HMAC_KEY_LEN {
        return Err(MCPError::Security(SecurityError::InternalError(
            format!("Invalid HMAC key length: expected {}, got {}", HMAC_KEY_LEN, key.len()),
        )));
    }

    let v_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    
    let result = hmac::verify(&v_key, data, signature).is_ok();
    debug!("Signature verification result: {}", result);
    
    Ok(result)
}

/// Hash algorithm options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256 hash
    Sha256,
    /// SHA-512 hash
    Sha512,
    /// BLAKE3 hash
    Blake3,
}

/// Hash data using the specified algorithm
///
/// # Arguments
///
/// * `data` - Data to hash
/// * `algorithm` - The hashing algorithm to use
///
/// # Returns
///
/// * `Vec<u8>` - The computed hash
#[must_use]
pub fn hash(data: &[u8], algorithm: HashAlgorithm) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha256 => {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha512 => {
            use sha2::Digest;
            let mut hasher = sha2::Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Blake3 => {
            blake3::hash(data).as_bytes().to_vec()
        }
    }
}

/// Encode a byte slice as a base64 string
#[must_use] pub fn base64_encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

/// Decode a base64 string to bytes
///
/// # Arguments
///
/// * `encoded` - The base64 encoded string.
///
/// # Errors
///
/// Returns `MCPError::Security(SecurityError::InternalError)` if the input
/// string `encoded` is not valid base64 according to the standard alphabet.
pub fn base64_decode(encoded: &str) -> Result<Vec<u8>> {
    BASE64.decode(encoded).map_err(|err| {
        MCPError::Security(SecurityError::InternalError(
            format!("Failed to decode base64: {err}"),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip_aes_gcm() {
        let data = b"test data for AES-GCM encryption";
        let key = generate_key(EncryptionFormat::Aes256Gcm);
        let encrypted = encrypt(data, &key, EncryptionFormat::Aes256Gcm).unwrap();
        let decrypted = decrypt(&encrypted, &key, EncryptionFormat::Aes256Gcm).unwrap();
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_encryption_roundtrip_chacha20_poly1305() {
        let data = b"test data for ChaCha20-Poly1305 encryption";
        let key = generate_key(EncryptionFormat::ChaCha20Poly1305);
        let encrypted = encrypt(data, &key, EncryptionFormat::ChaCha20Poly1305).unwrap();
        let decrypted = decrypt(&encrypted, &key, EncryptionFormat::ChaCha20Poly1305).unwrap();
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn test_signing_verification() {
        let data = b"test data for signing";
        let mut key = [0u8; HMAC_KEY_LEN];
        OsRng.fill_bytes(&mut key);
        
        let signature = sign(data, &key).unwrap();
        let valid = verify(data, &signature, &key).unwrap();
        assert!(valid);
        
        // Test with modified data
        let mut modified_data = data.to_vec();
        modified_data[0] ^= 1;
        let invalid = verify(&modified_data, &signature, &key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_hashing_sha256() {
        let data = b"test data for SHA-256 hashing";
        let hash_result = hash(data, HashAlgorithm::Sha256);
        assert_eq!(hash_result.len(), 32); // SHA-256 produces 32-byte digests
        
        // Hash the same data again to verify determinism
        let hash_result2 = hash(data, HashAlgorithm::Sha256);
        assert_eq!(hash_result, hash_result2);
    }

    #[test]
    fn test_hashing_sha512() {
        let data = b"test data for SHA-512 hashing";
        let hash_result = hash(data, HashAlgorithm::Sha512);
        assert_eq!(hash_result.len(), 64); // SHA-512 produces 64-byte digests
    }
    
    #[test]
    fn test_base64() {
        let data = b"test data for base64 encoding";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());
    }
    
    #[test]
    fn test_key_generation() {
        let aes_key = generate_key(EncryptionFormat::Aes256Gcm);
        assert_eq!(aes_key.len(), AES_256_GCM_KEY_LEN);
        
        let chacha_key = generate_key(EncryptionFormat::ChaCha20Poly1305);
        assert_eq!(chacha_key.len(), CHACHA20_POLY1305_KEY_LEN);
        
        // Make sure two generated keys are different
        let another_key = generate_key(EncryptionFormat::Aes256Gcm);
        assert_ne!(aes_key, another_key);
    }
    
    #[test]
    fn test_wrong_key_length() {
        let data = b"test data";
        let key = vec![0u8; 16]; // Wrong key length for AES-256-GCM
        
        let result = encrypt(data, &key, EncryptionFormat::Aes256Gcm);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_tampered_data() {
        let data = b"test data for integrity check";
        let key = generate_key(EncryptionFormat::Aes256Gcm);
        
        let mut encrypted = encrypt(data, &key, EncryptionFormat::Aes256Gcm).unwrap();
        
        // Tamper with the encrypted data (not the nonce)
        encrypted[AES_256_GCM_NONCE_LEN + 5] ^= 0x01;
        
        // Attempt to decrypt
        let result = decrypt(&encrypted, &key, EncryptionFormat::Aes256Gcm);
        assert!(result.is_err());
    }
} 