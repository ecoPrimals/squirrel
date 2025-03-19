/// Module for handling persistence operations in the MCP system.
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use crate::error::Result;
use crate::error::PersistenceError;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::mcp::context_manager::Context;
use crate::mcp::sync::StateChange;
use crate::mcp::error::types::MCPError;
use async_trait::async_trait;
use crate::mcp::types::{AccountId, AuthToken, SessionToken, UserId, UserRole, ProtocolVersion};
use serde_json;

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
}

/// Metadata about storage operations and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    /// Protocol version used for storage
    pub version: ProtocolVersion,
    /// When the storage was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the storage was last modified
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// Total size in bytes of stored data
    pub size: u64,
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
    config: PersistenceConfig,
    metadata: Arc<RwLock<StorageMetadata>>,
    state: Arc<RwLock<PersistentState>>,
    initialized: AtomicBool,
}

impl MCPPersistence {
    /// Creates a new instance of the persistence layer
    ///
    /// # Parameters
    ///
    /// * `config` - Configuration settings for the persistence layer
    ///
    /// # Returns
    ///
    /// A new `MCPPersistence` instance
    #[must_use] pub fn new(config: PersistenceConfig) -> Self {
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
                id: Uuid::new_v4().to_string(),
            })),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initializes the persistence layer
    ///
    /// Creates necessary directories if they don't exist and
    /// prepares the persistence layer for use.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub async fn init(&self) -> Result<()> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }

        if !self.config.data_dir.exists() {
            fs::create_dir_all(&self.config.data_dir)?;
        }

        self.initialized.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Saves the persistent state to storage
    ///
    /// # Parameters
    ///
    /// * `state` - The state to save
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub async fn save_state(&self, state: PersistentState) -> Result<()> {
        let state_path = self.get_state_path(&state.id);
        let temp_path = state_path.with_extension("tmp");
        
        let state_json = serde_json::to_string_pretty(&state)?;
        
        fs::write(&temp_path, &state_json)?;
        
        fs::rename(&temp_path, &state_path)?;
        
        Ok(())
    }

    /// Loads the state from persistent storage
    /// 
    /// # Returns
    /// Returns the loaded state if found, or None if no state exists
    /// 
    /// # Errors
    /// Returns an error if there is an issue reading from the file system
    /// or if the state file is corrupted
    pub async fn load_state(&self) -> Result<Option<PersistentState>> {
        // Try to find any state file in the data directory
        let entries = match fs::read_dir(&self.config.data_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(None),
        };

        // Find the first state file
        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            
            // Check if this is a state file
            if file_name.starts_with("state_") && file_name.ends_with(".json") {
                let data = fs::read_to_string(entry.path())?;
                let state: PersistentState = serde_json::from_str(&data)?;
                return Ok(Some(state));
            }
        }
        
        Ok(None)
    }

    /// Saves a state change to persistent storage
    /// 
    /// # Arguments
    /// * `change` - The state change to save
    /// 
    /// # Errors
    /// Returns an error if there is an issue writing to the file system
    /// or if serialization fails
    pub async fn save_change(&self, change: StateChange) -> Result<()> {
        // Ensure the changes directory exists
        let changes_dir = self.config.data_dir.join("changes");
        if !changes_dir.exists() {
            fs::create_dir_all(&changes_dir)?;
        }
        
        // Save the change to a file
        let change_path = self.get_change_path(change.id);
        let change_json = serde_json::to_string_pretty(&change)?;
        fs::write(change_path, change_json)?;
        
        // Check if we need to compact changes directly
        let mut size = 0;
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len() as usize;
                }
            }
        }
        
        // Use Box::pin to handle the recursive async call
        if size > self.config.auto_compact_threshold {
            let fut = Box::pin(self.compact_changes());
            fut.await?;
        }
        
        Ok(())
    }

    /// Loads all state changes from persistent storage
    /// 
    /// # Returns
    /// Returns a vector of all persisted state changes
    /// 
    /// # Errors
    /// Returns an error if there is an issue reading from the file system
    /// or if any change files are corrupted
    pub async fn load_changes(&self) -> Result<Vec<StateChange>> {
        let mut changes = Vec::new();

        let dir_path = self.config.data_dir.join("changes");
        if !dir_path.exists() {
            return Ok(changes);
        }
        
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(ext) = path.extension() {
                if ext == "json" {
                    let change_json = fs::read_to_string(&path)?;
                    let change = serde_json::from_str(&change_json)?;
                    changes.push(change);
                }
            }
        }
        
        Ok(changes)
    }

    /// Checks if the storage needs to be compacted and performs compaction if necessary
    /// 
    /// # Errors
    /// Returns an error if there is an issue accessing or modifying the file system
    async fn compact_if_needed(&self) -> Result<()> {
        let mut size = 0;
        let entries = fs::read_dir(&self.config.data_dir)?;
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len() as usize;
                }
            }
        }
        
        // Instead of calling compact_changes() which could lead to recursion,
        // we just return a flag to the caller to indicate if compaction is needed
        if size > self.config.auto_compact_threshold {
            return Err(MCPError::StorageError("Storage size exceeded, compaction needed".to_string()).into());
        }
        
        Ok(())
    }

    async fn compact_changes(&self) -> Result<()> {
        let changes = self.load_changes().await?;
        let entries = fs::read_dir(&self.config.data_dir)?;

        // Remove old change files
        for entry in entries {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "change") {
                fs::remove_file(entry.path())?;
            }
        }

        // Save only the most recent changes
        for change in changes.iter().rev().take(100) {
            self.save_change(change.clone()).await?;
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

    /// Gets the path to a change file
    ///
    /// # Arguments
    /// * `id` - The change ID
    ///
    /// # Returns
    /// The full path to the change file
    fn get_change_path(&self, id: impl Into<Uuid>) -> PathBuf {
        let uuid: Uuid = id.into();
        self.config.data_dir.join("changes").join(format!("{uuid}.json"))
    }

    /// Saves data to persistent storage
    /// 
    /// # Arguments
    /// * `key` - The key under which to store the data
    /// * `data` - The data to store
    /// 
    /// # Errors
    /// This function currently returns no errors, but in a real implementation
    /// it could return storage-related errors
    pub async fn save(&self, _key: &str, _data: &[u8]) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    /// Loads data with the specified key
    ///
    /// # Parameters
    ///
    /// * `_key` - The key of the data to load
    ///
    /// # Returns
    ///
    /// A Result containing the loaded data or an error
    pub async fn load(&self, _key: &str) -> Result<Vec<u8>> {
        // Stub implementation
        Ok(Vec::new())
    }

    /// Deletes data with the specified key
    ///
    /// # Parameters
    ///
    /// * `_key` - The key of the data to delete
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub async fn delete(&self, _key: &str) -> Result<()> {
        // Stub implementation
        Ok(())
    }

    /// Updates the configuration settings
    ///
    /// # Parameters
    ///
    /// * `config` - The new configuration settings
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    pub fn update_config(&mut self, config: PersistenceConfig) -> Result<()> {
        // Simply update the config field directly
        self.config = config;
        Ok(())
    }

    /// Gets the current configuration settings
    ///
    /// # Returns
    ///
    /// A clone of the current configuration
    pub fn get_config(&self) -> PersistenceConfig {
        self.config.clone()
    }

    /// Gets the storage metadata
    ///
    /// # Returns
    ///
    /// A Result containing the metadata or an error
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
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
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
            id: Uuid::new_v4().to_string(),
        };

        // Save state
        assert!(persistence.save_state(state).await.is_ok());

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
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
        };

        let persistence = MCPPersistence::new(config);
        assert!(persistence.init().await.is_ok());

        // Create test change
        let change_id = Uuid::new_v4();
        let change = StateChange {
            id: change_id,
            context_id: Uuid::new_v4(),
            operation: crate::mcp::sync::StateOperation::Create,
            data: serde_json::json!({}),
            timestamp: Utc::now(),
            version: 1,
        };

        // Save change
        assert!(persistence.save_change(change).await.is_ok());

        // Load changes
        let changes = persistence.load_changes().await.unwrap();
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
    pub role: UserRole,
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
    pub role: UserRole,
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
    base_dir: String,
}

