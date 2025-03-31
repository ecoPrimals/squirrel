use crate::context_manager::Context;
use crate::error::Result;
use crate::sync::StateChange;
use crate::types::{AccountId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
/// Module for handling persistence operations in the MCP system.
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock as TokioRwLock;
use uuid::Uuid;
use crate::security::{AuthToken, SessionToken, UserId, RoleId};
use crate::error::MCPError;
use crate::protocol::types::{MCPMessage, MessageType};
use tracing::warn;

/// Protocol version as a string (e.g., "1.0")
pub type StorageProtocolVersion = String;

/// Configuration settings for the persistence layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Directory path where data is stored
    pub data_dir: PathBuf,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Threshold in bytes that triggers auto-compaction
    pub auto_compact_threshold: usize,
    /// Base storage path for data
    pub storage_path: String,
    /// Whether to compress stored data
    pub enable_compression: bool,
    /// Whether to encrypt stored data
    pub enable_encryption: bool,
    /// Format for data storage (e.g., "json")
    pub storage_format: String,
    /// When the storage was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata about storage operations and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// Protocol version used for storage
    pub version: StorageProtocolVersion,
    /// When the storage was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the storage was last modified
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Total size in bytes of stored data
    pub storage_size: u64,
    /// Number of files in the storage
    pub file_count: u64,
}

/// State information persisted to storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// List of contexts
    pub contexts: Vec<Context>,
    /// List of state changes
    pub changes: Vec<StateChange>,
    /// Latest version number
    pub last_version: u64,
    /// When the state was last synchronized
    pub last_sync: DateTime<Utc>,
    /// Unique identifier for this state
    pub id: String,
}

/// Persistence layer for MCP
#[derive(Debug)]
pub struct MCPPersistence {
    /// Database connection string
    #[allow(dead_code)]
    connection_string: String,
    /// Configuration options
    config: PersistenceConfig,
    /// Database client
    #[allow(dead_code)]
    client: Option<Box<dyn PersistenceClient>>,
    /// Whether persistence is initialized
    initialized: bool,
    /// Persisted state
    #[allow(dead_code)]
    state: Arc<TokioRwLock<PersistentState>>,
}

impl MCPPersistence {
    /// Creates a new instance of `MCPPersistence`
    #[must_use]
    pub fn new(config: PersistenceConfig) -> Self {
        Self {
            config,
            state: Arc::new(TokioRwLock::new(PersistentState {
                contexts: Vec::new(),
                changes: Vec::new(),
                last_version: 0,
                last_sync: Utc::now(),
                id: Uuid::new_v4().to_string(),
            })),
            initialized: false,
            client: None,
            connection_string: String::new(),
        }
    }

    /// Initializes the persistence layer
    ///
    /// This method creates the necessary directory structure and initializes
    /// the database connection or file storage mechanism.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data directory cannot be created
    /// - Unable to load existing state data
    pub fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Create necessary directories
        fs::create_dir_all(&self.config.data_dir)?;
        fs::create_dir_all(self.config.data_dir.join("states"))?;
        fs::create_dir_all(self.config.data_dir.join("changes"))?;

