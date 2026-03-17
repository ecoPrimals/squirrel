// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    ///   the ID used when initializing the SDK.
    /// * `session_id` - A unique identifier for the user session. This allows the plugin
    ///   to maintain separate state for different user sessions.
    ///
    /// # Returns
    ///
    /// Returns a new `PluginContext` instance with empty data and metadata.
    ///
    /// # Examples
    ///
    /// ```ignore
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_data_new() {
        let data = ContextData::new();
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);
        assert_eq!(data.version, 1);
        assert!(data.updated_at > 0);
    }

    #[test]
    fn test_context_data_default() {
        let data = ContextData::default();
        assert!(data.is_empty());
        assert_eq!(data.version, 1);
    }

    #[test]
    fn test_context_data_serde() {
        let data = ContextData::new();
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ContextData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.version, data.version);
        assert!(deserialized.is_empty());
    }

    #[test]
    fn test_plugin_context_new() {
        let ctx = PluginContext::new("plugin-1".to_string(), "session-abc".to_string());
        assert_eq!(ctx.plugin_id, "plugin-1");
        assert_eq!(ctx.session_id, "session-abc");
        assert!(ctx.data.is_empty());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_plugin_context_set_and_get() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("name", "test_value".to_string()).unwrap();
        let result: Option<String> = ctx.get("name").unwrap();
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[test]
    fn test_plugin_context_get_missing_key() {
        let ctx = PluginContext::new("p".to_string(), "s".to_string());
        let result: Option<String> = ctx.get("missing").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_plugin_context_set_increments_version() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        let initial_version = ctx.data.version;
        ctx.set("key", 42).unwrap();
        assert_eq!(ctx.data.version, initial_version + 1);
    }

    #[test]
    fn test_plugin_context_set_updates_timestamp() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        let initial_ts = ctx.data.updated_at;
        // Small sleep to ensure timestamp changes (ms resolution)
        std::thread::sleep(std::time::Duration::from_millis(10));
        ctx.set("key", "value").unwrap();
        assert!(ctx.data.updated_at >= initial_ts);
    }

    #[test]
    fn test_plugin_context_remove() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key", "value").unwrap();
        let removed = ctx.remove("key");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), serde_json::json!("value"));
        let result: Option<String> = ctx.get("key").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_plugin_context_remove_missing() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        let removed = ctx.remove("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_plugin_context_remove_increments_version() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key", "value").unwrap();
        let version_after_set = ctx.data.version;
        ctx.remove("key");
        assert_eq!(ctx.data.version, version_after_set + 1);
    }

    #[test]
    fn test_plugin_context_clear() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key1", "value1").unwrap();
        ctx.set("key2", "value2").unwrap();
        assert_eq!(ctx.data.len(), 2);
        ctx.clear();
        assert!(ctx.data.is_empty());
        assert_eq!(ctx.data.len(), 0);
    }

    #[test]
    fn test_plugin_context_clear_increments_version() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key", "value").unwrap();
        let version_after_set = ctx.data.version;
        ctx.clear();
        assert_eq!(ctx.data.version, version_after_set + 1);
    }

    #[test]
    fn test_plugin_context_set_various_types() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("string", "hello").unwrap();
        ctx.set("number", 42).unwrap();
        ctx.set("float", 2.5).unwrap();
        ctx.set("bool", true).unwrap();
        ctx.set("vec", vec![1, 2, 3]).unwrap();

        let string_val: Option<String> = ctx.get("string").unwrap();
        assert_eq!(string_val, Some("hello".to_string()));

        let number_val: Option<i32> = ctx.get("number").unwrap();
        assert_eq!(number_val, Some(42));

        let float_val: Option<f64> = ctx.get("float").unwrap();
        assert!((float_val.unwrap() - 2.5).abs() < f64::EPSILON);

        let bool_val: Option<bool> = ctx.get("bool").unwrap();
        assert_eq!(bool_val, Some(true));

        let vec_val: Option<Vec<i32>> = ctx.get("vec").unwrap();
        assert_eq!(vec_val, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_plugin_context_serde() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key", "value").unwrap();
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: PluginContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.plugin_id, "p");
        assert_eq!(deserialized.session_id, "s");
        assert_eq!(deserialized.data.len(), 1);
    }

    #[test]
    fn test_plugin_context_overwrite_key() {
        let mut ctx = PluginContext::new("p".to_string(), "s".to_string());
        ctx.set("key", "first").unwrap();
        ctx.set("key", "second").unwrap();
        let result: Option<String> = ctx.get("key").unwrap();
        assert_eq!(result, Some("second".to_string()));
        assert_eq!(ctx.data.len(), 1);
    }
}