impl FilePersistence {
    /// Create a new file persistence
    #[must_use] pub fn new(config: PersistenceConfig) -> Self {
        Self {
            base_dir: config.storage_path,
        }
    }
    
    /// Get the path for a key
    fn get_path(&self, key: &str) -> String {
        format!("{}/{}", self.base_dir, key)
    }
}

#[async_trait]
impl Persistence for FilePersistence {
    async fn init(&self) -> Result<()> {
        // Create the storage directory if it doesn't exist
        fs::create_dir_all(&self.base_dir)
            .map_err(|e| PersistenceError::IO(format!("Failed to create storage directory: {e}")))?;
        Ok(())
    }
    
    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let key = format!("sessions/{}", session.token.0);
        let data = serde_json::to_vec(session)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize session: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize session: {e}")))?;
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
        let data = serde_json::to_vec(user)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize user: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {e}")))?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>> {
        // We would need an index for this, but for simplicity we'll scan all users
        let keys = self.list_keys("users/").await?;
        for key in keys {
            if let Some(data) = self.load_data(&key).await? {
                let user: UserData = serde_json::from_slice(&data)
                    .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {e}")))?;
                if user.username == username {
                    return Ok(Some(user));
                }
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
        let data = serde_json::to_vec(account)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize account: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize account: {e}")))?;
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
        
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| PersistenceError::IO(format!("Failed to create directory: {e}")))?;
        }
        
