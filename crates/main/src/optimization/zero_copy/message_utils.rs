//! Zero-copy message utilities

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Message structure that uses Arc<str> for zero-copy sharing
#[derive(Debug, Clone)]
pub struct ZeroCopyMessage {
    pub message_type: Arc<str>,
    pub content: Arc<str>,
    pub metadata: HashMap<Arc<str>, Arc<str>>,
}

impl ZeroCopyMessage {
    /// Create a new zero-copy message
    pub fn new(message_type: Arc<str>, content: Arc<str>) -> Self {
        Self {
            message_type,
            content,
            metadata: HashMap::new(),
        }
    }

    /// Get message type without cloning
    pub fn get_type(&self) -> &str {
        &self.message_type
    }

    /// Get content without cloning
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Get metadata value without cloning
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v.as_ref())
    }

    /// Add metadata efficiently
    pub fn add_metadata(&mut self, key: Arc<str>, value: Arc<str>) {
        self.metadata.insert(key, value);
    }

    /// Serialize to JSON efficiently
    pub fn to_json(&self) -> serde_json::Result<String> {
        #[derive(Serialize)]
        struct MessageJson<'a> {
            message_type: &'a str,
            content: &'a str,
            metadata: HashMap<&'a str, &'a str>,
        }

        let metadata_refs: HashMap<&str, &str> = self
            .metadata
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect();

        let json_msg = MessageJson {
            message_type: &self.message_type,
            content: &self.content,
            metadata: metadata_refs,
        };

        serde_json::to_string(&json_msg)
    }
} 