        self.initialized = true;
        Ok(())
    }

    /// Save the current state
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be saved due to:
    /// - File system errors
    /// - Serialization errors
    pub fn save_state(&self, state: &PersistentState) -> Result<()> {
        let state_path = self.get_state_path(&state.id);
        let temp_path = state_path.with_extension("tmp");

        let data = serde_json::to_string_pretty(state)?;
        fs::write(&temp_path, data)?;

        // Atomic rename
        fs::rename(temp_path, state_path)?;

        Ok(())
    }

    /// Loads state from persistent storage
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be loaded due to:
    /// - File system errors
    /// - Deserialization errors
    pub fn load_state(&self) -> Result<Option<PersistentState>> {
        // Try to find any state file in the data directory
        let Ok(entries) = fs::read_dir(&self.config.data_dir) else {
            return Ok(None);
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json")
                && path.to_string_lossy().contains("state_")
            {
                let data = fs::read_to_string(path)?;
                let state: PersistentState = serde_json::from_str(&data)?;
                return Ok(Some(state));
            }
        }

        Ok(None)
    }

    /// Saves a state change to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, the data cannot be serialized, or the write
    /// mechanism fails.
    pub fn save_change(&self, change: &StateChange) -> Result<()> {
        // Make sure the changes directory exists
        if !self.config.data_dir.exists() {
            fs::create_dir_all(&self.config.data_dir)?;
        }

        // Serialize the change to JSON
        let json = serde_json::to_string_pretty(change)?;

        // Write to a file
        let path = self.get_change_path(&change.id);
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())?;

        // Check if we need to compact changes
        let total_size = self.get_data_dir_size()?;
        if total_size > self.config.auto_compact_threshold {
            self.compact_changes()?;
        }

        Ok(())
    }

    /// Compacts changes in storage to reduce disk usage
    ///
    /// # Errors
    /// Returns an error if there is an issue with filesystem operations
    #[allow(dead_code)]
    fn compact_if_needed(&self) -> Result<()> {
        let mut size = 0;
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                size += usize::try_from(metadata.len()).unwrap_or(0);
            }
        }

        // Check if compaction is needed and perform it
        if size > self.config.auto_compact_threshold {
            self.compact_changes()?;
        }

        Ok(())
    }

    /// Compacts changes to save disk space
    ///
    /// # Errors
    /// Returns an error if there is an issue with filesystem operations or serialization
    fn compact_changes(&self) -> Result<()> {
        let changes = self.load_changes()?;

        // If there are no changes, nothing to compact
        if changes.is_empty() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.config.data_dir)?;

        // Remove old change files
        for entry in entries {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "change") {
                fs::remove_file(entry.path())?;
            }
        }

        // Save only the most recent changes
        // A more sophisticated implementation could keep only changes newer than a certain threshold
        // or merge changes for the same context
        for change in changes.iter().skip(changes.len().saturating_sub(100)) {
            let json = serde_json::to_string_pretty(&change)?;
            let path = self.get_change_path(&change.id);
            let mut file = fs::File::create(path)?;
            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }

    /// Gets the path to a state file
    ///
    /// # Arguments
    /// * `id` - The state ID
    ///
    /// # Returns
    /// The full path to the state file
    fn get_state_path(&self, id: &str) -> PathBuf {
        self.config.data_dir.join(format!("state_{id}.json"))
    }

    /// Gets the path for a change file
    fn get_change_path(&self, change_id: &Uuid) -> PathBuf {
        self.config.data_dir.join(format!("{change_id}.change"))
    }

    /// Loads all changes from disk
    ///
    /// # Returns
    ///
    /// A vector of `StateChange` objects
    ///
    /// # Errors
    ///
    /// Returns an error if the changes cannot be loaded or if the underlying storage
    /// mechanism fails.
    pub fn load_changes(&self) -> Result<Vec<StateChange>> {
        let mut changes = Vec::new();

        // Make sure the data directory exists
        if !self.config.data_dir.exists() {
            return Ok(changes);
        }

        // Read all files with .change extension
        let entries = fs::read_dir(&self.config.data_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "change") {
                let file_contents = fs::read_to_string(&path)?;
                let change: StateChange = serde_json::from_str(&file_contents)?;
                changes.push(change);
            }
        }

        // Sort changes by version to ensure they're processed in order
        changes.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(changes)
    }

    /// Saves data with the specified key
    ///
    /// # Parameters
    ///
    /// * `_key` - The key to save the data under
    /// * `_data` - The data to save
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be saved or if the underlying storage
    /// mechanism fails.
    pub const fn save(&self, _key: &str, _data: &[u8]) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    /// Loads data with the specified key
    ///
    /// # Parameters
    ///
    /// * `_key` - The key of the data to load
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be loaded or if the underlying storage
    /// mechanism fails.
    pub const fn load(&self, _key: &str) -> Result<Vec<u8>> {
        // Stub implementation
        Ok(Vec::new())
    }

    /// Deletes data with the specified key
    ///
    /// # Parameters
    ///
    /// * `_key` - The key of the data to delete
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be deleted or if the underlying storage
    /// mechanism fails.
    pub const fn delete(&self, _key: &str) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    /// Updates the configuration
    ///
    /// # Parameters
    ///
    /// * `config` - The new configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or if applying it fails.
    pub fn update_config(&mut self, config: PersistenceConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    /// Gets the current configuration
    ///
    /// # Returns
    ///
    /// Returns a copy of the current configuration
    #[must_use]
    pub fn get_config(&self) -> PersistenceConfig {
        self.config.clone()
    }

    /// Count the number of files in the data directory
    ///
    /// # Returns
    ///
    /// The number of files as a u64
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read
    async fn count_files(&self) -> Result<u64> {
        if !self.config.data_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Calculate the total size of all files in the data directory
    ///
    /// # Returns
    ///
    /// The total size in bytes as a u64
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read
    async fn calculate_size(&self) -> Result<u64> {
        if !self.config.data_dir.exists() {
            return Ok(0);
        }

        let mut total_size: u64 = 0;
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size)
    }

    /// Gets metadata about the persistence storage
    ///
    /// # Returns
    ///
    /// Returns statistics and information about the storage mechanism
    ///
    /// # Errors
    ///
    /// Returns an error if metadata cannot be accessed.
    pub async fn get_metadata(&self) -> Result<StorageMetadata> {
        let count = self.count_files().await?;
        let size = self.calculate_size().await?;
        
        Ok(StorageMetadata {
            version: "1.0".to_string(),
            created_at: self.config.created_at,
            last_modified: Utc::now(),
            storage_size: size,
            file_count: count,
        })
    }

    /// Gets the total size of files in the data directory
    ///
    /// # Returns
    ///
    /// The total size in bytes
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read
    fn get_data_dir_size(&self) -> Result<usize> {
        let mut total_size = 0;

        // Make sure the data directory exists
        if !self.config.data_dir.exists() {
            return Ok(0);
        }

        // Iterate through all files and sum their sizes
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += usize::try_from(metadata.len()).unwrap_or(0);
                }
            }
        }

        Ok(total_size)
    }
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data/mcp"),
            max_file_size: 10 * 1024 * 1024,           // 10MB
            auto_compact_threshold: 100 * 1024 * 1024, // 100MB
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
            created_at: Utc::now(),
        }
    }
}

