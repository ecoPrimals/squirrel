// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Sovereign Data Management for Federation
//!
//! This module implements secure, user-controlled data management across
//! the federation network, ensuring data sovereignty and privacy.

use super::{
    DataId, DataPermissions, EncryptionMetadata, FederationError, FederationResult, SovereignData,
    SovereignDataManager,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sovereign data store implementation
pub struct DefaultSovereignDataManager {
    /// Data storage
    data_store: Arc<RwLock<HashMap<DataId, SovereignData>>>,
    /// Encryption key manager
    key_manager: Arc<dyn EncryptionKeyManager>,
    /// Access control manager
    access_control: Arc<dyn AccessControlManager>,
    /// Configuration
    config: SovereignDataConfig,
}

/// Configuration for sovereign data management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignDataConfig {
    /// Maximum data size in bytes
    pub max_data_size: usize,
    /// Default encryption algorithm
    pub default_encryption: String,
    /// Enable automatic encryption
    pub auto_encrypt: bool,
    /// Data retention period in days
    pub retention_days: u32,
    /// Backup configuration
    pub backup_config: BackupConfig,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    /// Backup interval in hours
    pub interval_hours: u32,
    /// Number of backups to retain
    pub retention_count: u32,
    /// Backup encryption enabled
    pub encrypt_backups: bool,
}

/// Data lifecycle stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataLifecycleStage {
    /// Data is actively being used
    Active,
    /// Data is archived but accessible
    Archived,
    /// Data is marked for deletion
    PendingDeletion,
    /// Data has been deleted
    Deleted,
}

/// Data access log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessLog {
    /// Access timestamp
    pub timestamp: DateTime<Utc>,
    /// User who accessed the data
    pub user: String,
    /// Type of access (read, write, delete)
    pub access_type: DataAccessType,
    /// Source IP address
    pub source_ip: Option<String>,
    /// Success status
    pub success: bool,
    /// Error message if access failed
    pub error: Option<String>,
}

/// Type of data access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataAccessType {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Delete access
    Delete,
    /// Admin access
    Admin,
}

/// Encryption key manager trait
#[async_trait]
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

/// Access control manager trait
#[async_trait]
pub trait AccessControlManager: Send + Sync {
    /// Check if user has permission to access data
    async fn check_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<bool>;

    /// Grant permission to user
    async fn grant_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()>;

    /// Revoke permission from user
    async fn revoke_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()>;

    /// List permissions for data
    async fn list_permissions(&self, data_id: DataId) -> FederationResult<DataPermissions>;
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

#[async_trait]
impl EncryptionKeyManager for DefaultEncryptionKeyManager {
    async fn generate_key(&self, algorithm: &str) -> FederationResult<Vec<u8>> {
        match algorithm {
            "AES-256-GCM" => {
                // Generate 32-byte key for AES-256
                let mut key = vec![0u8; 32];
                // In a real implementation, use a proper random number generator
                for (i, byte) in key.iter_mut().enumerate() {
                    *byte = (i as u8).wrapping_mul(7).wrapping_add(13);
                }
                Ok(key)
            }
            "ChaCha20-Poly1305" => {
                // Generate 32-byte key for ChaCha20
                let mut key = vec![0u8; 32];
                for (i, byte) in key.iter_mut().enumerate() {
                    *byte = (i as u8).wrapping_mul(11).wrapping_add(17);
                }
                Ok(key)
            }
            _ => Err(FederationError::UnsupportedPlatform(format!(
                "Unsupported encryption algorithm: {algorithm}"
            ))),
        }
    }

    async fn encrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>> {
        match algorithm {
            "AES-256-GCM" => {
                // Simplified encryption - in real implementation use proper crypto library
                let mut encrypted = Vec::new();
                for (i, &byte) in data.iter().enumerate() {
                    let key_byte = key[i % key.len()];
                    encrypted.push(byte ^ key_byte);
                }
                Ok(encrypted)
            }
            "ChaCha20-Poly1305" => {
                // Simplified encryption
                let mut encrypted = Vec::new();
                for (i, &byte) in data.iter().enumerate() {
                    let key_byte = key[i % key.len()];
                    encrypted.push(byte ^ key_byte ^ 0x5A);
                }
                Ok(encrypted)
            }
            _ => Err(FederationError::UnsupportedPlatform(format!(
                "Unsupported encryption algorithm: {algorithm}"
            ))),
        }
    }

