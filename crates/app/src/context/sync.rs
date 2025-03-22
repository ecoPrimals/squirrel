use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{debug, error};
use thiserror::Error;
use async_trait::async_trait;
use json_ptr::JsonPointer;
use crate::error::Result;

/// Context state containing the full context data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// Version number
    pub version: u64,
    /// State data
    pub data: serde_json::Value,
    /// Last modified timestamp
    pub last_modified: SystemTime,
    /// Source identifier
    pub source: String,
}

impl ContextState {
    /// Create a new context state
    pub fn new(data: serde_json::Value, source: String) -> Self {
        Self {
            version: 1,
            data,
            last_modified: SystemTime::now(),
            source,
        }
    }
    
    /// Increment the version number
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.last_modified = SystemTime::now();
    }
}

/// Change record for a context modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    /// Change ID
    pub id: Uuid,
    /// Change timestamp
    pub timestamp: DateTime<Utc>,
    /// JSON path to the changed value
    pub path: String,
    /// Previous value
    pub previous_value: Option<serde_json::Value>,
    /// New value
    pub new_value: serde_json::Value,
    /// Change origin/source
    pub origin: String,
}

impl ChangeRecord {
    /// Create a new change record
    pub fn new(
        path: String,
        previous_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        origin: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            path,
            previous_value,
            new_value,
            origin,
        }
    }
    
    /// Check if this change conflicts with another change
    pub fn conflicts_with(&self, other: &ChangeRecord) -> bool {
        // Same path changes conflict
        if self.path == other.path {
            // Unless they are identical changes
            !(self.new_value == other.new_value && self.previous_value == other.previous_value)
        } else {
            // Check if one path is a prefix of the other (parent-child relationship)
            self.path.starts_with(&other.path) || other.path.starts_with(&self.path)
        }
    }
}

/// Synchronization errors
#[derive(Debug, Error)]
pub enum SyncError {
    /// Conflict detected
    #[error("Conflict detected: {0}")]
    ConflictDetected(String),
    
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Version mismatch
    #[error("Version mismatch: {0}")]
    VersionMismatch(String),
    
    /// Sync failed
    #[error("Sync failed: {0}")]
    SyncFailed(String),
}

impl From<SyncError> for crate::error::SquirrelError {
    fn from(err: SyncError) -> Self {
        match err {
            SyncError::ConflictDetected(msg) => Self::context(format!("Sync conflict detected: {}", msg)),
            SyncError::InvalidPath(msg) => Self::context(format!("Invalid JSON path: {}", msg)),
            SyncError::InvalidState(msg) => Self::context(format!("Invalid sync state: {}", msg)),
            SyncError::VersionMismatch(msg) => Self::context(format!("Version mismatch: {}", msg)),
            SyncError::SyncFailed(msg) => Self::context(format!("Sync failed: {}", msg)),
        }
    }
}

// Add implementation for CoreError conversion
impl From<SyncError> for crate::error::CoreError {
    fn from(err: SyncError) -> Self {
        Self::Context(format!("Sync error: {}", err))
    }
}

/// Conflict resolution strategy
#[async_trait]
pub trait ConflictResolution: Send + Sync + std::fmt::Debug {
    /// Resolve conflict between changes
    async fn resolve(&self, local: &ChangeRecord, remote: &ChangeRecord) -> Result<ChangeRecord>;
    
    /// Check if changes conflict
    fn conflicts(&self, local: &ChangeRecord, remote: &ChangeRecord) -> bool;
}

/// Latest-wins conflict resolution strategy
#[derive(Debug)]
pub struct LatestWinsResolution;

impl LatestWinsResolution {
    /// Create a new latest-wins resolution strategy
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ConflictResolution for LatestWinsResolution {
    async fn resolve(&self, local: &ChangeRecord, remote: &ChangeRecord) -> Result<ChangeRecord> {
        // Choose the latest change
        if local.timestamp > remote.timestamp {
            Ok(local.clone())
        } else {
            Ok(remote.clone())
        }
    }
    
    fn conflicts(&self, local: &ChangeRecord, remote: &ChangeRecord) -> bool {
        local.conflicts_with(remote)
    }
}

/// Origin-priority conflict resolution strategy
#[derive(Debug)]
pub struct OriginPriorityResolution {
    /// Priority map for origins (higher number means higher priority)
    pub priorities: HashMap<String, u32>,
    /// Default priority for unknown origins
    pub default_priority: u32,
}

impl OriginPriorityResolution {
    /// Create a new origin priority resolution strategy
    pub fn new() -> Self {
        let mut priorities = HashMap::new();
        priorities.insert("user".to_string(), 100);
        priorities.insert("system".to_string(), 50);
        priorities.insert("plugin".to_string(), 25);
        
        Self {
            priorities,
            default_priority: 10,
        }
    }
    
