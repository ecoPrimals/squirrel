// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
#[derive(Debug, Clone, Default)]
pub enum ContextStorage {
    /// Store contexts locally in memory
    #[default]
    Local,
    /// Store contexts in `NestGate`
    NestGate {
        /// `NestGate` base URL or connection target for context storage.
        endpoint: String,
    },
    /// Store contexts across federation nodes
    Federation {
        /// Peer node identifiers or endpoints participating in shared context sync.
        nodes: Vec<String>,
    },
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
    #[must_use]
    pub fn new(storage: Option<ContextStorage>) -> Self {
        Self {
            persistent_contexts: DashMap::new(),
            shared_contexts: DashMap::new(),
            context_storage: storage,
        }
    }

    /// Get context data for a specific agent
    #[must_use]
    pub fn get_context(&self, agent_id: &str) -> Option<Value> {
        // Look for persistent context first
        if let Some(persistent_context) = self.get_persistent_context(agent_id) {
            return Some(persistent_context.data);
        }

        // Look for shared contexts
        let shared_contexts = self.get_shared_contexts_for_agent(agent_id);
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the context cannot be updated.
    pub fn update_persistent_context(&self, agent_id: &str, data: Value) -> Result<()> {
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the shared context does not exist.
    pub fn update_shared_context(&self, context_id: &str, data: Value) -> Result<()> {
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the context cannot be stored.
    pub fn create_shared_context(
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
    fn get_persistent_context(&self, agent_id: &str) -> Option<PersistentContext> {
        let context_id = format!("persistent_{agent_id}");
        self.persistent_contexts.get(&context_id).map(|entry| {
            let mut context = entry.value().clone();
            context.last_accessed = Utc::now();
            context
        })
    }

    /// Get shared contexts for an agent
    fn get_shared_contexts_for_agent(&self, agent_id: &str) -> Vec<SharedContext> {
        self.shared_contexts
            .iter()
            .filter(|entry| entry.value().shared_with.contains(&agent_id.to_string()))
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Add agent to shared context
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the shared context does not exist.
    pub fn add_agent_to_shared_context(&self, context_id: &str, agent_id: &str) -> Result<()> {
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the shared context does not exist.
    pub fn remove_agent_from_shared_context(&self, context_id: &str, agent_id: &str) -> Result<()> {
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
    #[must_use]
    pub fn get_context_value(&self, agent_id: &str, key: &str) -> Option<Value> {
        if let Some(Value::Object(obj)) = self.get_context(agent_id) {
            obj.get(key).cloned()
        } else {
            None
        }
    }

    /// Set context value by key
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the context shape is invalid or persistence fails.
    pub fn set_context_value(&self, agent_id: &str, key: &str, value: Value) -> Result<()> {
        let mut context_data = self
            .get_context(agent_id)
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        if let Value::Object(ref mut obj) = context_data {
            obj.insert(key.to_string(), value);
            self.update_persistent_context(agent_id, context_data)?;
            Ok(())
        } else {
            Err(Error::InvalidContext(
                "Context data is not an object".to_string(),
            ))
        }
    }

    /// Check if agent has access to a specific context
    #[must_use]
    pub fn agent_has_context_access(&self, agent_id: &str, context_id: &str) -> bool {
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
    pub fn cleanup_expired_contexts(&self) -> usize {
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
    #[must_use]
    pub fn get_context_statistics(&self) -> ContextStatistics {
        let persistent_count = self.persistent_contexts.len();
        let shared_count = self.shared_contexts.len();
        let total_size = self.calculate_total_context_size();

        ContextStatistics {
            persistent_contexts: persistent_count,
            shared_contexts: shared_count,
            total_contexts: persistent_count + shared_count,
            total_size_bytes: total_size,
            storage_backend: self.context_storage.clone(),
        }
    }

    /// Calculate total size of all contexts
    fn calculate_total_context_size(&self) -> usize {
        let mut total_size = 0;

        // Calculate size of persistent contexts
        for context in &self.persistent_contexts {
            if let Ok(serialized) = serde_json::to_string(&context.data) {
                total_size += serialized.len();
            }
        }

        // Calculate size of shared contexts
        for context in &self.shared_contexts {
            if let Ok(serialized) = serde_json::to_string(&context.data) {
                total_size += serialized.len();
            }
        }

        total_size
    }

    /// Export all contexts for backup
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if export cannot be assembled.
    pub fn export_contexts(&self) -> Result<ContextExport> {
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if import cannot be applied.
    pub fn import_contexts(&self, export: &ContextExport) -> Result<()> {
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
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if required context keys cannot be set or shared context updates fail.
    pub fn apply_context_requirements(
        &self,
        agent_id: &str,
        requirements: &ContextRequirements,
    ) -> Result<()> {
        // Set required context values
        for (key, value) in &requirements.required_context {
            self.set_context_value(agent_id, key, serde_json::Value::String(value.clone()))?;
        }

        // Add to shared contexts if specified
        for shared_context_id in &requirements.shared_contexts {
            if let Err(e) = self.add_agent_to_shared_context(shared_context_id, agent_id) {
                warn!(
                    "Failed to add agent '{}' to shared context '{}': {}",
                    agent_id, shared_context_id, e
                );
            }
        }

        Ok(())
    }

    /// Get storage backend
    #[must_use]
    pub const fn get_storage_backend(&self) -> Option<&ContextStorage> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContextRequirements;
    use crate::Error;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn context_storage_variants_and_default() {
        assert!(matches!(ContextStorage::default(), ContextStorage::Local));
        let nest = ContextStorage::NestGate {
            endpoint: "http://ng".to_string(),
        };
        let fed = ContextStorage::Federation {
            nodes: vec!["a".to_string()],
        };
        assert!(matches!(nest, ContextStorage::NestGate { .. }));
        assert!(matches!(fed, ContextStorage::Federation { .. }));
    }

    #[test]
    fn persistent_context_roundtrip_and_get_context() {
        let mgr = ContextManager::new(None);
        mgr.update_persistent_context("agent1", json!({"k": 1}))
            .expect("upd");
        let v = mgr.get_context("agent1").expect("ctx");
        assert_eq!(v["k"], 1);
        assert_eq!(mgr.get_context_value("agent1", "k"), Some(json!(1)));
    }

    #[test]
    fn shared_context_create_update_merge_and_access() {
        let mgr = ContextManager::new(None);
        mgr.create_shared_context(
            "sc1",
            json!({"a": 1}),
            vec!["u1".to_string(), "u2".to_string()],
        )
        .expect("create");
        mgr.update_shared_context("sc1", json!({"b": 2}))
            .expect("upd");
        let merged = mgr.get_context("u1").expect("merged");
        assert!(merged.get("b").is_some());
        assert!(mgr.agent_has_context_access("u1", "sc1"));
        assert!(!mgr.agent_has_context_access("other", "sc1"));
        mgr.add_agent_to_shared_context("sc1", "u3").expect("add");
        mgr.remove_agent_from_shared_context("sc1", "u1")
            .expect("rm");
        assert!(!mgr.agent_has_context_access("u1", "sc1"));
    }

    #[test]
    fn shared_context_missing_errors() {
        let mgr = ContextManager::new(None);
        assert!(matches!(
            mgr.update_shared_context("nope", json!({})),
            Err(Error::ContextNotFound(_))
        ));
        assert!(matches!(
            mgr.add_agent_to_shared_context("nope", "a"),
            Err(Error::ContextNotFound(_))
        ));
        assert!(matches!(
            mgr.remove_agent_from_shared_context("nope", "a"),
            Err(Error::ContextNotFound(_))
        ));
    }

    #[test]
    fn set_context_value_requires_object_shape() {
        let mgr = ContextManager::new(None);
        mgr.update_persistent_context("a", json!("scalar"))
            .expect("init");
        let err = mgr
            .set_context_value("a", "x", json!(1))
            .expect_err("not object");
        assert!(matches!(err, Error::InvalidContext(_)));
    }

    #[test]
    fn cleanup_expired_noop_without_expiry_and_statistics_track_counts() {
        let mgr = ContextManager::new(Some(ContextStorage::Local));
        mgr.update_persistent_context("z", json!({})).expect("u");
        assert_eq!(mgr.cleanup_expired_contexts(), 0);
        let stats = mgr.get_context_statistics();
        assert_eq!(stats.persistent_contexts, 1);
        assert!(matches!(stats.storage_backend, Some(ContextStorage::Local)));
    }

    #[test]
    fn export_import_roundtrip() {
        let a = ContextManager::new(None);
        a.update_persistent_context("p", json!({"x": 1}))
            .expect("u");
        a.create_shared_context("s", json!({"y": 2}), vec!["u".to_string()])
            .expect("c");
        let exp = a.export_contexts().expect("exp");
        let b = ContextManager::new(None);
        b.import_contexts(&exp).expect("imp");
        assert_eq!(b.get_context("p").expect("p")["x"], 1);
    }

    #[test]
    fn apply_context_requirements_sets_keys_and_tolerates_missing_shared() {
        let mgr = ContextManager::new(None);
        let req = ContextRequirements {
            persistent_context: false,
            shared_context: HashMap::new(),
            shared_contexts: vec!["missing-shared".to_string()],
            required_context: HashMap::from([("rk".to_string(), "rv".to_string())]),
            context_keys: vec![],
        };
        mgr.apply_context_requirements("agent", &req)
            .expect("apply");
        let v = mgr.get_context_value("agent", "rk").expect("rk");
        assert_eq!(v, json!("rv"));
    }

    #[test]
    fn default_context_manager_matches_local_storage() {
        let mgr = ContextManager::default();
        assert!(matches!(
            mgr.get_storage_backend(),
            Some(ContextStorage::Local)
        ));
    }
}