    async fn decrypt(&self, data: &[u8], key: &[u8], algorithm: &str) -> FederationResult<Vec<u8>> {
        match algorithm {
            "AES-256-GCM" => {
                // Simplified decryption - same as encryption for XOR
                let mut decrypted = Vec::new();
                for (i, &byte) in data.iter().enumerate() {
                    let key_byte = key[i % key.len()];
                    decrypted.push(byte ^ key_byte);
                }
                Ok(decrypted)
            }
            "ChaCha20-Poly1305" => {
                // Simplified decryption
                let mut decrypted = Vec::new();
                for (i, &byte) in data.iter().enumerate() {
                    let key_byte = key[i % key.len()];
                    decrypted.push(byte ^ key_byte ^ 0x5A);
                }
                Ok(decrypted)
            }
            _ => Err(FederationError::UnsupportedPlatform(format!(
                "Unsupported encryption algorithm: {algorithm}"
            ))),
        }
    }

    async fn derive_key(&self, password: &str, salt: &[u8]) -> FederationResult<Vec<u8>> {
        // Simplified key derivation - in real implementation use PBKDF2 or similar
        let mut key = Vec::new();
        let password_bytes = password.as_bytes();

        for i in 0..32 {
            let mut byte = 0u8;
            for (j, &p_byte) in password_bytes.iter().enumerate() {
                let salt_byte = salt[j % salt.len()];
                byte = byte
                    .wrapping_add(p_byte)
                    .wrapping_add(salt_byte)
                    .wrapping_add(i as u8);
            }
            key.push(byte);
        }

        Ok(key)
    }
}

/// Default access control manager
pub struct DefaultAccessControlManager {
    /// Permission storage
    permissions: Arc<RwLock<HashMap<DataId, DataPermissions>>>,
    /// Access logs
    access_logs: Arc<RwLock<HashMap<DataId, Vec<DataAccessLog>>>>,
}

impl DefaultAccessControlManager {
    /// Create a new access control manager
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            access_logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Log data access
    async fn log_access(
        &self,
        data_id: DataId,
        user: &str,
        access_type: DataAccessType,
        success: bool,
        error: Option<String>,
    ) {
        let log_entry = DataAccessLog {
            timestamp: Utc::now(),
            user: user.to_string(),
            access_type,
            source_ip: None, // Would be populated in real implementation
            success,
            error,
        };

        let mut logs = self.access_logs.write().await;
        logs.entry(data_id).or_insert_with(Vec::new).push(log_entry);
    }
}

#[async_trait]
impl AccessControlManager for DefaultAccessControlManager {
    async fn check_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<bool> {
        let permissions = self.permissions.read().await;

        if let Some(data_permissions) = permissions.get(&data_id) {
            let has_permission = match access_type {
                DataAccessType::Read => {
                    data_permissions.public_read
                        || data_permissions.read_users.contains(&user.to_string())
                        || data_permissions.admin_users.contains(&user.to_string())
                }
                DataAccessType::Write => {
                    data_permissions.public_write
                        || data_permissions.write_users.contains(&user.to_string())
                        || data_permissions.admin_users.contains(&user.to_string())
                }
                DataAccessType::Delete | DataAccessType::Admin => {
                    data_permissions.admin_users.contains(&user.to_string())
                }
            };

            // Log access attempt
            self.log_access(data_id, user, access_type, has_permission, None)
                .await;

            Ok(has_permission)
        } else {
            self.log_access(
                data_id,
                user,
                access_type,
                false,
                Some("Data not found".to_string()),
            )
            .await;
            Ok(false)
        }
    }

