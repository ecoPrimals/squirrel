use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{debug, warn};
use crate::error::Result;
use super::{Plugin, PluginState};

/// Plugin state storage trait
#[async_trait]
pub trait PluginStateStorage: Send + Sync {
    /// Save plugin state
    async fn save_state(&self, state: &PluginState) -> Result<()>;
    
    /// Load plugin state
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// Delete plugin state
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()>;
    
    /// List all plugin states
    async fn list_states(&self) -> Result<Vec<PluginState>>;
}

/// File system plugin state storage
#[derive(Debug, Clone)]
pub struct FileSystemStateStorage {
    /// Base directory for plugin state
    base_dir: PathBuf,
}

impl FileSystemStateStorage {
    /// Create a new file system state storage
    #[must_use]
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
    
    /// Get state file path for a plugin
    fn get_state_path(&self, plugin_id: Uuid) -> PathBuf {
        self.base_dir.join(format!("{plugin_id}.json"))
    }
}

#[async_trait]
impl PluginStateStorage for FileSystemStateStorage {
    async fn save_state(&self, state: &PluginState) -> Result<()> {
        let path = self.get_state_path(state.plugin_id);
        
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Serialize state to JSON
        let json = serde_json::to_string_pretty(state)?;
        
        // Write to file
        fs::write(path, json)?;
        
        debug!("Saved plugin state: {}", state.plugin_id);
        
        Ok(())
    }
    
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let path = self.get_state_path(plugin_id);
        
        if !path.exists() {
            debug!("Plugin state not found: {}", plugin_id);
            return Ok(None);
        }
        
        // Read file content
        let content = fs::read_to_string(path)?;
        
        // Deserialize from JSON
        let state: PluginState = serde_json::from_str(&content)?;
        
        debug!("Loaded plugin state: {}", plugin_id);
        
        Ok(Some(state))
    }
    
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        let path = self.get_state_path(plugin_id);
        
        if path.exists() {
            fs::remove_file(path)?;
            debug!("Deleted plugin state: {}", plugin_id);
        }
        
        Ok(())
    }
    
    async fn list_states(&self) -> Result<Vec<PluginState>> {
        let mut states = Vec::new();
        
        if !self.base_dir.exists() {
            return Ok(states);
        }
        
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                // Read and parse file
                let content = fs::read_to_string(&path)?;
                match serde_json::from_str::<PluginState>(&content) {
                    Ok(state) => states.push(state),
                    Err(e) => warn!("Failed to parse plugin state: {:?} - {}", path, e),
                }
            }
        }
        
        Ok(states)
    }
}

/// Memory plugin state storage for testing
#[derive(Debug)]
pub struct MemoryStateStorage {
    /// In-memory state storage
    states: Arc<RwLock<HashMap<Uuid, PluginState>>>,
}

impl MemoryStateStorage {
    /// Create a new memory state storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl PluginStateStorage for MemoryStateStorage {
    async fn save_state(&self, state: &PluginState) -> Result<()> {
        let mut states = self.states.write().await;
        states.insert(state.plugin_id, state.clone());
        Ok(())
    }
    
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let states = self.states.read().await;
        Ok(states.get(&plugin_id).cloned())
    }
    
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        let mut states = self.states.write().await;
        states.remove(&plugin_id);
        Ok(())
    }
    
    async fn list_states(&self) -> Result<Vec<PluginState>> {
        let states = self.states.read().await;
        Ok(states.values().cloned().collect())
    }
}

impl Default for MemoryStateStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin state manager for handling state persistence
pub struct PluginStateManager {
    /// State storage
    storage: Box<dyn PluginStateStorage>,
}

// Manual Debug implementation since we can't derive it for Box<dyn PluginStateStorage>
impl std::fmt::Debug for PluginStateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginStateManager")
            .field("storage", &"Box<dyn PluginStateStorage>")
            .finish()
    }
}

impl PluginStateManager {
    /// Create a new plugin state manager
    #[must_use]
    pub fn new(storage: Box<dyn PluginStateStorage>) -> Self {
        Self { storage }
    }
    
