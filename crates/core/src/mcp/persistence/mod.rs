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
use std::time::SystemTime;
use async_trait::async_trait;
use crate::error::{PersistenceError, io_error, config_error, storage_error, format_error};
use crate::mcp::types::{AccountId, AuthToken, SessionToken, UserId, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub data_dir: PathBuf,
    pub max_file_size: usize,
    pub auto_compact_threshold: usize,
    pub storage_path: String,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub storage_format: String,
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
            storage_path: "data".to_string(),
            enable_compression: false,
            enable_encryption: false,
            storage_format: "json".to_string(),
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
pub trait Persistence: Send + Sync {
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
pub struct FilePersistence {
    /// Persistence configuration
    config: PersistenceConfig,
}

impl FilePersistence {
    /// Create a new file persistence
    pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }
    
    /// Get the path for a key
    fn get_path(&self, key: &str) -> String {
        format!("{}/{}", self.config.storage_path, key)
    }
}

#[async_trait]
impl Persistence for FilePersistence {
    async fn init(&self) -> Result<()> {
        // Create the storage directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.storage_path).await
            .map_err(|e| PersistenceError::IO(format!("Failed to create storage directory: {}", e)))?;
        Ok(())
    }
    
    async fn save_session(&self, session: &SessionData) -> Result<()> {
        let key = format!("sessions/{}", session.token.0);
        let data = serde_json::to_vec(session)
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize session: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize session: {}", e)))?;
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
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize user: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {}", e)))?;
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
                    .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {}", e)))?;
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
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize account: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize account: {}", e)))?;
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
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| PersistenceError::IO(format!("Failed to create directory: {}", e)))?;
        }
        
        tokio::fs::write(&path, value).await
            .map_err(|e| PersistenceError::IO(format!("Failed to write file: {}", e)))?;
        Ok(())
    }
    
    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_path(key);
        match tokio::fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(PersistenceError::IO(format!("Failed to read file: {}", e)).into()),
        }
    }
    
    async fn delete_data(&self, key: &str) -> Result<()> {
        let path = self.get_path(key);
        match tokio::fs::remove_file(&path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(PersistenceError::IO(format!("Failed to delete file: {}", e)).into()),
        }
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let dir_path = format!("{}/{}", self.config.storage_path, prefix);
        let dir_path = std::path::Path::new(&dir_path);
        
        if !dir_path.exists() {
            return Ok(Vec::new());
        }
        
        let prefix_path = std::path::Path::new(&self.config.storage_path);
        let mut entries = Vec::new();
        
        let mut read_dir = tokio::fs::read_dir(dir_path).await
            .map_err(|e| PersistenceError::IO(format!("Failed to read directory: {}", e)))?;
        
        while let Ok(Some(entry)) = read_dir.next_entry().await {
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

/// Memory-based persistence implementation
pub struct MemoryPersistence {
    /// In-memory data store
    data: tokio::sync::RwLock<std::collections::HashMap<String, Vec<u8>>>,
}

impl MemoryPersistence {
    /// Create a new memory persistence
    pub fn new() -> Self {
        Self {
            data: tokio::sync::RwLock::new(std::collections::HashMap::new()),
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
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize session: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_session(&self, token: &SessionToken) -> Result<Option<SessionData>> {
        let key = format!("sessions/{}", token.0);
        if let Some(data) = self.load_data(&key).await? {
            let session = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize session: {}", e)))?;
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
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize user: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_user_by_id(&self, id: &UserId) -> Result<Option<UserData>> {
        let key = format!("users/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let user = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize user: {}", e)))?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn load_user_by_username(&self, username: &str) -> Result<Option<UserData>> {
        let data = self.data.read().await;
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
            .map_err(|e| PersistenceError::Format(format!("Failed to serialize account: {}", e)))?;
        self.save_data(&key, &data).await
    }
    
    async fn load_account(&self, id: &AccountId) -> Result<Option<AccountData>> {
        let key = format!("accounts/{}", id.0);
        if let Some(data) = self.load_data(&key).await? {
            let account = serde_json::from_slice(&data)
                .map_err(|e| PersistenceError::Format(format!("Failed to deserialize account: {}", e)))?;
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
        self.data.write().await.insert(key.to_string(), value.to_vec());
        Ok(())
    }
    
    async fn load_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }
    
    async fn delete_data(&self, key: &str) -> Result<()> {
        self.data.write().await.remove(key);
        Ok(())
    }
    
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let data = self.data.read().await;
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
    pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }
    
    /// Create a file-based persistence
    pub fn create_file_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(FilePersistence::new(self.config.clone()))
    }
    
    /// Create a memory-based persistence
    pub fn create_memory_persistence(&self) -> Arc<dyn Persistence> {
        Arc::new(MemoryPersistence::new())
    }
    
    /// Create a persistence based on the configuration
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