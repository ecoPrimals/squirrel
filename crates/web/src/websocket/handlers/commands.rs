//! WebSocket handlers for command-related events
//!
//! This module contains WebSocket handlers for command-related events,
//! including command status updates and command list updates.

use std::sync::Arc;
use async_trait::async_trait;
use serde_json::json;
use tracing::error;

use crate::api::commands::{CommandStatusEvent, CommandListUpdateEvent};
use crate::state::AppState;
use crate::websocket::{
    WebSocketHandler,
    WebSocketMessage,
    WebSocketContext,
    error::WebSocketError,
    ChannelCategory,
};

/// Command WebSocket handler
pub struct CommandWebSocketHandler {
    app_state: Arc<AppState>,
}

impl CommandWebSocketHandler {
    /// Create a new CommandWebSocketHandler
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[async_trait]
impl WebSocketHandler for CommandWebSocketHandler {
    async fn handle_message(
        &self,
        context: &WebSocketContext,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, WebSocketError> {
        match message.action.as_str() {
            "subscribe" => self.handle_subscribe(context, message).await,
            "unsubscribe" => self.handle_unsubscribe(context, message).await,
            _ => Err(WebSocketError::UnknownCommand(format!(
                "Unknown command action: {}",
                message.action
            ))),
        }
    }
}

impl CommandWebSocketHandler {
    /// Handle subscribe command
    async fn handle_subscribe(
        &self,
        context: &WebSocketContext,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, WebSocketError> {
        let topic = message.data.get("topic")
            .and_then(|t| t.as_str())
            .ok_or_else(|| WebSocketError::MissingParameter("topic".to_string()))?;

        match topic {
            "command_status" => {
                let command_id = message.data.get("commandId")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| WebSocketError::MissingParameter("commandId".to_string()))?;

                // Subscribe to command status updates
                let channel = format!("command:{}", command_id);
                context.subscribe(&channel).await.map_err(|e| {
                    WebSocketError::SubscriptionError(format!("Failed to subscribe to command status: {}", e))
                })?;

                // Return confirmation
                Ok(Some(WebSocketMessage {
                    action: "subscribed".to_string(),
                    data: json!({
                        "topic": "command_status",
                        "commandId": command_id,
                        "channel": channel
                    }),
                }))
            },
            "command_list" => {
                // Subscribe to command list updates
                context.subscribe("commands").await.map_err(|e| {
                    WebSocketError::SubscriptionError(format!("Failed to subscribe to command list: {}", e))
                })?;

                // Return confirmation
                Ok(Some(WebSocketMessage {
                    action: "subscribed".to_string(),
                    data: json!({
                        "topic": "command_list",
                        "channel": "commands"
                    }),
                }))
            },
            _ => Err(WebSocketError::InvalidCommand(format!("Unknown topic: {}", topic))),
        }
    }

    /// Handle unsubscribe command
    async fn handle_unsubscribe(
        &self,
        context: &WebSocketContext,
        message: WebSocketMessage,
    ) -> Result<Option<WebSocketMessage>, WebSocketError> {
        let topic = message.data.get("topic")
            .and_then(|t| t.as_str())
            .ok_or_else(|| WebSocketError::MissingParameter("topic".to_string()))?;

        match topic {
            "command_status" => {
                let command_id = message.data.get("commandId")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| WebSocketError::MissingParameter("commandId".to_string()))?;

                // Unsubscribe from command status updates
                let channel = format!("command:{}", command_id);
                context.unsubscribe(&channel).await.map_err(|e| {
                    WebSocketError::SubscriptionError(format!("Failed to unsubscribe from command status: {}", e))
                })?;

                // Return confirmation
                Ok(Some(WebSocketMessage {
                    action: "unsubscribed".to_string(),
                    data: json!({
                        "topic": "command_status",
                        "commandId": command_id
                    }),
                }))
            },
            "command_list" => {
                // Unsubscribe from command list updates
                context.unsubscribe("commands").await.map_err(|e| {
                    WebSocketError::SubscriptionError(format!("Failed to unsubscribe from command list: {}", e))
                })?;

                // Return confirmation
                Ok(Some(WebSocketMessage {
                    action: "unsubscribed".to_string(),
                    data: json!({
                        "topic": "command_list"
                    }),
                }))
            },
            _ => Err(WebSocketError::InvalidCommand(format!("Unknown topic: {}", topic))),
        }
    }
}

/// Broadcast a command status update to all subscribers of the command's status channel
pub async fn broadcast_command_status_update(
    app_state: &Arc<AppState>,
    event: CommandStatusEvent,
) -> Result<(), WebSocketError> {
    let _channel = format!("command:{}", event.id);
    let message = json!({
        "event": "command_status",
        "command_id": event.id,
        "status": event.status,
        "progress": event.progress,
        "result": event.result,
        "error": event.error,
        "timestamp": event.timestamp,
    });

    app_state.ws_manager.broadcast_to_channel(
        ChannelCategory::Command,
        &event.id,
        "status_update",
        message,
    ).await.map_err(|e| {
        error!("Failed to broadcast command status: {:?}", e);
        WebSocketError::SendError(format!("Failed to broadcast: {}", e))
    })?;

    Ok(())
}

/// Broadcast a command list update to all subscribers of the commands channel
pub async fn broadcast_command_list_update(
    app_state: &Arc<AppState>,
    event: CommandListUpdateEvent,
) -> Result<(), WebSocketError> {
    let message = json!({
        "event": "command_list_update",
        "update_type": event.update_type,
        "command": {
            "id": event.command.id,
            "name": event.command.name,
            "status": event.command.status,
        }
    });

    app_state.ws_manager.broadcast_to_channel(
        ChannelCategory::System,
        "commands",
        "list_update",
        message,
    ).await.map_err(|e| {
        error!("Failed to broadcast command list update: {:?}", e);
        WebSocketError::SendError(format!("Failed to broadcast: {}", e))
    })?;

    Ok(())
} 