// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A simplified context for plugin testing that doesn't rely on tokio
#[derive(Default)]
pub struct TestPluginContext {
    /// Data stored in the context
    data: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl TestPluginContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a value in the context
    pub fn set<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        if let Ok(mut data) = self.data.write() {
            data.insert(key.to_string(), Box::new(value));
        }
    }

    /// Get a value from the context
    pub fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        if let Ok(data) = self.data.read() {
            data.get(key)
                .and_then(|value| value.downcast_ref::<T>())
                .cloned()
        } else {
            None
        }
    }

    /// Remove a value from the context
    pub fn remove(&self, key: &str) -> bool {
        if let Ok(mut data) = self.data.write() {
            data.remove(key).is_some()
        } else {
            false
        }
    }

    /// Check if the context contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        if let Ok(data) = self.data.read() {
            data.contains_key(key)
        } else {
            false
        }
    }

    /// Get all keys in the context
    pub fn keys(&self) -> Vec<String> {
        if let Ok(data) = self.data.read() {
            data.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Clear all data in the context
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_plugin_context_set_get() {
        let context = TestPluginContext::new();

        // Set a value
        context.set("test_key", "test_value".to_string());

        // Get the value
        let value: Option<String> = context.get("test_key");
        assert_eq!(value, Some("test_value".to_string()));

        // Get a non-existent value
        let value: Option<String> = context.get("non_existent");
        assert_eq!(value, None);
    }

    #[test]
    fn test_test_plugin_context_remove() {
        let context = TestPluginContext::new();

        // Set a value
        context.set("test_key", "test_value".to_string());

        // Remove the value
        let removed = context.remove("test_key");
        assert!(removed);

        // Get the removed value
        let value: Option<String> = context.get("test_key");
        assert_eq!(value, None);

        // Remove a non-existent value
        let removed = context.remove("non_existent");
        assert!(!removed);
    }

    #[test]
    fn test_test_plugin_context_contains_key() {
        let context = TestPluginContext::new();

        // Set a value
        context.set("test_key", "test_value".to_string());

        // Check if the key exists
        let contains = context.contains_key("test_key");
        assert!(contains);

        // Check if a non-existent key exists
        let contains = context.contains_key("non_existent");
        assert!(!contains);
    }
}
