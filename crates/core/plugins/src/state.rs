// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin state management
//!
//! This module provides functionality for managing plugin state.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::RwLock;

/// Plugin state manager trait
pub trait PluginStateManager: Send + Sync + Debug {
    /// Get plugin state
    fn get_state(&self, plugin_id: &str) -> impl std::future::Future<Output = Result<Option<Value>>> + Send;

    /// Set plugin state
    fn set_state(&self, plugin_id: &str, state: Value) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Remove plugin state
    fn remove_state(&self, plugin_id: &str) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// In-memory plugin state manager
#[derive(Debug, Default)]
pub struct MemoryStateManager {
    /// Plugin state storage
    states: RwLock<HashMap<String, Value>>,
}

impl MemoryStateManager {
    /// Create a new memory state manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }
}

impl PluginStateManager for MemoryStateManager {
    async fn get_state(&self, plugin_id: &str) -> Result<Option<Value>> {
        let states = self.states.read().await;
        Ok(states.get(plugin_id).cloned())
    }

    async fn set_state(&self, plugin_id: &str, state: Value) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(plugin_id.to_string(), state);
        drop(states);
        Ok(())
    }

    async fn remove_state(&self, plugin_id: &str) -> Result<()> {
        let mut states = self.states.write().await;
        states.remove(plugin_id);
        drop(states);
        Ok(())
    }
}

/// File-based plugin state manager
#[derive(Debug)]
pub struct FileStateManager {
    /// Base directory for state files
    base_dir: String,
    /// Memory cache for state
    cache: RwLock<HashMap<String, Value>>,
}

impl FileStateManager {
    /// Create a new file state manager
    #[must_use]
    pub fn new(base_dir: String) -> Self {
        Self {
            base_dir,
            cache: RwLock::new(HashMap::new()),
        }
    }
}

impl PluginStateManager for FileStateManager {
    async fn get_state(&self, plugin_id: &str) -> Result<Option<Value>> {
        // Check cache first
        let cache = self.cache.read().await;
        if let Some(value) = cache.get(plugin_id) {
            let result = Ok(Some(value.clone()));
            drop(cache);
            return result;
        }

        // If not in cache, try to read from file
        let file_path = format!("{}/{}.json", self.base_dir, plugin_id);
        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => {
                let value: Value = serde_json::from_str(&content)?;
                // Update cache
                drop(cache);
                let mut cache = self.cache.write().await;
                cache.insert(plugin_id.to_string(), value.clone());
                drop(cache);
                Ok(Some(value))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn set_state(&self, plugin_id: &str, state: Value) -> Result<()> {
        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(plugin_id.to_string(), state.clone());

        // Ensure directory exists
        tokio::fs::create_dir_all(&self.base_dir).await?;

        // Write to file
        let file_path = format!("{}/{}.json", self.base_dir, plugin_id);
        let content = serde_json::to_string(&state)?;
        tokio::fs::write(&file_path, content).await?;
        drop(cache);

        Ok(())
    }

    async fn remove_state(&self, plugin_id: &str) -> Result<()> {
        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(plugin_id);

        // Remove file if exists
        let file_path = format!("{}/{}.json", self.base_dir, plugin_id);
        let result = match tokio::fs::remove_file(&file_path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        };
        drop(cache);
        result
    }
}

/// Plugin state storage backend (enum dispatch instead of `Box<dyn PluginStateManager>`).
#[derive(Debug)]
pub enum StateManagerBackend {
    /// In-memory state only.
    Memory(MemoryStateManager),
    /// Persistent JSON files under a base directory.
    File(FileStateManager),
}

impl PluginStateManager for StateManagerBackend {
    async fn get_state(&self, plugin_id: &str) -> Result<Option<Value>> {
        match self {
            Self::Memory(m) => m.get_state(plugin_id).await,
            Self::File(f) => f.get_state(plugin_id).await,
        }
    }

    async fn set_state(&self, plugin_id: &str, state: Value) -> Result<()> {
        match self {
            Self::Memory(m) => m.set_state(plugin_id, state).await,
            Self::File(f) => f.set_state(plugin_id, state).await,
        }
    }

    async fn remove_state(&self, plugin_id: &str) -> Result<()> {
        match self {
            Self::Memory(m) => m.remove_state(plugin_id).await,
            Self::File(f) => f.remove_state(plugin_id).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_memory_state_manager_set_and_get() {
        let manager = MemoryStateManager::new();
        let plugin_id = "test-plugin-id";
        let state = json!({"key": "value", "count": 42});

        manager
            .set_state(plugin_id, state.clone())
            .await
            .expect("set_state");
        let retrieved = manager.get_state(plugin_id).await.expect("get_state");
        assert_eq!(retrieved, Some(state));
    }

    #[tokio::test]
    async fn test_memory_state_manager_get_missing_returns_none() {
        let manager = MemoryStateManager::new();
        let plugin_id = "missing-plugin-id";
        let retrieved = manager.get_state(plugin_id).await.expect("get_state");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_state_manager_remove_state() {
        let manager = MemoryStateManager::new();
        let plugin_id = "remove-me";
        manager
            .set_state(plugin_id, json!({"x": 1}))
            .await
            .expect("set");
        manager.remove_state(plugin_id).await.expect("remove");
        let retrieved = manager.get_state(plugin_id).await.expect("get");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_state_manager_overwrite() {
        let manager = MemoryStateManager::new();
        let plugin_id = "overwrite-me";
        manager
            .set_state(plugin_id, json!({"v": 1}))
            .await
            .expect("set");
        manager
            .set_state(plugin_id, json!({"v": 2}))
            .await
            .expect("overwrite");
        let retrieved = manager.get_state(plugin_id).await.expect("get");
        assert_eq!(retrieved, Some(json!({"v": 2})));
    }

    #[tokio::test]
    async fn test_memory_state_manager_multiple_plugins() {
        let manager = MemoryStateManager::new();
        let id1 = "plugin-a";
        let id2 = "plugin-b";
        manager.set_state(id1, json!({"a": 1})).await.expect("set");
        manager.set_state(id2, json!({"b": 2})).await.expect("set");

        assert_eq!(
            manager.get_state(id1).await.expect("get"),
            Some(json!({"a": 1}))
        );
        assert_eq!(
            manager.get_state(id2).await.expect("get"),
            Some(json!({"b": 2}))
        );
    }

    #[test]
    fn test_memory_state_manager_default() {
        let _ = MemoryStateManager::default();
    }
}
