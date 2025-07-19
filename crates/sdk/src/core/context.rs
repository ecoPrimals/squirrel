//! Plugin Context Module
//!
//! This module provides context management for plugins running in the sandbox environment.

use crate::error::PluginResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plugin execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Plugin identifier
    pub plugin_id: String,
    /// Session identifier  
    pub session_id: String,
    /// Context data
    pub data: ContextData,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Context data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Key-value data storage
    pub values: HashMap<String, serde_json::Value>,
    /// Timestamp of last update
    pub updated_at: u64,
    /// Version for conflict resolution
    pub version: u64,
}

impl PluginContext {
    /// Create a new plugin context
    ///
    /// This constructor initializes a new plugin context with the specified plugin ID
    /// and session ID. The context provides a way to store and retrieve plugin-specific
    /// data during execution, and associate it with a particular user session.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - A unique identifier for the plugin instance. This should match
    ///                 the ID used when initializing the SDK.
    /// * `session_id` - A unique identifier for the user session. This allows the plugin
    ///                  to maintain separate state for different user sessions.
    ///
    /// # Returns
    ///
    /// Returns a new `PluginContext` instance with empty data and metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::context::PluginContext;
    ///
    /// let context = PluginContext::new(
    ///     "my-plugin".to_string(),
    ///     "user-session-123".to_string()
    /// );
    /// ```
    pub fn new(plugin_id: String, session_id: String) -> Self {
        Self {
            plugin_id,
            session_id,
            data: ContextData::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get a value from context
    pub fn get<T>(&self, key: &str) -> PluginResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(value) = self.data.values.get(key) {
            Ok(Some(serde_json::from_value(value.clone())?))
        } else {
            Ok(None)
        }
    }

    /// Set a value in context
    pub fn set<T>(&mut self, key: &str, value: T) -> PluginResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.data.values.insert(key.to_string(), json_value);
        self.data.updated_at = crate::utils::current_timestamp();
        self.data.version += 1;
        Ok(())
    }

    /// Remove a value from context
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        let result = self.data.values.remove(key);
        if result.is_some() {
            self.data.updated_at = crate::utils::current_timestamp();
            self.data.version += 1;
        }
        result
    }

    /// Clear all context data
    pub fn clear(&mut self) {
        self.data.values.clear();
        self.data.updated_at = crate::utils::current_timestamp();
        self.data.version += 1;
    }
}

impl ContextData {
    /// Create new empty context data
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            updated_at: crate::utils::current_timestamp(),
            version: 1,
        }
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the number of stored values
    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Default for ContextData {
    fn default() -> Self {
        Self::new()
    }
}
