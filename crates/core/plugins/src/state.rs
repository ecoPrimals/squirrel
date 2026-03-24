// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin state management
//!
//! This module provides functionality for managing plugin state.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Plugin state manager trait
pub trait PluginStateManager: Send + Sync + Debug {
    /// Get plugin state
    fn get_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>>;

    /// Set plugin state
    fn set_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
        state: Value,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

    /// Remove plugin state
    fn remove_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

/// In-memory plugin state manager
#[derive(Debug, Default)]
pub struct MemoryStateManager {
    /// Plugin state storage
    states: RwLock<HashMap<Uuid, Value>>,
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
    fn get_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>> {
        Box::pin(async move {
            let states = self.states.read().await;
            Ok(states.get(plugin_id).cloned())
        })
    }

    fn set_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
        state: Value,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut states = self.states.write().await;
            states.insert(*plugin_id, state);
            drop(states);
            Ok(())
        })
    }

    fn remove_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut states = self.states.write().await;
            states.remove(plugin_id);
            drop(states);
            Ok(())
        })
    }
}

/// File-based plugin state manager
#[derive(Debug)]
pub struct FileStateManager {
    /// Base directory for state files
    base_dir: String,
    /// Memory cache for state
    cache: RwLock<HashMap<Uuid, Value>>,
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
    fn get_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>> {
        Box::pin(async move {
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
                    cache.insert(*plugin_id, value.clone());
                    drop(cache);
                    Ok(Some(value))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.into()),
            }
        })
    }

    fn set_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
        state: Value,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Update cache
            let mut cache = self.cache.write().await;
            cache.insert(*plugin_id, state.clone());

            // Ensure directory exists
            tokio::fs::create_dir_all(&self.base_dir).await?;

            // Write to file
            let file_path = format!("{}/{}.json", self.base_dir, plugin_id);
            let content = serde_json::to_string(&state)?;
            tokio::fs::write(&file_path, content).await?;
            drop(cache);

            Ok(())
        })
    }

    fn remove_state<'a>(
        &'a self,
        plugin_id: &'a Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_memory_state_manager_set_and_get() {
        let manager = MemoryStateManager::new();
        let plugin_id = Uuid::new_v4();
        let state = json!({"key": "value", "count": 42});

        manager
            .set_state(&plugin_id, state.clone())
            .await
            .expect("set_state");
        let retrieved = manager.get_state(&plugin_id).await.expect("get_state");
        assert_eq!(retrieved, Some(state));
    }

    #[tokio::test]
    async fn test_memory_state_manager_get_missing_returns_none() {
        let manager = MemoryStateManager::new();
        let plugin_id = Uuid::new_v4();
        let retrieved = manager.get_state(&plugin_id).await.expect("get_state");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_state_manager_remove_state() {
        let manager = MemoryStateManager::new();
        let plugin_id = Uuid::new_v4();
        manager
            .set_state(&plugin_id, json!({"x": 1}))
            .await
            .expect("set");
        manager.remove_state(&plugin_id).await.expect("remove");
        let retrieved = manager.get_state(&plugin_id).await.expect("get");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_state_manager_overwrite() {
        let manager = MemoryStateManager::new();
        let plugin_id = Uuid::new_v4();
        manager
            .set_state(&plugin_id, json!({"v": 1}))
            .await
            .expect("set");
        manager
            .set_state(&plugin_id, json!({"v": 2}))
            .await
            .expect("overwrite");
        let retrieved = manager.get_state(&plugin_id).await.expect("get");
        assert_eq!(retrieved, Some(json!({"v": 2})));
    }

    #[tokio::test]
    async fn test_memory_state_manager_multiple_plugins() {
        let manager = MemoryStateManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        manager.set_state(&id1, json!({"a": 1})).await.expect("set");
        manager.set_state(&id2, json!({"b": 2})).await.expect("set");

        assert_eq!(
            manager.get_state(&id1).await.expect("get"),
            Some(json!({"a": 1}))
        );
        assert_eq!(
            manager.get_state(&id2).await.expect("get"),
            Some(json!({"b": 2}))
        );
    }

    #[test]
    fn test_memory_state_manager_default() {
        let _ = MemoryStateManager::default();
    }
}
