use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{error, info, instrument, warn};
use thiserror::Error;

use super::state::{State, StateError};

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("State not found: {0}")]
    NotFound(String),

    #[error("Invalid state data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub state: State,
    pub metadata: PersistentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentMetadata {
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checksum: String,
}

pub struct StatePersistence {
    storage_path: PathBuf,
    states: HashMap<String, PersistentState>,
}

impl StatePersistence {
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            states: HashMap::new(),
        }
    }

    #[instrument(skip(self, state))]
    pub async fn save_state(&mut self, state: State) -> Result<(), PersistenceError> {
        let state_path = self.get_state_path(&state.name);
        
        // Create persistent state
        let metadata = PersistentMetadata {
            version: state.version,
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
        self.states.insert(state.name.clone(), persistent_state);

        info!(state_name = %state.name, "State persisted to storage");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn load_state(&mut self, state_name: &str) -> Result<State, PersistenceError> {
        let state_path = self.get_state_path(state_name);

        // Try to read from file
        let state_json = fs::read_to_string(&state_path).await
            .map_err(|_| PersistenceError::NotFound(state_name.to_string()))?;

        // Deserialize state
        let persistent_state: PersistentState = serde_json::from_str(&state_json)?;

        // Validate checksum
        let calculated_checksum = self.calculate_checksum(&persistent_state.state)?;
        if calculated_checksum != persistent_state.metadata.checksum {
            return Err(PersistenceError::InvalidData(
                format!("Checksum mismatch for state: {}", state_name)
            ));
        }

        // Update in-memory cache
        self.states.insert(state_name.to_string(), persistent_state.clone());

        info!(state_name = %state_name, "State loaded from storage");
        Ok(persistent_state.state)
    }

    #[instrument(skip(self))]
    pub async fn delete_state(&mut self, state_name: &str) -> Result<(), PersistenceError> {
        let state_path = self.get_state_path(state_name);

        // Remove file if it exists
        if state_path.exists() {
            fs::remove_file(&state_path).await?;
        }

        // Remove from in-memory cache
        self.states.remove(state_name);

        info!(state_name = %state_name, "State deleted from storage");
        Ok(())
    }

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

    fn get_state_path(&self, state_name: &str) -> PathBuf {
        self.storage_path.join(format!("{}.json", state_name))
    }

    fn calculate_checksum(&self, state: &State) -> Result<String, PersistenceError> {
        let state_json = serde_json::to_string(state)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(state_json.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }
} 