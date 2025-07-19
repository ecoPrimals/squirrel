//! Context management for MCP task routing
//!
//! This module handles persistent and shared contexts for MCP tasks, including
//! context storage, retrieval, and synchronization across agents.

use crate::{ContextRequirements, Error, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_json::Value;
use tracing::{debug, info, warn};

/// Storage backend for contexts
#[derive(Debug, Clone)]
pub enum ContextStorage {
    /// Store contexts locally in memory
    Local,
    /// Store contexts in NestGate
    NestGate { endpoint: String },
    /// Store contexts across federation nodes
    Federation { nodes: Vec<String> },
}

/// A persistent context that is tied to a specific agent
#[derive(Debug, Clone)]
pub struct PersistentContext {
    /// Unique context identifier
    pub context_id: String,
    /// Agent that owns this context
    pub agent_id: String,
    /// Context data
    pub data: Value,
    /// When the context was created
    pub created_at: DateTime<Utc>,
    /// When the context was last accessed
    pub last_accessed: DateTime<Utc>,
    /// When the context expires (if any)
    pub expiry: Option<DateTime<Utc>>,
}

/// A shared context that can be accessed by multiple agents
#[derive(Debug, Clone)]
pub struct SharedContext {
    /// Unique context identifier
    pub context_id: String,
    /// List of agent IDs that have access to this context
    pub shared_with: Vec<String>,
    /// Context data
    pub data: Value,
    /// Version number for optimistic locking
    pub version: u64,
    /// When the context was last modified
    pub last_modified: DateTime<Utc>,
}

/// Manager for handling persistent and shared contexts
#[derive(Debug)]
pub struct ContextManager {
    /// Persistent contexts by context ID
    persistent_contexts: DashMap<String, PersistentContext>,
    /// Shared contexts by context ID
    shared_contexts: DashMap<String, SharedContext>,
    /// Storage backend
    context_storage: Option<ContextStorage>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(storage: Option<ContextStorage>) -> Self {
        Self {
            persistent_contexts: DashMap::new(),
            shared_contexts: DashMap::new(),
            context_storage: storage,
        }
    }

    /// Get context data for a specific agent
    pub async fn get_context(&self, agent_id: &str) -> Option<Value> {
        // Look for persistent context first
        if let Some(persistent_context) = self.get_persistent_context(agent_id).await {
            return Some(persistent_context.data);
        }

        // Look for shared contexts
        let shared_contexts = self.get_shared_contexts_for_agent(agent_id).await;
        if !shared_contexts.is_empty() {
            // Merge shared contexts if there are multiple
            let mut merged_data = serde_json::Map::new();
            for context in shared_contexts {
                if let Value::Object(obj) = context.data {
                    merged_data.extend(obj);
                }
            }
            return Some(Value::Object(merged_data));
        }

        None
    }

    /// Update persistent context for an agent
    pub async fn update_persistent_context(&self, agent_id: &str, data: Value) -> Result<()> {
        let context_id = format!("persistent_{agent_id}");
        let now = Utc::now();

        let context = PersistentContext {
            context_id: context_id.clone(),
            agent_id: agent_id.to_string(),
            data,
            created_at: now,
            last_accessed: now,
            expiry: None,
        };

        self.persistent_contexts.insert(context_id, context);
        debug!("Updated persistent context for agent '{}'", agent_id);
        Ok(())
    }

    /// Update shared context
    pub async fn update_shared_context(&self, context_id: &str, data: Value) -> Result<()> {
        match self.shared_contexts.get_mut(context_id) {
            Some(mut context) => {
                context.data = data;
                context.version += 1;
                context.last_modified = Utc::now();
                debug!(
                    "Updated shared context '{}' to version {}",
                    context_id, context.version
                );
                Ok(())
            }
            None => Err(Error::ContextNotFound(context_id.to_string())),
        }
    }

    /// Create a new shared context
    pub async fn create_shared_context(
        &self,
        context_id: &str,
        data: Value,
        shared_with: Vec<String>,
    ) -> Result<()> {
        let context = SharedContext {
            context_id: context_id.to_string(),
            shared_with,
            data,
            version: 1,
            last_modified: Utc::now(),
        };

        let shared_count = context.shared_with.len();
        self.shared_contexts.insert(context_id.to_string(), context);
        info!(
            "Created shared context '{}' shared with {} agents",
            context_id, shared_count
        );
        Ok(())
    }

    /// Get persistent context for an agent
    async fn get_persistent_context(&self, agent_id: &str) -> Option<PersistentContext> {
        let context_id = format!("persistent_{agent_id}");
        self.persistent_contexts.get(&context_id).map(|entry| {
            let mut context = entry.value().clone();
            context.last_accessed = Utc::now();
            context
        })
    }

    /// Get shared contexts for an agent
    async fn get_shared_contexts_for_agent(&self, agent_id: &str) -> Vec<SharedContext> {
        self.shared_contexts
            .iter()
            .filter(|entry| entry.value().shared_with.contains(&agent_id.to_string()))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Add agent to shared context
    pub async fn add_agent_to_shared_context(
        &self,
        context_id: &str,
        agent_id: &str,
    ) -> Result<()> {
        match self.shared_contexts.get_mut(context_id) {
            Some(mut context) => {
                if !context.shared_with.contains(&agent_id.to_string()) {
                    context.shared_with.push(agent_id.to_string());
                    debug!(
                        "Added agent '{}' to shared context '{}'",
                        agent_id, context_id
                    );
                }
                Ok(())
            }
            None => Err(Error::ContextNotFound(context_id.to_string())),
        }
    }

    /// Remove agent from shared context
    pub async fn remove_agent_from_shared_context(
        &self,
        context_id: &str,
        agent_id: &str,
    ) -> Result<()> {
        match self.shared_contexts.get_mut(context_id) {
            Some(mut context) => {
                context.shared_with.retain(|id| id != agent_id);
                debug!(
                    "Removed agent '{}' from shared context '{}'",
                    agent_id, context_id
                );
                Ok(())
            }
            None => Err(Error::ContextNotFound(context_id.to_string())),
        }
    }

    /// Get context value by key
    pub async fn get_context_value(&self, agent_id: &str, key: &str) -> Option<Value> {
        if let Some(Value::Object(obj)) = self.get_context(agent_id).await {
            obj.get(key).cloned()
        } else {
            None
        }
    }

    /// Set context value by key
    pub async fn set_context_value(&self, agent_id: &str, key: &str, value: Value) -> Result<()> {
        let mut context_data = self
            .get_context(agent_id)
            .await
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(ref mut obj) = context_data {
            obj.insert(key.to_string(), value);
            self.update_persistent_context(agent_id, context_data)
                .await?;
            Ok(())
        } else {
            Err(Error::InvalidContext(
                "Context data is not an object".to_string(),
            ))
        }
    }

    /// Check if agent has access to a specific context
    pub async fn agent_has_context_access(&self, agent_id: &str, context_id: &str) -> bool {
        // Check persistent context
        if context_id.starts_with("persistent_") {
            let expected_id = format!("persistent_{agent_id}");
            return context_id == expected_id;
        }

        // Check shared context
        if let Some(context) = self.shared_contexts.get(context_id) {
            return context.shared_with.contains(&agent_id.to_string());
        }

        false
    }

    /// Clean up expired contexts
    pub async fn cleanup_expired_contexts(&self) -> usize {
        let now = Utc::now();
        let mut removed_count = 0;

        // Clean up persistent contexts
        self.persistent_contexts.retain(|_, context| {
            if let Some(expiry) = context.expiry {
                if now > expiry {
                    debug!(
                        "Removing expired persistent context '{}'",
                        context.context_id
                    );
                    removed_count += 1;
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        if removed_count > 0 {
            info!("Cleaned up {} expired contexts", removed_count);
        }

        removed_count
    }

    /// Get context statistics
    pub async fn get_context_statistics(&self) -> ContextStatistics {
        let persistent_count = self.persistent_contexts.len();
        let shared_count = self.shared_contexts.len();
        let total_size = self.calculate_total_context_size().await;

        ContextStatistics {
            persistent_contexts: persistent_count,
            shared_contexts: shared_count,
            total_contexts: persistent_count + shared_count,
            total_size_bytes: total_size,
            storage_backend: self.context_storage.clone(),
        }
    }

    /// Calculate total size of all contexts
    async fn calculate_total_context_size(&self) -> usize {
        let mut total_size = 0;

        // Calculate size of persistent contexts
        for context in self.persistent_contexts.iter() {
            if let Ok(serialized) = serde_json::to_string(&context.data) {
                total_size += serialized.len();
            }
        }

        // Calculate size of shared contexts
        for context in self.shared_contexts.iter() {
            if let Ok(serialized) = serde_json::to_string(&context.data) {
                total_size += serialized.len();
            }
        }

        total_size
    }

    /// Export all contexts for backup
    pub async fn export_contexts(&self) -> Result<ContextExport> {
        let persistent_contexts: Vec<PersistentContext> = self
            .persistent_contexts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        let shared_contexts: Vec<SharedContext> = self
            .shared_contexts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        Ok(ContextExport {
            persistent_contexts,
            shared_contexts,
            exported_at: Utc::now(),
        })
    }

    /// Import contexts from backup
    pub async fn import_contexts(&self, export: ContextExport) -> Result<()> {
        // Import persistent contexts
        for context in &export.persistent_contexts {
            self.persistent_contexts
                .insert(context.context_id.clone(), context.clone());
        }

        // Import shared contexts
        for context in &export.shared_contexts {
            self.shared_contexts
                .insert(context.context_id.clone(), context.clone());
        }

        info!(
            "Imported {} persistent and {} shared contexts",
            export.persistent_contexts.len(),
            export.shared_contexts.len()
        );

        Ok(())
    }

    /// Apply context requirements from a task
    pub async fn apply_context_requirements(
        &self,
        agent_id: &str,
        requirements: &ContextRequirements,
    ) -> Result<()> {
        // Set required context values
        for (key, value) in &requirements.required_context {
            self.set_context_value(agent_id, key, serde_json::Value::String(value.clone()))
                .await?;
        }

        // Add to shared contexts if specified
        for shared_context_id in &requirements.shared_contexts {
            if let Err(e) = self
                .add_agent_to_shared_context(shared_context_id, agent_id)
                .await
            {
                warn!(
                    "Failed to add agent '{}' to shared context '{}': {}",
                    agent_id, shared_context_id, e
                );
            }
        }

        Ok(())
    }

    /// Get storage backend
    pub fn get_storage_backend(&self) -> Option<&ContextStorage> {
        self.context_storage.as_ref()
    }
}

/// Statistics about context usage
#[derive(Debug, Clone)]
pub struct ContextStatistics {
    /// Number of persistent contexts
    pub persistent_contexts: usize,
    /// Number of shared contexts
    pub shared_contexts: usize,
    /// Total number of contexts
    pub total_contexts: usize,
    /// Total size of all contexts in bytes
    pub total_size_bytes: usize,
    /// Storage backend being used
    pub storage_backend: Option<ContextStorage>,
}

/// Export structure for backing up contexts
#[derive(Debug, Clone)]
pub struct ContextExport {
    /// All persistent contexts
    pub persistent_contexts: Vec<PersistentContext>,
    /// All shared contexts
    pub shared_contexts: Vec<SharedContext>,
    /// When this export was created
    pub exported_at: DateTime<Utc>,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(Some(ContextStorage::Local))
    }
}

impl Default for ContextStorage {
    fn default() -> Self {
        Self::Local
    }
}
