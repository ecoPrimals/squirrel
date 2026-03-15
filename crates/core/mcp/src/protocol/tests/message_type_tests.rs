// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

#[cfg(test)]
mod tests {
    use crate::message::{Message, MessageBuilder, MessageType};
    use crate::types::{MCPMessage, MessageId, MessageType as MCPMessageType, SecurityMetadata, ProtocolVersion};
    use serde_json::json;
    use std::collections::HashMap;
    use chrono::Utc;

    #[tokio::test]
    async fn test_message_to_mcp_message_conversion() {
        // Create a standard Message
        let message = MessageBuilder::new()
            .with_message_type("request")
            .with_content("Test content")
            .with_source("client-123")
            .with_destination("server-456")
            .with_metadata("version", "1.0")
            .with_metadata("app", "test-app")
            .with_correlation_id("corr-123")
            .build();

        // Convert to MCPMessage
        let mcp_message = message.to_mcp_message().await;

        // Verify basic field mapping
        assert_eq!(mcp_message.id.0, message.id);
        assert_eq!(mcp_message.type_, MCPMessageType::Command); // Request -> Command
        
        // Verify payload includes important fields
        if let Some(content) = mcp_message.payload.get("content") {
            assert_eq!(content.as_str().unwrap(), "Test content");
        } else {
            panic!("Content field missing in payload");
        }

        if let Some(source) = mcp_message.payload.get("source") {
            assert_eq!(source.as_str().unwrap(), "client-123");
        } else {
            panic!("Source field missing in payload");
        }

        if let Some(in_reply_to) = mcp_message.payload.get("in_reply_to") {
            assert_eq!(in_reply_to.as_str().unwrap(), "corr-123");
        } else {
            panic!("in_reply_to field missing in payload");
        }
    }

    #[tokio::test]
    async fn test_mcp_message_to_message_conversion() {
        // Create an MCPMessage
        let mcp_message = MCPMessage {
            id: MessageId("msg-456".to_string()),
            type_: MCPMessageType::Response,
            payload: json!({
                "content": "Response content",
                "source": "server-456",
                "destination": "client-123",
                "in_reply_to": "req-789",
                "metadata": {
                    "status": "success",
                    "processed": true
                }
            }),
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: Utc::now(),
            version: ProtocolVersion::new(1, 0),
            trace_id: None,
        };

        // Convert to Message
        let message = Message::from_mcp_message(&mcp_message).await.unwrap();

        // Verify basic field mapping
        assert_eq!(message.id, "msg-456");
        assert_eq!(message.message_type, MessageType::Response);
        assert_eq!(message.content, "Response content");
        assert_eq!(message.source, "server-456");
        assert_eq!(message.destination, "client-123");
        assert_eq!(message.in_reply_to, Some("req-789".to_string()));
        
        // Verify metadata
        assert!(message.metadata.contains_key("status"));
        assert_eq!(message.metadata.get("status").unwrap(), "success");
    }

    #[tokio::test]
    async fn test_roundtrip_conversion() {
        // Create a standard Message
        let original_message = MessageBuilder::new()
            .with_message_type("notification")
            .with_content("Event notification")
            .with_source("system")
            .with_destination("*")
            .with_topic("system.event")
            .with_metadata("category", "alert")
            .build();

        // Convert to MCPMessage and back
        let mcp_message = original_message.to_mcp_message().await;
        let converted_message = Message::from_mcp_message(&mcp_message).await.unwrap();

        // Verify core fields preserved
        assert_eq!(converted_message.message_type, original_message.message_type);
        assert_eq!(converted_message.content, original_message.content);
        assert_eq!(converted_message.source, original_message.source);
        assert_eq!(converted_message.destination, original_message.destination);
        assert_eq!(converted_message.topic, original_message.topic);
        
        // Metadata should be preserved
        assert_eq!(
            converted_message.metadata.get("category"), 
            original_message.metadata.get("category")
        );
    }
} 