    /// Get priority for an origin
    fn get_priority(&self, origin: &str) -> u32 {
        *self.priorities.get(origin).unwrap_or(&self.default_priority)
    }
}

#[async_trait]
impl ConflictResolution for OriginPriorityResolution {
    async fn resolve(&self, local: &ChangeRecord, remote: &ChangeRecord) -> Result<ChangeRecord> {
        // Choose the higher priority change
        let local_priority = self.get_priority(&local.origin);
        let remote_priority = self.get_priority(&remote.origin);
        
        if local_priority > remote_priority {
            Ok(local.clone())
        } else if remote_priority > local_priority {
            Ok(remote.clone())
        } else {
            // If equal priority, choose the latest change
            if local.timestamp > remote.timestamp {
                Ok(local.clone())
            } else {
                Ok(remote.clone())
            }
        }
    }
    
    fn conflicts(&self, local: &ChangeRecord, remote: &ChangeRecord) -> bool {
        local.conflicts_with(remote)
    }
}

/// Context synchronization manager
#[derive(Debug)]
pub struct SyncManager {
    /// Context state
    state: Arc<RwLock<ContextState>>,
    /// Change history
    change_history: Arc<RwLock<VecDeque<ChangeRecord>>>,
    /// Conflict resolution strategy
    #[allow(dead_code)]
    conflict_strategy: Box<dyn ConflictResolution>,
    /// Maximum history size
    max_history_size: usize,
    /// Source identifier
    source: String,
}

impl SyncManager {
    /// Create a new sync manager with latest-wins conflict resolution
    pub fn new(initial_state: ContextState, source: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            change_history: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            conflict_strategy: Box::new(LatestWinsResolution::new()),
            max_history_size: 100,
            source,
        }
    }
    
