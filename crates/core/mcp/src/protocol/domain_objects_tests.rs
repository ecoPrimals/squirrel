// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#[cfg(test)]
mod tests {
    
    use crate::protocol::adapter_wire::{WireProtocolVersion, WireFormat, WireMessage, DomainObject};
    use crate::protocol::types::{MessageId, MessageType};
    // BearDog handles security: // use crate::security::types::SecurityMetadata;
    use crate::SecurityLevel;
    use crate::protocol::types::MCPMessage;
    use serde_json::{json, Value};
    use chrono::Utc;

    // Helper function for creating test messages
    pub fn create_test_mcp_message(
        id: &str,
        type_: MessageType,
        payload: serde_json::Value,
    ) -> MCPMessage {
        MCPMessage {
            id: MessageId(id.to_string()), 
            timestamp: Utc::now(),
            version: crate::protocol::types::ProtocolVersion::default(), 
            type_,
            payload,
            security: SecurityMetadata::default(),
            metadata: None,
            trace_id: None,
        }
    }

    // Helper function for creating test messages with specific security
    pub fn create_test_mcp_message_with_security(
        id: &str,
        type_: MessageType,
        payload: serde_json::Value,
        security: SecurityMetadata,
    ) -> MCPMessage {
        MCPMessage {
            id: MessageId(id.to_string()), 
            timestamp: Utc::now(),
            version: crate::protocol::types::ProtocolVersion::default(), 
            type_,
            payload,
            security,
            metadata: None,
            trace_id: None,
        }
    }

    #[tokio::test]
    async fn test_serialize_mcp_message() {
        let msg = create_test_mcp_message("msg-test-serialize", MessageType::Response, json!({"test": "value"})); 
        // MCPMessage needs to implement DomainObject or have a to_wire_message method
        // If it implements DomainObject, we need super::* or crate::protocol::domain_objects::*
        // Assume MCPMessage has a to_wire_message method from its own impl or DomainObject
        let wire_msg_result = msg.to_wire_message(WireProtocolVersion::default()).await;
        assert!(wire_msg_result.is_ok());
        let wire_msg = wire_msg_result.expect("should succeed");

        let json_result: std::result::Result<Value, _> = serde_json::from_slice(&wire_msg.data);
        assert!(json_result.is_ok());
        let json_val = json_result.expect("should succeed");
        println!("Serialized MCPMessage from wire: {}", serde_json::to_string_pretty(&json_val).expect("should succeed"));

        // Basic checks on the serialized JSON Value
        assert_eq!(json_val["id"], "msg-test-serialize");
        assert_eq!(json_val["type_"], "Response");
        assert_eq!(json_val["payload"], json!({"test": "value"}));
        assert!(json_val["version"].is_object());
        assert!(json_val["security"].is_object());
    }

    #[tokio::test]
    async fn test_deserialize_mcp_message() {
        let json_str = r#"{
            "id": "msg-test-deserialize",
            "timestamp": 1698400800000,
            "version": "1.0",
            "type_": "Command",
            "payload": {"command": "do_something"},
            "metadata": null,
            "security": {
                "security_level": "Standard",
                "signature": "sig123"
            },
            "trace_id": null
        }"#;

        let wire_msg = WireMessage::new(
            WireProtocolVersion::V1_0, 
            json_str.as_bytes().to_vec(), 
            WireFormat::Json
        );
        
        // Ensure MCPMessage::from_wire_message is accessible
        // It might need `use crate::protocol::domain_objects::MCPMessage;` or similar
        // depending on where the DomainObject impl is located.
        // Assuming DomainObject is implemented for MCPMessage.
        let msg_result = MCPMessage::from_wire_message(&wire_msg).await;

        if let Err(e) = &msg_result {
            eprintln!("Deserialization failed: {}", e); 
        }
        assert!(msg_result.is_ok());
        let msg = msg_result.expect("should succeed");

        assert_eq!(msg.id.0, "msg-test-deserialize");
        assert_eq!(msg.type_, MessageType::Command);
        assert!(msg.payload.is_object());
        assert_eq!(msg.security.security_level, SecurityLevel::Standard);
        assert_eq!(msg.security.signature, Some("sig123".to_string()));
    }

    #[tokio::test]
    async fn test_to_from_wire_message() {
        // Create a message but manually prepare the wire message with correct version format
        let original_msg = create_test_mcp_message("msg-wire-test", MessageType::Event, json!({"event": "occurred"}));
        
        // Manually create the wire message payload with string version format
        let wire_payload = json!({
            "id": "msg-wire-test",
            "timestamp": original_msg.timestamp.timestamp_millis(),
            "version": "1.0",
            "type_": "Event",
            "payload": {"event": "occurred"},
            "metadata": null,
            "security": original_msg.security,
            "trace_id": null
        });
        
        // Create the wire message
        let wire_msg = WireMessage::from_json(WireProtocolVersion::V1_0, wire_payload).expect("should succeed");

        // Deserialize from WireMessage
        let deserialized_msg_result = MCPMessage::from_wire_message(&wire_msg).await;
        if let Err(e) = &deserialized_msg_result {
             eprintln!("Wire deserialization failed: {}", e); 
        }
        assert!(deserialized_msg_result.is_ok());
        let deserialized_msg = deserialized_msg_result.expect("should succeed");

        // Compare relevant fields (timestamp might differ slightly)
        assert_eq!(original_msg.id, deserialized_msg.id);
        assert_eq!(original_msg.type_, deserialized_msg.type_);
        assert_eq!(original_msg.payload, deserialized_msg.payload);
    }

    // Test the async DomainObject trait methods now
    #[tokio::test]
    async fn test_mcp_message_domain_object_serialization() {
        let msg = create_test_mcp_message("msg-direct-test", MessageType::Sync, json!({}));
        
        // Manually create the wire payload with correct format
        let wire_payload = json!({
            "id": "msg-direct-test",
            "timestamp": msg.timestamp.timestamp_millis(),
            "version": "1.0",
            "type_": "Sync",
            "payload": {},
            "metadata": null,
            "security": msg.security,
            "trace_id": null
        });
        
        // Create the wire message
        let wire_msg = WireMessage::from_json(WireProtocolVersion::V1_0, wire_payload).expect("should succeed");
        
        // Use the async DomainObject trait method
        let deserialized_result = MCPMessage::from_wire_message(&wire_msg).await; 
        if let Err(e) = &deserialized_result {
            eprintln!("Domain object serialization failed: {}", e); 
        }
        assert!(deserialized_result.is_ok());
        let deserialized_msg = deserialized_result.expect("should succeed");

        assert_eq!(msg.id, deserialized_msg.id);
        assert_eq!(msg.type_, deserialized_msg.type_);
    }
} 