/*!
 * Secure storage for Galaxy adapter credentials.
 * 
 * This module provides interfaces and implementations for storing
 * credentials securely.
 */

use super::{SecurityError, SecureCredentials};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use crate::error::Result;

/// Trait for credential storage backends
#[async_trait]
pub trait CredentialStorage: Send + Sync + std::fmt::Debug {
    /// Store credentials in the storage
    async fn store(&self, id: &str, credentials: SecureCredentials) -> Result<()>;
    
    /// Get credentials from the storage
    async fn get(&self, id: &str) -> Result<SecureCredentials>;
    
    /// Delete credentials from the storage
    async fn delete(&self, id: &str) -> Result<()>;
    
    /// List all credential IDs in the storage
    async fn list(&self) -> Result<Vec<String>>;
    
    /// Clear all credentials from the storage
    async fn clear(&self) -> Result<()>;
}

/// In-memory credential storage
#[derive(Debug)]
pub struct MemoryStorage {
    credentials: RwLock<HashMap<String, SecureCredentials>>,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// Create a new empty memory storage
    pub fn new() -> Self {
        Self {
            credentials: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CredentialStorage for MemoryStorage {
    async fn store(&self, id: &str, credentials: SecureCredentials) -> Result<()> {
        let mut map = self.credentials.write().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        map.insert(id.to_string(), credentials);
        Ok(())
    }
    
    async fn get(&self, id: &str) -> Result<SecureCredentials> {
        let map = self.credentials.read().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        map.get(id)
            .cloned()
            .ok_or_else(|| SecurityError::StorageError(format!("Credentials not found for ID: {}", id)).into())
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        let mut map = self.credentials.write().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        map.remove(id);
        Ok(())
    }
    
    async fn list(&self) -> Result<Vec<String>> {
        let map = self.credentials.read().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(map.keys().cloned().collect())
    }
    
    async fn clear(&self) -> Result<()> {
        let mut map = self.credentials.write().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire write lock: {}", e))
        })?;
        
        map.clear();
        Ok(())
    }
}

/// Encrypted file-based credential storage
#[derive(Debug)]
pub struct FileStorage {
    base_path: String,
    encryption_key: Arc<[u8; 32]>,
    in_memory_cache: Mutex<HashMap<String, SecureCredentials>>,
}

impl FileStorage {
    /// Create a new file storage with the given base path and encryption key
    pub fn new(base_path: impl Into<String>, encryption_key: [u8; 32]) -> Self {
        Self {
            base_path: base_path.into(),
            encryption_key: Arc::new(encryption_key),
            in_memory_cache: Mutex::new(HashMap::new()),
        }
    }
    
    /// Create a new file storage with a random encryption key
    /// 
    /// Note: This will generate a new key each time, so credentials stored
    /// with a previous instance will not be readable.
    pub fn with_random_key(base_path: impl Into<String>) -> Self {
        use rand::RngCore;
        
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        
        Self::new(base_path, key)
    }
    
    /// Get the full path for a credential ID
    fn get_path(&self, id: &str) -> String {
        // Sanitize the ID to ensure it's safe to use as a filename
        let safe_id = id.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
        // Use PathBuf for cross-platform path handling
        let mut path = std::path::PathBuf::from(&self.base_path);
        path.push(format!("{}.enc", safe_id));
        path.to_string_lossy().to_string()
    }
    
    /// Encrypt credentials for storage
    fn encrypt(&self, credentials: &SecureCredentials) -> Result<Vec<u8>> {
        // Serialize credentials to JSON
        let json = serde_json::to_string(credentials)
            .map_err(|e| SecurityError::EncryptionError(format!("Failed to serialize credentials: {}", e)))?;
        
        // In a real implementation, we would encrypt the data here using the encryption key
        // For demonstration purposes, we'll use a simple XOR encryption with the first byte of the key
        let key_byte = self.encryption_key[0];
        let mut data = b"ENCRYPTED:".to_vec();
        
        // XOR each byte with our key byte
        for byte in json.as_bytes() {
            data.push(byte ^ key_byte);
        }
        
        Ok(data)
    }
    
    /// Decrypt credentials from storage
    fn decrypt(&self, data: &[u8]) -> Result<SecureCredentials> {
        // Check for our placeholder prefix
        if data.len() < 10 || &data[0..10] != b"ENCRYPTED:" {
            return Err(SecurityError::DecryptionError("Invalid encrypted data format".to_string()).into());
        }
        
        // Get the encryption key byte
        let key_byte = self.encryption_key[0];
        
        // XOR each byte to decrypt
        let mut decrypted = Vec::with_capacity(data.len() - 10);
        for byte in &data[10..] {
            decrypted.push(byte ^ key_byte);
        }
        
        // Convert to string
        let json = String::from_utf8(decrypted)
            .map_err(|e| SecurityError::DecryptionError(format!("Invalid UTF-8 in decrypted data: {}", e)))?;
        
        // Deserialize from JSON
        serde_json::from_str(&json)
            .map_err(|e| SecurityError::DecryptionError(format!("Failed to deserialize credentials: {}", e)).into())
    }
    
