use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tokio::sync::mpsc::{self, Sender};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use super::{ContextState, ContextError, ContextSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub id: String,
    pub timestamp: SystemTime,
    pub operation: SyncOperation,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperation {
    StateUpdate(ContextState),
    SnapshotCreate(ContextSnapshot),
    SnapshotDelete(String),
    Conflict(ConflictInfo),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub state_id: String,
    pub conflicting_versions: Vec<ContextState>,
    pub resolution_strategy: ConflictResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    KeepLatest,
    KeepOldest,
    Merge,
    Manual,
}

#[derive(Debug, Clone)]
pub enum SyncEvent {
    StateUpdated {
        version: u64,
        timestamp: SystemTime,
    },
    // Add other event types as needed
}

#[derive(Default)]
pub struct SyncManager {
    subscribers: HashMap<String, Sender<SyncEvent>>,
}

impl SyncManager {
    pub fn subscribe(&mut self, sender: Sender<SyncEvent>) -> String {
        let id = Uuid::new_v4().to_string();
        self.subscribers.insert(id.clone(), sender);
        id
    }

    pub fn unsubscribe(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.subscribers.remove(id).is_none() {
            return Err("Subscription not found".into());
        }
        Ok(())
    }

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

    pub fn resolve_conflict(&self, state1: &ContextState, state2: &ContextState) -> ContextState {
        if state1.version > state2.version {
            state1.clone()
        } else if state2.version > state1.version || state2.last_modified > state1.last_modified {
            state2.clone()
        } else {
            state1.clone()
        }
    }
}

pub trait ConflictResolver: Send + Sync {
    fn resolve(&self, conflict: &ConflictInfo) -> Result<ContextState, ContextError>;
}

pub struct DefaultConflictResolver;

impl ConflictResolver for DefaultConflictResolver {
    fn resolve(&self, conflict: &ConflictInfo) -> Result<ContextState, ContextError> {
        match conflict.resolution_strategy {
            ConflictResolutionStrategy::KeepLatest => {
                conflict.conflicting_versions.iter()
                    .max_by_key(|state| state.last_modified)
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::KeepOldest => {
                conflict.conflicting_versions.iter()
                    .min_by_key(|state| state.last_modified)
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::Merge => {
                // Implement custom merge logic here
                // For now, just keep the latest version
                conflict.conflicting_versions.iter()
                    .max_by_key(|state| state.last_modified)
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            },
            ConflictResolutionStrategy::Manual => {
                Err(ContextError::InvalidState("Manual resolution required".to_string()))
            },
        }
    }
}

pub struct SyncCoordinator {
    node_id: String,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    message_tx: Sender<SyncMessage>,
    message_rx: mpsc::Receiver<SyncMessage>,
    conflict_resolver: Box<dyn ConflictResolver>,
}

#[derive(Debug)]
struct PeerInfo {
    last_seen: SystemTime,
    state_version: u64,
}

impl PeerInfo {
    fn new(state_version: u64) -> Self {
        Self {
            last_seen: SystemTime::now(),
            state_version,
        }
    }

    fn update_last_seen(&mut self) {
        self.last_seen = SystemTime::now();
    }
}

impl SyncCoordinator {
    pub fn new(
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

    pub async fn start(&mut self) -> Result<(), ContextError> {
        while let Some(message) = self.message_rx.recv().await {
            self.handle_sync_message(message).await?;
        }
        Ok(())
    }

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

    async fn handle_snapshot_create(&self, snapshot: ContextSnapshot) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotCreate(snapshot)).await
    }

    async fn handle_snapshot_delete(&self, id: String) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotDelete(id)).await
    }

    async fn handle_conflict(&self, conflict: ConflictInfo) -> Result<(), ContextError> {
        let resolved_state = self.conflict_resolver.resolve(&conflict)?;
        self.broadcast_message(SyncOperation::StateUpdate(resolved_state)).await
    }

    async fn broadcast_message(&self, operation: SyncOperation) -> Result<(), ContextError> {
        let message = SyncMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            operation,
            source: self.node_id.clone(),
        };

        self.message_tx.send(message).await.map_err(|e| {
            ContextError::InvalidState(format!("Failed to broadcast message: {e}"))
        })
    }

    pub async fn send_state_update(&self, state: ContextState) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::StateUpdate(state)).await
    }

    pub async fn send_snapshot_create(&self, snapshot: ContextSnapshot) -> Result<(), ContextError> {
        self.broadcast_message(SyncOperation::SnapshotCreate(snapshot)).await
    }

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