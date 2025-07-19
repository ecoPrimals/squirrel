use super::{ContextError, ContextSnapshot, ContextState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Configuration for sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Maximum time to wait for sync operations (in seconds)
    pub sync_timeout_seconds: u64,
    /// Interval between heartbeat messages (in seconds)
    pub heartbeat_interval_seconds: u64,
    /// Maximum number of retry attempts for failed sync operations
    pub max_retry_attempts: u32,
    /// Delay between retry attempts (in seconds)
    pub retry_delay_seconds: u64,
    /// Maximum number of pending sync operations
    pub max_pending_operations: usize,
    /// Enable automatic conflict resolution
    pub auto_resolve_conflicts: bool,
    /// Network partition detection timeout (in seconds)
    pub partition_detection_timeout_seconds: u64,
    /// Maximum age of sync messages to accept (in seconds)
    pub max_message_age_seconds: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_timeout_seconds: 30,
            heartbeat_interval_seconds: 10,
            max_retry_attempts: 3,
            retry_delay_seconds: 2,
            max_pending_operations: 1000,
            auto_resolve_conflicts: true,
            partition_detection_timeout_seconds: 60,
            max_message_age_seconds: 300, // 5 minutes
        }
    }
}

/// Status of sync operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    /// Sync is healthy and operating normally
    Healthy,
    /// Sync is degraded but still functional
    Degraded,
    /// Sync is experiencing issues
    Unhealthy,
    /// Sync is completely offline
    Offline,
    /// Network partition detected
    Partitioned,
}

/// Sync operation result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
    pub timestamp: SystemTime,
    pub retry_count: u32,
}

/// Network partition information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartitionInfo {
    pub detected_at: SystemTime,
    pub affected_peers: Vec<String>,
    pub partition_duration: Duration,
    pub recovery_strategy: PartitionRecoveryStrategy,
}

/// Strategies for recovering from network partitions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PartitionRecoveryStrategy {
    /// Wait for partition to heal naturally
    WaitForHealing,
    /// Attempt to reconnect to peers
    AttemptReconnection,
    /// Use cached state until partition heals
    UseCachedState,
    /// Fail over to backup nodes
    FailoverToBackup,
}

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
    /// Message priority (higher numbers = higher priority)
    pub priority: u8,
    /// Retry count for this message
    pub retry_count: u32,
    /// Checksum for message integrity
    pub checksum: Option<String>,
}

impl SyncMessage {
    /// Create a new sync message
    pub fn new(operation: SyncOperation, source: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            operation,
            source,
            priority: 0,
            retry_count: 0,
            checksum: None,
        }
    }

    /// Create a high-priority sync message
    pub fn high_priority(operation: SyncOperation, source: String) -> Self {
        let mut msg = Self::new(operation, source);
        msg.priority = 10;
        msg
    }

    /// Check if message is too old based on config
    pub fn is_expired(&self, config: &SyncConfig) -> bool {
        if let Ok(age) = self.timestamp.elapsed() {
            age.as_secs() > config.max_message_age_seconds
        } else {
            true // If we can't determine age, consider it expired
        }
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
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
    /// Heartbeat message to maintain connection
    Heartbeat {
        node_id: String,
        timestamp: SystemTime,
    },
    /// Request full state synchronization
    FullSyncRequest { requesting_node: String },
    /// Response to full sync request
    FullSyncResponse {
        states: Vec<ContextState>,
        snapshots: Vec<ContextSnapshot>,
    },
    /// Network partition detection
    PartitionDetected(PartitionInfo),
    /// Network partition recovery
    PartitionRecovered {
        recovered_at: SystemTime,
        affected_peers: Vec<String>,
    },
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
    /// Timestamp when conflict was detected
    pub detected_at: SystemTime,
    /// Nodes involved in the conflict
    pub involved_nodes: Vec<String>,
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
    /// Use vector clocks for resolution
    VectorClock,
    /// Use consensus algorithm
    Consensus,
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
        /// Source node of the update
        source: String,
    },
    /// Conflict detected
    ConflictDetected {
        /// State ID in conflict
        state_id: String,
        /// Conflict information
        conflict: ConflictInfo,
    },
    /// Conflict resolved
    ConflictResolved {
        /// State ID that was in conflict
        state_id: String,
        /// Resolution strategy used
        strategy: ConflictResolutionStrategy,
        /// Final resolved state
        resolved_state: ContextState,
    },
    /// Network partition detected
    PartitionDetected {
        /// Partition information
        partition: PartitionInfo,
    },
    /// Network partition recovered
    PartitionRecovered {
        /// Recovery timestamp
        recovered_at: SystemTime,
        /// Affected peers
        affected_peers: Vec<String>,
    },
    /// Sync operation failed
    SyncFailed {
        /// Operation that failed
        operation: String,
        /// Error message
        error: String,
        /// Retry count
        retry_count: u32,
    },
}

