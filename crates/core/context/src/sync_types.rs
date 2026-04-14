// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration and status types for the sync subsystem.
//!
//! Extracted from [`super::sync`] for module size management.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

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
            max_message_age_seconds: 300,
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
    /// Result message or error description
    pub message: String,
    /// Timestamp of the sync operation
    pub timestamp: SystemTime,
    /// Number of retries attempted
    pub retry_count: u32,
}

/// Network partition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    /// Timestamp when partition was detected
    pub detected_at: SystemTime,
    /// List of peer IDs affected by partition
    pub affected_peers: Vec<String>,
    /// Duration of the partition
    pub partition_duration: Duration,
    /// Strategy to use for partition recovery
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
