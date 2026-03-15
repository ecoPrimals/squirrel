// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Chat History Integration for MCP
//! 
//! This module provides functionality for storing, synchronizing, and
//! managing chat history through the Machine Context Protocol.

use crate::client::MCPClient;
use crate::error::{MCPError, Result};
use crate::message::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Chat Message representation for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message content
    pub content: String,
    
    /// Flag indicating if the message is from a user (true) or AI (false)
    pub is_user: bool,
    
    /// Timestamp as Unix timestamp (seconds since epoch)
    pub timestamp: u64,
    
    /// Optional source identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

impl ChatMessage {
    /// Create a new chat message
    pub fn new(content: String, is_user: bool) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            content,
            is_user,
            timestamp,
            source: None,
            metadata: None,
        }
    }
    
    /// Create a new user message
    pub fn new_user(content: String) -> Self {
        Self::new(content, true)
    }
    
    /// Create a new AI message
    pub fn new_ai(content: String) -> Self {
        Self::new(content, false)
    }
    
    /// Set the source identifier
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Convert to a tuple format (content, is_user, timestamp)
    pub fn to_tuple(&self) -> (String, bool, u64) {
        (self.content.clone(), self.is_user, self.timestamp)
    }
    
    /// Create from a tuple format (content, is_user, timestamp)
    pub fn from_tuple(tuple: (String, bool, u64)) -> Self {
        let (content, is_user, timestamp) = tuple;
        Self {
            content,
            is_user,
            timestamp,
            source: None,
            metadata: None,
        }
    }
}

/// Subscription handle for chat history updates
pub struct ChatHistorySubscription {
    /// Message receiver for updates
    receiver: tokio::sync::broadcast::Receiver<Vec<ChatMessage>>,
}

impl ChatHistorySubscription {
    /// Create a new subscription handle
    pub fn new(receiver: tokio::sync::broadcast::Receiver<Vec<ChatMessage>>) -> Self {
        Self { receiver }
    }
    
    /// Wait for the next update
    pub async fn next_update(&mut self) -> Option<Vec<ChatMessage>> {
        match self.receiver.recv().await {
            Ok(messages) => Some(messages),
            Err(_) => None,
        }
    }
}

/// Extension trait for MCPClient to handle chat history
pub trait ChatHistoryClient {
    /// Get the current chat history
    fn get_chat_history(&self) -> impl std::future::Future<Output = Result<Vec<ChatMessage>>> + Send;
    
    /// Update the chat history
    fn update_chat_history(&self, messages: Vec<ChatMessage>) -> impl std::future::Future<Output = Result<()>> + Send;
    
    /// Update chat history from tuple format
    fn update_chat_history_from_tuples(&self, history: Vec<(String, bool, u64)>) -> impl std::future::Future<Output = Result<()>> + Send;
    
    /// Subscribe to chat history updates
    fn subscribe_to_chat_updates(&self) -> impl std::future::Future<Output = Result<ChatHistorySubscription>> + Send;
    
    /// Get relevant messages for a context
    fn get_relevant_messages(&self, context: &str, limit: usize) -> impl std::future::Future<Output = Result<Vec<ChatMessage>>> + Send;
    
