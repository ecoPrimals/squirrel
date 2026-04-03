// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Key storage for MCP security
//!
//! This module provides key storage functionality for the MCP system.
//! Actual key storage operations are delegated to the BearDog framework.

use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Stored key material and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    /// Unique key record id.
    pub id: Uuid,
    /// Human-readable key name.
    pub name: String,
    /// Semantic type label (e.g. signing, encryption).
    pub key_type: String,
    /// Raw key bytes or opaque blob.
    pub data: Vec<u8>,
    /// When the key was created.
    pub created_at: DateTime<Utc>,
    /// Optional expiry after which the key should not be used.
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the key is still valid for use.
    pub active: bool,
}

/// Trait for key storage backends.
///
/// The default in-memory implementation is suitable for development and
/// single-process deployments. Production deployments should provide a
/// BearDog-backed or HSM-backed implementation via capability discovery.
#[async_trait]
pub trait KeyStorage: Send + Sync + std::fmt::Debug {
    /// Store a key with optional expiry and return its id.
    async fn store_key(
        &self,
        name: String,
        key_type: String,
        data: Vec<u8>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid>;

    /// Get a key by id.
    async fn get_key(&self, id: &Uuid) -> Result<Option<StoredKey>>;

    /// Find a key by name.
    async fn get_key_by_name(&self, name: &str) -> Result<Option<StoredKey>>;

    /// Update a stored key.
    async fn update_key(&self, key: StoredKey) -> Result<()>;

    /// Delete a key by id.
    async fn delete_key(&self, id: &Uuid) -> Result<()>;

    /// List active keys.
    async fn list_keys(&self) -> Result<Vec<StoredKey>>;

    /// Check if a key is expired.
    async fn is_key_expired(&self, id: &Uuid) -> Result<bool>;

    /// Remove expired keys, return count removed.
    async fn cleanup_expired_keys(&self) -> Result<usize>;
}

/// In-memory key storage implementation
///
/// This provides basic key storage that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct InMemoryKeyStorage {
    keys: Arc<RwLock<HashMap<Uuid, StoredKey>>>,
}

impl InMemoryKeyStorage {
    /// Create a new key storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Inserts a new key with optional expiry and returns its id.
    pub async fn store_key(
        &self,
        name: String,
        key_type: String,
        data: Vec<u8>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        let key = StoredKey {
            id: Uuid::new_v4(),
            name,
            key_type,
            data,
            created_at: Utc::now(),
            expires_at,
            active: true,
        };

        let mut keys = self.keys.write().await;
        let id = key.id;
        keys.insert(id, key);
        Ok(id)
    }

