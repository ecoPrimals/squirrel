// Plugin State Management Module
//
// This module provides functionality for managing plugin state,
// including persistence, versioning, and migration.

use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;

/// Plugin state error
#[derive(Debug, Error)]
pub enum StateError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Version mismatch
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        expected: String,
        found: String,
    },
    
    /// State not found
    #[error("State not found for plugin: {0}")]
    NotFound(Uuid),
    
    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),
}

/// Plugin state with versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// Plugin ID
    pub plugin_id: Uuid,
    
    /// State version
    pub version: String,
    
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    
    /// Plugin state data
    pub data: Value,
}

impl PluginState {
    /// Create a new plugin state
    pub fn new(plugin_id: Uuid, version: impl Into<String>, data: Value) -> Self {
        Self {
            plugin_id,
            version: version.into(),
            updated_at: chrono::Utc::now(),
            data,
        }
    }
    
    /// Check if state is compatible with a version
    pub fn is_compatible_with(&self, version: &str) -> bool {
        // Simple version check for now, could use semver for more sophisticated checks
        self.version == version
    }
    
    /// Update the state data
    pub fn update_data(&mut self, data: Value) {
        self.data = data;
        self.updated_at = chrono::Utc::now();
    }
}

/// State migration strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// Fail if versions don't match
    Fail,
    
    /// Replace with new state if versions don't match
    Replace,
    
    /// Attempt to migrate the state
    Migrate,
}

/// State storage interface
#[async_trait]
pub trait StateStorage: Send + Sync + Debug {
    /// Save plugin state
    async fn save_state(&self, state: &PluginState) -> Result<()>;
    
    /// Load plugin state
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// Delete plugin state
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()>;
    
    /// List all plugin states
    async fn list_states(&self) -> Result<Vec<PluginState>>;
}

/// File-based state storage
#[derive(Debug)]
pub struct FileStateStorage {
    /// Directory where states are stored
    directory: PathBuf,
}

impl FileStateStorage {
    /// Create a new file-based state storage
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        let directory = directory.into();
        if !directory.exists() {
            if let Err(e) = std::fs::create_dir_all(&directory) {
                error!("Failed to create state directory: {}", e);
            }
        }
        Self { directory }
    }
    
    /// Get the file path for a plugin state
    fn get_state_path(&self, plugin_id: Uuid) -> PathBuf {
        self.directory.join(format!("{}.json", plugin_id))
    }
}

#[async_trait]
impl StateStorage for FileStateStorage {
    async fn save_state(&self, state: &PluginState) -> Result<()> {
        let path = self.get_state_path(state.plugin_id);
        let json = serde_json::to_string_pretty(state)?;
        
        // Write to a temporary file first, then rename
        let temp_path = path.with_extension("tmp");
        tokio::task::spawn_blocking(move || -> Result<()> {
            fs::write(&temp_path, json)?;
            fs::rename(&temp_path, &path)?;
            Ok(())
        })
        .await??;
        
        debug!("Saved state for plugin {}", state.plugin_id);
        Ok(())
    }
    
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        let path = self.get_state_path(plugin_id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let path_clone = path.clone();
        let state = tokio::task::spawn_blocking(move || -> Result<PluginState> {
            let json = fs::read_to_string(&path_clone)?;
            let state = serde_json::from_str(&json)?;
            Ok(state)
        })
        .await??;
        
        debug!("Loaded state for plugin {}", plugin_id);
        Ok(Some(state))
    }
    
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        let path = self.get_state_path(plugin_id);
        
        if path.exists() {
            tokio::task::spawn_blocking(move || -> Result<()> {
                fs::remove_file(&path)?;
                Ok(())
            })
            .await??;
            
            debug!("Deleted state for plugin {}", plugin_id);
        }
        
        Ok(())
    }
    
    async fn list_states(&self) -> Result<Vec<PluginState>> {
        let directory = self.directory.clone();
        
        let states = tokio::task::spawn_blocking(move || -> Result<Vec<PluginState>> {
            let mut states = Vec::new();
            
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(json) = fs::read_to_string(&path) {
                        if let Ok(state) = serde_json::from_str::<PluginState>(&json) {
                            states.push(state);
                        }
                    }
                }
            }
            
            Ok(states)
        })
        .await??;
        
        Ok(states)
    }
}

