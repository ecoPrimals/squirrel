use async_trait::async_trait;
use crate::core::error::Result;
use super::types::{Key, KeyType};

#[async_trait]
pub trait EncryptionProvider: Send + Sync + 'static {
    /// Encrypt data
    async fn encrypt(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;

    /// Decrypt data
    async fn decrypt(&self, data: &[u8], key_id: &str) -> Result<Vec<u8>>;

    /// Generate a new key
    async fn generate_key(&self, key_type: KeyType) -> Result<Key>;

    /// Import an existing key
    async fn import_key(&self, key_data: &[u8], key_type: KeyType) -> Result<Key>;
} 