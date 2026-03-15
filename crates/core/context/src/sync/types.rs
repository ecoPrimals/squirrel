// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Sync type definitions
//!
//! This module contains all type definitions for synchronization operations.

use crate::{ContextSnapshot, ContextState};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
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
    /// Whether the sync operation succeeded
    pub success: bool,
    /// Human-readable message about the sync result
    pub message: String,
    /// When the sync operation completed
    pub timestamp: SystemTime,
    /// Number of retry attempts made
    pub retry_count: u32,
}

/// Network partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Timestamp when the partition was detected
    pub detected_at: SystemTime,
    /// List of peer nodes affected by the partition
    pub affected_peers: Vec<String>,
    /// How long the partition has lasted
    pub partition_duration: Duration,
    /// Strategy to use for recovering from this partition
    pub recovery_strategy: PartitionRecoveryStrategy,
}

/// Strategies for recovering from network partitions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    SnapshotCreate(SnapshotCreateRequest),
    /// Delete an existing snapshot
    SnapshotDelete(SnapshotDeleteRequest),
    /// Handle a conflict between states
    Conflict(ConflictInfo),
    /// Heartbeat message to maintain connection
    Heartbeat {
        /// Node identifier sending the heartbeat
        node_id: String,
        /// Timestamp of the heartbeat
        timestamp: SystemTime,
    },
    /// Request full state synchronization
    FullSyncRequest(FullSyncRequest),
    /// Response to full sync request
    FullSyncResponse(FullSyncResponse),
    /// Network partition detection
    PartitionDetected(PartitionInfo),
    /// Network partition recovery
    PartitionRecovered {
        /// Timestamp when partition was recovered
        recovered_at: SystemTime,
        /// Peer nodes that were affected
        affected_peers: Vec<String>,
    },
}

/// Snapshot creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotCreateRequest {
    /// Snapshot data
    pub snapshot: ContextSnapshot,
}

/// Snapshot deletion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDeleteRequest {
    /// ID of the snapshot to delete
    pub snapshot_id: String,
}

/// Full sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSyncRequest {
    /// IDs of states to synchronize
    pub requested_state_ids: Vec<String>,
}

/// Full sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSyncResponse {
    /// Request ID this is responding to
    pub request_id: String,
    /// State snapshot data
    pub state_snapshot: Vec<StateEntry>,
    /// Optional checksum for integrity
    pub checksum: Option<String>,
}

/// State entry in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct StateEntry {
    /// State identifier
    pub id: String,
    /// State data
    pub data: Vec<u8>,
    /// Timestamp of the state
    pub timestamp: SystemTime,
}

/// Information about a conflict between different versions of state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Identifier of the state in conflict
    pub state_id: String,
    /// Local version of the state
    pub local_version: ContextState,
    /// Remote version of the state
    pub remote_version: ContextState,
    /// When the conflict was detected
    pub detected_at: SystemTime,
    /// Strategy to use for resolving the conflict
    pub resolution_strategy: ConflictResolutionStrategy,
}

/// Strategies for resolving conflicts between different versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Use the version with the latest timestamp
    LastWriteWins,
    /// Use the version with the earliest timestamp
    FirstWriteWins,
    /// Merge both versions semantically
    MergeVersions,
    /// Use version from highest priority source
    HighestPriorityWins,
    /// Ask user to manually resolve
    ManualResolution,
}

/// Events that can occur during sync operations
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Sync operation started
    Started {
        /// ID of the sync operation
        operation_id: String,
        /// Type of operation
        operation_type: String,
    },
    /// Sync operation completed successfully
    Completed {
        /// ID of the sync operation
        operation_id: String,
        /// Result of the operation
        result: SyncResult,
    },
    /// Sync operation failed
    Failed {
        /// ID of the sync operation
        operation_id: String,
        /// Error message
        error: String,
        /// Whether retry is possible
        can_retry: bool,
    },
    /// Status changed
    StatusChanged {
        /// Previous status
        old_status: SyncStatus,
        /// New status
        new_status: SyncStatus,
    },
    /// Conflict detected
    ConflictDetected {
        /// Information about the conflict
        conflict: Box<ConflictInfo>,
    },
    /// Conflict resolved
    ConflictResolved {
        /// ID of the state that had conflict
        state_id: String,
        /// Resolution strategy used
        resolution_strategy: String,
    },
    /// Snapshot created
    SnapshotCreated {
        /// ID of the created snapshot
        snapshot_id: String,
        /// Source that created it
        source: String,
        /// Timestamp of creation
        timestamp: SystemTime,
    },
    /// Snapshot deleted
    SnapshotDeleted {
        /// ID of the deleted snapshot
        snapshot_id: String,
        /// Source that deleted it
        source: String,
    },
    /// Full sync completed
    FullSyncCompleted {
        /// ID of peer that was synced
        peer_id: String,
        /// Number of items synchronized
        items_synced: usize,
    },
    /// Network partition detected
    PartitionDetected {
        /// Information about the partition
        partition: PartitionInfo,
    },
    /// Network partition recovered
    PartitionRecovered {
        /// Timestamp of recovery
        recovered_at: SystemTime,
        /// Affected peers
        affected_peers: Vec<String>,
    },
    /// Heartbeat received from a peer
    HeartbeatReceived {
        /// Peer that sent the heartbeat
        peer_id: String,
        /// Timestamp of the heartbeat
        timestamp: SystemTime,
    },
}

/// Sync statistics
#[derive(Debug, Clone)]
pub struct SyncStatistics {
    /// Current synchronization status
    pub status: SyncStatus,
    /// Number of operations waiting to be processed
    pub pending_operations: usize,
    /// Number of operations that have failed
    pub failed_operations: usize,
    /// Number of active network partitions
    pub active_partitions: usize,
    /// Number of peers currently connected
    pub connected_peers: usize,
    /// Number of active event subscribers
    pub subscribers: usize,
}
