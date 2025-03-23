//! Plugin state management
//!
//! This module provides functionality for managing plugin state.

use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio::sync::RwLock;
use crate::errors::{Result, PluginError};
use serde_json::Value;

/// Plugin state manager trait
pub trait PluginStateManager: Send + Sync + Debug {
    /// Get plugin state
    async fn get_state(&self, plugin_id: &Uuid) -> Result<Option<Value>>;
    
    /// Set plugin state
    async fn set_state(&self, plugin_id: &Uuid, state: Value) -> Result<()>;
    
    /// Remove plugin state
    async fn remove_state(&self, plugin_id: &Uuid) -> Result<()>;
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

#[async_trait::async_trait]
impl PluginStateManager for MemoryStateManager {
    async fn get_state(&self, plugin_id: &Uuid) -> Result<Option<Value>> {
        let states = self.states.read().await;
        Ok(states.get(plugin_id).cloned())
    }
    
    async fn set_state(&self, plugin_id: &Uuid, state: Value) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(*plugin_id, state);
        Ok(())
    }
    
    async fn remove_state(&self, plugin_id: &Uuid) -> Result<()> {
        let mut states = self.states.write().await;
        states.remove(plugin_id);
        Ok(())
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

#[async_trait::async_trait]
impl PluginStateManager for FileStateManager {
    async fn get_state(&self, plugin_id: &Uuid) -> Result<Option<Value>> {
        // For now, just use the in-memory cache
        let cache = self.cache.read().await;
        Ok(cache.get(plugin_id).cloned())
    }
    
    async fn set_state(&self, plugin_id: &Uuid, state: Value) -> Result<()> {
        // For now, just use the in-memory cache
        let mut cache = self.cache.write().await;
        cache.insert(*plugin_id, state);
        Ok(())
    }
    
    async fn remove_state(&self, plugin_id: &Uuid) -> Result<()> {
        // For now, just use the in-memory cache
        let mut cache = self.cache.write().await;
        cache.remove(plugin_id);
        Ok(())
    }
} 