    /// Create a new plugin state manager with file system storage
    /// 
    /// # Errors
    /// 
    /// Returns an error if the base directory cannot be created or accessed.
    pub fn with_file_storage(base_dir: PathBuf) -> Result<Self> {
        // Create directory if it doesn't exist
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir)?;
        }
        
        Ok(Self {
            storage: Box::new(FileSystemStateStorage::new(base_dir)),
        })
    }
    
    /// Create a new plugin state manager with memory storage
    #[must_use]
    pub fn with_memory_storage() -> Self {
        Self {
            storage: Box::new(MemoryStateStorage::new()),
        }
    }
    
    /// Save plugin state
    /// 
    /// # Errors
    /// 
    /// Returns an error if saving the state fails due to serialization or storage issues.
    pub async fn save_state(&self, plugin: &dyn Plugin) -> Result<()> {
        if let Ok(Some(state)) = plugin.get_state().await {
            self.storage.save_state(&state).await?;
        }
        
        Ok(())
    }
    
    /// Load plugin state
    /// 
    /// # Errors
    /// 
    /// Returns an error if loading the state fails due to deserialization or storage issues.
    pub async fn load_state(&self, plugin: &dyn Plugin) -> Result<()> {
        let plugin_id = plugin.metadata().id;
        
        if let Some(state) = self.storage.load_state(plugin_id).await? {
            plugin.set_state(state).await?;
        }
        
        Ok(())
    }
    
    /// Delete plugin state
    /// 
    /// # Errors
    /// 
    /// Returns an error if deleting the state fails due to storage issues.
    pub async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        self.storage.delete_state(plugin_id).await
    }
    
    /// Save states for all plugins
    /// 
    /// # Errors
    /// 
    /// Returns an error if saving any plugin state fails.
    pub async fn save_all_states(&self, plugins: &[Box<dyn Plugin>]) -> Result<()> {
        for plugin in plugins {
            self.save_state(plugin.as_ref()).await?;
        }
        
        Ok(())
    }
    
    /// Load states for all plugins
    /// 
    /// # Errors
    /// 
    /// Returns an error if loading any plugin state fails.
    pub async fn load_all_states(&self, plugins: &[Box<dyn Plugin>]) -> Result<()> {
        for plugin in plugins {
            self.load_state(plugin.as_ref()).await?;
        }
        
        Ok(())
    }
    
    /// Save plugin state directly
    /// 
    /// # Errors
    /// 
    /// Returns an error if saving the state fails due to serialization or storage issues.
    pub async fn save_plugin_state(&self, state: &PluginState) -> Result<()> {
        self.storage.save_state(state).await
    }
    
    /// Load plugin state directly
    /// 
    /// # Errors
    /// 
    /// Returns an error if loading the state fails due to deserialization or storage issues.
    pub async fn load_plugin_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        self.storage.load_state(plugin_id).await
    }
    
    /// Delete plugin state directly
    /// 
    /// # Errors
    /// 
    /// Returns an error if deleting the state fails due to storage issues.
    pub async fn delete_plugin_state(&self, plugin_id: Uuid) -> Result<()> {
        self.storage.delete_state(plugin_id).await
    }
    
    /// List all plugin states
    /// 
    /// # Errors
    /// 
    /// Returns an error if listing the states fails due to storage issues.
    pub async fn list_plugin_states(&self) -> Result<Vec<PluginState>> {
        self.storage.list_states().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use crate::plugin::{Plugin, PluginMetadata, PluginState};
    
    #[derive(Debug)]
    struct TestPlugin {
        metadata: PluginMetadata,
        state: Arc<RwLock<Option<PluginState>>>,
    }
    
    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }
        
        async fn initialize(&self) -> Result<()> {
            Ok(())
        }
        
        async fn shutdown(&self) -> Result<()> {
            Ok(())
        }
        
        async fn get_state(&self) -> Result<Option<PluginState>> {
            Ok(self.state.read().await.clone())
        }
        
        async fn set_state(&self, state: PluginState) -> Result<()> {
            *self.state.write().await = Some(state);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_memory_state_storage() {
        let storage = MemoryStateStorage::new();
        
        // Create a test state
        let plugin_id = Uuid::new_v4();
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"key": "value"}),
            last_modified: Utc::now(),
        };
        
        // Save state
        storage.save_state(&state).await.unwrap();
        
        // Load state
        let loaded_state = storage.load_state(plugin_id).await.unwrap().unwrap();
        
        // Verify
        assert_eq!(loaded_state.plugin_id, state.plugin_id);
        assert_eq!(loaded_state.data, state.data);
        
        // Delete state
        storage.delete_state(plugin_id).await.unwrap();
        
        // Verify deleted
        let deleted_state = storage.load_state(plugin_id).await.unwrap();
        assert!(deleted_state.is_none());
    }
    
    #[tokio::test]
    async fn test_plugin_state_manager() {
        let manager = PluginStateManager::with_memory_storage();
        
        // Create a test plugin
        let plugin_id = Uuid::new_v4();
        let plugin = TestPlugin {
            metadata: PluginMetadata {
                id: plugin_id,
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                description: "Test plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["test".to_string()],
            },
            state: Arc::new(RwLock::new(None)),
        };
        
        // Create a test state
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"key": "value"}),
            last_modified: Utc::now(),
        };
        
        // Set plugin state
        plugin.set_state(state.clone()).await.unwrap();
        
        // Save state
        manager.save_state(&plugin).await.unwrap();
        
        // Clear plugin state
        *plugin.state.write().await = None;
        
        // Load state
        manager.load_state(&plugin).await.unwrap();
        
        // Verify state was loaded
        let loaded_state = plugin.get_state().await.unwrap().unwrap();
        assert_eq!(loaded_state.plugin_id, state.plugin_id);
        assert_eq!(loaded_state.data, state.data);
    }
    
    #[tokio::test]
    async fn test_file_system_state_storage() {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileSystemStateStorage::new(temp_dir.path().to_path_buf());
        
        // Create a test state
        let plugin_id = Uuid::new_v4();
        let state = PluginState {
            plugin_id,
            data: serde_json::json!({"key": "value"}),
            last_modified: Utc::now(),
        };
        
        // Save state
        storage.save_state(&state).await.unwrap();
        
        // Verify file exists
        let state_path = storage.get_state_path(plugin_id);
        assert!(state_path.exists());
        
        // Load state
        let loaded_state = storage.load_state(plugin_id).await.unwrap().unwrap();
        
        // Verify
        assert_eq!(loaded_state.plugin_id, state.plugin_id);
        assert_eq!(loaded_state.data, state.data);
        
        // Delete state
        storage.delete_state(plugin_id).await.unwrap();
        
        // Verify file deleted
        assert!(!state_path.exists());
    }
} 