    async fn grant_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()> {
        let mut permissions = self.permissions.write().await;
        let data_permissions = permissions
            .entry(data_id)
            .or_insert_with(DataPermissions::default);

        match access_type {
            DataAccessType::Read => {
                if !data_permissions.read_users.contains(&user.to_string()) {
                    data_permissions.read_users.push(user.to_string());
                }
            }
            DataAccessType::Write => {
                if !data_permissions.write_users.contains(&user.to_string()) {
                    data_permissions.write_users.push(user.to_string());
                }
            }
            DataAccessType::Admin => {
                if !data_permissions.admin_users.contains(&user.to_string()) {
                    data_permissions.admin_users.push(user.to_string());
                }
            }
            DataAccessType::Delete => {
                // Delete permission is same as admin
                if !data_permissions.admin_users.contains(&user.to_string()) {
                    data_permissions.admin_users.push(user.to_string());
                }
            }
        }

        Ok(())
    }

    async fn revoke_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()> {
        let mut permissions = self.permissions.write().await;

        if let Some(data_permissions) = permissions.get_mut(&data_id) {
            match access_type {
                DataAccessType::Read => {
                    data_permissions.read_users.retain(|u| u != user);
                }
                DataAccessType::Write => {
                    data_permissions.write_users.retain(|u| u != user);
                }
                DataAccessType::Admin | DataAccessType::Delete => {
                    data_permissions.admin_users.retain(|u| u != user);
                }
            }
        }

        Ok(())
    }

    async fn list_permissions(&self, data_id: DataId) -> FederationResult<DataPermissions> {
        let permissions = self.permissions.read().await;

        if let Some(data_permissions) = permissions.get(&data_id) {
            Ok(data_permissions.clone())
        } else {
            Ok(DataPermissions::default())
        }
    }
}

impl DefaultSovereignDataManager {
    /// Create a new sovereign data manager
    pub fn new(config: SovereignDataConfig) -> Self {
        Self {
            data_store: Arc::new(RwLock::new(HashMap::new())),
            key_manager: Arc::new(DefaultEncryptionKeyManager::new()),
            access_control: Arc::new(DefaultAccessControlManager::new()),
            config,
        }
    }

    /// Set custom key manager
    pub fn with_key_manager(mut self, key_manager: Arc<dyn EncryptionKeyManager>) -> Self {
        self.key_manager = key_manager;
        self
    }

    /// Set custom access control manager
    pub fn with_access_control(mut self, access_control: Arc<dyn AccessControlManager>) -> Self {
        self.access_control = access_control;
        self
    }

    /// Encrypt data if required
    async fn encrypt_data(
        &self,
        data: &[u8],
        metadata: &mut EncryptionMetadata,
    ) -> FederationResult<Vec<u8>> {
        if self.config.auto_encrypt && !metadata.encrypted {
            let key = self.key_manager.generate_key(&metadata.algorithm).await?;
            let encrypted = self
                .key_manager
                .encrypt(data, &key, &metadata.algorithm)
                .await?;

            metadata.encrypted = true;
            metadata.iv = key; // Simplified - in real implementation, IV would be separate

            Ok(encrypted)
        } else {
            Ok(data.to_vec())
        }
    }

    /// Decrypt data if needed
    async fn decrypt_data(
        &self,
        data: &[u8],
        metadata: &EncryptionMetadata,
    ) -> FederationResult<Vec<u8>> {
        if metadata.encrypted {
            self.key_manager
                .decrypt(data, &metadata.iv, &metadata.algorithm)
                .await
        } else {
            Ok(data.to_vec())
        }
    }
}

#[async_trait]
impl SovereignDataManager for DefaultSovereignDataManager {
    async fn store_data(&self, mut data: SovereignData) -> FederationResult<DataId> {
        // Check data size limits
        if data.content.len() > self.config.max_data_size {
            return Err(FederationError::ResourceLimitExceeded(format!(
                "Data size {} exceeds limit {}",
                data.content.len(),
                self.config.max_data_size
            )));
        }

        // Encrypt data if needed
        let encrypted_content = self
            .encrypt_data(&data.content, &mut data.encryption)
            .await?;
        data.content = encrypted_content;

        // Set timestamps
        data.created_at = Utc::now();
        data.modified_at = data.created_at;

        // Store data
        let data_id = data.id;
        let owner = data.owner.clone();
        {
            let mut store = self.data_store.write().await;
            store.insert(data_id, data);
        }

        // Set up permissions
        self.access_control
            .grant_permission(&owner, data_id, DataAccessType::Admin)
            .await?;

        Ok(data_id)
    }

