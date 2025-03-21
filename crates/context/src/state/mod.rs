//! Context state module
//!
//! This module provides state representation and management for the context system.

use std::collections::HashMap;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::Result;

/// Context state structure
///
/// This structure maintains the state of a context along with metadata
/// for versioning and synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier for this state
    pub id: String,
    /// Version number of this state
    pub version: u64,
    /// Timestamp when this state was created/updated (Unix timestamp)
    pub timestamp: u64,
    /// State data as key-value pairs
    pub data: HashMap<String, String>,
    /// Metadata about this state
    pub metadata: HashMap<String, String>,
    /// Whether this state has been synchronized with persistence
    #[serde(default)]
    pub synchronized: bool,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            version: 1,
            timestamp,
            data: HashMap::new(),
            metadata: HashMap::new(),
            synchronized: false,
        }
    }
    
    /// Create a new state with the given ID
    pub fn with_id(id: String) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        Self {
            id,
            version: 1,
            timestamp,
            data: HashMap::new(),
            metadata: HashMap::new(),
            synchronized: false,
        }
    }
    
    /// Create a new state with the given data
    pub fn with_data(data: HashMap<String, String>) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            version: 1,
            timestamp,
            data,
            metadata: HashMap::new(),
            synchronized: false,
        }
    }
    
    /// Load state from storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - State not found
    /// - Deserialization failure
    /// - Storage access error
    pub fn load(id: &str, storage: &impl StateStorage) -> Result<Self> {
        storage.load_state(id)
    }
    
    /// Save state to storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Serialization failure
    /// - Storage access error
    pub fn save(&mut self, storage: &impl StateStorage) -> Result<()> {
        let result = storage.save_state(&self.id, self);
        if result.is_ok() {
            self.synchronized = true;
        }
        result
    }
    
    /// Update state with new data
    ///
    /// This increments the version number and updates the timestamp.
    pub fn update(&mut self, data: HashMap<String, String>) {
        self.data = data;
        self.version += 1;
        self.timestamp = Utc::now().timestamp() as u64;
        self.synchronized = false;
    }
    
    /// Update a single value in the state
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
        self.version += 1;
        self.timestamp = Utc::now().timestamp() as u64;
        self.synchronized = false;
    }
    
    /// Get a value from the state
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    /// Check if the state contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    /// Remove a key from the state
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let result = self.data.remove(key);
        if result.is_some() {
            self.version += 1;
            self.timestamp = Utc::now().timestamp() as u64;
            self.synchronized = false;
        }
        result
    }
    
    /// Get all key-value pairs in the state
    pub fn all(&self) -> &HashMap<String, String> {
        &self.data
    }
    
    /// Clear all data from the state
    pub fn clear(&mut self) {
        self.data.clear();
        self.version += 1;
        self.timestamp = Utc::now().timestamp() as u64;
        self.synchronized = false;
    }
    
    /// Set metadata for the state
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata from the state
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Mark the state as synchronized
    pub fn mark_synchronized(&mut self) {
        self.synchronized = true;
    }
    
    /// Check if the state has been synchronized
    pub fn is_synchronized(&self) -> bool {
        self.synchronized
    }
    
    /// Create a snapshot of the current state
    pub fn create_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            id: format!("{}-{}", self.id, Uuid::new_v4()),
            state_id: self.id.clone(),
            version: self.version,
            timestamp: self.timestamp,
            data: self.data.clone(),
        }
    }
}

/// State snapshot structure for recovery points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Unique identifier for this snapshot
    pub id: String,
    /// ID of the state this snapshot was created from
    pub state_id: String,
    /// Version number when this snapshot was created
    pub version: u64,
    /// Timestamp when this snapshot was created
    pub timestamp: u64,
    /// State data at the time of snapshot
    pub data: HashMap<String, String>,
}

/// State storage trait for persistence operations
pub trait StateStorage {
    /// Load a state from storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - State not found
    /// - Deserialization failure
    /// - Storage access error
    fn load_state(&self, id: &str) -> Result<State>;
    
    /// Save a state to storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Serialization failure
    /// - Storage access error
    fn save_state(&self, id: &str, state: &State) -> Result<()>;
    
    /// Delete a state from storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - State not found
    /// - Storage access error
    fn delete_state(&self, id: &str) -> Result<()>;
    
    /// List all state IDs in storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Storage access error
    fn list_states(&self) -> Result<Vec<String>>;
    
    /// Save a snapshot to storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Serialization failure
    /// - Storage access error
    fn save_snapshot(&self, snapshot: &StateSnapshot) -> Result<()>;
    
    /// Load a snapshot from storage
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Snapshot not found
    /// - Deserialization failure
    /// - Storage access error
    fn load_snapshot(&self, id: &str) -> Result<StateSnapshot>;
    
    /// List all snapshots for a state
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Storage access error
    fn list_snapshots(&self, state_id: &str) -> Result<Vec<StateSnapshot>>;
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
} 