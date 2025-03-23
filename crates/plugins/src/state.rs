//! Plugin state management
//!
//! This module provides functionality for managing plugin state persistence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::Result;
use crate::PluginError;

/// Plugin state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// State data as a JSON value
    pub data: serde_json::Value,
    
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PluginState {
    /// Create new plugin state
    pub fn new(plugin_id: Uuid, data: serde_json::Value) -> Self {
        Self {
            plugin_id,
            data,
            last_updated: chrono::Utc::now(),
        }
    }
    
    /// Update the state data
    pub fn update(&mut self, data: serde_json::Value) {
        self.data = data;
        self.last_updated = chrono::Utc::now();
    }
}

/// Plugin state manager trait
#[async_trait]
pub trait PluginStateManager: Send + Sync {
    /// Get plugin state
    async fn get_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// Set plugin state
    async fn set_state(&self, state: PluginState) -> Result<()>;
    
    /// Delete plugin state
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()>;
}

/// In-memory plugin state manager
#[derive(Debug, Default)]
pub struct MemoryStateManager {
    /// Plugin state storage
    states: RwLock<HashMap<Uuid, PluginState>>,
}

impl MemoryStateManager {
    /// Create new memory state manager
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl PluginStateManager for MemoryStateManager {
    /// Get plugin state
    async fn get_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let states = self.states.read().await;
        Ok(states.get(&plugin_id).cloned())
    }
    
    /// Set plugin state
    async fn set_state(&self, state: PluginState) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(state.plugin_id, state);
        Ok(())
    }
    
    /// Delete plugin state
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        let mut states = self.states.write().await;
        states.remove(&plugin_id);
        Ok(())
    }
}

/// File-based plugin state manager
#[derive(Debug)]
pub struct FileStateManager {
    /// State directory
    state_dir: PathBuf,
}

impl FileStateManager {
    /// Create new file state manager
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
        }
    }
    
    /// Get state file path for plugin
    fn get_state_path(&self, plugin_id: Uuid) -> PathBuf {
        self.state_dir.join(format!("{}.json", plugin_id))
    }
}

#[async_trait]
impl PluginStateManager for FileStateManager {
    /// Get plugin state
    async fn get_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let path = self.get_state_path(plugin_id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(path).await?;
        let state = serde_json::from_str(&content)
            .map_err(|e| PluginError::SerializationError(e))?;
        
        Ok(Some(state))
    }
    
    /// Set plugin state
    async fn set_state(&self, state: PluginState) -> Result<()> {
        let path = self.get_state_path(state.plugin_id);
        
        // Ensure state directory exists
        if !self.state_dir.exists() {
            fs::create_dir_all(&self.state_dir).await?;
        }
        
        let content = serde_json::to_string_pretty(&state)
            .map_err(|e| PluginError::SerializationError(e))?;
        
        fs::write(path, content).await?;
        
        Ok(())
    }
    
    /// Delete plugin state
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        let path = self.get_state_path(plugin_id);
        
        if path.exists() {
            fs::remove_file(path).await?;
        }
        
        Ok(())
    }
} 