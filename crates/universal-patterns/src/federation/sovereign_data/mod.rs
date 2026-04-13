// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Sovereign Data Management for Federation
//!
//! This module implements secure, user-controlled data management across
//! the federation network, ensuring data sovereignty and privacy.
//!
//! Organized into focused submodules:
//! - [`encryption`] — Key management traits and default implementation
//! - [`access_control`] — Permission management, audit logging, and types

mod access_control;
mod encryption;

pub use access_control::{
    AccessControlManager, DataAccessLog, DataAccessType, DataLifecycleStage,
    DefaultAccessControlManager,
};
pub use encryption::{DefaultEncryptionKeyManager, EncryptionKeyManager};

use super::{
    DataId, EncryptionMetadata, FederationError, FederationResult, SovereignData,
    SovereignDataManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Sovereign data store implementation
pub struct DefaultSovereignDataManager<
    E: EncryptionKeyManager = DefaultEncryptionKeyManager,
    A: AccessControlManager = DefaultAccessControlManager,
> {
    /// Data storage
    data_store: Arc<RwLock<HashMap<DataId, SovereignData>>>,
    /// Encryption key manager
    key_manager: Arc<E>,
    /// Access control manager
    access_control: Arc<A>,
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
    /// Whether to auto-encrypt data
    pub auto_encrypt: bool,
    /// Data retention period in days
    pub retention_days: u32,
    /// Backup configuration
    pub backup_config: BackupConfig,
}

/// Backup configuration for sovereign data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Whether backups are enabled
    pub enabled: bool,
    /// Backup interval in hours
    pub interval_hours: u32,
    /// Number of backups to retain
    pub retention_count: u32,
    /// Whether to encrypt backups
    pub encrypt_backups: bool,
}

impl DefaultSovereignDataManager<DefaultEncryptionKeyManager, DefaultAccessControlManager> {
    /// Create a new sovereign data manager
    pub fn new(
        config: SovereignDataConfig,
    ) -> DefaultSovereignDataManager<DefaultEncryptionKeyManager, DefaultAccessControlManager> {
        DefaultSovereignDataManager {
            data_store: Arc::new(RwLock::new(HashMap::new())),
            key_manager: Arc::new(DefaultEncryptionKeyManager::new()),
            access_control: Arc::new(DefaultAccessControlManager::new()),
            config,
        }
    }
}

