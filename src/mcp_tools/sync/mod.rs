use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub interval: std::time::Duration,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: String,
    pub data: serde_json::Value,
}

pub struct SyncManager {
    config: SyncConfig,
    events: Arc<RwLock<Vec<SyncEvent>>>,
}

impl SyncManager {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn record_event(&self, event: SyncEvent) {
        self.events.write().await.push(event);
    }

    pub async fn get_events(&self) -> Vec<SyncEvent> {
        self.events.read().await.clone()
    }
} 