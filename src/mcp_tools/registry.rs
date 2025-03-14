use std::fmt;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub metadata: serde_json::Value,
}

pub struct Registry {
    entries: HashMap<String, RegistryEntry>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn register(&mut self, entry: RegistryEntry) -> Result<()> {
        if self.entries.contains_key(&entry.id) {
            return Err("Entry already exists".into());
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> Option<RegistryEntry> {
        self.entries.remove(id)
    }

    pub fn get_entry(&self, id: &str) -> Option<&RegistryEntry> {
        self.entries.get(id)
    }

    pub fn list_entries(&self) -> Vec<&RegistryEntry> {
        self.entries.values().collect()
    }

    pub fn update_entry(&mut self, entry: RegistryEntry) -> Result<()> {
        if !self.entries.contains_key(&entry.id) {
            return Err("Entry not found".into());
        }
        self.entries.insert(entry.id.clone(), entry);
        Ok(())
    }
} 