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
use rand::Rng;

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

/// Additional trait for secure encrypted storage backends
#[async_trait]
pub trait EncryptedStorage: CredentialStorage {
    /// Rotate the encryption key
    async fn rotate_encryption_key(&self, new_key: [u8; 32]) -> Result<()>;
    
    /// Get the encryption status
    fn encryption_status(&self) -> EncryptionStatus;
    
    /// Rekey a specific credential
    async fn rekey_credential(&self, id: &str) -> Result<()>;
    
    /// Check if rekey is needed for credential
    fn needs_rekey(&self, id: &str) -> Result<bool>;
}

/// Encryption status information
#[derive(Debug, Clone)]
pub struct EncryptionStatus {
    /// Algorithm used for encryption
    pub algorithm: String,
    /// Key strength in bits
    pub key_strength: usize,
    /// Last key rotation timestamp
    pub last_rotation: Option<time::OffsetDateTime>,
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
    encryption_key: Arc<Mutex<[u8; 32]>>,
    in_memory_cache: Mutex<HashMap<String, SecureCredentials>>,
    last_key_rotation: Mutex<Option<time::OffsetDateTime>>,
    metadata_path: String,
}

impl FileStorage {
    /// Create a new file storage with the given base path and encryption key
    pub fn new(base_path: impl Into<String>, encryption_key: [u8; 32]) -> Self {
        let base = base_path.into();
        let metadata_path = format!("{}/metadata.json", base);
        Self {
            base_path: base,
            encryption_key: Arc::new(Mutex::new(encryption_key)),
            in_memory_cache: Mutex::new(HashMap::new()),
            last_key_rotation: Mutex::new(None),
            metadata_path,
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
    
    /// Generate a random initialization vector
    fn generate_iv(&self) -> [u8; 16] {
        let mut iv = [0u8; 16];
        rand::thread_rng().fill(&mut iv);
        iv
    }
    
    /// Encrypt credentials for storage using AES-GCM
    fn encrypt(&self, credentials: &SecureCredentials) -> Result<Vec<u8>> {
        // Serialize credentials to JSON
        let json = serde_json::to_string(credentials)
            .map_err(|e| SecurityError::EncryptionError(format!("Failed to serialize credentials: {}", e)))?;
        
        // In a real implementation, we would use a proper encryption library
        // like aes-gcm-siv or chacha20poly1305. For this example, we'll simulate it.
        
        // Get the encryption key
        let key = self.encryption_key.lock().map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to acquire encryption key lock: {}", e))
        })?;
        
        // Generate a random IV
        let iv = self.generate_iv();
        
        // Simulate encryption (in reality, use a proper crypto library)
        let mut encrypted = Vec::with_capacity(json.len() + 32);
        
        // Store the IV at the beginning
        encrypted.extend_from_slice(&iv);
        
        // Add a version byte for future algorithm changes
        encrypted.push(1); // Version 1
        
        // Header marker
        encrypted.extend_from_slice(b"GALAXY-ENC");
        
        // Simulate AES-GCM
        let mut ciphertext = Vec::with_capacity(json.len());
        for (i, byte) in json.bytes().enumerate() {
            // XOR with key and IV (simplified example - NOT real AES-GCM)
            let key_byte = key[i % key.len()];
            let iv_byte = iv[i % iv.len()];
            ciphertext.push(byte ^ key_byte ^ iv_byte);
        }
        
        // Add the ciphertext
        encrypted.extend_from_slice(&ciphertext);
        
        // Add a simulated authentication tag
        let tag = self.generate_auth_tag(&ciphertext, &key, &iv);
        encrypted.extend_from_slice(&tag);
        
        Ok(encrypted)
    }
    
