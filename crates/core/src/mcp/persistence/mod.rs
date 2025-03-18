use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::error::{MCPError, Result};
use crate::mcp::types::{ProtocolVersion};
use tokio::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::mcp::context_manager::Context;
use crate::mcp::sync::StateChange;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub data_dir: PathBuf,
    pub max_file_size: usize,
    pub auto_compact_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub version: ProtocolVersion,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    pub contexts: Vec<Context>,
    pub changes: Vec<StateChange>,
    pub last_version: u64,
    pub last_sync: DateTime<Utc>,
}

/// Persistence layer for MCP
#[derive(Debug)]
pub struct MCPPersistence {
    config: PersistenceConfig,
    metadata: Arc<RwLock<StorageMetadata>>,
    state: Arc<RwLock<PersistentState>>,
}

impl MCPPersistence {
    pub fn new(config: PersistenceConfig) -> Self {
        let metadata = StorageMetadata {
            version: ProtocolVersion::default(),
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            size: 0,
        };

        Self {
            config,
            metadata: Arc::new(RwLock::new(metadata)),
            state: Arc::new(RwLock::new(PersistentState {
                contexts: Vec::new(),
                changes: Vec::new(),
                last_version: 0,
                last_sync: Utc::now(),
            })),
        }
    }

    pub async fn init(&self) -> Result<()> {
        // Create data directory if it doesn't exist
        if !self.config.data_dir.exists() {
            fs::create_dir_all(&self.config.data_dir).await?;
        }
        Ok(())
    }

    pub async fn save_state(&self, state: &PersistentState) -> Result<()> {
        let state_path = self.config.data_dir.join("state.json");
        let temp_path = self.config.data_dir.join("state.json.tmp");

        // Write to temporary file first
        let state_json = serde_json::to_string_pretty(state)?;
        fs::write(&temp_path, state_json).await?;

        // Atomically replace old state file
        fs::rename(&temp_path, &state_path).await?;

        // Cleanup old changes if needed
        self.compact_if_needed().await?;

        Ok(())
    }

    pub async fn load_state(&self) -> Result<Option<PersistentState>> {
        let state_path = self.config.data_dir.join("state.json");

        if !state_path.exists() {
            return Ok(None);
        }

        let state_json = fs::read_to_string(&state_path).await?;
        let state = serde_json::from_str(&state_json)?;
        Ok(Some(state))
    }

    pub async fn save_change(&self, change: &StateChange) -> Result<()> {
        let change_path = self.get_change_path(change.id);
        let change_json = serde_json::to_string_pretty(change)?;
        fs::write(&change_path, change_json).await?;
        Ok(())
    }

    pub async fn load_changes(&self) -> Result<Vec<StateChange>> {
        let mut changes = Vec::new();
        let mut entries = fs::read_dir(&self.config.data_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "change") {
                let change_json = fs::read_to_string(&path).await?;
                let change: StateChange = serde_json::from_str(&change_json)?;
                changes.push(change);
            }
        }

        // Sort changes by version
        changes.sort_by_key(|c| c.version);
        Ok(changes)
    }

    async fn compact_if_needed(&self) -> Result<()> {
        let mut size = 0;
        let mut entries = fs::read_dir(&self.config.data_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "change") {
                size += entry.metadata().await?.len() as usize;
            }
        }

        if size > self.config.auto_compact_threshold {
            self.compact_changes().await?;
        }

        Ok(())
    }

    async fn compact_changes(&self) -> Result<()> {
        let changes = self.load_changes().await?;
        let mut entries = fs::read_dir(&self.config.data_dir).await?;

        // Remove old change files
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().map_or(false, |ext| ext == "change") {
                fs::remove_file(entry.path()).await?;
            }
        }

        // Save only the most recent changes
        for change in changes.iter().rev().take(100) {
            self.save_change(change).await?;
        }

        Ok(())
    }

    fn get_change_path(&self, id: Uuid) -> PathBuf {
        self.config.data_dir.join(format!("{}.change", id))
    }

    pub async fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        // TODO: Implement actual persistence
        let mut metadata = self.metadata.write().await;
        metadata.last_modified = chrono::Utc::now();
        metadata.size += data.len() as u64;
        Ok(())
    }

    pub async fn load(&self, key: &str) -> Result<Vec<u8>> {
        // TODO: Implement actual loading
        Ok(Vec::new())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        // TODO: Implement actual deletion
        Ok(())
    }

    pub async fn update_config(&self, config: PersistenceConfig) -> Result<()> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }

    pub async fn get_config(&self) -> Result<PersistenceConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    pub async fn get_metadata(&self) -> Result<StorageMetadata> {
        let metadata = self.metadata.read().await;
        Ok(metadata.clone())
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data/mcp"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            auto_compact_threshold: 100 * 1024 * 1024, // 100MB
        }
    }
}

impl Default for MCPPersistence {
    fn default() -> Self {
        Self::new(PersistenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_persistence_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let config = PersistenceConfig {
            data_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024,
            auto_compact_threshold: 4096,
        };

        let persistence = MCPPersistence::new(config);
        assert!(persistence.init().await.is_ok());

        // Create test state
        let state = PersistentState {
            contexts: vec![Context {
                id: Uuid::new_v4(),
                name: "test".to_string(),
                data: serde_json::json!({}),
                metadata: None,
                parent_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                expires_at: None,
            }],
            changes: vec![],
            last_version: 1,
            last_sync: Utc::now(),
        };

        // Save state
        assert!(persistence.save_state(&state).await.is_ok());

        // Load state
        let loaded = persistence.load_state().await.unwrap().unwrap();
        assert_eq!(loaded.contexts.len(), 1);
        assert_eq!(loaded.contexts[0].name, "test");
    }

    #[tokio::test]
    async fn test_change_persistence() {
        let temp_dir = tempdir().unwrap();
        let config = PersistenceConfig {
            data_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024,
            auto_compact_threshold: 4096,
        };

        let persistence = MCPPersistence::new(config);
        assert!(persistence.init().await.is_ok());

        // Create test change
        let change = StateChange {
            id: Uuid::new_v4(),
            context_id: Uuid::new_v4(),
            operation: crate::mcp::sync::StateOperation::Create,
            data: serde_json::json!({}),
            timestamp: Utc::now(),
            version: 1,
        };

        // Save change
        assert!(persistence.save_change(&change).await.is_ok());

        // Load changes
        let changes = persistence.load_changes().await.unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].id, change.id);
    }
} 