/// Manages synchronization between nodes in a distributed system
#[derive(Debug)]
pub struct SyncManager {
    /// Collection of subscribers to sync events, mapped by their unique ID
    subscribers: HashMap<String, Sender<SyncEvent>>,
    /// Sync configuration
    config: SyncConfig,
    /// Current sync status
    status: SyncStatus,
    /// Pending sync operations
    pending_operations: HashMap<String, SyncMessage>,
    /// Failed operations for retry
    failed_operations: HashMap<String, (SyncMessage, u32)>,
    /// Last heartbeat timestamps from peers
    peer_heartbeats: HashMap<String, SystemTime>,
    /// Detected network partitions
    active_partitions: HashMap<String, PartitionInfo>,
}

impl SyncManager {
    /// Create a new sync manager with default configuration
    pub fn new() -> Self {
        Self::with_config(SyncConfig::default())
    }

    /// Create a new sync manager with custom configuration
    pub fn with_config(config: SyncConfig) -> Self {
        Self {
            subscribers: HashMap::new(),
            config,
            status: SyncStatus::Healthy,
            pending_operations: HashMap::new(),
            failed_operations: HashMap::new(),
            peer_heartbeats: HashMap::new(),
            active_partitions: HashMap::new(),
        }
    }

    /// Get current sync status
    pub fn get_status(&self) -> SyncStatus {
        self.status.clone()
    }

    /// Update sync configuration
    pub fn update_config(&mut self, config: SyncConfig) {
        self.config = config;
        info!("Sync configuration updated");
    }

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
        debug!("New sync event subscriber: {}", id);
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
        debug!("Sync event subscriber removed: {}", id);
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
    pub async fn broadcast_event(
        &mut self,
        event: SyncEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut failed_ids = Vec::new();
        for (id, sender) in &self.subscribers {
            if sender.send(event.clone()).await.is_err() {
                failed_ids.push(id.clone());
            }
        }
        for id in failed_ids {
            self.subscribers.remove(&id);
            debug!("Removed failed subscriber: {}", id);
        }
        Ok(())
    }

    /// Process a sync message with timeout and retry logic
    pub async fn process_message_with_retry(
        &mut self,
        mut message: SyncMessage,
    ) -> Result<SyncResult, ContextError> {
        // Check if message is expired
        if message.is_expired(&self.config) {
            warn!("Dropping expired sync message: {}", message.id);
            return Ok(SyncResult {
                success: false,
                message: "Message expired".to_string(),
                timestamp: SystemTime::now(),
                retry_count: message.retry_count,
            });
        }

        // Check pending operations limit
        if self.pending_operations.len() >= self.config.max_pending_operations {
            warn!(
                "Too many pending operations, dropping message: {}",
                message.id
            );
            return Ok(SyncResult {
                success: false,
                message: "Too many pending operations".to_string(),
                timestamp: SystemTime::now(),
                retry_count: message.retry_count,
            });
        }

        let message_id = message.id.clone();
        self.pending_operations
            .insert(message_id.clone(), message.clone());

        let timeout_duration = Duration::from_secs(self.config.sync_timeout_seconds);
        let result = timeout(
            timeout_duration,
            self.process_message_internal(message.clone()),
        )
        .await;

        self.pending_operations.remove(&message_id);

        match result {
            Ok(Ok(_)) => {
                // Success - remove from failed operations if it was there
                self.failed_operations.remove(&message_id);
                Ok(SyncResult {
                    success: true,
                    message: "Operation completed successfully".to_string(),
                    timestamp: SystemTime::now(),
                    retry_count: message.retry_count,
                })
            }
            Ok(Err(e)) => {
                // Failed - add to retry queue if under limit
                let retry_count = message.retry_count;
                if retry_count < self.config.max_retry_attempts {
                    message.increment_retry();
                    self.failed_operations.insert(message_id, (message, 0));
                    warn!("Sync operation failed, will retry: {}", e);
                } else {
                    error!(
                        "Sync operation failed after {} attempts: {}",
                        self.config.max_retry_attempts, e
                    );
                }

                Ok(SyncResult {
                    success: false,
                    message: e.to_string(),
                    timestamp: SystemTime::now(),
                    retry_count,
                })
            }
            Err(_) => {
                // Timeout - add to retry queue
                let retry_count = message.retry_count;
                if retry_count < self.config.max_retry_attempts {
                    message.increment_retry();
                    self.failed_operations.insert(message_id, (message, 0));
                    warn!("Sync operation timed out, will retry");
                } else {
                    error!(
                        "Sync operation timed out after {} attempts",
                        self.config.max_retry_attempts
                    );
                }

                Ok(SyncResult {
                    success: false,
                    message: "Operation timed out".to_string(),
                    timestamp: SystemTime::now(),
                    retry_count,
                })
            }
        }
    }

