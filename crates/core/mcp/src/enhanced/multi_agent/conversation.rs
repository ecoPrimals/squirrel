// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Conversation Management
//!
//! Handles multi-agent conversations and message dispatching.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::error::types::MCPError;
use super::{MultiAgentConfig, Conversation, ConversationState};

/// Conversation manager
#[derive(Debug)]
pub struct ConversationManager {
    /// Active conversations
    conversations: Arc<RwLock<HashMap<String, Arc<RwLock<Conversation>>>>>,
    /// Conversation configurations
    config: Arc<MultiAgentConfig>,
    /// Message dispatcher
    message_dispatcher: Arc<MessageDispatcher>,
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new(config: Arc<MultiAgentConfig>) -> Self {
        let message_dispatcher = MessageDispatcher::new();
        
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
            config,
            message_dispatcher: Arc::new(message_dispatcher),
        }
    }
    
    /// Start a new conversation
    pub async fn start_conversation(&self, participants: Vec<String>) -> Result<String, MCPError> {
        let conversation_id = uuid::Uuid::new_v4().to_string();
        
        let conversation = Conversation {
            id: conversation_id.clone(),
            participants: participants.clone(),
            state: ConversationState::Active,
            messages: Vec::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            timeout: self.config.conversation_timeout,
        };
        
        let mut conversations = self.conversations.write().await;
        conversations.insert(conversation_id.clone(), Arc::new(RwLock::new(conversation)));
        
        info!("Started conversation {} with {} participants", conversation_id, participants.len());
        Ok(conversation_id)
    }
    
    /// Send message in conversation
    pub async fn send_message(
        &self,
        conversation_id: &str,
        sender: &str,
        content: serde_json::Value,
    ) -> Result<(), MCPError> {
        let conversations = self.conversations.read().await;
        
        if let Some(conversation_arc) = conversations.get(conversation_id) {
            let mut conversation = conversation_arc.write().await;
            
            // Check if conversation is active
            if conversation.state != ConversationState::Active {
                return Err(MCPError::InvalidState(
                    format!("Conversation {} is not active", conversation_id)
                ));
            }
            
            // Add message
            let message = super::ConversationMessage {
                id: uuid::Uuid::new_v4().to_string(),
                sender: sender.to_string(),
                content,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            
            conversation.messages.push(message);
            conversation.last_activity = chrono::Utc::now();
            
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Conversation not found: {}", conversation_id)))
        }
    }
    
    /// Get conversation history
    pub async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Conversation, MCPError> {
        let conversations = self.conversations.read().await;
        
        if let Some(conversation_arc) = conversations.get(conversation_id) {
            let conversation = conversation_arc.read().await;
            Ok(conversation.clone())
        } else {
            Err(MCPError::NotFound(format!("Conversation not found: {}", conversation_id)))
        }
    }
    
    /// End conversation
    pub async fn end_conversation(&self, conversation_id: &str) -> Result<(), MCPError> {
        let conversations = self.conversations.read().await;
        
        if let Some(conversation_arc) = conversations.get(conversation_id) {
            let mut conversation = conversation_arc.write().await;
            conversation.state = ConversationState::Completed;
            
            info!("Ended conversation {}", conversation_id);
            Ok(())
        } else {
            Err(MCPError::NotFound(format!("Conversation not found: {}", conversation_id)))
        }
    }
    
    /// Clean up inactive conversations
    pub async fn cleanup_inactive(&self, max_age: Duration) -> Result<usize, MCPError> {
        let mut conversations = self.conversations.write().await;
        let now = chrono::Utc::now();
        let mut removed = 0;
        
        conversations.retain(|_id, conversation_arc| {
            let conversation = conversation_arc.blocking_read();
            let age = now - conversation.last_activity;
            let should_keep = age.to_std().unwrap_or(Duration::from_secs(0)) < max_age;
            
            if !should_keep {
                removed += 1;
            }
            
            should_keep
        });
        
        if removed > 0 {
            info!("Cleaned up {} inactive conversations", removed);
        }
        
        Ok(removed)
    }
}

/// Message dispatcher
#[derive(Debug)]
pub struct MessageDispatcher {
    /// Message routing table
    routes: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MessageDispatcher {
    /// Create a new message dispatcher
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register route for message type
    pub async fn register_route(&self, message_type: String, agents: Vec<String>) -> Result<(), MCPError> {
        let mut routes = self.routes.write().await;
        routes.insert(message_type.clone(), agents.clone());
        
        info!("Registered route for message type {}: {} agents", message_type, agents.len());
        Ok(())
    }
    
    /// Get routes for message type
    pub async fn get_routes(&self, message_type: &str) -> Option<Vec<String>> {
        let routes = self.routes.read().await;
        routes.get(message_type).cloned()
    }
    
    /// Dispatch message to agents
    pub async fn dispatch_message(
        &self,
        message_type: &str,
        message: serde_json::Value,
    ) -> Result<Vec<String>, MCPError> {
        let routes = self.routes.read().await;
        
        if let Some(agents) = routes.get(message_type) {
            // In a real implementation, this would send messages to agents
            // For now, just return the agent IDs that would receive the message
            Ok(agents.clone())
        } else {
            warn!("No routes found for message type: {}", message_type);
            Ok(Vec::new())
        }
    }
}

impl Default for MessageDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