impl<E, A> DefaultSovereignDataManager<E, A>
where
    E: EncryptionKeyManager,
    A: AccessControlManager,
{
    /// Set custom key manager
    pub fn with_key_manager<E2: EncryptionKeyManager>(
        self,
        key_manager: Arc<E2>,
    ) -> DefaultSovereignDataManager<E2, A> {
        DefaultSovereignDataManager {
            data_store: self.data_store,
            key_manager,
            access_control: self.access_control,
            config: self.config,
        }
    }

    /// Set custom access control manager
    pub fn with_access_control<A2: AccessControlManager>(
        self,
        access_control: Arc<A2>,
    ) -> DefaultSovereignDataManager<E, A2> {
        DefaultSovereignDataManager {
            data_store: self.data_store,
            key_manager: self.key_manager,
            access_control,
            config: self.config,
        }
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

impl<E, A> SovereignDataManager for DefaultSovereignDataManager<E, A>
where
    E: EncryptionKeyManager,
    A: AccessControlManager,
{
    async fn store_data(&self, mut data: SovereignData) -> FederationResult<DataId> {
        if data.content.len() > self.config.max_data_size {
            return Err(FederationError::ResourceLimitExceeded(format!(
                "Data size {} exceeds limit {}",
                data.content.len(),
                self.config.max_data_size
            )));
        }

        let id = data.id;
        let mut encryption = data.encryption.clone();
        let encrypted_content = self.encrypt_data(&data.content, &mut encryption).await?;
        data.content = encrypted_content;
        data.encryption = encryption;

        self.access_control
            .grant_permission(&data.owner, id, DataAccessType::Admin)
            .await?;

        let mut store = self.data_store.write().await;
        store.insert(id, data);
        Ok(id)
    }

    async fn retrieve_data(&self, id: DataId) -> FederationResult<SovereignData> {
        let store = self.data_store.read().await;
        if let Some(data) = store.get(&id) {
            let mut result = data.clone();
            result.content = self.decrypt_data(&data.content, &data.encryption).await?;
            Ok(result)
        } else {
            Err(FederationError::PeerNotFound(id.to_string()))
        }
    }

    async fn delete_data(&self, id: DataId) -> FederationResult<()> {
        let mut store = self.data_store.write().await;
        if store.remove(&id).is_some() {
            Ok(())
        } else {
            Err(FederationError::PeerNotFound(id.to_string()))
        }
    }

    async fn list_data(&self, owner: &str) -> FederationResult<Vec<DataId>> {
        let store = self.data_store.read().await;
        Ok(store
            .iter()
            .filter(|(_, data)| data.owner == owner)
            .map(|(id, _)| *id)
            .collect())
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

#[cfg(test)]
mod tests {
    use super::super::{DataPermissions, EncryptionMetadata};
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn test_data(owner: &str, content: &[u8]) -> SovereignData {
        SovereignData {
            id: Uuid::new_v4(),
            owner: owner.to_string(),
            content: content.to_vec(),
            permissions: DataPermissions::default(),
            encryption: EncryptionMetadata::default(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_sovereign_data_manager_creation() {
        let config = SovereignDataConfig::default();
        let manager = DefaultSovereignDataManager::new(config);

        let data = SovereignData {
            id: Uuid::new_v4(),
            owner: "test_user".to_string(),
            content: b"test data".to_vec(),
            permissions: DataPermissions::default(),
            encryption: EncryptionMetadata::default(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        let data_id = manager.store_data(data).await.expect("should succeed");
        assert!(!data_id.is_nil());
    }

    #[tokio::test]
    async fn test_data_encryption() {
        let key_manager = DefaultEncryptionKeyManager::new();

        let data = b"sensitive data";
        let key = key_manager
            .generate_key("AES-256-GCM")
            .await
            .expect("should succeed");

        let encrypted = key_manager
            .encrypt(data, &key, "AES-256-GCM")
            .await
            .expect("should succeed");
        assert_ne!(encrypted, data);

        let decrypted = key_manager
            .decrypt(&encrypted, &key, "AES-256-GCM")
            .await
            .expect("should succeed");
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn test_access_control() {
        let access_control = DefaultAccessControlManager::new();
        let data_id = Uuid::new_v4();

        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .expect("should succeed");
        assert!(!has_permission);

        access_control
            .grant_permission("user1", data_id, DataAccessType::Read)
            .await
            .expect("should succeed");

        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .expect("should succeed");
        assert!(has_permission);

        access_control
            .revoke_permission("user1", data_id, DataAccessType::Read)
            .await
            .expect("should succeed");

        let has_permission = access_control
            .check_permission("user1", data_id, DataAccessType::Read)
            .await
            .expect("should succeed");
        assert!(!has_permission);
    }

    #[tokio::test]
    async fn test_data_size_limits() {
        let config = SovereignDataConfig {
            max_data_size: 10,
            ..Default::default()
        };

        let manager = DefaultSovereignDataManager::new(config);

        let data = SovereignData {
            id: Uuid::new_v4(),
            owner: "test_user".to_string(),
            content: b"this is too much data".to_vec(),
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
            _ => unreachable!("Expected ResourceLimitExceeded error"),
        }
    }

    #[tokio::test]
    async fn test_retrieve_data() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let data = test_data("user1", b"hello");
        let id = manager.store_data(data).await.expect("store");

        let retrieved = manager.retrieve_data(id).await.expect("retrieve");
        assert_eq!(retrieved.owner, "user1");
        assert_eq!(retrieved.content, b"hello");
    }

    #[tokio::test]
    async fn test_retrieve_data_not_found() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let result = manager.retrieve_data(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_data() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let data = test_data("user1", b"to-delete");
        let id = manager.store_data(data).await.expect("store");

        manager.delete_data(id).await.expect("delete");

        let result = manager.retrieve_data(id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_data_not_found() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let result = manager.delete_data(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_data() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let d1 = test_data("alice", b"data1");
        let d2 = test_data("alice", b"data2");
        let d3 = test_data("bob", b"data3");

        let id1 = manager.store_data(d1).await.expect("store1");
        let id2 = manager.store_data(d2).await.expect("store2");
        let _id3 = manager.store_data(d3).await.expect("store3");

        let alice_data = manager.list_data("alice").await.expect("list");
        assert_eq!(alice_data.len(), 2);
        assert!(alice_data.contains(&id1));
        assert!(alice_data.contains(&id2));

        let bob_data = manager.list_data("bob").await.expect("list");
        assert_eq!(bob_data.len(), 1);

        let empty = manager.list_data("nobody").await.expect("list");
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_check_access() {
        let manager = DefaultSovereignDataManager::new(SovereignDataConfig::default());
        let data = test_data("owner", b"content");
        let id = manager.store_data(data).await.expect("store");

        let has_access = manager.check_access(id, "stranger").await.expect("check");
        assert!(!has_access);
    }

    #[tokio::test]
    async fn test_derive_key_deterministic() {
        let key_mgr = DefaultEncryptionKeyManager::new();
        let key1 = key_mgr
            .derive_key("password", b"salt123")
            .await
            .expect("derive1");
        let key2 = key_mgr
            .derive_key("password", b"salt123")
            .await
            .expect("derive2");
        assert_eq!(key1, key2);
        assert!(!key1.is_empty());

        let key3 = key_mgr
            .derive_key("password", b"different_salt")
            .await
            .expect("derive3");
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_access_control_write_permission() {
        let ac = DefaultAccessControlManager::new();
        let id = Uuid::new_v4();

        ac.grant_permission("user1", id, DataAccessType::Write)
            .await
            .expect("grant");
        assert!(
            ac.check_permission("user1", id, DataAccessType::Write)
                .await
                .expect("check")
        );
        assert!(
            !ac.check_permission("user1", id, DataAccessType::Read)
                .await
                .expect("check")
        );

        ac.revoke_permission("user1", id, DataAccessType::Write)
            .await
            .expect("revoke");
        assert!(
            !ac.check_permission("user1", id, DataAccessType::Write)
                .await
                .expect("check")
        );
    }

    #[tokio::test]
    async fn test_access_control_admin_permission() {
        let ac = DefaultAccessControlManager::new();
        let id = Uuid::new_v4();

        ac.grant_permission("admin", id, DataAccessType::Admin)
            .await
            .expect("grant");
        assert!(
            ac.check_permission("admin", id, DataAccessType::Admin)
                .await
                .expect("check")
        );
    }

    #[tokio::test]
    async fn test_access_control_delete_permission() {
        let ac = DefaultAccessControlManager::new();
        let id = Uuid::new_v4();

        ac.grant_permission("user1", id, DataAccessType::Delete)
            .await
            .expect("grant");
        assert!(
            ac.check_permission("user1", id, DataAccessType::Delete)
                .await
                .expect("check")
        );

        ac.revoke_permission("user1", id, DataAccessType::Delete)
            .await
            .expect("revoke");
        assert!(
            !ac.check_permission("user1", id, DataAccessType::Delete)
                .await
                .expect("check")
        );
    }

    #[tokio::test]
    async fn test_list_permissions_empty() {
        let ac = DefaultAccessControlManager::new();
        let perms = ac.list_permissions(Uuid::new_v4()).await.expect("list");
        assert!(perms.read_users.is_empty());
        assert!(perms.write_users.is_empty());
    }

    #[tokio::test]
    async fn test_list_permissions_populated() {
        let ac = DefaultAccessControlManager::new();
        let id = Uuid::new_v4();

        ac.grant_permission("user1", id, DataAccessType::Read)
            .await
            .expect("grant");
        ac.grant_permission("user2", id, DataAccessType::Write)
            .await
            .expect("grant");

        let perms = ac.list_permissions(id).await.expect("list");
        assert!(perms.read_users.contains(&"user1".to_string()));
        assert!(perms.write_users.contains(&"user2".to_string()));
    }

    #[tokio::test]
    async fn test_sovereign_data_config_default() {
        let config = SovereignDataConfig::default();
        assert!(config.max_data_size > 0);
        assert!(config.auto_encrypt);
    }
}