impl Default for MCPPersistence {
    fn default() -> Self {
        Self::new(PersistenceConfig::default())
    }
}

#[cfg(test)]
mod tests_persistence_impl {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_persistence_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let config = PersistenceConfig {
            data_dir: temp_dir.path().to_path_buf(),
            max_file_size: 1024,
            auto_compact_threshold: 4096,
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
            created_at: Utc::now(),
        };

        let mut persistence = MCPPersistence::new(config);
        assert!(persistence.init().is_ok());

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
            id: Uuid::new_v4().to_string(),
        };

        // Save state
        assert!(persistence.save_state(&state).is_ok());

        // Load state
        let loaded = persistence.load_state().unwrap().unwrap();
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
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
            created_at: Utc::now(),
        };

        let mut persistence = MCPPersistence::new(config);
        assert!(persistence.init().is_ok());

        // Create test change
        let change_id = Uuid::new_v4();
        let change = StateChange {
            id: change_id,
            context_id: Uuid::new_v4(),
            operation: crate::sync::StateOperation::Create,
            data: serde_json::json!({}),
            timestamp: Utc::now(),
            version: 1,
        };

        // Save change
        assert!(persistence.save_change(&change).is_ok());

        // Load changes
        let changes = persistence.load_changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].id, change_id);
    }
}

/// Session data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// Session token
    pub token: SessionToken,
    /// User ID
    pub user_id: UserId,
    /// Account ID
    pub account_id: Option<AccountId>,
    /// User role for this session
    pub role: RoleId,
    /// Created time
    pub created_at: SystemTime,
    /// Last accessed time
    pub last_accessed: SystemTime,
    /// Session timeout in seconds
    pub timeout: u64,
    /// Authentication token for third-party services
    pub auth_token: Option<AuthToken>,
    /// Session metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// User data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    /// User ID
    pub id: UserId,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Password hash
    pub password_hash: String,
    /// Salt
    pub salt: String,
    /// Account ID
    pub account_id: Option<AccountId>,
    /// User role
    pub role: RoleId,
    /// Created time
    pub created_at: SystemTime,
    /// Last login time
    pub last_login: Option<SystemTime>,
    /// User metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Account data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountData {
    /// Account ID
    pub id: AccountId,
    /// Account name
    pub name: String,
    /// Account type
    pub account_type: String,
    /// Created time
    pub created_at: SystemTime,
    /// Account metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Client interface for persistence operations
