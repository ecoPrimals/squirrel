#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::adapter_wire::{ProtocolVersion as WireProtocolVersion, WireFormat, WireMessage, DomainObject};
    use crate::protocol::types::{MessageId, MessageType, ProtocolVersion, SecurityMetadata, SecurityLevel};
    use crate::error::Result;
    use serde_json::{json, Value};
    use chrono::Utc;
    use crate::types::MCPMessage; // Make sure MCPMessage is in scope

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
        let wire_msg = wire_msg_result.unwrap();

        let json_result: std::result::Result<Value, _> = serde_json::from_slice(&wire_msg.data);
        assert!(json_result.is_ok());
        let json_val = json_result.unwrap();
        println!("Serialized MCPMessage from wire: {}", serde_json::to_string_pretty(&json_val).unwrap());

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
            "timestamp": "2023-10-27T10:00:00Z",
            "version": {"major": 1, "minor": 0, "patch": 0},
            "type_": "Command",
            "payload": {"command": "do_something"},
            "security": {
                "security_level": "Standard",
                "signature": "sig123"
            }
        }"#;

        let wire_msg = WireMessage::new(
            WireProtocolVersion::default(), 
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
        let msg = msg_result.unwrap();

        assert_eq!(msg.id.0, "msg-test-deserialize");
        assert_eq!(msg.type_, MessageType::Command);
        assert!(msg.payload.is_object());
        assert_eq!(msg.security.security_level, SecurityLevel::Standard);
        assert_eq!(msg.security.signature, Some("sig123".to_string()));
    }

    #[tokio::test]
    async fn test_to_from_wire_message() {
        let original_msg = create_test_mcp_message("msg-wire-test", MessageType::Event, json!({"event": "occurred"}));

        // Serialize to WireMessage
        let wire_msg_result = original_msg.to_wire_message(WireProtocolVersion::default()).await;
        assert!(wire_msg_result.is_ok());
        let wire_msg = wire_msg_result.unwrap();

        // Deserialize from WireMessage
        let deserialized_msg_result = MCPMessage::from_wire_message(&wire_msg).await;
        if let Err(e) = &deserialized_msg_result {
             eprintln!("Wire deserialization failed: {}", e); 
        }
        assert!(deserialized_msg_result.is_ok());
        let deserialized_msg = deserialized_msg_result.unwrap();

        // Compare relevant fields (timestamp might differ slightly)
        assert_eq!(original_msg.id, deserialized_msg.id);
        assert_eq!(original_msg.version, deserialized_msg.version);
        assert_eq!(original_msg.type_, deserialized_msg.type_);
        assert_eq!(original_msg.payload, deserialized_msg.payload);
        assert_eq!(original_msg.security, deserialized_msg.security);
    }

    // Test the async DomainObject trait methods now
    #[tokio::test]
    async fn test_mcp_message_domain_object_serialization() {
        let msg = create_test_mcp_message("msg-direct-test", MessageType::Sync, json!({}));
        
        // Use the async DomainObject trait method
        let wire_msg_result = msg.to_wire_message(WireProtocolVersion::default()).await; 
        assert!(wire_msg_result.is_ok());
        let wire_msg = wire_msg_result.unwrap();

        // Use the async DomainObject trait method
        let deserialized_result = MCPMessage::from_wire_message(&wire_msg).await; 
        assert!(deserialized_result.is_ok());
        let deserialized_msg = deserialized_result.unwrap();

        assert_eq!(msg.id, deserialized_msg.id);
        assert_eq!(msg.type_, deserialized_msg.type_);
    }
} 