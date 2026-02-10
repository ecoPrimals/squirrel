// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Key storage for MCP security
//!
//! This module provides key storage functionality for the MCP system.
//! Actual key storage operations are delegated to the BearDog framework.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::Result;

/// Stored key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKey {
    pub id: Uuid,
    pub name: String,
    pub key_type: String,
    pub data: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
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
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a key
    pub async fn store_key(&self, name: String, key_type: String, data: Vec<u8>, expires_at: Option<DateTime<Utc>>) -> Result<Uuid> {
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

    /// Get a key by ID
    pub async fn get_key(&self, id: &Uuid) -> Result<Option<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(id).cloned())
    }

    /// Get a key by name
    pub async fn get_key_by_name(&self, name: &str) -> Result<Option<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().find(|k| k.name == name).cloned())
    }

    /// Update a key
    pub async fn update_key(&self, key: StoredKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id, key);
        Ok(())
    }

    /// Delete a key
    pub async fn delete_key(&self, id: &Uuid) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(id);
        Ok(())
    }

    /// List all active keys
    pub async fn list_keys(&self) -> Result<Vec<StoredKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().filter(|k| k.active).cloned().collect())
    }

    /// Check if a key is expired
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

    /// Clean up expired keys
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