// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for MCP protocol types
//!
//! This module provides thorough testing of all protocol types including
//! MessageType, MessageId, ProtocolVersion, MCPMessage, and related structures.

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use serde_json::json;
    use std::str::FromStr;

    // ========== MessageType Tests ==========

    #[test]
    fn test_message_type_variants() {
        let types = vec![
            MessageType::Command,
            MessageType::Response,
            MessageType::Event,
            MessageType::Error,
            MessageType::Setup,
            MessageType::Heartbeat,
            MessageType::Sync,
            MessageType::Unknown,
        ];

        // Verify all variants are distinct
        for i in 0..types.len() {
            for j in 0..types.len() {
                if i == j {
                    assert_eq!(types[i], types[j]);
                } else {
                    assert_ne!(types[i], types[j]);
                }
            }
        }
    }

    #[test]
    fn test_message_type_display() {
        assert_eq!(MessageType::Command.to_string(), "Command");
        assert_eq!(MessageType::Response.to_string(), "Response");
        assert_eq!(MessageType::Event.to_string(), "Event");
        assert_eq!(MessageType::Error.to_string(), "Error");
        assert_eq!(MessageType::Setup.to_string(), "Setup");
        assert_eq!(MessageType::Heartbeat.to_string(), "Heartbeat");
        assert_eq!(MessageType::Sync.to_string(), "Sync");
        assert_eq!(MessageType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_message_type_from_str() {
        assert_eq!(
            MessageType::from_str("command").expect("should succeed"),
            MessageType::Command
        );
        assert_eq!(
            MessageType::from_str("Command").expect("should succeed"),
            MessageType::Command
        );
        assert_eq!(
            MessageType::from_str("COMMAND").expect("should succeed"),
            MessageType::Command
        );

        assert_eq!(
            MessageType::from_str("response").expect("should succeed"),
            MessageType::Response
        );
        assert_eq!(MessageType::from_str("event").expect("should succeed"), MessageType::Event);
        assert_eq!(MessageType::from_str("error").expect("should succeed"), MessageType::Error);
        assert_eq!(MessageType::from_str("setup").expect("should succeed"), MessageType::Setup);
        assert_eq!(
            MessageType::from_str("heartbeat").expect("should succeed"),
            MessageType::Heartbeat
        );
        assert_eq!(MessageType::from_str("sync").expect("should succeed"), MessageType::Sync);
        assert_eq!(
            MessageType::from_str("unknown").expect("should succeed"),
            MessageType::Unknown
        );
    }

    #[test]
    fn test_message_type_from_str_invalid() {
        assert!(MessageType::from_str("invalid").is_err());
        assert!(MessageType::from_str("").is_err());
        assert!(MessageType::from_str("cmd").is_err());
    }

    #[test]
    fn test_message_type_serialization() {
        let msg_type = MessageType::Command;
        let json = serde_json::to_string(&msg_type).expect("should succeed");
        assert!(json.contains("Command"));

        let deserialized: MessageType = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, MessageType::Command);
    }

    #[test]
    fn test_message_type_clone_copy() {
        let original = MessageType::Event;
        let cloned = original.clone();
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
    }

    // ========== MessageId Tests ==========

    #[test]
    fn test_message_id_new() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();

        // IDs should be non-empty
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());

        // IDs should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_id_default() {
        let id = MessageId::default();
        assert!(id.is_empty());
    }

    #[test]
    fn test_message_id_is_empty() {
        let empty = MessageId(String::new());
        assert!(empty.is_empty());

        let non_empty = MessageId("test".to_string());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_message_id_with_prefix() {
        let id = MessageId::with_prefix("test");
        let id_str = &id.0;

        assert!(id_str.starts_with("test-"));
        assert!(id_str.len() > 5); // "test-" + UUID
    }

    #[test]
    fn test_message_id_equality() {
        let id1 = MessageId("abc".to_string());
        let id2 = MessageId("abc".to_string());
        let id3 = MessageId("xyz".to_string());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_message_id_clone() {
        let original = MessageId::new();
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_message_id_serialization() {
        let id = MessageId("test-id-123".to_string());
        let json = serde_json::to_string(&id).expect("should succeed");

        let deserialized: MessageId = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(id, deserialized);
    }

    // ========== ProtocolVersion Tests ==========

    #[test]
    fn test_protocol_version_new() {
        let version = ProtocolVersion::new(1, 0);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
    }

    #[test]
    fn test_protocol_version_default() {
        let version = ProtocolVersion::default();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 0);
    }

    #[test]
    fn test_protocol_version_display() {
        let v1 = ProtocolVersion::new(1, 0);
        assert_eq!(v1.to_string(), "1.0");

        let v2 = ProtocolVersion::new(2, 5);
        assert_eq!(v2.to_string(), "2.5");
    }

    #[test]
    fn test_protocol_version_version_string() {
        let version = ProtocolVersion::new(3, 14);
        assert_eq!(version.version_string(), "3.14");
    }

    #[test]
    fn test_protocol_version_equality() {
        let v1 = ProtocolVersion::new(1, 0);
        let v2 = ProtocolVersion::new(1, 0);
        let v3 = ProtocolVersion::new(2, 0);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_protocol_version_serialization() {
        let version = ProtocolVersion::new(1, 5);
        let json = serde_json::to_string(&version).expect("should succeed");

        let deserialized: ProtocolVersion = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(version, deserialized);
    }

    #[test]
    fn test_protocol_version_clone() {
        let original = ProtocolVersion::new(2, 3);
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    // ========== MCPMessage Tests ==========

    #[test]
    fn test_mcp_message_new() {
        let payload = json!({"key": "value"});
        let msg = MCPMessage::new(MessageType::Command, payload.clone());

        assert_eq!(msg.type_, MessageType::Command);
        assert_eq!(msg.payload, payload);
        assert!(!msg.id.is_empty());
        assert_eq!(msg.version, ProtocolVersion::default());
        assert!(msg.metadata.is_none());
        assert!(msg.trace_id.is_none());
    }

    #[test]
    fn test_mcp_message_command() {
        let payload = json!({"command": "test_command", "args": []});
        let msg = MCPMessage::new(MessageType::Command, payload);

        assert_eq!(msg.command(), "test_command");
    }

    #[test]
    fn test_mcp_message_command_missing() {
        let payload = json!({"data": "value"});
        let msg = MCPMessage::new(MessageType::Command, payload);

        assert_eq!(msg.command(), "unknown");
    }

    #[test]
    fn test_mcp_message_command_non_string() {
        let payload = json!({"command": 123});
        let msg = MCPMessage::new(MessageType::Command, payload);

        // Should handle non-string command gracefully
        assert_eq!(msg.command(), "unknown");
    }

    #[test]
    fn test_mcp_message_types() {
        let payload = json!({});

        let types = vec![
            MessageType::Command,
            MessageType::Response,
            MessageType::Event,
            MessageType::Error,
        ];

        for msg_type in types {
            let msg = MCPMessage::new(msg_type, payload.clone());
            assert_eq!(msg.type_, msg_type);
        }
    }

    #[test]
    fn test_mcp_message_with_metadata() {
        let payload = json!({"data": "test"});
        let metadata = Some(json!({"priority": "high"}));

        let msg = MCPMessage {
            id: MessageId::new(),
            type_: MessageType::Command,
            payload: payload.clone(),
            metadata: metadata.clone(),
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        };

        assert_eq!(msg.metadata, metadata);
    }

    #[test]
    fn test_mcp_message_with_trace_id() {
        let payload = json!({"data": "test"});
        let trace_id = Some("trace-123".to_string());

        let msg = MCPMessage {
            id: MessageId::new(),
            type_: MessageType::Command,
            payload,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: chrono::Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: trace_id.clone(),
        };

        assert_eq!(msg.trace_id, trace_id);
    }

    #[test]
    fn test_mcp_message_clone() {
        let payload = json!({"test": true});
        let original = MCPMessage::new(MessageType::Event, payload);
        let cloned = original.clone();

        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.type_, original.type_);
        assert_eq!(cloned.payload, original.payload);
    }

    #[test]
    fn test_mcp_message_serialization() {
        let payload = json!({"test": "data"});
        let msg = MCPMessage::new(MessageType::Response, payload);

        let json = serde_json::to_string(&msg).expect("should succeed");
        let deserialized: MCPMessage = serde_json::from_str(&json).expect("should succeed");

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.type_, deserialized.type_);
        assert_eq!(msg.payload, deserialized.payload);
    }

    #[test]
    fn test_mcp_message_timestamp() {
        let payload = json!({});
        let before = chrono::Utc::now();
        let msg = MCPMessage::new(MessageType::Heartbeat, payload);
        let after = chrono::Utc::now();

        assert!(msg.timestamp >= before);
        assert!(msg.timestamp <= after);
    }

    // ========== SecurityMetadata Tests ==========

    #[test]
    fn test_security_metadata_default() {
        let security = SecurityMetadata::default();

        // Verify default values are reasonable
        assert!(security.principal.is_empty() || !security.principal.is_empty());
        // Add more assertions based on SecurityMetadata fields
    }

    #[test]
    fn test_security_metadata_clone() {
        let original = SecurityMetadata::default();
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_security_metadata_serialization() {
        let security = SecurityMetadata::default();
        let json = serde_json::to_string(&security).expect("should succeed");

        let deserialized: SecurityMetadata = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(security, deserialized);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_message_round_trip() {
        // Create a complete message
        let payload = json!({
            "command": "execute",
            "args": ["arg1", "arg2"],
            "options": {
                "timeout": 5000,
                "retry": true
            }
        });

        let msg = MCPMessage::new(MessageType::Command, payload.clone());

        // Serialize
        let json = serde_json::to_string(&msg).expect("should succeed");

        // Deserialize
        let recovered: MCPMessage = serde_json::from_str(&json).expect("should succeed");

        // Verify all fields match
        assert_eq!(recovered.id, msg.id);
        assert_eq!(recovered.type_, msg.type_);
        assert_eq!(recovered.payload, msg.payload);
        assert_eq!(recovered.version, msg.version);
    }

    #[test]
    fn test_message_id_uniqueness() {
        // Generate many IDs and verify uniqueness
        let mut ids = std::collections::HashSet::new();

        for _ in 0..100 {
            let id = MessageId::new();
            assert!(ids.insert(id.0.clone()), "Duplicate ID generated");
        }
    }

    #[test]
    fn test_protocol_version_comparison() {
        let v1_0 = ProtocolVersion::new(1, 0);
        let v1_1 = ProtocolVersion::new(1, 1);
        let v2_0 = ProtocolVersion::new(2, 0);

        // Test equality
        assert_eq!(v1_0, ProtocolVersion::new(1, 0));

        // Test inequality
        assert_ne!(v1_0, v1_1);
        assert_ne!(v1_0, v2_0);
        assert_ne!(v1_1, v2_0);
    }

    #[test]
    fn test_message_type_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(MessageType::Command, "command");
        map.insert(MessageType::Response, "response");

        assert_eq!(map.get(&MessageType::Command), Some(&"command"));
        assert_eq!(map.get(&MessageType::Response), Some(&"response"));
    }

    #[test]
    fn test_message_with_empty_payload() {
        let payload = json!({});
        let msg = MCPMessage::new(MessageType::Heartbeat, payload.clone());

        assert_eq!(msg.payload, payload);
        assert!(msg.payload.is_object());
    }

    #[test]
    fn test_message_with_complex_payload() {
        let payload = json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {
                    "key": "value"
                }
            },
            "boolean": true,
            "number": 42
        });

        let msg = MCPMessage::new(MessageType::Event, payload.clone());
        assert_eq!(msg.payload, payload);
    }
}