    /// Returns the key record by id, if present.
    pub async fn get_key(&self, id: &Uuid) -> Result<Option<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(id).cloned())
    }

    /// Finds a key by its display name.
    pub async fn get_key_by_name(&self, name: &str) -> Result<Option<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().find(|k| k.name == name).cloned())
    }

    /// Replaces the stored record for the key id.
    pub async fn update_key(&self, key: StoredKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id, key);
        Ok(())
    }

    /// Deletes a key by id.
    pub async fn delete_key(&self, id: &Uuid) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(id);
        Ok(())
    }

    /// Lists keys that are still marked active.
    pub async fn list_keys(&self) -> Result<Vec<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().filter(|k| k.active).cloned().collect())
    }

    /// Returns whether the key is past its expiry or missing from the store.
    pub async fn is_key_expired(&self, id: &Uuid) -> Result<bool> {
        let keys = self.keys.read().await;
        if let Some(key) = keys.get(id) {
            if let Some(expires_at) = key.expires_at {
                Ok(Utc::now() > expires_at)
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    /// Removes keys whose expiry is in the past and returns how many were removed.
    pub async fn cleanup_expired_keys(&self) -> Result<usize> {
        let mut keys = self.keys.write().await;
        let now = Utc::now();
        let mut removed = 0;

        keys.retain(|_, key| {
            if let Some(expires_at) = key.expires_at {
                if now > expires_at {
                    removed += 1;
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        Ok(removed)
    }
}

#[async_trait]
impl KeyStorage for InMemoryKeyStorage {
    async fn store_key(
        &self,
        name: String,
        key_type: String,
        data: Vec<u8>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid> {
        Self::store_key(self, name, key_type, data, expires_at).await
    }

    async fn get_key(&self, id: &Uuid) -> Result<Option<StoredKey>> {
        Self::get_key(self, id).await
    }

    async fn get_key_by_name(&self, name: &str) -> Result<Option<StoredKey>> {
        Self::get_key_by_name(self, name).await
    }

    async fn update_key(&self, key: StoredKey) -> Result<()> {
        Self::update_key(self, key).await
    }

    async fn delete_key(&self, id: &Uuid) -> Result<()> {
        Self::delete_key(self, id).await
    }

    async fn list_keys(&self) -> Result<Vec<StoredKey>> {
        Self::list_keys(self).await
    }

    async fn is_key_expired(&self, id: &Uuid) -> Result<bool> {
        Self::is_key_expired(self, id).await
    }

    async fn cleanup_expired_keys(&self) -> Result<usize> {
        Self::cleanup_expired_keys(self).await
    }
}

impl Default for InMemoryKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn stored_key_serde_round_trip() {
        let k = StoredKey {
            id: Uuid::new_v4(),
            name: String::new(),
            key_type: "signing".to_string(),
            data: vec![],
            created_at: Utc::now(),
            expires_at: None,
            active: true,
        };
        let json = serde_json::to_string(&k).expect("stored key serializes");
        let back: StoredKey = serde_json::from_str(&json).expect("stored key deserializes");
        assert_eq!(back.id, k.id);
        assert_eq!(back.name, k.name);
        assert_eq!(back.data, k.data);
        assert_eq!(back.active, k.active);
    }

    #[tokio::test]
    async fn new_default_store_get_update_delete_list() {
        let s = InMemoryKeyStorage::new();
        let _ = InMemoryKeyStorage::default();

        let id = s
            .store_key("k1".to_string(), "type".to_string(), vec![1, 2, 3], None)
            .await
            .expect("store_key");

        assert_eq!(
            s.get_key(&id)
                .await
                .expect("get_key")
                .expect("key exists")
                .name,
            "k1"
        );
        assert_eq!(
            s.get_key_by_name("k1")
                .await
                .expect("get_key_by_name")
                .expect("key exists")
                .id,
            id
        );
        assert!(s.get_key(&Uuid::new_v4()).await.expect("get_key").is_none());
        assert!(
            s.get_key_by_name("missing")
                .await
                .expect("get_key_by_name")
                .is_none()
        );

        let mut rec = s.get_key(&id).await.expect("get_key").expect("key exists");
        rec.name = "renamed".to_string();
        s.update_key(rec).await.expect("update_key");
        assert_eq!(
            s.get_key_by_name("renamed")
                .await
                .expect("get_key_by_name")
                .expect("key exists")
                .id,
            id
        );

        let keys = s.list_keys().await.expect("list_keys");
        assert_eq!(keys.len(), 1);

        let mut inactive = s.get_key(&id).await.expect("get_key").expect("key exists");
        inactive.active = false;
        s.update_key(inactive).await.expect("update_key");
        assert!(s.list_keys().await.expect("list_keys").is_empty());

        s.delete_key(&id).await.expect("delete_key");
        assert!(s.get_key(&id).await.expect("get_key").is_none());
    }

    #[tokio::test]
    async fn is_key_expired_missing_and_future() {
        let s = InMemoryKeyStorage::new();
        assert!(
            s.is_key_expired(&Uuid::new_v4())
                .await
                .expect("is_key_expired")
        );

        let id = s
            .store_key(
                "e".to_string(),
                "t".to_string(),
                vec![],
                Some(Utc::now() + Duration::hours(24)),
            )
            .await
            .expect("store_key");
        assert!(!s.is_key_expired(&id).await.expect("is_key_expired"));

        let mut k = s.get_key(&id).await.expect("get_key").expect("key exists");
        k.expires_at = Some(Utc::now() - Duration::hours(1));
        s.update_key(k).await.expect("update_key");
        assert!(s.is_key_expired(&id).await.expect("is_key_expired"));
    }

    #[tokio::test]
    async fn cleanup_expired_keys() {
        let s = InMemoryKeyStorage::new();
        let past = Utc::now() - Duration::minutes(1);
        let _ = s
            .store_key("gone".to_string(), "t".to_string(), vec![], Some(past))
            .await
            .expect("store_key");
        let future = Utc::now() + Duration::hours(1);
        let keep = s
            .store_key("stay".to_string(), "t".to_string(), vec![], Some(future))
            .await
            .expect("store_key");
        let no_exp = s
            .store_key("forever".to_string(), "t".to_string(), vec![], None)
            .await
            .expect("store_key");

        let removed = s
            .cleanup_expired_keys()
            .await
            .expect("cleanup_expired_keys");
        assert_eq!(removed, 1);
        assert!(
            s.get_key_by_name("gone")
                .await
                .expect("get_key_by_name")
                .is_none()
        );
        assert!(s.get_key(&keep).await.expect("get_key").is_some());
        assert!(s.get_key(&no_exp).await.expect("get_key").is_some());
    }
}