    /// Create a new sync manager with custom conflict resolution
    pub fn with_resolution_strategy(
        initial_state: ContextState,
        source: String,
        strategy: Box<dyn ConflictResolution>,
        max_history_size: usize,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            change_history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history_size))),
            conflict_strategy: strategy,
            max_history_size,
            source,
        }
    }
    
    /// Get the current state
    pub async fn get_state(&self) -> ContextState {
        self.state.read().await.clone()
    }
    
    /// Apply a change to the state
    pub async fn apply_change(&self, change: ChangeRecord) -> Result<()> {
        // Get current state
        let mut state = self.state.write().await;
        
        // Use a JSON pointer to apply the change
        let path = if change.path.starts_with('/') {
            change.path.clone()
        } else {
            format!("/{}", change.path)
        };
        
        // Try to get the current value at the path
        let pointer = JsonPointer::new(path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>());
        let current_value = pointer.get(&state.data).ok().cloned();
        
        // Check if the previous value matches our current value
        if let Some(prev) = &change.previous_value {
            if current_value.is_none() || current_value.as_ref() != Some(prev) {
                return Err(SyncError::VersionMismatch(format!(
                    "Previous value mismatch at {}: expected {:?}, found {:?}",
                    path, prev, current_value
                )).into());
            }
        }
        
        // Add the value manually since json-ptr doesn't have a direct set method
        let mut ref_val = &mut state.data;
        let tokens = path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();
        
        // Navigate to the parent location
        for (_i, token) in tokens.iter().enumerate().take(tokens.len().saturating_sub(1)) {
            if let Some(idx) = token.parse::<usize>().ok() {
                // Array access
                if let serde_json::Value::Array(arr) = ref_val {
                    if idx < arr.len() {
                        ref_val = &mut arr[idx];
                    } else {
                        return Err(SyncError::InvalidPath(format!("Array index out of bounds: {}", idx)).into());
                    }
                } else {
                    return Err(SyncError::InvalidPath(format!("Expected array at {}", token)).into());
                }
            } else {
                // Object access
                if let serde_json::Value::Object(obj) = ref_val {
                    if let Some(value) = obj.get_mut(*token) {
                        ref_val = value;
                    } else {
                        return Err(SyncError::InvalidPath(format!("Object key not found: {}", token)).into());
                    }
                } else {
                    return Err(SyncError::InvalidPath(format!("Expected object at {}", token)).into());
                }
            }
        }
        
        // Set the value at the final position
        if let Some(last_token) = tokens.last() {
            match ref_val {
                serde_json::Value::Object(obj) => {
                    obj.insert(last_token.to_string(), change.new_value.clone());
                    Ok::<(), crate::error::CoreError>(())
                },
                serde_json::Value::Array(arr) => {
                    if let Ok(idx) = last_token.parse::<usize>() {
                        if idx < arr.len() {
                            arr[idx] = change.new_value.clone();
                            Ok::<(), crate::error::CoreError>(())
                        } else {
                            Err(SyncError::InvalidPath(format!("Array index out of bounds: {}", idx)).into())
                        }
                    } else {
                        Err(SyncError::InvalidPath(format!("Invalid array index: {}", last_token)).into())
                    }
                },
                _ => Err::<(), crate::error::CoreError>(SyncError::InvalidPath(format!("Cannot set value on primitive at {}", last_token)).into())
            }
        } else {
            // If path is empty or just "/", replace the entire data
            *ref_val = change.new_value.clone();
            Ok::<(), crate::error::CoreError>(())
        }?;

        // Update state version and timestamp after setting the value
        state.increment_version();
        state.source = self.source.clone();
        
        // Add to change history
        let mut history = self.change_history.write().await;
        history.push_back(change);
        
        // Trim history if needed
        if history.len() > self.max_history_size {
            history.pop_front();
        }
        
        Ok(())
    }
    
    /// Merge a remote state with the local state
    pub async fn merge_state(&self, remote: ContextState) -> Result<()> {
        if remote.version < self.state.read().await.version {
            // Remote state is older, ignore
            debug!("Ignoring older remote state (version {} < {})",
                remote.version, self.state.read().await.version);
            return Ok(());
        }
        
        // If remote is newer or from a different source, we need to merge
        let paths_to_check = collect_paths(&remote.data);
        let mut conflicts = Vec::new();
        
        // Check for conflicts
        for path in &paths_to_check {
            let pointer = JsonPointer::new(path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>());
            
            let remote_value = pointer.get(&remote.data).ok().cloned();
            let local_value = pointer.get(&self.state.read().await.data).ok().cloned();
            
            if remote_value != local_value {
                // Create change records for both
                let local_change = ChangeRecord::new(
                    path.clone(),
                    None,
                    local_value.unwrap_or(serde_json::Value::Null),
                    self.source.clone(),
                );
                
                let remote_change = ChangeRecord::new(
                    path.clone(),
                    None,
                    remote_value.unwrap_or(serde_json::Value::Null),
                    remote.source.clone(),
                );
                
                if self.conflict_strategy.conflicts(&local_change, &remote_change) {
                    conflicts.push((local_change, remote_change));
                }
            }
        }
        
        // Resolve conflicts
        let mut resolved_changes = Vec::new();
        for (local, remote) in conflicts {
            let resolved = self.conflict_strategy.resolve(&local, &remote).await?;
            resolved_changes.push(resolved);
        }
        
        // Apply resolved changes
        let mut state = self.state.write().await;
        state.version = remote.version.max(state.version) + 1;
        state.last_modified = SystemTime::now();
        
        for change in resolved_changes {
            let path = if change.path.starts_with('/') {
                change.path.clone()
            } else {
                format!("/{}", change.path)
            };
            
            // Use the same approach to set values in the state as in apply_change
            let tokens = path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();
            let mut ref_val = &mut state.data;
            
            // Navigate to the parent location
            for (_i, token) in tokens.iter().enumerate().take(tokens.len().saturating_sub(1)) {
                if let Some(idx) = token.parse::<usize>().ok() {
                    // Array access
                    if let serde_json::Value::Array(arr) = ref_val {
                        if idx < arr.len() {
                            ref_val = &mut arr[idx];
                        } else {
                            return Err(SyncError::InvalidPath(format!("Array index out of bounds: {}", idx)).into());
                        }
                    } else {
                        return Err(SyncError::InvalidPath(format!("Expected array at {}", token)).into());
                    }
                } else {
                    // Object access
                    if let serde_json::Value::Object(obj) = ref_val {
                        if let Some(value) = obj.get_mut(*token) {
                            ref_val = value;
                        } else {
                            return Err(SyncError::InvalidPath(format!("Object key not found: {}", token)).into());
                        }
                    } else {
                        return Err(SyncError::InvalidPath(format!("Expected object at {}", token)).into());
                    }
                }
            }
            
            // Set the value at the final position
            if let Some(last_token) = tokens.last() {
                match ref_val {
                    serde_json::Value::Object(obj) => {
                        obj.insert(last_token.to_string(), change.new_value.clone());
                    },
                    serde_json::Value::Array(arr) => {
                        if let Ok(idx) = last_token.parse::<usize>() {
                            if idx < arr.len() {
                                arr[idx] = change.new_value.clone();
                            } else {
                                return Err(SyncError::InvalidPath(format!("Array index out of bounds: {}", idx)).into());
                            }
                        } else {
                            return Err(SyncError::InvalidPath(format!("Invalid array index: {}", last_token)).into());
                        }
                    },
                    _ => return Err(SyncError::InvalidPath(format!("Cannot set value on primitive at {}", last_token)).into())
                }
            } else {
                // If path is empty or just "/", replace the entire data
                *ref_val = change.new_value.clone();
            }
            
            // Add to change history
            let mut history = self.change_history.write().await;
            history.push_back(change);
            
            // Trim history if needed
            if history.len() > self.max_history_size {
                history.pop_front();
            }
        }
        
        Ok(())
    }
    
    /// Get change history
    pub async fn get_change_history(&self) -> Vec<ChangeRecord> {
        self.change_history.read().await.iter().cloned().collect()
    }
    
    /// Clear change history
    pub async fn clear_history(&self) {
        self.change_history.write().await.clear();
    }
}

