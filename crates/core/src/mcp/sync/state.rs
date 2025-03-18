use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::{MCPError, Result};
use crate::mcp::types::ProtocolVersion;
use crate::mcp::monitoring::MCPMonitor;
use crate::mcp::context_manager::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub id: Uuid,
    pub context_id: Uuid,
    pub operation: StateOperation,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateOperation {
    Create,
    Update,
    Delete,
    Sync,
}

#[derive(Debug)]
pub struct StateSyncManager {
    changes: Arc<RwLock<HashMap<Uuid, StateChange>>>,
    version: Arc<RwLock<u64>>,
    change_tx: broadcast::Sender<StateChange>,
    change_rx: broadcast::Receiver<StateChange>,
}

impl StateSyncManager {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(1024); // Buffer size for change notifications
        
        Self {
            changes: Arc::new(RwLock::new(HashMap::new())),
            version: Arc::new(RwLock::new(0)),
            change_tx: tx,
            change_rx: rx,
        }
    }

    pub async fn record_change(&self, context: &Context, operation: StateOperation) -> Result<()> {
        let mut version = self.version.write().await;
        *version += 1;

        let change = StateChange {
            id: Uuid::new_v4(),
            context_id: context.id,
            operation,
            data: serde_json::to_value(context)?,
            timestamp: Utc::now(),
            version: *version,
        };

        // Store change
        let mut changes = self.changes.write().await;
        changes.insert(change.id, change.clone());

        // Broadcast change
        if let Err(e) = self.change_tx.send(change) {
            tracing::error!("Failed to broadcast state change: {}", e);
        }

        Ok(())
    }

    pub async fn subscribe_changes(&self) -> broadcast::Receiver<StateChange> {
        self.change_tx.subscribe()
    }

    pub async fn get_changes_since(&self, version: u64) -> Result<Vec<StateChange>> {
        let changes = self.changes.read().await;
        Ok(changes
            .values()
            .filter(|c| c.version > version)
            .cloned()
            .collect())
    }

    pub async fn apply_change(&self, change: StateChange) -> Result<()> {
        let mut changes = self.changes.write().await;
        let mut version = self.version.write().await;

        // Only apply if version is newer
        if change.version > *version {
            changes.insert(change.id, change.clone());
            *version = change.version;

            // Broadcast change
            if let Err(e) = self.change_tx.send(change) {
                tracing::error!("Failed to broadcast applied state change: {}", e);
            }
        }

        Ok(())
    }

    pub async fn get_current_version(&self) -> Result<u64> {
        Ok(*self.version.read().await)
    }

    pub async fn cleanup_old_changes(&self, before: DateTime<Utc>) -> Result<u64> {
        let mut changes = self.changes.write().await;
        let initial_len = changes.len();
        
        changes.retain(|_, change| change.timestamp > before);
        
        Ok((initial_len - changes.len()) as u64)
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