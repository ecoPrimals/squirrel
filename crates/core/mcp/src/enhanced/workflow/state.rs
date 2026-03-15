// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow State Manager
//!
//! Manages workflow state persistence, recovery, and synchronization.
//! Provides state snapshots, rollback capabilities, and distributed state management.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::{Result, types::MCPError};
use super::types::*;

/// Workflow state manager
///
/// Manages workflow state persistence, recovery, and synchronization.
/// Provides state snapshots, rollback capabilities, and distributed state management.
#[derive(Debug)]
pub struct WorkflowStateManager {
    /// In-memory state store (for fast access)
    state_store: Arc<RwLock<HashMap<String, WorkflowState>>>,
    
    /// State snapshots for recovery
    snapshots: Arc<RwLock<HashMap<String, Vec<StateSnapshot>>>>,
    
    /// Configuration
    config: StateManagerConfig,
}

/// State manager configuration
#[derive(Debug, Clone)]
pub struct StateManagerConfig {
    /// Enable persistent state storage
    pub enable_persistence: bool,
    
    /// Snapshot interval
    pub snapshot_interval: Duration,
    
    /// Maximum snapshots to keep
    pub max_snapshots: usize,
    
    /// Enable distributed state sync
    pub enable_distributed_sync: bool,
}

impl Default for StateManagerConfig {
    fn default() -> Self {
        Self {
            enable_persistence: true,
            snapshot_interval: Duration::from_secs(60),
            max_snapshots: 10,
            enable_distributed_sync: false,
        }
    }
}

/// State snapshot for recovery
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    /// Snapshot timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Workflow state at snapshot time
    pub state: WorkflowState,
    
    /// Snapshot metadata
    pub metadata: HashMap<String, String>,
}

impl WorkflowStateManager {
    /// Create a new state manager
    pub fn new(config: StateManagerConfig) -> Self {
        Self {
            state_store: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Save workflow state
    pub async fn save_state(&self, workflow_id: &str, state: WorkflowState) -> Result<()> {
        debug!("Saving state for workflow: {}", workflow_id);
        
        // Store in memory
        let mut store = self.state_store.write().await;
        store.insert(workflow_id.to_string(), state.clone());
        
        // Create snapshot if needed
        if self.config.enable_persistence {
            drop(store); // Release lock before creating snapshot
            self.create_snapshot(workflow_id, state).await?;
        }
        
        Ok(())
    }
    
    /// Load workflow state
    pub async fn load_state(&self, workflow_id: &str) -> Result<Option<WorkflowState>> {
        let store = self.state_store.read().await;
        Ok(store.get(workflow_id).cloned())
    }
    
    /// Create state snapshot
    async fn create_snapshot(&self, workflow_id: &str, state: WorkflowState) -> Result<()> {
        let snapshot = StateSnapshot {
            timestamp: chrono::Utc::now(),
            state,
            metadata: HashMap::new(),
        };
        
        let mut snapshots = self.snapshots.write().await;
        let workflow_snapshots = snapshots
            .entry(workflow_id.to_string())
            .or_insert_with(Vec::new);
        workflow_snapshots.push(snapshot);
        
        // Keep only max_snapshots
        if workflow_snapshots.len() > self.config.max_snapshots {
            workflow_snapshots.remove(0);
        }
        
        debug!(
            "Created snapshot for workflow: {} (total: {})",
            workflow_id,
            workflow_snapshots.len()
        );
        
        Ok(())
    }
    
    /// Restore from snapshot
    pub async fn restore_from_snapshot(
        &self,
        workflow_id: &str,
        snapshot_index: Option<usize>,
    ) -> Result<WorkflowState> {
        let snapshots = self.snapshots.read().await;
        let workflow_snapshots = snapshots.get(workflow_id).ok_or_else(|| {
            MCPError::InvalidArgument(format!("No snapshots found for workflow: {}", workflow_id))
        })?;
        
        let snapshot = if let Some(index) = snapshot_index {
            workflow_snapshots.get(index).ok_or_else(|| {
                MCPError::InvalidArgument(format!("Snapshot index out of bounds: {}", index))
            })?
        } else {
            // Get latest snapshot
            workflow_snapshots
                .last()
                .ok_or_else(|| MCPError::InvalidArgument("No snapshots available".to_string()))?
        };
        
        debug!(
            "Restored snapshot for workflow: {} (timestamp: {})",
            workflow_id, snapshot.timestamp
        );
        
        Ok(snapshot.state.clone())
    }
    
    /// Delete workflow state
    pub async fn delete_state(&self, workflow_id: &str) -> Result<()> {
        let mut store = self.state_store.write().await;
        store.remove(workflow_id);
        
        let mut snapshots = self.snapshots.write().await;
        snapshots.remove(workflow_id);
        
        debug!("Deleted state for workflow: {}", workflow_id);
        
        Ok(())
    }
    
    /// Get all workflow states
    pub async fn list_states(&self) -> Result<Vec<(String, WorkflowState)>> {
        let store = self.state_store.read().await;
        Ok(store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }
    
    /// List snapshots for a workflow
    pub async fn list_snapshots(&self, workflow_id: &str) -> Result<Vec<StateSnapshot>> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots
            .get(workflow_id)
            .map(|s| s.clone())
            .unwrap_or_default())
    }
    
    /// Get snapshot count for a workflow
    pub async fn snapshot_count(&self, workflow_id: &str) -> usize {
        let snapshots = self.snapshots.read().await;
        snapshots
            .get(workflow_id)
            .map(|s| s.len())
            .unwrap_or(0)
    }
    
    /// Clear old snapshots based on age
    pub async fn cleanup_old_snapshots(&self, max_age: Duration) -> Result<usize> {
        let mut snapshots = self.snapshots.write().await;
        let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(max_age).expect("max_age should be representable as chrono::Duration");
        let mut removed_count = 0;
        
        for workflow_snapshots in snapshots.values_mut() {
            let initial_len = workflow_snapshots.len();
            workflow_snapshots.retain(|s| s.timestamp > cutoff_time);
            removed_count += initial_len - workflow_snapshots.len();
        }
        
        debug!("Cleaned up {} old snapshots", removed_count);
        
        Ok(removed_count)
    }
    
    /// Get state store size
    pub async fn state_count(&self) -> usize {
        let store = self.state_store.read().await;
        store.len()
    }
    
    /// Get total snapshot count across all workflows
    pub async fn total_snapshot_count(&self) -> usize {
        let snapshots = self.snapshots.read().await;
        snapshots.values().map(|s| s.len()).sum()
    }
    
    /// Get configuration
    pub fn config(&self) -> &StateManagerConfig {
        &self.config
    }
}

