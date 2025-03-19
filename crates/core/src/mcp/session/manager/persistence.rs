/// Persistence functionality for session management.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{error, info, instrument};
use thiserror::Error;
use sha2::Digest;

// TODO: Uncomment when state module is implemented
// use super::state::{State, StateError};

/// Errors that can occur during persistence operations
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// Input/output errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization or deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// State not found errors
    #[error("State not found: {0}")]
    NotFound(String),

    /// Invalid data errors
    #[error("Invalid state data: {0}")]
    InvalidData(String),
}

// TODO: Define State here temporarily until state module is implemented
/// Represents the state of a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier for the state
    pub id: String,
    /// Arbitrary state data
    pub data: serde_json::Value,
    /// When the state was created
    pub created_at: DateTime<Utc>,
    /// When the state was last updated
    pub updated_at: DateTime<Utc>,
}

/// A state together with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// The state object
    pub state: State,
    /// Metadata about the state
    pub metadata: PersistentMetadata,
}

/// Metadata for a persistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentMetadata {
    /// Version number of the state
    pub version: u64,
    /// When the state was created
    pub created_at: DateTime<Utc>,
    /// When the state was last updated
    pub updated_at: DateTime<Utc>,
    /// Checksum for data validation
    pub checksum: String,
}

/// Handles the persistence of state objects
pub struct StatePersistence {
    storage_path: PathBuf,
    states: HashMap<String, PersistentState>,
}

impl StatePersistence {
    /// Creates a new `StatePersistence` instance
    ///
    /// # Parameters
    ///
    /// * `storage_path` - The path where states will be stored
    ///
    /// # Returns
    ///
    /// A new `StatePersistence` instance
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            states: HashMap::new(),
        }
    }

    /// Saves a state to persistent storage
    ///
    /// # Parameters
    ///
    /// * `state` - The state to save
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    #[instrument(skip(self, state))]
    pub async fn save_state(&mut self, state: State) -> Result<(), PersistenceError> {
        let state_path = self.get_state_path(&state.id);
        
        // Create persistent state
        let metadata = PersistentMetadata {
            version: 1, // Use a hardcoded version since State doesn't have a version field
            created_at: state.created_at,
            updated_at: Utc::now(),
            checksum: self.calculate_checksum(&state)?,
        };

        let persistent_state = PersistentState {
            state: state.clone(),
            metadata,
        };

        // Serialize state
        let state_json = serde_json::to_string_pretty(&persistent_state)?;

        // Ensure directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write state to file
        fs::write(&state_path, state_json).await?;

        // Update in-memory cache
        self.states.insert(state.id.clone(), persistent_state);

        info!(state_id = %state.id, "State persisted to storage");
        Ok(())
    }

    /// Loads a state from persistent storage
    ///
    /// # Parameters
    ///
    /// * `state_id` - The ID of the state to load
    ///
    /// # Returns
    ///
    /// A Result containing the loaded state or an error
    #[instrument(skip(self))]
    pub async fn load_state(&mut self, state_id: &str) -> Result<State, PersistenceError> {
        let state_path = self.get_state_path(state_id);

        // Try to read from file
        let state_json = fs::read_to_string(&state_path).await
            .map_err(|_| PersistenceError::NotFound(state_id.to_string()))?;

        // Deserialize state
        let persistent_state: PersistentState = serde_json::from_str(&state_json)?;

        // Validate checksum
        let calculated_checksum = self.calculate_checksum(&persistent_state.state)?;
        if calculated_checksum != persistent_state.metadata.checksum {
            return Err(PersistenceError::InvalidData(
                format!("Checksum mismatch for state: {state_id}")
            ));
        }

        // Update in-memory cache
        self.states.insert(state_id.to_string(), persistent_state.clone());

        info!(state_id = %state_id, "State loaded from storage");
        Ok(persistent_state.state)
    }

    /// Deletes a state from persistent storage
    ///
    /// # Parameters
    ///
    /// * `state_id` - The ID of the state to delete
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    #[instrument(skip(self))]
    pub async fn delete_state(&mut self, state_id: &str) -> Result<(), PersistenceError> {
        let state_path = self.get_state_path(state_id);

        // Remove file if it exists
        if state_path.exists() {
            fs::remove_file(&state_path).await?;
        }

        // Remove from in-memory cache
        self.states.remove(state_id);

        info!(state_id = %state_id, "State deleted from storage");
        Ok(())
    }

    /// Lists all states in persistent storage
    ///
    /// # Returns
    ///
    /// A Result containing a vector of state IDs or an error
    #[instrument(skip(self))]
    pub async fn list_states(&self) -> Result<Vec<String>, PersistenceError> {
        let mut states = Vec::new();

        let mut entries = fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    states.push(file_name[..file_name.len() - 5].to_string());
                }
            }
        }

        Ok(states)
    }

    /// Gets the file path for a state
    ///
    /// # Parameters
    ///
    /// * `state_id` - The ID of the state
    ///
    /// # Returns
    ///
    /// The path where the state is stored
    fn get_state_path(&self, state_id: &str) -> PathBuf {
        self.storage_path.join(format!("{state_id}.json"))
    }

    /// Calculates a checksum for a state
    ///
    /// # Parameters
    ///
    /// * `state` - The state to calculate a checksum for
    ///
    /// # Returns
    ///
    /// A Result containing the checksum as a string or an error
    fn calculate_checksum(&self, state: &State) -> Result<String, PersistenceError> {
        let state_json = serde_json::to_string(state)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(state_json.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }
} 