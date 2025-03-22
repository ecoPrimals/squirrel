//! WebSocket command handlers for processing client commands.

use serde_json::Value;
use tracing::{debug, warn};

use super::error::WebSocketError;
use super::manager::ConnectionManager;
use super::models::{ChannelCategory, WebSocketCommand, WebSocketResponse};

/// Command handler for processing WebSocket commands.
pub struct CommandHandler {
    /// The connection manager.
    connection_manager: ConnectionManager,
}

impl CommandHandler {
    /// Create a new command handler.
    pub fn new(connection_manager: ConnectionManager) -> Self {
        Self {
            connection_manager,
        }
    }

    /// Handle a WebSocket command.
    pub async fn handle_command(
        &self,
        connection_id: &str,
        command: WebSocketCommand,
    ) -> Result<WebSocketResponse, WebSocketError> {
        debug!("Handling command: {}", command.command);

        match command.command.as_str() {
            "ping" => self.handle_ping(command).await,
            "subscribe" => self.handle_subscribe(connection_id, command).await,
            "unsubscribe" => self.handle_unsubscribe(connection_id, command).await,
            "info" => self.handle_info(connection_id, command).await,
            _ => Err(WebSocketError::UnknownCommand(format!(
                "Unknown command: {}",
                command.command
            ))),
        }
    }

    /// Handle a ping command.
    async fn handle_ping(&self, command: WebSocketCommand) -> Result<WebSocketResponse, WebSocketError> {
        // Extract data to echo back, if provided
        let data = command.params.get("data").cloned().unwrap_or(Value::Null);

        Ok(WebSocketResponse {
            success: true,
            event: "pong".to_string(),
            data,
            error: None,
            id: command.id,
        })
    }

    /// Handle a subscribe command.
    async fn handle_subscribe(
        &self,
        connection_id: &str,
        command: WebSocketCommand,
    ) -> Result<WebSocketResponse, WebSocketError> {
        // Extract category
        let category_str = command
            .params
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or(WebSocketError::MissingParameter("category".to_string()))?;

        // Convert to ChannelCategory
        let category = ChannelCategory::from_str(category_str).ok_or(WebSocketError::InvalidParameterType(
            "Invalid category".to_string(),
        ))?;

        // Extract channel
        let channel = command
            .params
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or(WebSocketError::MissingParameter("channel".to_string()))?;

        // Subscribe
        self.connection_manager
            .subscribe(connection_id, category.clone(), channel)
            .await?;

        // Return success response
        Ok(WebSocketResponse {
            success: true,
            event: "subscribed".to_string(),
            data: serde_json::json!({
                "category": category_str,
                "channel": channel,
            }),
            error: None,
            id: command.id,
        })
    }

    /// Handle an unsubscribe command.
    async fn handle_unsubscribe(
        &self,
        connection_id: &str,
        command: WebSocketCommand,
    ) -> Result<WebSocketResponse, WebSocketError> {
        // Extract category
        let category_str = command
            .params
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or(WebSocketError::MissingParameter("category".to_string()))?;

        // Convert to ChannelCategory
        let category = ChannelCategory::from_str(category_str).ok_or(WebSocketError::InvalidParameterType(
            "Invalid category".to_string(),
        ))?;

        // Extract channel
        let channel = command
            .params
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or(WebSocketError::MissingParameter("channel".to_string()))?;

        // Unsubscribe
        self.connection_manager
            .unsubscribe(connection_id, category.clone(), channel)
            .await?;

        // Return success response
        Ok(WebSocketResponse {
            success: true,
            event: "unsubscribed".to_string(),
            data: serde_json::json!({
                "category": category_str,
                "channel": channel,
            }),
            error: None,
            id: command.id,
        })
    }

    /// Handle an info command to get information about the connection.
    async fn handle_info(
        &self,
        connection_id: &str,
        command: WebSocketCommand,
    ) -> Result<WebSocketResponse, WebSocketError> {
        // Get connection information
        let connections = self.connection_manager.connection_count().await;
        
        // Get connection's subscriptions
        let connections_lock = self
            .connection_manager
            .clone();
        
        let active_subscriptions = connections_lock
            .get_active_subscriptions()
            .await;
            
        let my_subscriptions: Vec<_> = active_subscriptions
            .iter()
            .filter(|sub| sub.user_id == connection_id)
            .collect();

        // Return info response
        Ok(WebSocketResponse {
            success: true,
            event: "info".to_string(),
            data: serde_json::json!({
                "connections": connections,
                "subscriptions": my_subscriptions.len(),
            }),
            error: None,
            id: command.id,
        })
    }

    /// Create an error response.
    pub fn create_error_response(
        error: WebSocketError,
        command_id: Option<String>,
    ) -> WebSocketResponse {
        WebSocketResponse {
            success: false,
            event: "error".to_string(),
            data: serde_json::json!({
                "code": error.code(),
                "message": error.to_string(),
            }),
            error: Some(error.to_string()),
            id: command_id,
        }
    }
} 