/// Memory-based state storage (for testing)
#[derive(Debug, Default)]
pub struct MemoryStateStorage {
    /// States stored in memory
    states: RwLock<HashMap<Uuid, PluginState>>,
}

impl MemoryStateStorage {
    /// Create a new memory-based state storage
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StateStorage for MemoryStateStorage {
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

/// State manager interface
#[async_trait]
pub trait StateManager: Send + Sync + Debug {
    /// Save state for a plugin
    async fn save_state(&self, state: PluginState) -> Result<()>;
    
    /// Load state for a plugin
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>>;
    
    /// Delete state for a plugin
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Update state for a plugin
    async fn update_state(&self, plugin_id: Uuid, data: Value) -> Result<PluginState>;
    
    /// Create a transaction
    async fn begin_transaction(&self, plugin_id: Uuid) -> Result<StateTransaction>;
    
    /// Perform a state migration
    async fn migrate_state(
        &self,
        plugin_id: Uuid,
        target_version: &str,
        migration_fn: impl Fn(PluginState) -> Result<PluginState> + Send + Sync,
    ) -> Result<Option<PluginState>>;
}

/// State transaction
#[derive(Debug)]
pub struct StateTransaction {
    /// Plugin ID
    plugin_id: Uuid,
    
    /// Original state
    original_state: Option<PluginState>,
    
    /// Current state
    current_state: Option<PluginState>,
    
    /// Whether the transaction has been committed
    committed: bool,
}

impl StateTransaction {
    /// Create a new transaction
    pub fn new(plugin_id: Uuid, original_state: Option<PluginState>) -> Self {
        Self {
            plugin_id,
            original_state: original_state.clone(),
            current_state: original_state,
            committed: false,
        }
    }
    
    /// Get the current state
    pub fn current_state(&self) -> Option<&PluginState> {
        self.current_state.as_ref()
    }
    
    /// Update the state
    pub fn update_state(&mut self, data: Value) -> Result<&PluginState> {
        let state = match self.current_state.take() {
            Some(mut state) => {
                state.update_data(data);
                state
            }
            None => {
                return Err(PluginError::InvalidOperation(
                    "Cannot update state: state not found".to_string(),
                ))
            }
        };
        
        self.current_state = Some(state);
        Ok(self.current_state.as_ref().unwrap())
    }
    
    /// Create a new state
    pub fn create_state(&mut self, version: &str, data: Value) -> &PluginState {
        let state = PluginState::new(self.plugin_id, version, data);
        self.current_state = Some(state);
        self.current_state.as_ref().unwrap()
    }
    
    /// Check if the state has been modified
    pub fn is_modified(&self) -> bool {
        match (&self.original_state, &self.current_state) {
            (Some(original), Some(current)) => {
                original.version != current.version || original.data != current.data
            }
            (None, Some(_)) => true,
            _ => false,
        }
    }
    
    /// Get the plugin ID
    pub fn plugin_id(&self) -> Uuid {
        self.plugin_id
    }
}

/// Default state manager implementation
#[derive(Debug)]
pub struct DefaultStateManager {
    /// State storage
    storage: Arc<dyn StateStorage>,
    
    /// In-memory cache
    cache: RwLock<HashMap<Uuid, PluginState>>,
    
    /// Active transactions
    transactions: RwLock<HashMap<Uuid, StateTransaction>>,
    
    /// Default migration strategy
    default_migration_strategy: MigrationStrategy,
}

impl DefaultStateManager {
    /// Create a new state manager
    pub fn new(storage: Arc<dyn StateStorage>) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
            transactions: RwLock::new(HashMap::new()),
            default_migration_strategy: MigrationStrategy::Fail,
        }
    }
    
    /// Set the default migration strategy
    pub fn with_migration_strategy(mut self, strategy: MigrationStrategy) -> Self {
        self.default_migration_strategy = strategy;
        self
    }
    