/// Collect all JSON paths in a value
fn collect_paths(value: &serde_json::Value) -> Vec<String> {
    let mut paths = Vec::new();
    collect_paths_recursive(value, "", &mut paths);
    paths
}

/// Recursively collect JSON paths
fn collect_paths_recursive(value: &serde_json::Value, prefix: &str, paths: &mut Vec<String>) {
    paths.push(prefix.to_string());
    
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let new_prefix = if prefix.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", prefix, key)
                };
                collect_paths_recursive(val, &new_prefix, paths);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_prefix = if prefix.is_empty() {
                    format!("/{}", i)
                } else {
                    format!("{}/{}", prefix, i)
                };
                collect_paths_recursive(val, &new_prefix, paths);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_apply_change() {
        // Create initial state
        let initial_data = serde_json::json!({
            "name": "test",
            "values": [1, 2, 3],
            "nested": {
                "key": "value"
            }
        });
        
        let state = ContextState::new(initial_data, "test".to_string());
        let manager = SyncManager::new(state, "local".to_string());
        
        // Apply a change
        let change = ChangeRecord::new(
            "name".to_string(),
            Some(serde_json::json!("test")),
            serde_json::json!("updated"),
            "user".to_string(),
        );
        
        manager.apply_change(change).await.unwrap();
        
        // Check the state was updated
        let updated_state = manager.get_state().await;
        assert_eq!(updated_state.version, 2);
        assert_eq!(updated_state.data["name"], "updated");
    }
    
    #[tokio::test]
    async fn test_conflict_resolution() {
        // Create initial state
        let initial_data = serde_json::json!({
            "name": "test",
            "counter": 1
        });
        
        let state = ContextState::new(initial_data, "test".to_string());
        let manager = SyncManager::new(state, "local".to_string());
        
        // Create remote state with a different value
        let remote_data = serde_json::json!({
            "name": "remote",
            "counter": 2
        });
        
        let remote_state = ContextState {
            version: 2,
            data: remote_data,
            last_modified: SystemTime::now(),
            source: "remote".to_string(),
        };
        
        // Merge states
        manager.merge_state(remote_state).await.unwrap();
        
        // Check the result (latest wins should use remote values)
        let merged_state = manager.get_state().await;
        assert_eq!(merged_state.version, 3);
        assert_eq!(merged_state.data["name"], "remote");
        assert_eq!(merged_state.data["counter"], 2);
    }
    
    #[tokio::test]
    async fn test_origin_priority_resolution() {
        // Create initial state
        let initial_data = serde_json::json!({
            "name": "test",
            "counter": 1
        });
        
        let state = ContextState::new(initial_data, "test".to_string());
        
        // Use origin priority resolution
        let resolution = Box::new(OriginPriorityResolution::new());
        let manager = SyncManager::with_resolution_strategy(
            state,
            "system".to_string(),
            resolution,
            100,
        );
        
        // Create remote state with a different value, but from lower priority source
        let remote_data = serde_json::json!({
            "name": "plugin_value",
            "counter": 2
        });
        
        let remote_state = ContextState {
            version: 2,
            data: remote_data,
            last_modified: SystemTime::now(),
            source: "plugin".to_string(),
        };
        
        // Merge states
        manager.merge_state(remote_state).await.unwrap();
        
        // Check the result (system has higher priority than plugin)
        let merged_state = manager.get_state().await;
        assert_eq!(merged_state.version, 3);
        assert_eq!(merged_state.data["name"], "test"); // Kept local value from higher priority source
        assert_eq!(merged_state.data["counter"], 1);   // Kept local value from higher priority source
        
        // Now try with a higher priority source
        let user_data = serde_json::json!({
            "name": "user_value",
            "counter": 3
        });
        
        let user_state = ContextState {
            version: 3,
            data: user_data,
            last_modified: SystemTime::now(),
            source: "user".to_string(),
        };
        
        // Merge states
        manager.merge_state(user_state).await.unwrap();
        
        // Check the result (user has higher priority than system)
        let merged_state = manager.get_state().await;
        assert_eq!(merged_state.version, 4);
        assert_eq!(merged_state.data["name"], "user_value"); // Used higher priority value
        assert_eq!(merged_state.data["counter"], 3);         // Used higher priority value
    }
} 