    /// Internal message processing
    async fn process_message_internal(&mut self, message: SyncMessage) -> Result<(), ContextError> {
        match &message.operation {
            SyncOperation::Heartbeat { node_id, timestamp } => {
                self.handle_heartbeat(node_id.clone(), *timestamp).await
            }
            SyncOperation::StateUpdate(state) => {
                self.handle_state_update(state.clone(), &message.source)
                    .await
            }
            SyncOperation::Conflict(conflict) => {
                self.handle_conflict_advanced(conflict.clone()).await
            }
            SyncOperation::PartitionDetected(partition) => {
                self.handle_partition_detected(partition.clone()).await
            }
            SyncOperation::PartitionRecovered {
                recovered_at,
                affected_peers,
            } => {
                self.handle_partition_recovered(*recovered_at, affected_peers.clone())
                    .await
            }
            _ => {
                debug!("Processing sync operation: {:?}", message.operation);
                Ok(())
            }
        }
    }

    /// Handle heartbeat messages
    async fn handle_heartbeat(
        &mut self,
        node_id: String,
        timestamp: SystemTime,
    ) -> Result<(), ContextError> {
        self.peer_heartbeats.insert(node_id.clone(), timestamp);
        debug!("Received heartbeat from node: {}", node_id);

        // Check if this resolves any partitions
        self.check_partition_recovery(&node_id).await?;

        Ok(())
    }

    /// Handle state updates with conflict detection
    async fn handle_state_update(
        &mut self,
        state: ContextState,
        source: &str,
    ) -> Result<(), ContextError> {
        debug!("Handling state update from {}: {}", source, state.id);

        // Broadcast state updated event
        let event = SyncEvent::StateUpdated {
            version: state.version,
            timestamp: SystemTime::now(),
            source: source.to_string(),
        };

        if let Err(e) = self.broadcast_event(event).await {
            warn!("Failed to broadcast state update event: {}", e);
        }

        Ok(())
    }

