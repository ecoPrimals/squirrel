use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, ProtocolError, Result};
use crate::types::ProtocolVersion;

/// Registry entry for components in the MCP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Unique identifier for the entry
    pub id: String,
    /// Name of the entry
    pub name: String,
    /// Type of the entry (e.g., "adapter", "plugin", "service")
    pub entry_type: String,
    /// Protocol version supported by the entry
    pub version: ProtocolVersion,
    /// Additional metadata about the entry
    pub metadata: serde_json::Value,
}

/// Registry for MCP components
#[derive(Debug)]
pub struct Registry {
    entries: RwLock<HashMap<String, RegistryEntry>>,
}

impl Registry {
    /// Creates a new registry
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Registers a new entry in the registry
    pub async fn register(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(ProtocolError::ConfigurationError(
                format!("Entry already exists with id: {}", entry.id)
            )));
        }
        
        entries.insert(entry.id.clone(), entry);
        Ok(())
    }
    
    /// Gets an entry by ID
    pub async fn get_entry(&self, id: &str) -> Result<RegistryEntry> {
        let entries = self.entries.read().await;
        
        entries.get(id).cloned()
            .ok_or_else(|| MCPError::Protocol(ProtocolError::HandlerNotFound(
                format!("No entry found with id: {}", id)
            )))
    }
    
    /// Checks if an entry exists
    pub async fn has_entry(&self, id: &str) -> Result<bool> {
        let entries = self.entries.read().await;
        
        Ok(entries.contains_key(id))
    }
    
    /// Updates an existing entry
    pub async fn update_entry(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if !entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(ProtocolError::HandlerNotFound(
                format!("No entry found with id: {}", entry.id)
            )));
        }
        
        entries.insert(entry.id.clone(), entry);
        Ok(())
    }
    
    /// Removes an entry
    pub async fn remove_entry(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if !entries.contains_key(id) {
            return Err(MCPError::Protocol(ProtocolError::HandlerNotFound(
                format!("No entry found with id: {}", id)
            )));
        }
        
        entries.remove(id);
        Ok(())
    }
    
    /// Lists all entries
    pub async fn list_entries(&self) -> Vec<RegistryEntry> {
        let entries = self.entries.read().await;
        entries.values().cloned().collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
} 