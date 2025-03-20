use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tokio::sync::mpsc::{self, Sender};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use super::{ContextState, ContextError, ContextSnapshot};

/// Message representing a synchronization operation between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    /// Unique identifier for the sync message
    pub id: String,
    /// Timestamp when the message was created
    pub timestamp: SystemTime,
    /// Type of synchronization operation
    pub operation: SyncOperation,
    /// Identifier of the source node
    pub source: String,
}

/// Types of synchronization operations that can be performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    /// Update the state of a context
    StateUpdate(ContextState),
    /// Create a new snapshot
    SnapshotCreate(ContextSnapshot),
    /// Delete an existing snapshot
    SnapshotDelete(String),
    /// Handle a conflict between states
    Conflict(ConflictInfo),
}

/// Information about a conflict between different versions of state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Identifier of the state in conflict
    pub state_id: String,
    /// List of conflicting state versions
    pub conflicting_versions: Vec<ContextState>,
    /// Strategy to resolve the conflict
    pub resolution_strategy: ConflictResolutionStrategy,
}

/// Strategies for resolving conflicts between different versions of state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Keep the most recent version
    KeepLatest,
    /// Keep the oldest version
    KeepOldest,
    /// Merge the conflicting versions
    Merge,
    /// Require manual resolution
    Manual,
}

/// Events that can occur during synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    /// State has been updated
    StateUpdated {
        /// Version number of the updated state
        version: u64,
        /// Timestamp of the update
        timestamp: SystemTime,
    },
    // Add other event types as needed
}

/// Manages synchronization between nodes in a distributed system
#[derive(Debug)]
pub struct SyncManager {
    /// Collection of subscribers to sync events, mapped by their unique ID
    subscribers: HashMap<String, Sender<SyncEvent>>,
}

impl SyncManager {
    /// Subscribes to sync events
    ///
    /// # Arguments
    /// * `sender` - Channel to send sync events to
    ///
    /// # Returns
    /// * `String` - Unique identifier for the subscription
    pub fn subscribe(&mut self, sender: Sender<SyncEvent>) -> String {
        let id = Uuid::new_v4().to_string();
        self.subscribers.insert(id.clone(), sender);
        id
    }

    /// Unsubscribes from sync events
    ///
    /// # Arguments
    /// * `id` - Subscription identifier to remove
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error status
    /// 
    /// # Errors
    /// * Returns an error if the subscription ID is not found
    pub fn unsubscribe(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.subscribers.remove(id).is_none() {
            return Err("Subscription not found".into());
        }
        Ok(())
    }

    /// Broadcasts a sync event to all subscribers
    ///
    /// # Arguments
    /// * `event` - Event to broadcast
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Success or error status
    /// 
    /// # Errors
    /// * Returns an error if broadcasting to all subscribers fails
    pub async fn broadcast_event(&mut self, event: SyncEvent) -> Result<(), Box<dyn std::error::Error>> {
        let mut failed_ids = Vec::new();
        for (id, sender) in &self.subscribers {
            if sender.send(event.clone()).await.is_err() {
                failed_ids.push(id.clone());
            }
        }
        for id in failed_ids {
            self.subscribers.remove(&id);
        }
        Ok(())
    }

    /// Resolves a conflict between two states
    ///
    /// # Arguments
    /// * `state1` - First state in conflict
    /// * `state2` - Second state in conflict
    ///
    /// # Returns
    /// * `ContextState` - Resolved state
    #[must_use] pub fn resolve_conflict(&self, state1: &ContextState, state2: &ContextState) -> ContextState {
        if state1.version > state2.version {
            state1.clone()
        } else if state2.version > state1.version || state2.last_modified() > state1.last_modified() {
            state2.clone()
        } else {
            state1.clone()
        }
    }
}

/// Resolves conflicts between different versions of state
pub trait ConflictResolver: Send + Sync + std::fmt::Debug {
    /// Resolves a conflict using the specified strategy
    ///
    /// # Arguments
    /// * `conflict` - Information about the conflict to resolve
    ///
    /// # Returns
    /// * `Result<ContextState, ContextError>` - Resolved state or error
    /// 
    /// # Errors
    /// * Returns a `ContextError` if the conflict cannot be resolved
    fn resolve(&self, conflict: &ConflictInfo) -> Result<ContextState, ContextError>;
}

/// Default implementation of conflict resolution
#[derive(Debug, Default)]
pub struct DefaultConflictResolver;