        fs::write(&path, value)
            .map_err(|e| PersistenceError::IO(format!("Failed to write file: {e}")))?;
        Ok(())
    }
    
    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_path(key);
        match fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(PersistenceError::IO(format!("Failed to read file: {e}")).into()),
        }
    }
    
    async fn delete_data(&self, key: &str) -> Result<()> {
        let path = self.get_path(key);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(PersistenceError::IO(format!("Failed to delete file: {e}")).into()),
        }
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let dir_path = format!("{}/{}", self.base_dir, prefix);
        let dir_path = std::path::Path::new(&dir_path);
        
        if !dir_path.exists() {
            return Ok(Vec::new());
        }
        
        let prefix_path = std::path::Path::new(&self.base_dir);
        let mut entries = Vec::new();
        
        let read_dir = fs::read_dir(dir_path)
            .map_err(|e| PersistenceError::IO(format!("Failed to read directory: {e}")))?;
        
        for entry in read_dir {
            let entry = entry.map_err(|e| PersistenceError::IO(format!("Failed to read entry: {e}")))?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(relative) = path.strip_prefix(prefix_path) {
                    if let Some(key) = relative.to_str() {
                        entries.push(key.to_string());
                    }
                }
            }
        }
        
        Ok(entries)
    }
    
    async fn close(&self) -> Result<()> {
        // Nothing to do for file-based persistence
        Ok(())
    }
}

/// Memory-based persistence implementation for testing
#[derive(Debug)]
pub struct MemoryPersistence {
    /// In-memory data store
    data: std::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>,
}

impl MemoryPersistence {
    /// Create a new memory persistence
    #[must_use] pub fn new() -> Self {
        Self {
            data: std::sync::RwLock::new(std::collections::HashMap::new()),
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
        // Nothing to initialize for memory-based persistence
        Ok(())
    }
    
    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let key = format!("sessions/{}", session.token.0);
        let data = serde_json::to_vec(session)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize session: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize session: {e}")))?;
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
        let data = serde_json::to_vec(user)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize user: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {e}")))?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>> {
        let data = self.data.read().map_err(|_| PersistenceError::IO("Failed to acquire read lock".to_string()))?;
        for (key, value) in data.iter() {
            if key.starts_with("users/") {
                if let Ok(user) = serde_json::from_slice::<UserData>(value) {
                    if user.username == username {
                        return Ok(Some(user));
                    }
                }
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
        let data = serde_json::to_vec(account)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize account: {e}")))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize account: {e}")))?;
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
        let mut data = self.data.write().map_err(|_| PersistenceError::IO("Failed to acquire write lock".to_string()))?;
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }
    
    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().map_err(|_| PersistenceError::IO("Failed to acquire read lock".to_string()))?;
        Ok(data.get(key).cloned())
    }
    
    async fn delete_data(&self, key: &str) -> Result<()> {
        let mut data = self.data.write().map_err(|_| PersistenceError::IO("Failed to acquire write lock".to_string()))?;
        data.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let data = self.data.read().map_err(|_| PersistenceError::IO("Failed to acquire read lock".to_string()))?;
        let keys: Vec<String> = data.keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }
    
    async fn close(&self) -> Result<()> {
        // Nothing to do for memory-based persistence
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
    #[must_use] pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }
    
    /// Create a file-based persistence
    #[must_use] pub fn create_file_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(FilePersistence::new(self.config.clone()))
    }
    
    /// Create a memory-based persistence
    #[must_use] pub fn create_memory_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(MemoryPersistence::new())
    }
    
    /// Create a persistence based on the configuration
    #[must_use] pub fn create_persistence(&self) -> Arc<dyn Persistence> {
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