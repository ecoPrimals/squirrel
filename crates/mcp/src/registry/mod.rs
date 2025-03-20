use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, Result};
use crate::mcp::types::{MCPMessage, ProtocolVersion, ProtocolState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub version: ProtocolVersion,
    pub state: ProtocolState,
    pub metadata: serde_json::Value,
}

pub struct MCPRegistry {
    entries: Arc<RwLock<HashMap<String, RegistryEntry>>>,
}

impl MCPRegistry {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        if entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(format!("Entry already exists with id: {}", entry.id)));
        }
        entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    pub async fn unregister(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(id)
            .ok_or_else(|| MCPError::Protocol(format!("No entry found with id: {}", id)))?;
        Ok(())
    }

    pub async fn get_entry(&self, id: &str) -> Result<RegistryEntry> {
        let entries = self.entries.read().await;
        entries.get(id)
            .cloned()
            .ok_or_else(|| MCPError::Protocol(format!("No entry found with id: {}", id)))
    }

    pub async fn update_entry(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        if !entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(format!("No entry found with id: {}", entry.id)));
        }
        entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    pub async fn list_entries(&self) -> Result<Vec<RegistryEntry>> {
        let entries = self.entries.read().await;
        Ok(entries.values().cloned().collect())
    }
}

impl Default for MCPRegistry {
    fn default() -> Self {
        Self::new()
    }
} 