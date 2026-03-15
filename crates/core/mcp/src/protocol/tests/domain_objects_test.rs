// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for domain object translation.

use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::message::{Message, MessageType, MessagePriority};
use crate::types::{MCPMessage, MessageId, SecurityMetadata, ProtocolVersion as MCPProtocolVersion};
use crate::protocol::adapter_wire::{DomainObject, ProtocolVersion, WireMessage, WireFormat};
use crate::protocol::domain_objects;

#[tokio::test]
async fn test_message_roundtrip_translation() {
    // Create a test message
    let message = Message {
        id: Uuid::new_v4().to_string(),
        message_type: MessageType::Request,
        priority: MessagePriority::Normal,
        content: "Test integration content".to_string(),
        binary_payload: Some(vec![1, 2, 3, 4, 5]),
        timestamp: Utc::now(),
        in_reply_to: Some("parent-123".to_string()),
        source: "client-test".to_string(),
        destination: "server-test".to_string(),
        context_id: Some("ctx-987".to_string()),
        topic: Some("test-topic".to_string()),
        metadata: {
            let mut map = HashMap::new();
            map.insert("test-key".to_string(), "test-value".to_string());
            map
        },
    };
    
    // Convert to wire format
    let wire_message = message.to_wire_message(ProtocolVersion::V1_0).await.unwrap();
    
    // Verify wire format properties
    assert_eq!(wire_message.version, "1.0");
    assert_eq!(wire_message.format, WireFormat::Json);
    
    // Convert back to domain object
    let decoded = Message::from_wire_message(&wire_message).await.unwrap();
    
    // Verify the round-trip conversion
    assert_eq!(decoded.id, message.id);
    assert_eq!(decoded.message_type, message.message_type);
    assert_eq!(decoded.priority, message.priority);
    assert_eq!(decoded.content, message.content);
    assert_eq!(decoded.binary_payload, message.binary_payload);
    assert_eq!(decoded.in_reply_to, message.in_reply_to);
    assert_eq!(decoded.source, message.source);
    assert_eq!(decoded.destination, message.destination);
    assert_eq!(decoded.context_id, message.context_id);
    assert_eq!(decoded.topic, message.topic);
    assert_eq!(decoded.metadata, message.metadata);
}

#[tokio::test]
async fn test_message_version_transformation() {
    // Create a test message
    let message = Message {
        id: "version-test-123".to_string(),
        message_type: MessageType::Notification,
        priority: MessagePriority::High,
        content: "Version transformation test".to_string(),
        binary_payload: Some(vec![10, 20, 30]),
        timestamp: Utc::now(),
        in_reply_to: None,
        source: "source-system".to_string(),
        destination: "target-system".to_string(),
        context_id: Some("test-context".to_string()),
        topic: Some("version-topic".to_string()),
        metadata: {
            let mut map = HashMap::new();
            map.insert("version".to_string(), "test".to_string());
            map
        },
    };
    
    // Convert to v0.9 wire format
    let wire_message = message.to_wire_message(ProtocolVersion::V0_9).await.unwrap();
    
    // Verify wire format properties
    assert_eq!(wire_message.version, "0.9");
    
    // Parse the data to verify the format
    let data: serde_json::Value = serde_json::from_slice(&wire_message.data).unwrap();
    let obj = data.as_object().unwrap();
    
    // Verify that it's using the legacy format field names
    assert!(obj.contains_key("msg_type"));
    assert!(!obj.contains_key("message_type"));
    assert_eq!(obj.get("msg_type").unwrap().as_str().unwrap(), "notification");
    
    // Verify binary data is encoded as base64
    assert!(obj.contains_key("binary"));
    
    // Convert back to a message
    let decoded = Message::from_wire_message(&wire_message).await.unwrap();
    
    // Verify the round-trip conversion
    assert_eq!(decoded.id, message.id);
    assert_eq!(decoded.message_type, message.message_type);
    assert_eq!(decoded.content, message.content);
    assert_eq!(decoded.binary_payload, message.binary_payload);
}