    /// Commit a transaction
    async fn commit_transaction(&self, transaction: StateTransaction) -> Result<Option<PluginState>> {
        if transaction.committed {
            return Err(PluginError::InvalidOperation(
                "Transaction already committed".to_string(),
            ));
        }
        
        if !transaction.is_modified() {
            return Ok(transaction.current_state);
        }
        
        let plugin_id = transaction.plugin_id;
        
        if let Some(state) = transaction.current_state {
            // Update cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(plugin_id, state.clone());
            }
            
            // Save to storage
            self.storage.save_state(&state).await?;
            
            // Remove transaction
            {
                let mut transactions = self.transactions.write().await;
                transactions.remove(&plugin_id);
            }
            
            Ok(Some(state))
        } else {
            // Delete state
            self.delete_state(plugin_id).await?;
            Ok(None)
        }
    }
}

#[async_trait]
impl StateManager for DefaultStateManager {
    async fn save_state(&self, state: PluginState) -> Result<()> {
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(state.plugin_id, state.clone());
        }
        
        // Save to storage
        self.storage.save_state(&state).await
    }
    
    async fn load_state(&self, plugin_id: Uuid) -> Result<Option<PluginState>> {
        // First check cache
        {
            let cache = self.cache.read().await;
            if let Some(state) = cache.get(&plugin_id) {
                return Ok(Some(state.clone()));
            }
        }
        
        // Then check storage
        if let Some(state) = self.storage.load_state(plugin_id).await? {
            // Update cache
            {
                let mut cache = self.cache.write().await;
                cache.insert(plugin_id, state.clone());
            }
            
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
    
    async fn delete_state(&self, plugin_id: Uuid) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.cache.write().await;
            cache.remove(&plugin_id);
        }
        
        // Remove from storage
        self.storage.delete_state(plugin_id).await
    }
    
    async fn update_state(&self, plugin_id: Uuid, data: Value) -> Result<PluginState> {
        // Start a transaction
        let mut transaction = self.begin_transaction(plugin_id).await?;
        
        // Create or update state
        let state = if transaction.current_state().is_some() {
            transaction.update_state(data)?
        } else {
            transaction.create_state("1.0.0", data)
        };
        
        // Commit the transaction
        if let Some(state) = self.commit_transaction(transaction).await? {
            Ok(state)
        } else {
            Err(PluginError::InvalidOperation(
                "Failed to commit transaction".to_string(),
            ))
        }
    }
    
    async fn begin_transaction(&self, plugin_id: Uuid) -> Result<StateTransaction> {
        // Check if a transaction already exists
        {
            let transactions = self.transactions.read().await;
            if let Some(transaction) = transactions.get(&plugin_id) {
                return Err(PluginError::InvalidOperation(format!(
                    "Transaction already exists for plugin {}",
                    plugin_id
                )));
            }
        }
        
        // Load current state
        let state = self.load_state(plugin_id).await?;
        
        // Create transaction
        let transaction = StateTransaction::new(plugin_id, state);
        
        // Store transaction
        {
            let mut transactions = self.transactions.write().await;
            transactions.insert(plugin_id, transaction.clone());
        }
        
        Ok(transaction)
    }
    
    async fn migrate_state(
        &self,
        plugin_id: Uuid,
        target_version: &str,
        migration_fn: impl Fn(PluginState) -> Result<PluginState> + Send + Sync,
    ) -> Result<Option<PluginState>> {
        // Load current state
        let state = match self.load_state(plugin_id).await? {
            Some(state) => state,
            None => return Ok(None),
        };
        
        // Check if migration is needed
        if state.is_compatible_with(target_version) {
            return Ok(Some(state));
        }
        
        // Perform migration
        match self.default_migration_strategy {
            MigrationStrategy::Fail => {
                return Err(PluginError::InvalidOperation(format!(
                    "Version mismatch: expected {}, found {}",
                    target_version, state.version
                )))
            }
            MigrationStrategy::Replace => {
                // Just return the state, caller will need to create a new one
                return Ok(None);
            }
            MigrationStrategy::Migrate => {
                // Perform migration using provided function
                let migrated_state = migration_fn(state)?;
                
                // Save migrated state
                self.save_state(migrated_state.clone()).await?;
                
                Ok(Some(migrated_state))
            }
        }
    }
}

