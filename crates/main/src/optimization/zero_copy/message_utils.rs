// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Zero-copy message utilities

use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Message structure that uses `Arc<str>` for zero-copy sharing
#[derive(Debug, Clone)]
pub struct ZeroCopyMessage {
    pub message_type: Arc<str>,
    pub content: Arc<str>,
    pub metadata: HashMap<Arc<str>, Arc<str>>,
}

impl ZeroCopyMessage {
    /// Create a new zero-copy message
    #[must_use]
    pub fn new(message_type: Arc<str>, content: Arc<str>) -> Self {
        Self {
            message_type,
            content,
            metadata: HashMap::new(),
        }
    }

    /// Get message type without cloning
    #[must_use]
    pub fn get_type(&self) -> &str {
        &self.message_type
    }

    /// Get content without cloning
    #[must_use]
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Get metadata value without cloning
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_message() {
        let msg = ZeroCopyMessage::new(Arc::from("test_type"), Arc::from("hello world"));
        assert_eq!(msg.get_type(), "test_type");
        assert_eq!(msg.get_content(), "hello world");
        assert!(msg.metadata.is_empty());
    }

    #[test]
    fn test_get_type() {
        let msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("data"));
        assert_eq!(msg.get_type(), "request");
    }

    #[test]
    fn test_get_content() {
        let msg = ZeroCopyMessage::new(Arc::from("t"), Arc::from("content here"));
        assert_eq!(msg.get_content(), "content here");
    }

    #[test]
    fn test_add_and_get_metadata() {
        let mut msg = ZeroCopyMessage::new(Arc::from("type"), Arc::from("content"));
        msg.add_metadata(Arc::from("key1"), Arc::from("value1"));
        msg.add_metadata(Arc::from("key2"), Arc::from("value2"));

        assert_eq!(msg.get_metadata("key1"), Some("value1"));
        assert_eq!(msg.get_metadata("key2"), Some("value2"));
        assert_eq!(msg.get_metadata("missing"), None);
    }

    #[test]
    fn test_metadata_overwrite() {
        let mut msg = ZeroCopyMessage::new(Arc::from("type"), Arc::from("content"));
        msg.add_metadata(Arc::from("key"), Arc::from("old"));
        msg.add_metadata(Arc::from("key"), Arc::from("new"));

        assert_eq!(msg.get_metadata("key"), Some("new"));
    }

    #[test]
    fn test_to_json() {
        let mut msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("payload data"));
        msg.add_metadata(Arc::from("trace_id"), Arc::from("abc123"));

        let json = msg.to_json().unwrap();
        assert!(json.contains("request"));
        assert!(json.contains("payload data"));
        assert!(json.contains("trace_id"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn test_to_json_empty_metadata() {
        let msg = ZeroCopyMessage::new(Arc::from("type"), Arc::from("content"));
        let json = msg.to_json().unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["message_type"], "type");
        assert_eq!(parsed["content"], "content");
        assert!(parsed["metadata"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_clone() {
        let mut msg = ZeroCopyMessage::new(Arc::from("type"), Arc::from("content"));
        msg.add_metadata(Arc::from("key"), Arc::from("value"));

        let cloned = msg.clone();
        assert_eq!(cloned.get_type(), msg.get_type());
        assert_eq!(cloned.get_content(), msg.get_content());
        assert_eq!(cloned.get_metadata("key"), msg.get_metadata("key"));
    }

    #[test]
    fn test_zero_copy_sharing() {
        let msg_type: Arc<str> = Arc::from("shared_type");
        let content: Arc<str> = Arc::from("shared_content");

        let msg1 = ZeroCopyMessage::new(Arc::clone(&msg_type), Arc::clone(&content));
        let msg2 = ZeroCopyMessage::new(Arc::clone(&msg_type), Arc::clone(&content));

        // Both share the same underlying data
        assert_eq!(msg1.get_type(), msg2.get_type());
        assert_eq!(msg1.get_content(), msg2.get_content());

        // Arc strong count should be 3 (original + 2 messages)
        assert_eq!(Arc::strong_count(&msg_type), 3);
    }
}
