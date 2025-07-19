//! Tests for the MCP protocol message validation

use std::sync::Arc;
use serde_json::json;
use chrono::Utc;
use crate::protocol::{MCPProtocolImpl, ProtocolConfig};
use crate::protocol::MCPProtocol;
use crate::types::{MCPMessage, MessageType, MessageId, SecurityLevel, MessageMetadata};
use crate::error::{MCPError, ProtocolError};

#[tokio::test]
async fn test_valid_message_validation() {
    // Create a protocol instance
    let protocol = MCPProtocolImpl::new();
    
    // Create a valid message
    let valid_message = MCPMessage {
        id: MessageId(String::from("test-id-1")),
        message_type: MessageType::Command,
        payload: json!({
            "command": "test",
            "args": ["arg1", "arg2"]
        }),
        metadata: Some(MessageMetadata {
            timestamp: Some(Utc::now().timestamp() as u64),
            security_level: SecurityLevel::Normal,
            source: Some("test-source".to_string()),
            destination: None,
            compression: None,
            encryption: None,
        }),
    };
    
    // Should pass validation
    let result = protocol.validate_message(&valid_message).await;
    assert!(result.is_ok(), "Valid message failed validation: {:?}", result);
}

#[tokio::test]
async fn test_invalid_message_type() {
    // Create a protocol instance
    let protocol = MCPProtocolImpl::new();
    
    // Create a message with unknown type
    let invalid_message = MCPMessage {
        id: MessageId(String::from("test-id-2")),
        message_type: MessageType::Unknown,
        payload: json!({
            "command": "test",
        }),
        metadata: None,
    };
    
    // Should fail validation
    let result = protocol.validate_message(&invalid_message).await;
    assert!(result.is_err(), "Invalid message type should fail validation");
    
    if let Err(MCPError::Protocol(ProtocolError::InvalidFormat(msg))) = result {
        assert!(msg.contains("Unknown message type"), "Error message does not mention unknown type: {}", msg);
    } else {
        panic!("Unexpected error type: {:?}", result);
    }
}

#[tokio::test]
async fn test_message_too_large() {
    // Create a protocol instance with small max message size
    let config = ProtocolConfig {
        max_message_size: 50, // Very small size for testing
        ..ProtocolConfig::default()
    };
    let protocol = MCPProtocolImpl::with_config(config);
    
    // Create a message with large payload
    let large_message = MCPMessage {
        id: MessageId(String::from("test-id-3")),
        message_type: MessageType::Command,
        payload: json!({
            "command": "test",
            "data": "a".repeat(100), // Large string
        }),
        metadata: None,
    };
    
    // Should fail validation due to size
    let result = protocol.validate_message(&large_message).await;
    assert!(result.is_err(), "Message exceeding size limit should fail validation");
    
    if let Err(MCPError::Protocol(ProtocolError::MessageTooLarge(msg))) = result {
        assert!(msg.contains("exceeds maximum allowed size"), "Error message does not mention size limit: {}", msg);
    } else {
        panic!("Unexpected error type: {:?}", result);
    }
}

#[tokio::test]
async fn test_invalid_timestamp() {
    // Create a protocol instance
    let protocol = MCPProtocolImpl::new();
    
    // Create a message with future timestamp
    let future_time = (Utc::now().timestamp() + 3600) as u64; // 1 hour in the future
    let future_message = MCPMessage {
        id: MessageId(String::from("test-id-4")),
        message_type: MessageType::Command,
        payload: json!({
            "command": "test",
        }),
        metadata: Some(MessageMetadata {
            timestamp: Some(future_time),
            security_level: SecurityLevel::Normal,
            source: None,
            destination: None,
            compression: None,
            encryption: None,
        }),
    };
    
    // Should fail validation due to future timestamp
    let result = protocol.validate_message(&future_message).await;
    assert!(result.is_err(), "Message with future timestamp should fail validation");
    
    if let Err(MCPError::Protocol(ProtocolError::InvalidTimestamp(msg))) = result {
        assert!(msg.contains("is in the future"), "Error message does not mention future timestamp: {}", msg);
    } else {
        panic!("Unexpected error type: {:?}", result);
    }
}

#[tokio::test]
async fn test_invalid_payload_format() {
    // Create a protocol instance
    let protocol = MCPProtocolImpl::new();
    
    // Create a message with non-object payload
    let invalid_payload_message = MCPMessage {
        id: MessageId(String::from("test-id-5")),
        message_type: MessageType::Command,
        payload: json!([1, 2, 3]), // Array instead of object
        metadata: None,
    };
    
    // Should fail validation due to payload format
    let result = protocol.validate_message(&invalid_payload_message).await;
    assert!(result.is_err(), "Message with non-object payload should fail validation");
    
    if let Err(MCPError::Protocol(ProtocolError::InvalidPayload(msg))) = result {
        assert!(msg.contains("must be an object"), "Error message does not mention payload format: {}", msg);
    } else {
        panic!("Unexpected error type: {:?}", result);
    }
}

#[tokio::test]
async fn test_expired_message() {
    // Create a protocol instance with short timeout
    let config = ProtocolConfig {
        timeout_ms: 5000, // 5 seconds
        ..ProtocolConfig::default()
    };
    let protocol = MCPProtocolImpl::with_config(config);
    
    // Create a message with old timestamp
    let old_time = (Utc::now().timestamp() - 10) as u64; // 10 seconds in the past
    let old_message = MCPMessage {
        id: MessageId(String::from("test-id-6")),
        message_type: MessageType::Command,
        payload: json!({
            "command": "test",
        }),
        metadata: Some(MessageMetadata {
            timestamp: Some(old_time),
            security_level: SecurityLevel::Normal,
            source: None,
            destination: None,
            compression: None,
            encryption: None,
        }),
    };
    
    // Should fail validation due to expired timestamp
    let result = protocol.validate_message(&old_message).await;
    assert!(result.is_err(), "Expired message should fail validation");
    
    if let Err(MCPError::Protocol(ProtocolError::MessageTimeout(msg))) = result {
        assert!(msg.contains("too old"), "Error message does not mention message age: {}", msg);
    } else {
        panic!("Unexpected error type: {:?}", result);
    }
} 