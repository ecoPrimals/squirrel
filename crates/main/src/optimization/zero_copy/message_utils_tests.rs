// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for zero-copy message utilities

#[cfg(test)]
mod tests {
    use super::super::message_utils::*;
    use std::sync::Arc;

    #[test]
    fn test_zero_copy_message_new() {
        let msg_type: Arc<str> = Arc::from("request");
        let content: Arc<str> = Arc::from("test content");

        let msg = ZeroCopyMessage::new(msg_type, content);

        assert_eq!(msg.get_type(), "request");
        assert_eq!(msg.get_content(), "test content");
        assert_eq!(msg.metadata.len(), 0);
    }

    #[test]
    fn test_zero_copy_message_get_type() {
        let msg = ZeroCopyMessage::new(Arc::from("notification"), Arc::from("content"));

        assert_eq!(msg.get_type(), "notification");
    }

    #[test]
    fn test_zero_copy_message_get_content() {
        let msg = ZeroCopyMessage::new(Arc::from("type"), Arc::from("important content"));

        assert_eq!(msg.get_content(), "important content");
    }

    #[test]
    fn test_zero_copy_message_add_metadata() {
        let mut msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("content"));

        msg.add_metadata(Arc::from("user_id"), Arc::from("123"));
        msg.add_metadata(Arc::from("timestamp"), Arc::from("2025-11-20"));

        assert_eq!(msg.metadata.len(), 2);
        assert_eq!(msg.get_metadata("user_id"), Some("123"));
        assert_eq!(msg.get_metadata("timestamp"), Some("2025-11-20"));
    }

    #[test]
    fn test_zero_copy_message_get_metadata_missing() {
        let msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("content"));

        assert_eq!(msg.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_zero_copy_message_clone() {
        let mut msg1 = ZeroCopyMessage::new(Arc::from("request"), Arc::from("content"));
        msg1.add_metadata(Arc::from("key"), Arc::from("value"));

        let msg2 = msg1.clone();

        assert_eq!(msg2.get_type(), "request");
        assert_eq!(msg2.get_content(), "content");
        assert_eq!(msg2.get_metadata("key"), Some("value"));
    }

    #[test]
    fn test_zero_copy_message_to_json() {
        let mut msg = ZeroCopyMessage::new(Arc::from("test"), Arc::from("hello"));
        msg.add_metadata(Arc::from("author"), Arc::from("system"));

        let json = msg.to_json().unwrap();

        assert!(json.contains("\"message_type\":\"test\""));
        assert!(json.contains("\"content\":\"hello\""));
        assert!(json.contains("\"author\":\"system\""));
    }

    #[test]
    fn test_zero_copy_message_to_json_empty_metadata() {
        let msg = ZeroCopyMessage::new(Arc::from("simple"), Arc::from("message"));

        let json = msg.to_json().unwrap();

        assert!(json.contains("\"message_type\":\"simple\""));
        assert!(json.contains("\"content\":\"message\""));
        assert!(json.contains("\"metadata\":{}"));
    }

    #[test]
    fn test_zero_copy_message_multiple_metadata() {
        let mut msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("data"));

        for i in 0..10 {
            msg.add_metadata(
                Arc::from(format!("key{}", i)),
                Arc::from(format!("value{}", i)),
            );
        }

        assert_eq!(msg.metadata.len(), 10);

        for i in 0..10 {
            assert_eq!(
                msg.get_metadata(&format!("key{}", i)),
                Some(format!("value{}", i).as_str())
            );
        }
    }

    #[test]
    fn test_zero_copy_message_update_metadata() {
        let mut msg = ZeroCopyMessage::new(Arc::from("request"), Arc::from("content"));

        msg.add_metadata(Arc::from("counter"), Arc::from("1"));
        msg.add_metadata(Arc::from("counter"), Arc::from("2"));
        msg.add_metadata(Arc::from("counter"), Arc::from("3"));

        assert_eq!(msg.metadata.len(), 1);
        assert_eq!(msg.get_metadata("counter"), Some("3"));
    }

    #[test]
    fn test_zero_copy_message_empty_strings() {
        let msg = ZeroCopyMessage::new(Arc::from(""), Arc::from(""));

        assert_eq!(msg.get_type(), "");
        assert_eq!(msg.get_content(), "");
    }

    #[test]
    fn test_zero_copy_message_unicode() {
        let mut msg = ZeroCopyMessage::new(Arc::from("通知"), Arc::from("你好世界"));
        msg.add_metadata(Arc::from("言語"), Arc::from("中文"));

        assert_eq!(msg.get_type(), "通知");
        assert_eq!(msg.get_content(), "你好世界");
        assert_eq!(msg.get_metadata("言語"), Some("中文"));

        let json = msg.to_json().unwrap();
        assert!(json.contains("你好世界"));
    }

    #[test]
    fn test_zero_copy_message_large_content() {
        let large_content = "x".repeat(100000);
        let msg = ZeroCopyMessage::new(Arc::from("large"), Arc::from(large_content.as_str()));

        assert_eq!(msg.get_content().len(), 100000);
    }

    #[test]
    fn test_zero_copy_message_json_escaping() {
        let mut msg = ZeroCopyMessage::new(
            Arc::from("test"),
            Arc::from(r#"content with "quotes" and \backslashes\"#),
        );
        msg.add_metadata(Arc::from("special"), Arc::from(r#"value with "quotes""#));

        let json = msg.to_json().unwrap();

        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn test_zero_copy_message_sharing() {
        let msg_type: Arc<str> = Arc::from("shared");
        let content: Arc<str> = Arc::from("shared content");

        // Create multiple messages sharing the same Arc<str>
        let msg1 = ZeroCopyMessage::new(msg_type.clone(), content.clone());
        let msg2 = ZeroCopyMessage::new(msg_type.clone(), content.clone());

        assert_eq!(msg1.get_type(), msg2.get_type());
        assert_eq!(msg1.get_content(), msg2.get_content());

        // Verify they're actually sharing (pointer equality)
        assert!(Arc::ptr_eq(&msg1.message_type, &msg2.message_type));
        assert!(Arc::ptr_eq(&msg1.content, &msg2.content));
    }
}