#[tokio::test]
async fn test_mcpmessage_roundtrip_translation() {
    // Create a test MCPMessage
    let message = MCPMessage {
        id: MessageId("mcpmsg-integration-123".to_string()),
        type_: crate::types::MessageType::Command,
        payload: json!({
            "action": "integrate",
            "params": {
                "target": "domain-objects",
                "mode": "full"
            }
        }),
        metadata: Some(json!({
            "source_id": "test-integration",
            "priority": "high"
        })),
        security: SecurityMetadata {
            encrypted: true,
            signed: true,
            key_id: Some("test-key-integration".to_string()),
            signature: Some("test-signature-abc".to_string()),
        },
        timestamp: Utc::now(),
        version: MCPProtocolVersion("1.0".to_string()),
        trace_id: Some("trace-integration-xyz".to_string()),
    };
    
    // Convert to wire format
    let wire_message = message.to_wire_message(ProtocolVersion::V1_0).await.unwrap();
    
    // Verify wire format properties
    assert_eq!(wire_message.version, "1.0");
    assert_eq!(wire_message.format, WireFormat::Json);
    
    // Convert back to domain object
    let decoded = MCPMessage::from_wire_message(&wire_message).await.unwrap();
    
    // Verify the round-trip conversion
    assert_eq!(decoded.id.0, message.id.0);
    assert_eq!(decoded.type_, message.type_);
    assert_eq!(decoded.payload, message.payload);
    assert!(decoded.security.encrypted);
    assert!(decoded.security.signed);
    assert_eq!(decoded.security.key_id, message.security.key_id);
    assert_eq!(decoded.security.signature, message.security.signature);
    assert_eq!(decoded.trace_id, message.trace_id);
}

#[tokio::test]
async fn test_mcpmessage_version_transformation() {
    // Create a test MCPMessage for legacy conversion
    let message = MCPMessage {
        id: MessageId("legacy-mcpmsg-456".to_string()),
        type_: crate::types::MessageType::Event,
        payload: json!({
            "event_name": "system_status",
            "status": "ready"
        }),
        metadata: None,
        security: SecurityMetadata {
            encrypted: false,
            signed: true,
            key_id: None,
            signature: None,
        },
        timestamp: Utc::now(),
        version: MCPProtocolVersion("0.9".to_string()),
        trace_id: None,
    };
    
    // Convert to legacy wire format
    let wire_message = message.to_wire_message(ProtocolVersion::V0_9).await.unwrap();
    
    // Verify wire format properties
    assert_eq!(wire_message.version, "0.9");
    
    // Parse the data to verify the format
    let data: serde_json::Value = serde_json::from_slice(&wire_message.data).unwrap();
    let obj = data.as_object().unwrap();
    
    // Verify that it's using the legacy format
    assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "legacy-mcpmsg-456");
    assert_eq!(obj.get("type").unwrap().as_str().unwrap(), "event");
    
    // Verify security data
    let security = obj.get("security").unwrap().as_object().unwrap();
    assert_eq!(security.get("encrypted").unwrap().as_bool().unwrap(), false);
    assert_eq!(security.get("signed").unwrap().as_bool().unwrap(), true);
    
    // Convert back to an MCPMessage
    let decoded = MCPMessage::from_wire_message(&wire_message).await.unwrap();
    
    // Verify the round-trip conversion
    assert_eq!(decoded.id.0, message.id.0);
    assert_eq!(decoded.type_, message.type_);
    assert_eq!(decoded.payload, message.payload);
    assert_eq!(decoded.security.encrypted, message.security.encrypted);
    assert_eq!(decoded.security.signed, message.security.signed);
}

#[tokio::test]
async fn test_cross_version_compatibility() {
    // Create a message in legacy format
    let wire_data = json!({
        "id": "compat-test-789",
        "msg_type": "request",
        "content": "Cross-version compatibility test",
        "timestamp": Utc::now().timestamp(),
        "source": "legacy-system",
        "destination": "modern-system",
        "reply_to": "original-request-id",
        "binary": base64::encode([1, 2, 3, 4, 5, 6]),
        "metadata": {
            "legacy_key": "legacy_value",
            "compatibility": "true"
        }
    });
    
    // Create a wire message with v0.9 format
    let wire_message = WireMessage::from_json(ProtocolVersion::V0_9, wire_data).unwrap();
    
    // Convert to a modern Message
    let message = Message::from_wire_message(&wire_message).await.unwrap();
    
    // Verify the conversion worked
    assert_eq!(message.id, "compat-test-789");
    assert_eq!(message.message_type, MessageType::Request);
    assert_eq!(message.content, "Cross-version compatibility test");
    assert_eq!(message.source, "legacy-system");
    assert_eq!(message.destination, "modern-system");
    assert_eq!(message.in_reply_to, Some("original-request-id".to_string()));
    assert_eq!(message.binary_payload, Some(vec![1, 2, 3, 4, 5, 6]));
    
    // Check metadata was preserved
    assert!(message.metadata.contains_key("legacy_key"));
    assert_eq!(message.metadata.get("legacy_key").unwrap(), "legacy_value");
    
    // Now convert back to wire format but with the latest version
    let new_wire_message = message.to_wire_message(ProtocolVersion::Latest).await.unwrap();
    
    // Verify it's using the latest version now
    assert_eq!(new_wire_message.version, "1.0");
    
    // Parse the JSON to confirm the structure
    let new_data: serde_json::Value = serde_json::from_slice(&new_wire_message.data).unwrap();
    let new_obj = new_data.as_object().unwrap();
    
    // Verify it's using the new format
    assert!(new_obj.contains_key("message_type"));
    assert!(!new_obj.contains_key("msg_type"));
} 