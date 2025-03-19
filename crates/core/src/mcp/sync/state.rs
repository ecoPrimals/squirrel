use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::Result;
use crate::mcp::context_manager::Context;
use std::collections::VecDeque;

/// Represents a change in state that needs to be synchronized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// Unique identifier for this state change
    pub id: Uuid,
    /// Identifier of the context being changed
    pub context_id: Uuid,
    /// Type of operation performed on the context
    pub operation: StateOperation,
    /// Data associated with the change
    pub data: serde_json::Value,
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    /// Version number assigned to this change
    pub version: u64,
}

/// Types of operations that can be performed on contexts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StateOperation {
    /// Create a new context
    Create,
    /// Update an existing context
    Update,
    /// Delete a context
    Delete,
    /// Synchronize a context
    Sync,
}

/// Manages state changes and synchronization
#[derive(Debug)]
pub struct StateSyncManager {
    /// Queue of state changes
    changes: Arc<RwLock<VecDeque<StateChange>>>,
    /// Broadcast channel for publishing state changes to subscribers
    pub sender: broadcast::Sender<StateChange>,
    /// Current version counter for state changes
    current_version: Arc<RwLock<u64>>,
    /// Maximum number of changes to keep in history
    max_changes: usize,
}

impl Clone for StateSyncManager {
    fn clone(&self) -> Self {
        Self {
            changes: self.changes.clone(),
            sender: self.sender.clone(),
            current_version: self.current_version.clone(),
            max_changes: self.max_changes,
        }
    }
}

impl StateSyncManager {
    /// Creates a new StateSyncManager instance
    ///
    /// Initializes a new state synchronization manager with default configuration.
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1024); // Buffer size for change notifications
        
        Self {
            changes: Arc::new(RwLock::new(VecDeque::new())),
            sender: tx,
            current_version: Arc::new(RwLock::new(0)),
            max_changes: 10000, // Maximum number of changes to store in memory
        }
    }

    /// Records a change to a context
    ///
    /// When a context is created, updated, or deleted, this method records the
    /// change for synchronization across instances.
    ///
    /// # Arguments
    /// * `context` - The context being changed
    /// * `operation` - The type of operation performed
    ///
    /// # Errors
    /// Returns an error if the change cannot be recorded
    pub async fn record_change(&self, context: &Context, operation: StateOperation) -> Result<()> {
        let mut version = self.current_version.write().await;
        *version += 1;
        
        let change = StateChange {
            id: Uuid::new_v4(),
            context_id: context.id,
            operation,
            data: serde_json::to_value(context)?,
            timestamp: Utc::now(),
            version: *version,
        };
        
        let mut changes = self.changes.write().await;
        changes.push_back(change.clone());
        
        // If we have too many changes, remove the oldest ones
        while changes.len() > self.max_changes {
            changes.pop_front();
        }
        
        // Broadcast the change to any subscribers
        let _ = self.sender.send(change);
        
        Ok(())
    }

    /// Subscribes to change notifications
    ///
    /// Returns a receiver that will be notified of all context changes.
    pub async fn subscribe_changes(&self) -> broadcast::Receiver<StateChange> {
        self.sender.subscribe()
    }

    /// Gets all changes since a specific version
    ///
    /// # Arguments
    /// * `version` - The version number to get changes since
    ///
    /// # Errors
    /// Returns an error if the changes cannot be retrieved
    pub async fn get_changes_since(&self, version: u64) -> Result<Vec<StateChange>> {
        let changes = self.changes.read().await;
        let result: Vec<StateChange> = changes
            .iter()
            .filter(|change| change.version > version)
            .cloned()
            .collect();
        
        Ok(result)
    }

    /// Applies a state change
    ///
    /// # Arguments
    /// * `change` - The state change to apply
    ///
    /// # Errors
    /// Returns an error if the change cannot be applied
    pub async fn apply_change(&self, change: StateChange) -> Result<()> {
        let mut version = self.current_version.write().await;
        if change.version <= *version {
            // We've already applied this change or have a newer version
            return Ok(());
        }
        
        *version = change.version;
        
        let mut changes = self.changes.write().await;
        changes.push_back(change.clone());
        
        // If we have too many changes, remove the oldest ones
        while changes.len() > self.max_changes {
            changes.pop_front();
        }
        
        // Broadcast the change to any subscribers
        let _ = self.sender.send(change);
        
        Ok(())
    }

    /// Gets the current version
    ///
    /// # Errors
    /// Returns an error if the version cannot be retrieved
    pub async fn get_current_version(&self) -> Result<u64> {
        Ok(*self.current_version.read().await)
    }

    /// Cleans up old changes before a specific time
    ///
    /// # Arguments
    /// * `before` - The timestamp before which to clean up changes
    ///
    /// # Errors
    /// Returns an error if the changes cannot be cleaned up
    pub async fn cleanup_old_changes(&self, before: DateTime<Utc>) -> Result<u64> {
        let mut changes = self.changes.write().await;
        let original_len = changes.len();
        
        changes.retain(|change| change.timestamp >= before);
        
        Ok((original_len - changes.len()) as u64)
    }
}

impl Default for StateSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_change_recording() {
        let manager = StateSyncManager::new();
        let context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        assert!(manager.record_change(&context, StateOperation::Create).await.is_ok());
        
        let version = manager.get_current_version().await.unwrap();
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn test_change_subscription() {
        let manager = StateSyncManager::new();
        let mut rx = manager.subscribe_changes().await;

        let context = Context {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            data: serde_json::json!({}),
            metadata: None,
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
        };

        tokio::spawn({
            let manager = manager.clone();
            let context = context.clone();
            async move {
                manager.record_change(&context, StateOperation::Create).await.unwrap();
            }
        });

        let change = rx.recv().await.unwrap();
        assert_eq!(change.context_id, context.id);
        assert_eq!(change.version, 1);
    }
} 