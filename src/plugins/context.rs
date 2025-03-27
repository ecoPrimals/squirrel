// Plugin Context Module
//
// This module defines the context passed to plugins during execution.

use std::collections::HashMap;
use std::sync::Arc;
use std::any::Any;
use tokio::sync::RwLock;

/// A context that plugins can use to store and retrieve data
#[derive(Default)]
pub struct PluginContext {
    /// Data stored in the context
    data: Arc<RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>>,
}

impl PluginContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set a value in the context
    pub async fn set<T: Send + Sync + 'static>(&self, key: &str, value: T) {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), Box::new(value));
    }
    
    /// Get a value from the context
    pub async fn get<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        let data = self.data.read().await;
        data.get(key)
            .and_then(|value| value.downcast_ref::<T>())
            .cloned()
    }
    
    /// Remove a value from the context
    pub async fn remove(&self, key: &str) -> bool {
        let mut data = self.data.write().await;
        data.remove(key).is_some()
    }
    
    /// Check if the context contains a key
    pub async fn contains_key(&self, key: &str) -> bool {
        let data = self.data.read().await;
        data.contains_key(key)
    }
    
    /// Get all keys in the context
    pub async fn keys(&self) -> Vec<String> {
        let data = self.data.read().await;
        data.keys().cloned().collect()
    }
    
    /// Clear all data in the context
    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_context_set_get() {
        let context = PluginContext::new();
        
        // Set a value
        context.set("test_key", "test_value").await;
        
        // Get the value
        let value: Option<String> = context.get("test_key").await;
        assert_eq!(value, Some("test_value".to_string()));
        
        // Get a non-existent value
        let value: Option<String> = context.get("non_existent").await;
        assert_eq!(value, None);
    }
    
    #[tokio::test]
    async fn test_plugin_context_remove() {
        let context = PluginContext::new();
        
        // Set a value
        context.set("test_key", "test_value").await;
        
        // Remove the value
        let removed = context.remove("test_key").await;
        assert!(removed);
        
        // Get the removed value
        let value: Option<String> = context.get("test_key").await;
        assert_eq!(value, None);
        
        // Remove a non-existent value
        let removed = context.remove("non_existent").await;
        assert!(!removed);
    }
    
    #[tokio::test]
    async fn test_plugin_context_contains_key() {
        let context = PluginContext::new();
        
        // Set a value
        context.set("test_key", "test_value").await;
        
        // Check if the key exists
        let contains = context.contains_key("test_key").await;
        assert!(contains);
        
        // Check if a non-existent key exists
        let contains = context.contains_key("non_existent").await;
        assert!(!contains);
    }
    
    #[tokio::test]
    async fn test_plugin_context_keys() {
        let context = PluginContext::new();
        
        // Set some values
        context.set("key1", "value1").await;
        context.set("key2", "value2").await;
        context.set("key3", "value3").await;
        
        // Get all keys
        let keys = context.keys().await;
        
        // Verify that all keys are present
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }
    
    #[tokio::test]
    async fn test_plugin_context_clear() {
        let context = PluginContext::new();
        
        // Set some values
        context.set("key1", "value1").await;
        context.set("key2", "value2").await;
        
        // Clear the context
        context.clear().await;
        
        // Verify that all keys are removed
        let keys = context.keys().await;
        assert_eq!(keys.len(), 0);
        
        // Verify that a specific key is removed
        let contains = context.contains_key("key1").await;
        assert!(!contains);
    }
    
    #[tokio::test]
    async fn test_plugin_context_different_types() {
        let context = PluginContext::new();
        
        // Set values of different types
        context.set("string", "test_value".to_string()).await;
        context.set("integer", 42).await;
        context.set("float", 3.14).await;
        context.set("boolean", true).await;
        
        // Get values with the correct type
        let string_value: Option<String> = context.get("string").await;
        let integer_value: Option<i32> = context.get("integer").await;
        let float_value: Option<f64> = context.get("float").await;
        let boolean_value: Option<bool> = context.get("boolean").await;
        
        // Verify the values
        assert_eq!(string_value, Some("test_value".to_string()));
        assert_eq!(integer_value, Some(42));
        assert_eq!(float_value, Some(3.14));
        assert_eq!(boolean_value, Some(true));
        
        // Get a value with the wrong type
        let wrong_type: Option<u32> = context.get("string").await;
        assert_eq!(wrong_type, None);
    }
} 