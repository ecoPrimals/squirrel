//! Encryption module for MCP
//!
//! This module provides encryption functionality for sensitive data in MCP.

use crate::config::SecurityConfig;
use crate::error::{Result, SecurityError, MCPError};
use crate::protocol::types::EncryptionFormat;
use crate::security::crypto;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};
use ring::{aead, hmac};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Encryption trait for MCP components
#[async_trait]
pub trait Encryption: Send + Sync {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8], format: EncryptionFormat) -> Result<Vec<u8>>;
    /// Decrypt data
    async fn decrypt(&self, data: &[u8], format: EncryptionFormat) -> Result<Vec<u8>>;
    /// Generate a new encryption key
    async fn generate_key(&self, format: EncryptionFormat) -> Result<Vec<u8>>;
}

/// Encryption manager for MCP
#[derive(Debug)]
pub struct EncryptionManager {
    /// Default encryption format
    default_format: EncryptionFormat,
    /// Encryption keys by format
    keys: RwLock<std::collections::HashMap<EncryptionFormat, Vec<u8>>>,
}

impl EncryptionManager {
    /// Create a new encryption manager
    #[must_use] pub fn new(default_format: EncryptionFormat) -> Self {
        Self { 
            default_format,
            keys: RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Get the default encryption format
    pub const fn default_format(&self) -> EncryptionFormat {
        self.default_format
    }

    /// Set the default encryption format
    pub fn set_default_format(&mut self, format: EncryptionFormat) {
        self.default_format = format;
    }
    
    /// Get or generate a key for the specified format
    async fn get_or_generate_key(&self, format: EncryptionFormat) -> Result<Vec<u8>> {
        // Early return for None format
        if format == EncryptionFormat::None {
            return Ok(Vec::new());
        }
        
        // First check if the key exists (read lock)
        {
            let keys = self.keys.read().await;
            if let Some(key) = keys.get(&format) {
                return Ok(key.clone());
            }
        } // read lock is dropped here
        
        // Key doesn't exist, generate and store it with write lock
        let new_key = crypto::generate_key(format);
        {
            let mut keys = self.keys.write().await;
            // Check again in case another thread generated the key while we were waiting
            if keys.contains_key(&format) {
                // Another thread generated the key, use that one
                return Ok(keys.get(&format).cloned().unwrap_or_default());
            }
            keys.insert(format, new_key.clone());
        } // write lock is dropped here
        
        Ok(new_key)
    }
    
    /// Set a key for the specified format.
    ///
    /// Stores the provided `key` for the given `format`. If the format is
    /// `EncryptionFormat::None`, this operation is a no-op and returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `format` - The encryption format the key is for.
    /// * `key` - The cryptographic key bytes.
    ///
    /// # Errors
    ///
    /// This function generally returns `Ok(())`.
    /// However, it might theoretically return an error if the internal lock
    /// protecting the key map becomes poisoned due to a panic in another thread
    /// holding the lock.
    pub async fn set_key(&self, format: EncryptionFormat, key: Vec<u8>) -> Result<()> {
        if format == EncryptionFormat::None {
            return Ok(());
        }
        
        // Scope the write lock to minimize duration
        {
            let mut keys = self.keys.write().await;
            keys.insert(format, key);
        } // write lock is dropped here
        
        Ok(())
    }
}

#[async_trait]
impl Encryption for EncryptionManager {
    #[instrument(skip(self, data))]
    async fn encrypt(&self, data: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
        // Get the appropriate key for this format
        let key = self.get_or_generate_key(format).await?;
        
        // Use the crypto module to encrypt the data
        let encrypted = crypto::encrypt(data, &key, format)?;
        
        debug!("Encrypted {} bytes using {:?}", data.len(), format);
        Ok(encrypted)
    }

    #[instrument(skip(self, data))]
    async fn decrypt(&self, data: &[u8], format: EncryptionFormat) -> Result<Vec<u8>> {
        // Get the appropriate key for this format
        let key = self.get_or_generate_key(format).await?;
        
        // Use the crypto module to decrypt the data
        let decrypted = crypto::decrypt(data, &key, format)?;
        
        debug!("Decrypted {} bytes using {:?}", data.len(), format);
        Ok(decrypted)
    }

