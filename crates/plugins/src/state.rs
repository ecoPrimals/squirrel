//! Plugin state management
//!
//! This module provides functionality for managing plugin state.

use std::fmt::Debug;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::sync::RwLock;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use anyhow::Result;

/// Plugin state manager trait
pub trait PluginStateManager: Send + Sync + Debug {
    /// Get plugin state
    fn get_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>>;
    
    /// Set plugin state
    fn set_state<'a>(&'a self, plugin_id: &'a Uuid, state: Value) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    
    /// Remove plugin state
    fn remove_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
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
    fn get_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>> {
        Box::pin(async move {
            let states = self.states.read().await;
            Ok(states.get(plugin_id).cloned())
        })
    }
    
    fn set_state<'a>(&'a self, plugin_id: &'a Uuid, state: Value) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut states = self.states.write().await;
            states.insert(*plugin_id, state);
            Ok(())
        })
    }
    
    fn remove_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let mut states = self.states.write().await;
            states.remove(plugin_id);
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
    fn get_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<Option<Value>>> + Send + 'a>> {
        Box::pin(async move {
            // For now, just use the in-memory cache
            let cache = self.cache.read().await;
            Ok(cache.get(plugin_id).cloned())
        })
    }
    
    fn set_state<'a>(&'a self, plugin_id: &'a Uuid, state: Value) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // For now, just use the in-memory cache
            let mut cache = self.cache.write().await;
            cache.insert(*plugin_id, state);
            Ok(())
        })
    }
    
    fn remove_state<'a>(&'a self, plugin_id: &'a Uuid) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // For now, just use the in-memory cache
            let mut cache = self.cache.write().await;
            cache.remove(plugin_id);
            Ok(())
        })
    }
} 