impl ConflictResolver for DefaultConflictResolver {
    fn resolve(&self, conflict: &ConflictInfo) -> Result<ContextState, ContextError> {
        match conflict.resolution_strategy {
            ConflictResolutionStrategy::KeepLatest => {
                conflict.conflicting_versions.iter()
                    .max_by_key(|state| state.last_modified())
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::KeepOldest => {
                conflict.conflicting_versions.iter()
                    .min_by_key(|state| state.last_modified())
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::Merge => {
                // Implement custom merge logic here
                // For now, just keep the latest version
                conflict.conflicting_versions.iter()
                    .max_by_key(|state| state.last_modified())
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::Manual => {
                Err(ContextError::InvalidState("Manual resolution required".to_string()))
            },
        }
    }
}

/// Coordinates synchronization between nodes in a distributed system
#[derive(Debug)]
pub struct SyncCoordinator {
    /// Unique identifier for this node in the network
    node_id: String,
    /// Map of peers and their information, shared across threads
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Channel sender for outgoing sync messages
    message_tx: Sender<SyncMessage>,
    /// Channel receiver for incoming sync messages
    message_rx: mpsc::Receiver<SyncMessage>,
    /// Strategy for resolving conflicts between concurrent updates
    conflict_resolver: Box<dyn ConflictResolver>,
}

/// Information about a connected peer in the sync network
///
/// Tracks when the peer was last seen and their current state version
#[derive(Debug)]
struct PeerInfo {
    /// Last time the peer was seen/communicated with
    last_seen: SystemTime,
    /// Current version of the peer's state
    state_version: u64,
}

impl PeerInfo {
    /// Creates a new `PeerInfo` with the given state version
    ///
    /// # Arguments
    /// * `state_version` - The initial state version for this peer
    fn new(state_version: u64) -> Self {
        Self {
            last_seen: SystemTime::now(),
            state_version,
        }
    }

    /// Updates the `last_seen` timestamp to the current time
    fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
}

impl SyncCoordinator {
    /// Creates a new sync coordinator
    ///
    /// # Arguments
    /// * `node_id` - Identifier for this node
    /// * `conflict_resolver` - Strategy for resolving conflicts
    #[must_use] pub fn new(
        node_id: String,
        conflict_resolver: Box<dyn ConflictResolver>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            node_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_tx: tx,
            message_rx: rx,
            conflict_resolver,
        }
    }

    /// Starts the sync coordinator
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error status
    /// 
    /// # Errors
    /// * Returns a `ContextError` if message handling fails
    pub async fn start(&mut self) -> Result<(), ContextError> {
        while let Some(message) = self.message_rx.recv().await {
            self.handle_sync_message(message).await?;
        }
        Ok(())
    }

    /// Handles incoming sync messages and routes them to the appropriate handler
    ///
    /// # Arguments
    /// * `message` - The sync message to handle
    ///
    /// # Errors
    /// * Returns a `ContextError` if handling the message fails
    async fn handle_sync_message(&mut self, message: SyncMessage) -> Result<(), ContextError> {
        // Update peer info
        self.update_peer_info(&message.source, message.timestamp)?;

        match message.operation {
            SyncOperation::StateUpdate(state) => {
                self.handle_state_update(state, &message.source).await?;
            },
            SyncOperation::SnapshotCreate(snapshot) => {
                self.handle_snapshot_create(snapshot).await?;
            },
            SyncOperation::SnapshotDelete(id) => {
                self.handle_snapshot_delete(id).await?;
            },
            SyncOperation::Conflict(conflict) => {
                self.handle_conflict(conflict).await?;
            },
        }

        Ok(())
    }

    /// Updates the peer information in the peers registry
    ///
    /// # Arguments
    /// * `peer_id` - The ID of the peer to update
    /// * `_timestamp` - The timestamp of the message from the peer
    ///
    /// # Errors
    /// * Returns a `ContextError` if the peers lock cannot be acquired
    fn update_peer_info(&self, peer_id: &str, _timestamp: SystemTime) -> Result<(), ContextError> {
        let mut peers = self.peers.write().map_err(|_| {
            ContextError::InvalidState("Failed to acquire peers lock".to_string())
        })?;

        if let Some(peer_info) = peers.get_mut(peer_id) {
            peer_info.update_last_seen();
            peer_info.state_version += 1;  // Increment version on update
        } else {
            peers.insert(peer_id.to_string(), PeerInfo::new(0));  // Start with version 0
        }

        Ok(())
    }

    /// Handles a state update from another peer
    ///
    /// Checks for potential conflicts with the current state and broadcasts
    /// either a conflict message or the state update to other peers.
    ///
    /// # Arguments
    /// * `state` - The updated state from another peer
    /// * `source` - The ID of the peer that sent the update
    ///
    /// # Errors
    /// * Returns a `ContextError` if the peers lock cannot be acquired or broadcasting fails
    async fn handle_state_update(&self, state: ContextState, source: &str) -> Result<(), ContextError> {
        // Check for conflicts
        let conflict = {
            let peers = self.peers.read().map_err(|_| {
                ContextError::InvalidState("Failed to acquire peers lock".to_string())
            })?;

            if let Some(peer_info) = peers.get(source) {
                if peer_info.state_version >= state.version {
                    // Potential conflict detected
                    Some(ConflictInfo {
                        state_id: state.version.to_string(),
                        conflicting_versions: vec![state.clone()],
                        resolution_strategy: ConflictResolutionStrategy::KeepLatest,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(conflict) = conflict {
            self.broadcast_message(SyncOperation::Conflict(conflict)).await?;
            return Ok(());
        }

        // No conflict, broadcast state update
        self.broadcast_message(SyncOperation::StateUpdate(state)).await?;
        Ok(())
    }

    /// Handles a snapshot creation event from another node
    ///
    /// This method processes a snapshot creation message received from another node
    /// and updates the local state accordingly.
    ///
    /// # Arguments
    /// * `snapshot` - The snapshot to be created
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error
    ///
    /// # Errors
    /// Returns a `ContextError` if there are issues processing the snapshot
    async fn handle_snapshot_create(&self, snapshot: ContextSnapshot) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotCreate(snapshot)).await
    }

    /// Handles a snapshot deletion event from another node
    ///
    /// This method processes a snapshot deletion message received from another node
    /// and updates the local state accordingly.
    ///
    /// # Arguments
    /// * `id` - The ID of the snapshot to be deleted
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error
    ///
    /// # Errors
    /// Returns a `ContextError` if there are issues processing the deletion
    async fn handle_snapshot_delete(&self, id: String) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotDelete(id)).await
    }

    /// Handles a conflict detected between different versions of state
    ///
    /// This method resolves conflicts between different state versions using
    /// the configured conflict resolution strategy.
    ///
    /// # Arguments
    /// * `conflict` - Information about the conflict to be resolved
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error
    ///
    /// # Errors
    /// Returns a `ContextError` if:
    /// - The conflict cannot be resolved
    /// - The resolution strategy fails
    /// - The resolved state cannot be broadcast
    async fn handle_conflict(&self, conflict: ConflictInfo) -> Result<(), ContextError> {
        let resolved_state = self.conflict_resolver.resolve(&conflict)?;
        self.broadcast_message(SyncOperation::StateUpdate(resolved_state)).await
    }

    /// Broadcasts a sync message to all connected nodes
    ///
    /// This method creates and sends a sync message containing the specified
    /// operation to all connected nodes.
    ///
    /// # Arguments
    /// * `operation` - The sync operation to broadcast
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error
    ///
    /// # Errors
    /// Returns a `ContextError::SyncError` if:
    /// - The message cannot be created
    /// - The message channel is closed
    /// - There are network issues
    async fn broadcast_message(&self, operation: SyncOperation) -> Result<(), ContextError> {
        let message = SyncMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            operation,
            source: self.node_id.clone(),
        };
        
        self.message_tx.send(message).await.map_err(|e| {
            ContextError::SyncError(format!("Failed to broadcast message: {e}"))
        })
    }

    /// Sends a state update event to other nodes
    ///
    /// # Arguments
    /// * `state` - New state to propagate
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error status
    ///
    /// # Errors
    ///
    /// Returns a `ContextError::SyncError` if:
    /// - Broadcasting the message fails
    /// - Network communication issues occur
    /// - Message serialization fails
    pub async fn send_state_update(&self, state: ContextState) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::StateUpdate(state)).await
    }

    /// Sends a snapshot creation event to other nodes
    ///
    /// # Arguments
    /// * `snapshot` - New snapshot to propagate
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error status
    ///
    /// # Errors
    ///
    /// Returns a `ContextError::SyncError` if:
    /// - Broadcasting the message fails
    /// - Network communication issues occur
    /// - Message serialization fails
    /// - The snapshot data is invalid or corrupted
    pub async fn send_snapshot_create(&self, snapshot: ContextSnapshot) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotCreate(snapshot)).await
    }

    /// Sends a snapshot deletion event to other nodes
    ///
    /// # Arguments
    /// * `id` - ID of the snapshot to delete
    ///
    /// # Returns
    /// * `Result<(), ContextError>` - Success or error status
    ///
    /// # Errors
    ///
    /// Returns a `ContextError::SyncError` if:
    /// - Broadcasting the message fails
    /// - Network communication issues occur
    /// - Message serialization fails
    /// - The snapshot ID is invalid or not found
    pub async fn send_snapshot_delete(&self, id: String) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotDelete(id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_conflict_resolver() {
        let resolver = DefaultConflictResolver;
        let state1 = ContextState {
            version: 1,
            last_modified: SystemTime::now(),
            data: serde_json::json!({"key": "value1"}),
        };
        let state2 = ContextState {
            version: 2,
            last_modified: SystemTime::now() + Duration::from_secs(1),
            data: serde_json::json!({"key": "value2"}),
        };

        let conflict = ConflictInfo {
            state_id: "test".to_string(),
            conflicting_versions: vec![state1.clone(), state2.clone()],
            resolution_strategy: ConflictResolutionStrategy::KeepLatest,
        };

        let resolved = resolver.resolve(&conflict).unwrap();
        assert_eq!(resolved.version, 2);
    }

    #[tokio::test]
    async fn test_sync_coordinator() {
        let coordinator = SyncCoordinator::new(
            "test_node".to_string(),
            Box::new(DefaultConflictResolver),
        );

        let state = ContextState {
            version: 1,
            last_modified: SystemTime::now(),
            data: serde_json::json!({"key": "value"}),
        };

        assert!(coordinator.send_state_update(state).await.is_ok());
    }
} 