    #[instrument(skip(self))]
    async fn generate_key(&self, format: EncryptionFormat) -> Result<Vec<u8>> {
        // Generate a new key for this format
        let new_key = crypto::generate_key(format);
        
        // Store the key for future use
        if format != EncryptionFormat::None {
            // Scope the write lock to minimize duration
            {
                let mut keys = self.keys.write().await;
                keys.insert(format, new_key.clone());
            } // write lock is dropped here
        }
        
        debug!("Generated new key for {:?}", format);
        Ok(new_key)
    }
}

/// Create a new encryption manager with default settings
#[must_use] pub fn create_encryption_manager() -> Arc<dyn Encryption> {
    Arc::new(EncryptionManager::new(EncryptionFormat::Aes256Gcm))
}

/// Create a new encryption manager with a specific format
#[must_use] pub fn create_encryption_manager_with_format(format: EncryptionFormat) -> Arc<dyn Encryption> {
    Arc::new(EncryptionManager::new(format))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let manager = EncryptionManager::new(EncryptionFormat::Aes256Gcm);
        
        let data = b"test encryption data for secure communication";
        
        // Encrypt using the manager
        let encrypted = manager.encrypt(data, EncryptionFormat::Aes256Gcm).await.unwrap();
        
        // Make sure it's actually encrypted (different from original)
        assert_ne!(encrypted, data);
        
        // Decrypt using the manager
        let decrypted = manager.decrypt(&encrypted, EncryptionFormat::Aes256Gcm).await.unwrap();
        
        // Verify that we got the original data back
        assert_eq!(decrypted, data);
    }
    
    #[tokio::test]
    async fn test_format_specific_keys() {
        let manager = EncryptionManager::new(EncryptionFormat::Aes256Gcm);
        
        let data = b"multi-format test data";
        
        // Generate keys for both formats
        let aes_key = manager.generate_key(EncryptionFormat::Aes256Gcm).await.unwrap();
        let chacha_key = manager.generate_key(EncryptionFormat::ChaCha20Poly1305).await.unwrap();
        
        // Encrypt with each format
        let aes_encrypted = manager.encrypt(data, EncryptionFormat::Aes256Gcm).await.unwrap();
        let chacha_encrypted = manager.encrypt(data, EncryptionFormat::ChaCha20Poly1305).await.unwrap();
        
        // Verify the two ciphertexts are different
        assert_ne!(aes_encrypted, chacha_encrypted);
        
        // Decrypt with each format
        let aes_decrypted = manager.decrypt(&aes_encrypted, EncryptionFormat::Aes256Gcm).await.unwrap();
        let chacha_decrypted = manager.decrypt(&chacha_encrypted, EncryptionFormat::ChaCha20Poly1305).await.unwrap();
        
        // Both should return the original data
        assert_eq!(aes_decrypted, data);
        assert_eq!(chacha_decrypted, data);
    }
    
    #[tokio::test]
    async fn test_set_key() {
        let manager = EncryptionManager::new(EncryptionFormat::Aes256Gcm);
        
        // Generate a key externally
        let external_key = crypto::generate_key(EncryptionFormat::Aes256Gcm).unwrap();
        
        // Set it in the manager
        manager.set_key(EncryptionFormat::Aes256Gcm, external_key.clone()).await.unwrap();
        
        // Use it for encryption
        let data = b"data encrypted with external key";
        let encrypted = manager.encrypt(data, EncryptionFormat::Aes256Gcm).await.unwrap();
        
        // Decrypt directly with the external key to verify
        let decrypted = crypto::decrypt(&encrypted, &external_key, EncryptionFormat::Aes256Gcm).unwrap();
        
        assert_eq!(decrypted, data);
    }
    
    #[tokio::test]
    async fn test_default_format() {
        let mut manager = EncryptionManager::new(EncryptionFormat::Aes256Gcm);
        
        // Check default format
        assert_eq!(manager.default_format(), EncryptionFormat::Aes256Gcm);
        
        // Change default format
        manager.set_default_format(EncryptionFormat::ChaCha20Poly1305);
        
        // Verify it changed
        assert_eq!(manager.default_format(), EncryptionFormat::ChaCha20Poly1305);
    }
    
    #[tokio::test]
    async fn test_none_format() {
        let manager = EncryptionManager::new(EncryptionFormat::None);
        
        let data = b"unencrypted data";
        
        // "Encrypt" with None format
        let result = manager.encrypt(data, EncryptionFormat::None).await.unwrap();
        
        // Should be unchanged
        assert_eq!(result, data);
        
        // "Decrypt" with None format
        let decrypted = manager.decrypt(&result, EncryptionFormat::None).await.unwrap();
        
        // Still unchanged
        assert_eq!(decrypted, data);
    }
} 