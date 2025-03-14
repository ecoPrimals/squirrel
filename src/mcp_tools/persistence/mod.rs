use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

pub struct PersistenceManager {
    snapshots: Arc<RwLock<Vec<ContextSnapshot>>>,
}

impl PersistenceManager {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn save_snapshot(&self, snapshot: ContextSnapshot) {
        self.snapshots.write().await.push(snapshot);
    }

    pub async fn get_snapshots(&self) -> Vec<ContextSnapshot> {
        self.snapshots.read().await.clone()
    }
} 