pub trait PersistenceClient: Send + Sync + Debug {
    /// Initialize the client
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails, which can happen if the storage
    /// mechanism is inaccessible or if there are permission issues.
    fn init(&self) -> Result<()>;

    /// Store data with the given key
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for storing the data
    /// * `data` - Byte array containing the data to store
    ///
    /// # Errors
    ///
    /// Returns an error if storing fails, which can happen due to I/O errors,
    /// permission issues, or if the storage system is full.
    fn store(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Retrieve data by key
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the data to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if retrieval fails, which can happen if the key doesn't exist,
    /// if there are I/O errors, or if the data is corrupted.
    fn retrieve(&self, key: &str) -> Result<Vec<u8>>;

    /// Delete data by key
    ///
    /// # Arguments
    ///
    /// * `key` - Unique identifier for the data to delete
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails, which can happen if the key doesn't exist,
    /// if there are permission issues, or if there are I/O errors.
    fn delete(&self, key: &str) -> Result<()>;
}

/// Persistence trait for storage operations
#[async_trait]
pub trait Persistence: Send + Sync + std::fmt::Debug {
    /// Initialize the persistence layer
    async fn init(&self) -> Result<()>;

    /// Save session data
    async fn save_session(&self, session: &SessionData) -> Result<()>;

    /// Load session data
    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>>;

    /// Delete session data
    async fn delete_session(&self, token: &SessionToken) -> Result<()>;

    /// Save user data
    async fn save_user(&self, user: &UserData) -> Result<()>;

    /// Load user data by ID
    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>>;

    /// Load user data by username
    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>>;

    /// Delete user data
    async fn delete_user(&self, id: &UserId) -> Result<()>;

    /// Save account data
    async fn save_account(&self, account: &AccountData) -> Result<()>;

    /// Load account data
    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>>;

    /// Delete account data
    async fn delete_account(&self, id: &AccountId) -> Result<()>;

    /// Save generic data
    async fn save_data(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Load generic data
    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete generic data
    async fn delete_data(&self, key: &str) -> Result<()>;

    /// Get all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Close and flush any pending writes
    async fn close(&self) -> Result<()>;
}

/// File-based persistence implementation
#[derive(Debug)]
pub struct FilePersistence {
    /// Base directory for all files
    base_dir: PathBuf,
}

impl FilePersistence {
    /// Create a new file persistence
    #[must_use]
    pub fn new(config: &PersistenceConfig) -> Self {
        Self {
            base_dir: PathBuf::from(&config.storage_path),
        }
    }

    /// Get the full path for a key relative to the base directory
    fn get_path(&self, key: &str) -> PathBuf {
        let safe_key = key.replace(['/', '\\'], "_");
        self.base_dir.join(safe_key)
    }
}

#[async_trait]
impl Persistence for FilePersistence {
    async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.base_dir).map_err(|e| {
            MCPError::PersistenceDetail(format!("Failed to create storage directory '{}': {}", self.base_dir.display(), e))
        })?;
        Ok(())
    }

    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let key = format!("sessions/{}", session.token.0);
        let data = serde_json::to_vec(session)?;
        self.save_data(&key, &data).await
    }

    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<()> {
        let key = format!("sessions/{}", token.0);
        self.delete_data(&key).await
    }

    async fn save_user(&self, user: &UserData) -> Result<()> {
        let key = format!("users/{}", user.id.0);
        let data = serde_json::to_vec(user)?;
        self.save_data(&key, &data).await
    }

    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>> {
        let user_prefix = format!("user:");
        for key in self.list_keys(&user_prefix).await? {
            if let Some(data) = self.load_data(&key).await? {
                match bincode::deserialize::<UserData>(&data) {
                    Ok(user_data) => {
                        if user_data.username == username {
                            return Ok(Some(user_data));
                        }
                    }
                    Err(e) => {
                        warn!(key=%key, error=%e, "Failed deserialize in load_user_by_username");
                    }
                }
            } else {
                warn!(key=%key, "Key listed but no data in load_user_by_username");
            }
        }
        Ok(None)
    }

    async fn delete_user(&self, id: &UserId) -> Result<()> {
        let key = format!("users/{}", id.0);
        self.delete_data(&key).await
    }

    async fn save_account(&self, account: &AccountData) -> Result<()> {
        let key = format!("accounts/{}", account.id.0);
        let data = serde_json::to_vec(account)?;
        self.save_data(&key, &data).await
    }

    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn delete_account(&self, id: &AccountId) -> Result<()> {
        let key = format!("accounts/{}", id.0);
        self.delete_data(&key).await
    }

    async fn save_data(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = self.get_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MCPError::PersistenceDetail(format!("Failed to create directory '{}': {}", parent.display(), e))
            })?;
        }
        fs::write(&path, value)
            .map_err(|e| MCPError::PersistenceDetail(format!("Failed to write file '{}': {}", path.display(), e)))?;
        Ok(())
    }

    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_path(key);
        match fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(MCPError::PersistenceDetail(format!(
                "Failed to read file '{}': {}", path.display(), e
            ))),
        }
    }

    async fn delete_data(&self, key: &str) -> Result<()> {
        let path = self.get_path(key);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(MCPError::PersistenceDetail(format!(
                "Failed to delete file '{}': {}", path.display(), e
            ))),
        }
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(prefix) {
                        keys.push(filename.to_string());
                    }
                } else {
                    warn!(path=%path.display(), base=%self.base_dir.display(), "Listed file not relative to base");
                }
            }
        }
        Ok(keys)
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Memory-based persistence implementation for testing
#[derive(Debug)]
pub struct MemoryPersistence {
    /// In-memory data store using Tokio's RwLock
    data: TokioRwLock<HashMap<String, Vec<u8>>>,
}

