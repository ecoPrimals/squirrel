use async_trait::async_trait;
use crate::error::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Purpose of a cryptographic key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyPurpose {
    /// Key used for encryption/decryption
    Encryption,
    /// Key used for signing/verification
    Signing,
    /// Key used for token generation/validation
    TokenSigning,
}

/// Trait for secure key storage operations
#[async_trait]
pub trait KeyStorage: Send + Sync {
    /// Get a key by its ID
    async fn get_key(&self, key_id: &str) -> Result<Vec<u8>>;
    
    /// Store a new key with a generated ID and return the ID
    async fn store_key(&self, key: Vec<u8>, purpose: KeyPurpose) -> Result<String>;
    
    /// Get the latest key for a specific purpose
    async fn get_latest_key(&self, purpose: KeyPurpose) -> Result<(String, Vec<u8>)>;
    
    /// Delete a key by its ID
    async fn delete_key(&self, key_id: &str) -> Result<()>;
}

/// Simple in-memory implementation of KeyStorage for development/testing
pub struct InMemoryKeyStorage {
    keys: RwLock<HashMap<String, (Vec<u8>, KeyPurpose)>>,
    purpose_to_latest_key: RwLock<HashMap<KeyPurpose, String>>,
}

impl InMemoryKeyStorage {
    /// Create a new instance of the in-memory key storage
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            purpose_to_latest_key: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl KeyStorage for InMemoryKeyStorage {
    async fn get_key(&self, key_id: &str) -> Result<Vec<u8>> {
        let keys = self.keys.read().await;
        keys.get(key_id)
            .map(|key| key.0.clone())
            .ok_or_else(|| crate::error::SecurityError::NotFound(key_id.to_string()).into())
    }
    
    async fn store_key(&self, key: Vec<u8>, purpose: KeyPurpose) -> Result<String> {
        let key_id = Uuid::new_v4().to_string();
        
        // Store the key
        {
            let mut keys = self.keys.write().await;
            keys.insert(key_id.clone(), (key, purpose.clone()));
        }
        
        // Update the latest key for this purpose
        {
            let mut purpose_map = self.purpose_to_latest_key.write().await;
            purpose_map.insert(purpose, key_id.clone());
        }
        
        info!(key_id = %key_id, "Stored new key");
        Ok(key_id)
    }
    
    async fn get_latest_key(&self, purpose: KeyPurpose) -> Result<(String, Vec<u8>)> {
        // Get the ID of the latest key for this purpose
        let key_id = {
            let purpose_map = self.purpose_to_latest_key.read().await;
            purpose_map.get(&purpose)
                .cloned()
                .ok_or_else(|| crate::error::SecurityError::NotFound(format!("{:?}", purpose)))?
        };
        
        // Get the key itself
        let key = self.get_key(&key_id).await?;
        
        Ok((key_id, key))
    }
    
    async fn delete_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        let removed = keys.remove(key_id);
        
        if let Some((_, purpose)) = removed {
            // Check if this was the latest key for its purpose and remove the reference if so
            let mut purpose_map = self.purpose_to_latest_key.write().await;
            if let Some(latest_key_id) = purpose_map.get(&purpose) {
                if latest_key_id == key_id {
                    purpose_map.remove(&purpose);
                }
            }
            
            info!(key_id = %key_id, "Deleted key");
            Ok(())
        } else {
            Err(crate::error::SecurityError::NotFound(key_id.to_string()).into())
        }
    }
} 