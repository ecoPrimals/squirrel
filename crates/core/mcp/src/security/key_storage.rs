// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Key storage for MCP security
//!
//! This module provides key storage functionality for the MCP system.
//! Actual key storage operations are delegated to the BearDog framework.

use crate::error::Result;
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

impl Default for InMemoryKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn stored_key_serde_round_trip() {
        let k = StoredKey {
            id: Uuid::new_v4(),
            name: "".to_string(),
            key_type: "signing".to_string(),
            data: vec![],
            created_at: Utc::now(),
            expires_at: None,
            active: true,
        };
        let json = serde_json::to_string(&k).unwrap();
        let back: StoredKey = serde_json::from_str(&json).unwrap();
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
            .unwrap();

        assert_eq!(s.get_key(&id).await.unwrap().unwrap().name, "k1");
        assert_eq!(s.get_key_by_name("k1").await.unwrap().unwrap().id, id);
        assert!(s.get_key(&Uuid::new_v4()).await.unwrap().is_none());
        assert!(s.get_key_by_name("missing").await.unwrap().is_none());

        let mut rec = s.get_key(&id).await.unwrap().unwrap();
        rec.name = "renamed".to_string();
        s.update_key(rec).await.unwrap();
        assert_eq!(s.get_key_by_name("renamed").await.unwrap().unwrap().id, id);

        let keys = s.list_keys().await.unwrap();
        assert_eq!(keys.len(), 1);

        let mut inactive = s.get_key(&id).await.unwrap().unwrap();
        inactive.active = false;
        s.update_key(inactive).await.unwrap();
        assert!(s.list_keys().await.unwrap().is_empty());

        s.delete_key(&id).await.unwrap();
        assert!(s.get_key(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn is_key_expired_missing_and_future() {
        let s = InMemoryKeyStorage::new();
        assert!(s.is_key_expired(&Uuid::new_v4()).await.unwrap());

        let id = s
            .store_key(
                "e".to_string(),
                "t".to_string(),
                vec![],
                Some(Utc::now() + Duration::hours(24)),
            )
            .await
            .unwrap();
        assert!(!s.is_key_expired(&id).await.unwrap());

        let mut k = s.get_key(&id).await.unwrap().unwrap();
        k.expires_at = Some(Utc::now() - Duration::hours(1));
        s.update_key(k).await.unwrap();
        assert!(s.is_key_expired(&id).await.unwrap());
    }

    #[tokio::test]
    async fn cleanup_expired_keys() {
        let s = InMemoryKeyStorage::new();
        let past = Utc::now() - Duration::minutes(1);
        let _ = s
            .store_key("gone".to_string(), "t".to_string(), vec![], Some(past))
            .await
            .unwrap();
        let future = Utc::now() + Duration::hours(1);
        let keep = s
            .store_key("stay".to_string(), "t".to_string(), vec![], Some(future))
            .await
            .unwrap();
        let no_exp = s
            .store_key("forever".to_string(), "t".to_string(), vec![], None)
            .await
            .unwrap();

        let removed = s.cleanup_expired_keys().await.unwrap();
        assert_eq!(removed, 1);
        assert!(s.get_key_by_name("gone").await.unwrap().is_none());
        assert!(s.get_key(&keep).await.unwrap().is_some());
        assert!(s.get_key(&no_exp).await.unwrap().is_some());
    }
}