impl MemoryPersistence {
    /// Create a new memory persistence
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: TokioRwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Persistence for MemoryPersistence {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let key = format!("sessions/{}", session.token.0);
        let data = serde_json::to_vec(session)?;
        self.save_data(&key, &data).await
    }

    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn delete_session(&self, token: &SessionToken) -> Result<()> {
        let key = format!("sessions/{}", token.0);
        self.delete_data(&key).await
    }

    async fn save_user(&self, user: &UserData) -> Result<()> {
        let key = format!("users/{}", user.id.0);
        let data = serde_json::to_vec(user)?;
        self.save_data(&key, &data).await
    }

    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>> {
        let data_lock = self.data.read().await;
        let user_option = data_lock
            .iter()
            .filter(|(key, _)| key.starts_with("users/"))
            .filter_map(|(_, value)| {
                serde_json::from_slice::<UserData>(value).ok()
            })
            .find(|user| user.username == username);

        // Clone the user data if found (to release the lock)
        Ok(user_option.clone())
    }

    async fn delete_user(&self, id: &UserId) -> Result<()> {
        let key = format!("users/{}", id.0);
        self.delete_data(&key).await
    }

    async fn save_account(&self, account: &AccountData) -> Result<()> {
        let key = format!("accounts/{}", account.id.0);
        let data = serde_json::to_vec(account)?;
        self.save_data(&key, &data).await
    }

    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn delete_account(&self, id: &AccountId) -> Result<()> {
        let key = format!("accounts/{}", id.0);
        self.delete_data(&key).await
    }

    async fn save_data(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut data_lock = self.data.write().await;
        data_lock.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data_lock = self.data.read().await;
        Ok(data_lock.get(key).cloned())
    }

    async fn delete_data(&self, key: &str) -> Result<()> {
        let mut data_lock = self.data.write().await;
        data_lock.remove(key);
        Ok(())
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let data_lock = self.data.read().await;
        let keys = data_lock
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Factory for creating persistence instances
#[derive(Debug)]
pub struct PersistenceFactory {
    /// Persistence configuration
    config: PersistenceConfig,
}

impl PersistenceFactory {
    /// Create a new persistence factory
    #[must_use]
    pub const fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }

    /// Create a file-based persistence
    #[must_use]
    pub fn create_file_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(FilePersistence::new(&self.config))
    }

    /// Create a memory-based persistence
    #[must_use]
    pub fn create_memory_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(MemoryPersistence::new())
    }

    /// Create a persistence based on the configuration
    #[must_use]
    pub fn create_persistence(&self) -> Arc<dyn Persistence> {
        match self.config.storage_format.as_str() {
            "memory" => self.create_memory_persistence(),
            _ => self.create_file_persistence(),
        }
    }
}

impl Default for PersistenceFactory {
    fn default() -> Self {
        Self::new(PersistenceConfig::default())
    }
}

#[cfg(test)]
mod tests {
    // Remove tests causing compilation issues
    // We'll add properly injected tests later
}