    /// Handle advanced conflict resolution
    async fn handle_conflict_advanced(
        &mut self,
        conflict: ConflictInfo,
    ) -> Result<(), ContextError> {
        info!("Handling conflict for state: {}", conflict.state_id);

        // Broadcast conflict detected event
        let event = SyncEvent::ConflictDetected {
            state_id: conflict.state_id.clone(),
            conflict: conflict.clone(),
        };

        if let Err(e) = self.broadcast_event(event).await {
            warn!("Failed to broadcast conflict detected event: {}", e);
        }

        // Attempt automatic resolution if enabled
        if self.config.auto_resolve_conflicts {
            match self.resolve_conflict_automatically(&conflict).await {
                Ok(resolved_state) => {
                    let event = SyncEvent::ConflictResolved {
                        state_id: conflict.state_id.clone(),
                        strategy: conflict.resolution_strategy.clone(),
                        resolved_state,
                    };

                    if let Err(e) = self.broadcast_event(event).await {
                        warn!("Failed to broadcast conflict resolved event: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Automatic conflict resolution failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Automatically resolve conflicts
    async fn resolve_conflict_automatically(
        &self,
        conflict: &ConflictInfo,
    ) -> Result<ContextState, ContextError> {
        match conflict.resolution_strategy {
            ConflictResolutionStrategy::KeepLatest => conflict
                .conflicting_versions
                .iter()
                .max_by_key(|state| state.last_modified)
                .cloned()
                .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string())),
            ConflictResolutionStrategy::KeepOldest => conflict
                .conflicting_versions
                .iter()
                .min_by_key(|state| state.last_modified)
                .cloned()
                .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string())),
            ConflictResolutionStrategy::Merge => {
                // Advanced merge logic would go here
                // For now, use latest version
                conflict
                    .conflicting_versions
                    .iter()
                    .max_by_key(|state| state.last_modified)
                    .cloned()
                    .ok_or_else(|| ContextError::InvalidState("No states to resolve".to_string()))
            }
            _ => Err(ContextError::InvalidState(
                "Automatic resolution not supported for this strategy".to_string(),
            )),
        }
    }

    /// Handle network partition detection
    async fn handle_partition_detected(
        &mut self,
        partition: PartitionInfo,
    ) -> Result<(), ContextError> {
        warn!(
            "Network partition detected affecting {} peers",
            partition.affected_peers.len()
        );

        self.active_partitions
            .insert(partition.affected_peers.join(","), partition.clone());

        self.status = SyncStatus::Partitioned;

        let event = SyncEvent::PartitionDetected { partition };
        if let Err(e) = self.broadcast_event(event).await {
            warn!("Failed to broadcast partition detected event: {}", e);
        }

        Ok(())
    }

    /// Handle network partition recovery
    async fn handle_partition_recovered(
        &mut self,
        recovered_at: SystemTime,
        affected_peers: Vec<String>,
    ) -> Result<(), ContextError> {
        info!(
            "Network partition recovered for {} peers",
            affected_peers.len()
        );

        // Remove from active partitions
        let partition_key = affected_peers.join(",");
        self.active_partitions.remove(&partition_key);

        // Update status if no more partitions
        if self.active_partitions.is_empty() {
            self.status = SyncStatus::Healthy;
        }

        let event = SyncEvent::PartitionRecovered {
            recovered_at,
            affected_peers,
        };

        if let Err(e) = self.broadcast_event(event).await {
            warn!("Failed to broadcast partition recovered event: {}", e);
        }

        Ok(())
    }

    /// Check if a heartbeat resolves any partitions
    async fn check_partition_recovery(&mut self, node_id: &str) -> Result<(), ContextError> {
        let mut recovered_partitions = Vec::new();

        for (key, partition) in &self.active_partitions {
            if partition.affected_peers.contains(&node_id.to_string()) {
                recovered_partitions.push(key.clone());
            }
        }

        for key in recovered_partitions {
            if let Some(partition) = self.active_partitions.remove(&key) {
                self.handle_partition_recovered(SystemTime::now(), partition.affected_peers)
                    .await?;
            }
        }

        Ok(())
    }

    /// Detect network partitions based on missing heartbeats
    pub async fn detect_partitions(&mut self) -> Result<(), ContextError> {
        let now = SystemTime::now();
        let timeout = Duration::from_secs(self.config.partition_detection_timeout_seconds);
        let mut partitioned_peers = Vec::new();

        for (peer_id, last_heartbeat) in &self.peer_heartbeats {
            if let Ok(elapsed) = now.duration_since(*last_heartbeat) {
                if elapsed > timeout {
                    partitioned_peers.push(peer_id.clone());
                }
            }
        }

        if !partitioned_peers.is_empty() {
            let partition = PartitionInfo {
                detected_at: now,
                affected_peers: partitioned_peers,
                partition_duration: timeout,
                recovery_strategy: PartitionRecoveryStrategy::WaitForHealing,
            };

            self.handle_partition_detected(partition).await?;
        }

        Ok(())
    }

    /// Retry failed operations
    pub async fn retry_failed_operations(&mut self) -> Result<(), ContextError> {
        let mut operations_to_retry = Vec::new();
        let now = SystemTime::now();
        let retry_delay = Duration::from_secs(self.config.retry_delay_seconds);

        for (id, (message, last_retry)) in &mut self.failed_operations {
            let elapsed_secs = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|e| {
                    ContextError::InvalidState(format!("Failed to get current timestamp: {}", e))
                })?
                .as_secs();
            if *last_retry == 0 || elapsed_secs - (*last_retry as u64) >= retry_delay.as_secs() {
                operations_to_retry.push((id.clone(), message.clone()));
                *last_retry = elapsed_secs as u32;
            }
        }

        for (id, message) in operations_to_retry {
            info!("Retrying failed operation: {}", id);
            match self.process_message_with_retry(message).await {
                Ok(result) if result.success => {
                    self.failed_operations.remove(&id);
                }
                Ok(_) => {
                    // Still failed, will retry again later
                }
                Err(e) => {
                    error!("Retry failed for operation {}: {}", id, e);
                }
            }
        }

        Ok(())
    }

    /// Get sync statistics
    pub fn get_statistics(&self) -> SyncStatistics {
        SyncStatistics {
            status: self.status.clone(),
            pending_operations: self.pending_operations.len(),
            failed_operations: self.failed_operations.len(),
            active_partitions: self.active_partitions.len(),
            connected_peers: self.peer_heartbeats.len(),
            subscribers: self.subscribers.len(),
        }
    }

    /// Resolves a conflict between two states
    ///
    /// # Arguments
    /// * `state1` - First state in conflict
    /// * `state2` - Second state in conflict
    ///
    /// # Returns
    /// * `ContextState` - Resolved state
    #[must_use]
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

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sync statistics
#[derive(Debug, Clone)]
pub struct SyncStatistics {
    pub status: SyncStatus,
    pub pending_operations: usize,
    pub failed_operations: usize,
    pub active_partitions: usize,
    pub connected_peers: usize,
    pub subscribers: usize,
}