    async fn retrieve_data(&self, id: DataId) -> FederationResult<SovereignData> {
        let store = self.data_store.read().await;

        if let Some(data) = store.get(&id) {
            let mut retrieved_data = data.clone();

            // Decrypt data if needed
            let decrypted_content = self
                .decrypt_data(&retrieved_data.content, &retrieved_data.encryption)
                .await?;
            retrieved_data.content = decrypted_content;

            Ok(retrieved_data)
        } else {
            Err(FederationError::ExecutionNotFound(id))
        }
    }

    async fn delete_data(&self, id: DataId) -> FederationResult<()> {
        let mut store = self.data_store.write().await;

        if store.remove(&id).is_some() {
            Ok(())
        } else {
            Err(FederationError::ExecutionNotFound(id))
        }
    }

    async fn list_data(&self, owner: &str) -> FederationResult<Vec<DataId>> {
        let store = self.data_store.read().await;

        let data_ids: Vec<DataId> = store
            .values()
            .filter(|data| data.owner == owner)
            .map(|data| data.id)
            .collect();

        Ok(data_ids)
    }

    async fn check_access(&self, id: DataId, requester: &str) -> FederationResult<bool> {
        self.access_control
            .check_permission(requester, id, DataAccessType::Read)
            .await
    }
}

impl Default for SovereignDataConfig {
    fn default() -> Self {
        Self {
            max_data_size: 100 * 1024 * 1024, // 100MB
            default_encryption: "AES-256-GCM".to_string(),
            auto_encrypt: true,
            retention_days: 365,
            backup_config: BackupConfig::default(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_hours: 24,
            retention_count: 7,
            encrypt_backups: true,
        }
    }
}

impl Default for DefaultEncryptionKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DefaultAccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid; // Evolution: Add missing import

    #[tokio::test]
    async fn test_sovereign_data_manager_creation() {
        let config = SovereignDataConfig::default();
        let manager = DefaultSovereignDataManager::new(config);

        // Test basic functionality
        let data = SovereignData {
            id: Uuid::new_v4(),
            owner: "test_user".to_string(),
            content: b"test data".to_vec(),
            permissions: DataPermissions::default(),
            encryption: EncryptionMetadata::default(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        let data_id = manager.store_data(data).await.unwrap();
        assert!(!data_id.is_nil());
    }

    #[tokio::test]
    async fn test_data_encryption() {
        let key_manager = DefaultEncryptionKeyManager::new();

        let data = b"sensitive data";
        let key = key_manager.generate_key("AES-256-GCM").await.unwrap();

        let encrypted = key_manager
            .encrypt(data, &key, "AES-256-GCM")
            .await
            .unwrap();
        assert_ne!(encrypted, data);

        let decrypted = key_manager
            .decrypt(&encrypted, &key, "AES-256-GCM")
            .await
            .unwrap();
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn test_access_control() {
        let access_control = DefaultAccessControlManager::new();
        let data_id = Uuid::new_v4();

        // Initially no permission
        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .unwrap();
        assert!(!has_permission);

        // Grant permission
        access_control
            .grant_permission("user1", data_id, DataAccessType::Read)
            .await
            .unwrap();

        // Check permission again
        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .unwrap();
        assert!(has_permission);

        // Revoke permission
        access_control
            .revoke_permission("user1", data_id, DataAccessType::Read)
            .await
            .unwrap();

        // Check permission after revocation
        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .unwrap();
        assert!(!has_permission);
    }

    #[tokio::test]
    async fn test_data_size_limits() {
        let config = SovereignDataConfig {
            max_data_size: 10, // Very small limit for testing
            ..Default::default()
        };

        let manager = DefaultSovereignDataManager::new(config);

        let data = SovereignData {
            id: Uuid::new_v4(),
            owner: "test_user".to_string(),
            content: b"this is too much data".to_vec(), // Exceeds limit
            permissions: DataPermissions::default(),
            encryption: EncryptionMetadata::default(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        let result = manager.store_data(data).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            FederationError::ResourceLimitExceeded(msg) => {
                assert!(msg.contains("exceeds limit"));
            }
            _ => panic!("Expected ResourceLimitExceeded error"),
        }
    }
}