    /// Clear chat history
    fn clear_chat_history(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// Implementation of ChatHistoryClient for MCPClient
impl ChatHistoryClient for MCPClient {
    fn get_chat_history(&self) -> impl std::future::Future<Output = Result<Vec<ChatMessage>>> + Send {
        let client = self.clone();
        async move {
            let response = client.send_command_with_content(
                "get_chat_history",
                serde_json::json!({})
            ).await?;
            
            // Parse the string content as JSON
            let content: Value = serde_json::from_str(&response.content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse response content as JSON: {}", e)))?;
            
            let messages: Vec<ChatMessage> = serde_json::from_value(content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse chat history: {}", e)))?;
            
            Ok(messages)
        }
    }
    
    fn update_chat_history(&self, messages: Vec<ChatMessage>) -> impl std::future::Future<Output = Result<()>> + Send {
        let client = self.clone();
        async move {
            client.send_command_with_content(
                "update_chat_history",
                serde_json::json!({
                    "messages": messages,
                })
            ).await?;
            
            Ok(())
        }
    }
    
    fn update_chat_history_from_tuples(&self, history: Vec<(String, bool, u64)>) -> impl std::future::Future<Output = Result<()>> + Send {
        let messages: Vec<ChatMessage> = history.into_iter()
            .map(ChatMessage::from_tuple)
            .collect();
        
        self.update_chat_history(messages)
    }
    
    fn subscribe_to_chat_updates(&self) -> impl std::future::Future<Output = Result<ChatHistorySubscription>> + Send {
        let client = self.clone();
        async move {
            let response = client.send_command_with_content(
                "subscribe_to_chat_updates",
                serde_json::json!({})
            ).await?;
            
            // Parse the string content as JSON
            let content: Value = serde_json::from_str(&response.content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse response content as JSON: {}", e)))?;
            
            let subscription_id = content.get("subscription_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| MCPError::InvalidMessage("Missing subscription ID".to_string()))?;
            
            // Create a channel for updates
            let (tx, rx) = tokio::sync::broadcast::channel(100);
            let tx_clone = tx.clone();
            
            // Register an event handler for chat updates
            client.register_event_handler(std::sync::Arc::new(ChatUpdateHandler::new(
                subscription_id.clone(),
                tx_clone,
            ))).await?;
            
            Ok(ChatHistorySubscription::new(rx))
        }
    }
    
    fn get_relevant_messages(&self, context: &str, limit: usize) -> impl std::future::Future<Output = Result<Vec<ChatMessage>>> + Send {
        let client = self.clone();
        let context = context.to_string();
        async move {
            let response = client.send_command_with_content(
                "get_relevant_messages",
                serde_json::json!({
                    "context": context,
                    "limit": limit,
                })
            ).await?;
            
            // Parse the string content as JSON
            let content: Value = serde_json::from_str(&response.content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse response content as JSON: {}", e)))?;
            
            let messages: Vec<ChatMessage> = serde_json::from_value(content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse relevant messages: {}", e)))?;
            
            Ok(messages)
        }
    }
    
    fn clear_chat_history(&self) -> impl std::future::Future<Output = Result<()>> + Send {
        let client = self.clone();
        async move {
            client.send_command_with_content(
                "clear_chat_history",
                serde_json::json!({})
            ).await?;
            
            Ok(())
        }
    }
}

/// Event handler for chat history updates
struct ChatUpdateHandler {
    subscription_id: String,
    sender: tokio::sync::broadcast::Sender<Vec<ChatMessage>>,
}

impl ChatUpdateHandler {
    /// Create a new chat update handler
    fn new(subscription_id: String, sender: tokio::sync::broadcast::Sender<Vec<ChatMessage>>) -> Self {
        Self {
            subscription_id,
            sender,
        }
    }
}

impl crate::client::EventHandler for ChatUpdateHandler {
    fn handle_event<'a>(&'a self, event: &'a Message) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            // Parse the event content as JSON
            let content: Value = serde_json::from_str(&event.content)
                .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse event content as JSON: {}", e)))?;
            
            if let Some(subscription_id) = content.get("subscription_id") {
                if subscription_id.as_str() == Some(&self.subscription_id) {
                    if let Some(messages) = content.get("messages") {
                        let messages: Vec<ChatMessage> = serde_json::from_value(messages.clone())
                            .map_err(|e| MCPError::InvalidMessage(format!("Failed to parse chat update: {}", e)))?;
                        
                        // Send the update to subscribers
                        let _ = self.sender.send(messages);
                    }
                }
            }
            Ok(())
        })
    }
    
    fn supported_event_types(&self) -> Vec<String> {
        // This handler processes chat update events
        vec!["chat_update".to_string(), "chat_history_update".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chat_message_creation() {
        let user_msg = ChatMessage::new_user("Hello".to_string());
        assert!(user_msg.is_user);
        assert_eq!(user_msg.content, "Hello");
        
        let ai_msg = ChatMessage::new_ai("Hi there".to_string());
        assert!(!ai_msg.is_user);
        assert_eq!(ai_msg.content, "Hi there");
    }
    
    #[test]
    fn test_chat_message_tuple_conversion() {
        let msg = ChatMessage::new("Test".to_string(), true)
            .with_source("test")
            .with_metadata(serde_json::json!({"key": "value"}));
        
        let tuple = msg.to_tuple();
        assert_eq!(tuple.0, "Test");
        assert_eq!(tuple.1, true);
        
        let converted = ChatMessage::from_tuple(tuple);
        assert_eq!(converted.content, "Test");
        assert_eq!(converted.is_user, true);
        assert_eq!(converted.timestamp, msg.timestamp);
        assert_eq!(converted.source, None); // Note: source is not part of tuple
        assert_eq!(converted.metadata, None); // Note: metadata is not part of tuple
    }
} 