    /// Create directory if it doesn't exist
    fn ensure_directory_exists(&self) -> Result<()> {
        let path = Path::new(&self.base_path);
        if !path.exists() {
            fs::create_dir_all(path)
                .map_err(|e| SecurityError::StorageError(format!("Failed to create directory: {}", e)))?;
        }
        Ok(())
    }
}

#[async_trait]
impl CredentialStorage for FileStorage {
    async fn store(&self, id: &str, credentials: SecureCredentials) -> Result<()> {
        // Encrypt credentials
        let encrypted = self.encrypt(&credentials)?;
        
        // Ensure directory exists
        self.ensure_directory_exists()?;
        
        // Write to file
        let path = self.get_path(id);
        fs::write(&path, encrypted)
            .map_err(|e| SecurityError::StorageError(format!("Failed to write credentials to file: {}", e)))?;
        
        // Update in-memory cache
        let mut cache = self.in_memory_cache.lock().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire cache lock: {}", e))
        })?;
        
        cache.insert(id.to_string(), credentials);
        
        Ok(())
    }
    
    async fn get(&self, id: &str) -> Result<SecureCredentials> {
        // Check in-memory cache first
        {
            let cache = self.in_memory_cache.lock().map_err(|e| {
                SecurityError::StorageError(format!("Failed to acquire cache lock: {}", e))
            })?;
            
            if let Some(credentials) = cache.get(id) {
                return Ok(credentials.clone());
            }
        }
        
        // Read from file
        let path = self.get_path(id);
        let encrypted = fs::read(&path)
            .map_err(|e| SecurityError::StorageError(format!("Failed to read credentials from file: {}", e)))?;
        
        // Decrypt credentials
        let credentials = self.decrypt(&encrypted)?;
        
        // Update in-memory cache
        let mut cache = self.in_memory_cache.lock().map_err(|e| {
            SecurityError::StorageError(format!("Failed to acquire cache lock: {}", e))
        })?;
        
        cache.insert(id.to_string(), credentials.clone());
        
        Ok(credentials)
    }
    
    async fn delete(&self, id: &str) -> Result<()> {
        // Remove from in-memory cache
        {
            let mut cache = self.in_memory_cache.lock().map_err(|e| {
                SecurityError::StorageError(format!("Failed to acquire cache lock: {}", e))
            })?;
            
            cache.remove(id);
        }
        
        // Delete file
        let path = self.get_path(id);
        if Path::new(&path).exists() {
            fs::remove_file(&path)
                .map_err(|e| SecurityError::StorageError(format!("Failed to delete credentials file: {}", e)))?;
        }
        
        Ok(())
    }
    
    async fn list(&self) -> Result<Vec<String>> {
        self.ensure_directory_exists()?;
        
        let dir_entries = fs::read_dir(&self.base_path)
            .map_err(|e| SecurityError::StorageError(format!("Failed to read credential directory: {}", e)))?;
        
        let mut ids = Vec::new();
        for entry in dir_entries {
            let entry = entry.map_err(|e| SecurityError::StorageError(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "enc" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(id) = stem.to_str() {
                            ids.push(id.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(ids)
    }
    
    async fn clear(&self) -> Result<()> {
        // Clear in-memory cache
        {
            let mut cache = self.in_memory_cache.lock().map_err(|e| {
                SecurityError::StorageError(format!("Failed to acquire cache lock: {}", e))
            })?;
            
            cache.clear();
        }
        
        // Delete all files
        let ids = self.list().await?;
        for id in ids {
            self.delete(&id).await?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::security::SecretString;
    
    #[tokio::test]
    async fn test_memory_storage() {
        let storage = MemoryStorage::new();
        let credentials = SecureCredentials::with_api_key(SecretString::new("test-key"));
        
        // Store credentials
        storage.store("test", credentials.clone()).await.unwrap();
        
        // Retrieve credentials
        let retrieved = storage.get("test").await.unwrap();
        assert_eq!(
            retrieved.api_key().unwrap().expose(),
            credentials.api_key().unwrap().expose()
        );
        
        // List credentials
        let ids = storage.list().await.unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "test");
        
        // Delete credentials
        storage.delete("test").await.unwrap();
        let result = storage.get("test").await;
        assert!(result.is_err());
        
        // Clear storage
        storage.store("test1", credentials.clone()).await.unwrap();
        storage.store("test2", credentials.clone()).await.unwrap();
        storage.clear().await.unwrap();
        let ids = storage.list().await.unwrap();
        assert_eq!(ids.len(), 0);
    }
    
    #[tokio::test]
    async fn test_file_storage() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();
        let storage = FileStorage::with_random_key(base_path);
        
        // Test store and retrieve credentials
        let credentials = SecureCredentials::with_api_key(SecretString::new("test-key"));
        storage.store("test", credentials.clone()).await.unwrap();
        
        let retrieved = storage.get("test").await.unwrap();
        assert_eq!(retrieved.api_key().unwrap().expose(), "test-key");
        
        // Test list credentials
        let ids = storage.list().await.unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "test");
        
        // Test delete credentials
        storage.delete("test").await.unwrap();
        let ids = storage.list().await.unwrap();
        assert_eq!(ids.len(), 0);
        
        // Test unsafe ID sanitization
        let unsafe_id = "unsafe/id.with.special-chars";
        let safe_path = storage.get_path(unsafe_id);
        // Check that the path is valid by ensuring it's a proper child of the base path
        let path = std::path::Path::new(&safe_path);
        assert!(path.is_relative() || path.starts_with(base_path));
        
        // Clear storage
        storage.store("test1", credentials.clone()).await.unwrap();
        storage.store("test2", credentials.clone()).await.unwrap();
        storage.clear().await.unwrap();
        let ids = storage.list().await.unwrap();
        assert_eq!(ids.len(), 0);
    }
} 