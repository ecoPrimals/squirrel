//! WebSocket handlers for command-related events
//!
//! This module contains WebSocket handlers for command-related events,
//! including command status updates and command list updates.

use std::sync::Arc;
use serde_json::json;
use serde::Serialize;
use async_trait::async_trait;

use crate::api::commands::CommandStatus;
use crate::state::AppState;
use crate::websocket::{
    WebSocketContext,
    WebSocketHandler,
    WebSocketMessage,
    error::WebSocketError,
    ChannelCategory,
};

// Define the event types that were missing
#[derive(Debug, Serialize)]
pub struct CommandStatusEvent {
    pub command_id: String,
    pub status: CommandStatus,
    pub progress: f32,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandListUpdateEvent {
    pub action: String, // "added", "updated", "removed"
    pub command_id: String,
    pub command_name: String,
    pub status: CommandStatus,
}

/// Command message handler
pub struct CommandHandler {
    app_state: Arc<AppState>,
}

impl CommandHandler {
    /// Create a new CommandHandler
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl WebSocketHandler for CommandHandler {
    async fn handle_message(
        &self,
        context: &WebSocketContext,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, WebSocketError> {
        // Extract message content
        let action = &message.action;
        let data = &message.data;
        
        match action.as_str() {
            "subscribe_command_updates" => {
                // Get command ID from the message
                let command_id = data["command_id"].as_str()
                    .ok_or(WebSocketError::MissingParameter("Missing command_id".to_string()))?;
                
                // Subscribe the client to updates for this command
                let channel = format!("command:{}", command_id);
                context.subscribe(&channel).await?;
                
                // Send confirmation
                let response = WebSocketMessage {
                    action: "subscribed".to_string(),
                    data: json!({
                        "channel": channel
                    }),
                };
                
                Ok(Some(response))
            },
            "unsubscribe_command_updates" => {
                // Get command ID from the message
                let command_id = data["command_id"].as_str()
                    .ok_or(WebSocketError::MissingParameter("Missing command_id".to_string()))?;
                
                // Unsubscribe the client from this command's updates
                let channel = format!("command:{}", command_id);
                context.unsubscribe(&channel).await?;
                
                // Send confirmation
                let response = WebSocketMessage {
                    action: "unsubscribed".to_string(),
                    data: json!({
                        "channel": channel
                    }),
                };
                
                Ok(Some(response))
            },
            _ => {
                // Unknown command type
                Err(WebSocketError::UnknownCommand(
                    format!("Unknown command type: {}", action)
                ))
            }
        }
    }
}

/// Send command status update to WebSocket clients
pub async fn broadcast_command_status(
    app_state: &Arc<AppState>,
    command_id: &str,
    status: CommandStatus,
    progress: f32,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<usize, WebSocketError> {
    // Format channel name
    let channel = format!("command:{}", command_id);
    
    // Create JSON payload
    let payload = json!({
        "command_id": command_id,
        "status": status,
        "progress": progress,
        "result": result,
        "error": error,
    });
    
    // Broadcast to the channel
    app_state.ws_manager.broadcast_to_channel(
        ChannelCategory::Command,
        &channel,
        "command_status",
        payload
    ).await.map_err(|e| WebSocketError::SendError(format!("Failed to broadcast: {}", e)))
}

/// Send command list update to WebSocket clients
pub async fn broadcast_command_list_update(
    app_state: &Arc<AppState>,
    action: &str,
    command_id: &str,
    command_name: &str,
    status: CommandStatus,
) -> Result<usize, WebSocketError> {
    // Create payload
    let payload = json!({
        "action": action,
        "command": {
            "id": command_id,
            "name": command_name,
            "status": status,
        }
    });
    
    // Broadcast to all clients subscribed to command list updates
    app_state.ws_manager.broadcast_to_channel(
        ChannelCategory::Command,
        "commands",
        "command_list_update",
        payload
    ).await.map_err(|e| WebSocketError::SendError(format!("Failed to broadcast: {}", e)))
} 