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
    #[must_use] pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new entry
    ///
    /// # Errors
    /// Returns an error if an entry with the same ID already exists in the registry,
    /// or if the registry lock cannot be acquired.
    pub async fn register(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(
                ProtocolError::HandlerAlreadyExists(format!("Handler already exists with id: {}", entry.id))
            ).into());
        }
        
        entries.insert(entry.id.clone(), entry);
        drop(entries); // Early drop the mutex lock
        Ok(())
    }
    
    /// Get an entry by ID
    ///
    /// # Errors
    /// Returns an error if no entry with the given ID exists in the registry,
    /// or if the registry lock cannot be acquired.
    pub async fn get_entry(&self, id: &str) -> Result<RegistryEntry> {
        let entries = self.entries.read().await;
        
        entries.get(id)
            .cloned()
            .ok_or_else(|| MCPError::Protocol(
                ProtocolError::HandlerNotFound(format!("No entry found with id: {id}"))
            ).into())
    }
    
    /// Checks if an entry exists
    ///
    /// # Errors
    /// Returns an error if the registry lock cannot be acquired.
    pub async fn has_entry(&self, id: &str) -> Result<bool> {
        let entries = self.entries.read().await;
        
        Ok(entries.contains_key(id))
    }
    
    /// Updates an existing entry
    ///
    /// # Errors
    /// Returns an error if no entry with the given ID exists in the registry,
    /// or if the registry lock cannot be acquired.
    pub async fn update_entry(&self, entry: RegistryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if !entries.contains_key(&entry.id) {
            return Err(MCPError::Protocol(
                ProtocolError::HandlerNotFound(format!("No entry found with id: {}", entry.id))
            ).into());
        }
        
        entries.insert(entry.id.clone(), entry);
        drop(entries); // Early drop the mutex lock
        Ok(())
    }
    
    /// Removes an entry
    ///
    /// # Errors
    /// Returns an error if no entry with the given ID exists in the registry,
    /// or if the registry lock cannot be acquired.
    pub async fn remove_entry(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        if !entries.contains_key(id) {
            return Err(MCPError::Protocol(
                ProtocolError::HandlerNotFound(format!("No entry found with id: {id}"))
            ).into());
        }
        
        entries.remove(id);
        drop(entries); // Early drop the mutex lock
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