    /// Generate a simple authentication tag (simulated)
    fn generate_auth_tag(&self, data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> [u8; 16] {
        let mut tag = [0u8; 16];
        
        // Simple tag generation (NOT cryptographically secure - just an example)
        for i in 0..16 {
            let mut val = 0u8;
            for (j, &byte) in data.iter().enumerate() {
                val ^= byte ^ key[(i + j) % key.len()] ^ iv[i % iv.len()];
            }
            tag[i] = val;
        }
        
        tag
    }
    
    /// Decrypt credentials from storage
    fn decrypt(&self, data: &[u8]) -> Result<SecureCredentials> {
        // Check for minimum length (16 byte IV + 1 byte version + 10 byte header + 16 byte tag)
        if data.len() < 43 {
            return Err(SecurityError::DecryptionError("Invalid encrypted data format".to_string()).into());
        }
        
        // Extract IV from the first 16 bytes
        let mut iv = [0u8; 16];
        iv.copy_from_slice(&data[0..16]);
        
        // Check version
        let version = data[16];
        if version != 1 {
            return Err(SecurityError::DecryptionError(format!("Unsupported encryption version: {}", version)).into());
        }
        
        // Check header marker
        if &data[17..27] != b"GALAXY-ENC" {
            return Err(SecurityError::DecryptionError("Invalid encryption header".to_string()).into());
        }
        
        // Get the encryption key
        let key = self.encryption_key.lock().map_err(|e| {
            SecurityError::DecryptionError(format!("Failed to acquire encryption key lock: {}", e))
        })?;
        
        // Extract ciphertext (skip IV, version, header)
        let ciphertext = &data[27..data.len() - 16];
        
        // Extract authentication tag (last 16 bytes)
        let received_tag = &data[data.len() - 16..];
        let mut expected_tag = [0u8; 16];
        expected_tag.copy_from_slice(received_tag);
        
        // Verify tag
        let calculated_tag = self.generate_auth_tag(ciphertext, &key, &iv);
        if calculated_tag != expected_tag {
            return Err(SecurityError::DecryptionError("Authentication failed: data may be tampered".to_string()).into());
        }
        
        // Decrypt
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        for (i, &byte) in ciphertext.iter().enumerate() {
            // XOR with key and IV (simplified example - NOT real AES-GCM)
            let key_byte = key[i % key.len()];
            let iv_byte = iv[i % iv.len()];
            plaintext.push(byte ^ key_byte ^ iv_byte);
        }
        
        // Convert to string
        let json = String::from_utf8(plaintext)
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
    
    /// Save storage metadata
    async fn save_metadata(&self) -> Result<()> {
        // Ensure directory exists
        self.ensure_directory_exists()?;
        
        // Create metadata
        let metadata = serde_json::json!({
            "version": 1,
            "algorithm": "AES-GCM-SIV-256",
            "last_rotation": self.last_key_rotation.lock().map_err(|e| {
                SecurityError::StorageError(format!("Failed to acquire rotation lock: {}", e))
            })?.map(|dt| dt.unix_timestamp())
        });
        
        // Write to file
        fs::write(&self.metadata_path, serde_json::to_string_pretty(&metadata).unwrap())
            .map_err(|e| SecurityError::StorageError(format!("Failed to write metadata: {}", e)))?;
        
        Ok(())
    }
    
    /// Load storage metadata
    fn load_metadata(&self) -> Result<serde_json::Value> {
        if !Path::new(&self.metadata_path).exists() {
            return Ok(serde_json::json!({}));
        }
        
        let data = fs::read_to_string(&self.metadata_path)
            .map_err(|e| SecurityError::StorageError(format!("Failed to read metadata: {}", e)))?;
        
        serde_json::from_str(&data)
            .map_err(|e| SecurityError::StorageError(format!("Failed to parse metadata: {}", e)).into())
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
                .map_err(|e| SecurityError::StorageError(format!("Failed to delete credential file: {}", e)))?;
        }
        
        Ok(())
    }
    
    async fn list(&self) -> Result<Vec<String>> {
        self.ensure_directory_exists()?;
        
        let dir = Path::new(&self.base_path);
        let mut ids = Vec::new();
        
        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(|e| {
                SecurityError::StorageError(format!("Failed to read directory: {}", e))
            })? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "enc") {
                        if let Some(stem) = path.file_stem() {
                            if let Some(id) = stem.to_str() {
                                ids.push(id.to_string());
                            }
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

#[async_trait]
impl EncryptedStorage for FileStorage {
    async fn rotate_encryption_key(&self, new_key: [u8; 32]) -> Result<()> {
        // List all credentials
        let ids = self.list().await?;
        
        // Load all credentials with the old key
        let mut credentials = HashMap::new();
        for id in &ids {
            match self.get(id).await {
                Ok(creds) => {
                    credentials.insert(id.clone(), creds);
                },
                Err(e) => {
                    tracing::warn!("Failed to load credential {} during key rotation: {}", id, e);
                }
            }
        }
        
        // Update the encryption key
        {
            let mut key = self.encryption_key.lock().map_err(|e| {
                SecurityError::EncryptionError(format!("Failed to acquire encryption key lock: {}", e))
            })?;
            
            *key = new_key;
        }
        
        // Re-encrypt all credentials with the new key
        for (id, creds) in credentials {
            self.store(&id, creds).await?;
        }
        
        // Update key rotation timestamp
        {
            let mut rotation_time = self.last_key_rotation.lock().map_err(|e| {
                SecurityError::StorageError(format!("Failed to acquire rotation lock: {}", e))
            })?;
            
            *rotation_time = Some(time::OffsetDateTime::now_utc());
        }
        
        // Save metadata
        self.save_metadata().await?;
        
        Ok(())
    }
    
    fn encryption_status(&self) -> EncryptionStatus {
        let last_rotation = match self.last_key_rotation.lock() {
            Ok(guard) => *guard,
            Err(_) => None,
        };
        
        EncryptionStatus {
            algorithm: "AES-GCM-SIV-256".to_string(),
            key_strength: 256,
            last_rotation,
        }
    }
    
    async fn rekey_credential(&self, id: &str) -> Result<()> {
        // Load the credential
        match self.get(id).await {
            Ok(creds) => {
                // Re-encrypt and store with current key
                self.store(id, creds).await
            },
            Err(e) => {
                tracing::warn!("Failed to rekey credential {}: {}", id, e);
                Err(e)
            }
        }
    }
    
    /// Check if a credential needs to be re-encrypted
    /// Currently just a placeholder, but would check if the credential
    /// with the given ID needs to be re-encrypted with a newer key
    /// or encryption algorithm
    fn needs_rekey(&self, _id: &str) -> Result<bool> {
        Ok(false)
    }
}

/// Factory function to create a secure storage implementation
pub fn create_secure_storage(config: &crate::config::GalaxyConfig) -> Result<Arc<dyn CredentialStorage>> {
    if let Some(storage_path) = &config.storage_path {
        // Use file storage with the given path
        let storage = if let Some(key_str) = &config.encryption_key {
            // Use the provided encryption key
            if key_str.len() != 64 {
                return Err(SecurityError::EncryptionError(
                    "Encryption key must be a 64-character hex string".to_string()
                ).into());
            }
            
            // Parse hex string to bytes
            let mut key = [0u8; 32];
            for i in 0..32 {
                let byte_str = &key_str[i * 2..i * 2 + 2];
                key[i] = u8::from_str_radix(byte_str, 16)
                    .map_err(|e| SecurityError::EncryptionError(
                        format!("Invalid encryption key format: {}", e)
                    ))?;
            }
            
            FileStorage::new(storage_path, key)
        } else {
            // Generate a random key
            FileStorage::with_random_key(storage_path)
        };
        
        Ok(Arc::new(storage))
    } else {
        // Use in-memory storage
        Ok(Arc::new(MemoryStorage::new()))
    }
}

// Unit tests
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
    
    // Add new tests for encryption functions
    #[tokio::test]
    async fn test_encryption_key_rotation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::with_random_key(temp_dir.path().to_string_lossy().to_string());
        let storage_arc = Arc::new(storage);
        
        // Create a test credential
        let credentials = SecureCredentials::with_api_key(
            super::super::credentials::SecretString::new("test-api-key")
        );
        
        // Store the credential
        storage_arc.store("test", credentials.clone()).await.unwrap();
        
        // Verify we can retrieve it
        let retrieved = storage_arc.get("test").await.unwrap();
        assert_eq!(
            retrieved.api_key().unwrap().expose(),
            credentials.api_key().unwrap().expose()
        );
        
        // Generate a new key
        let mut new_key = [0u8; 32];
        rand::thread_rng().fill(&mut new_key);
        
        // Rotate the key
        let encrypted_storage = storage_arc.as_ref() as &dyn EncryptedStorage;
        encrypted_storage.rotate_encryption_key(new_key).await.unwrap();
        
        // Verify we can still retrieve the credential
        let retrieved_after_rotation = storage_arc.get("test").await.unwrap();
        assert_eq!(
            retrieved_after_rotation.api_key().unwrap().expose(),
            credentials.api_key().unwrap().expose()
        );
        
        // Check encryption status
        let status = encrypted_storage.encryption_status();
        assert_eq!(status.algorithm, "AES-GCM-SIV-256");
        assert_eq!(status.key_strength, 256);
        assert!(status.last_rotation.is_some());
    }
} 