impl Clone for StateTransaction {
    fn clone(&self) -> Self {
        Self {
            plugin_id: self.plugin_id,
            original_state: self.original_state.clone(),
            current_state: self.current_state.clone(),
            committed: self.committed,
        }
    }
}

/// Tests for state management
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_memory_storage() {
        let storage = Arc::new(MemoryStateStorage::new());
        let plugin_id = Uuid::new_v4();
        
        // Create state
        let state = PluginState::new(plugin_id, "1.0.0", json!({ "key": "value" }));
        
        // Save state
        assert!(storage.save_state(&state).await.is_ok());
        
        // Load state
        let loaded_state = storage.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(loaded_state.plugin_id, plugin_id);
        assert_eq!(loaded_state.version, "1.0.0");
        assert_eq!(loaded_state.data, json!({ "key": "value" }));
        
        // Delete state
        assert!(storage.delete_state(plugin_id).await.is_ok());
        
        // State should be gone
        assert!(storage.load_state(plugin_id).await.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn test_state_manager() {
        let storage = Arc::new(MemoryStateStorage::new());
        let manager = Arc::new(DefaultStateManager::new(storage));
        let plugin_id = Uuid::new_v4();
        
        // Create state
        let state = PluginState::new(plugin_id, "1.0.0", json!({ "key": "value" }));
        
        // Save state
        assert!(manager.save_state(state).await.is_ok());
        
        // Load state
        let loaded_state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(loaded_state.plugin_id, plugin_id);
        assert_eq!(loaded_state.version, "1.0.0");
        assert_eq!(loaded_state.data, json!({ "key": "value" }));
        
        // Update state
        let updated_state = manager
            .update_state(plugin_id, json!({ "key": "new_value" }))
            .await
            .unwrap();
        assert_eq!(updated_state.data, json!({ "key": "new_value" }));
        
        // Verify update
        let verified_state = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(verified_state.data, json!({ "key": "new_value" }));
        
        // Test transaction
        let mut transaction = manager.begin_transaction(plugin_id).await.unwrap();
        assert!(transaction.current_state().is_some());
        
        transaction
            .update_state(json!({ "key": "transaction_value" }))
            .unwrap();
        
        // State shouldn't change until committed
        let state_before_commit = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state_before_commit.data, json!({ "key": "new_value" }));
        
        // Commit transaction
        let committed_state = manager
            .commit_transaction(transaction)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(committed_state.data, json!({ "key": "transaction_value" }));
        
        // Verify after commit
        let state_after_commit = manager.load_state(plugin_id).await.unwrap().unwrap();
        assert_eq!(state_after_commit.data, json!({ "key": "transaction_value" }));
        
        // Delete state
        assert!(manager.delete_state(plugin_id).await.is_ok());
        
        // State should be gone
        assert!(manager.load_state(plugin_id).await.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn test_migration() {
        let storage = Arc::new(MemoryStateStorage::new());
        let manager = Arc::new(
            DefaultStateManager::new(storage).with_migration_strategy(MigrationStrategy::Migrate),
        );
        let plugin_id = Uuid::new_v4();
        
        // Create v1 state
        let state = PluginState::new(plugin_id, "1.0.0", json!({ "count": 1 }));
        
        // Save state
        assert!(manager.save_state(state).await.is_ok());
        
        // Define migration function
        let migration_fn = |state: PluginState| -> Result<PluginState> {
            // For v1 to v2, we need to add a "name" field
            let mut data = state.data.as_object().unwrap().clone();
            data.insert("name".to_string(), json!("default"));
            
            Ok(PluginState::new(
                state.plugin_id,
                "2.0.0", // New version
                Value::Object(data),
            ))
        };
        
        // Perform migration
        let migrated_state = manager
            .migrate_state(plugin_id, "2.0.0", migration_fn)
            .await
            .unwrap()
            .unwrap();
        
        // Verify migration
        assert_eq!(migrated_state.version, "2.0.0");
        assert_eq!(migrated_state.data["count"], 1);
        assert_eq!(migrated_state.data["name"], "default");
    }
} 