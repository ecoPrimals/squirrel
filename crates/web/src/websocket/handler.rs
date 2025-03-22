use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Extension, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::auth::extractor::AuthClaims;
use crate::state::AppState;
use super::{
    CommandHandler,
    WebSocketCommand,
    WebSocketError,
    WebSocketResponse,
};

/// Maximum number of queued messages for a connection
const MAX_QUEUE_SIZE: usize = 100;

/// WebSocket connection handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
    auth: Option<AuthClaims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, auth: Option<AuthClaims>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Get user info from auth, if available
    let user_id = auth.as_ref().map(|claims| claims.sub.clone());
    let roles = auth
        .as_ref()
        .map(|claims| claims.roles.clone())
        .unwrap_or_default();
    
    // Create a channel for sending messages to the client
    let (tx, mut rx) = mpsc::channel::<Result<String, WebSocketError>>(MAX_QUEUE_SIZE);
    
    // Register the connection
    let connection_id = state
        .ws_manager
        .register_connection(user_id.clone(), roles, tx)
        .await;
    
    info!(
        "WebSocket connection established: {}{}",
        connection_id,
        user_id
            .as_ref()
            .map(|id| format!(" (User: {})", id))
            .unwrap_or_default()
    );
    
    // Create command handler
    let cmd_handler = CommandHandler::new(state.ws_manager.clone());
    
    // Clone variables for use in tasks
    let connection_id_for_send = connection_id.clone();
    let connection_id_for_recv = connection_id.clone();
    let state_for_recv = state.clone();
    
    // Task for sending messages to the client
    let send_task = tokio::spawn(async move {
        while let Some(result) = rx.recv().await {
            match result {
                Ok(msg) => {
                    if let Err(e) = sender.send(Message::Text(msg)).await {
                        error!("Error sending message: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    // Try to send error message before closing
                    if let Err(send_err) = sender
                        .send(Message::Text(
                            serde_json::to_string(&WebSocketResponse {
                                success: false,
                                event: "error".to_string(),
                                data: serde_json::json!({
                                    "code": e.code(),
                                    "message": e.to_string(),
                                }),
                                error: Some(e.to_string()),
                                id: None,
                            })
                            .unwrap_or_else(|_| "\"Connection error\"".to_string()),
                        ))
                        .await
                    {
                        error!("Failed to send error message: {}", send_err);
                    }
                    break;
                }
            }
        }
        
        debug!("Send task for connection {} completed", connection_id_for_send);
        
        // Close the WebSocket connection properly
        let _ = sender.close().await;
    });
    
    // Task for receiving messages from the client
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => {
                            // Parse the message as a command
                            match serde_json::from_str::<WebSocketCommand>(&text) {
                                Ok(command) => {
                                    // Process the command
                                    let result = cmd_handler
                                        .handle_command(&connection_id_for_recv, command.clone())
                                        .await;
                                    
                                    // Send the response
                                    match result {
                                        Ok(response) => {
                                            state_for_recv
                                                .ws_manager
                                                .send_to_connection(&connection_id_for_recv, response)
                                                .await
                                                .unwrap_or_else(|e| {
                                                    error!(
                                                        "Failed to send response for command {}: {}",
                                                        command.command, e
                                                    );
                                                });
                                        }
                                        Err(err) => {
                                            let error_response = CommandHandler::create_error_response(
                                                err,
                                                command.id,
                                            );
                                            state_for_recv
                                                .ws_manager
                                                .send_to_connection(&connection_id_for_recv, error_response)
                                                .await
                                                .unwrap_or_else(|e| {
                                                    error!(
                                                        "Failed to send error response for command {}: {}",
                                                        command.command, e
                                                    );
                                                });
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Invalid command received: {}", e);
                                    let error_response = CommandHandler::create_error_response(
                                        WebSocketError::JsonError(e),
                                        None,
                                    );
                                    state_for_recv
                                        .ws_manager
                                        .send_to_connection(&connection_id_for_recv, error_response)
                                        .await
                                        .unwrap_or_else(|e| {
                                            error!("Failed to send error response: {}", e);
                                        });
                                }
                            }
                        }
                        Message::Binary(_) => {
                            // Binary messages are not supported yet
                            let error_response = CommandHandler::create_error_response(
                                WebSocketError::UnsupportedMessageType(
                                    "Binary messages are not supported".into(),
                                ),
                                None,
                            );
                            state_for_recv
                                .ws_manager
                                .send_to_connection(&connection_id_for_recv, error_response)
                                .await
                                .unwrap_or_else(|e| {
                                    error!("Failed to send error response: {}", e);
                                });
                        }
                        Message::Ping(_data) => {
                            // Respond to ping with pong
                            if let Err(e) = state_for_recv
                                .ws_manager
                                .send_to_connection(
                                    &connection_id_for_recv,
                                    WebSocketResponse {
                                        success: true,
                                        event: "pong".to_string(),
                                        data: serde_json::Value::Null,
                                        error: None,
                                        id: None,
                                    },
                                )
                                .await
                            {
                                error!("Failed to send pong: {}", e);
                            }
                        }
                        Message::Pong(_) => {
                            // Ignore pong messages
                        }
                        Message::Close(frame) => {
                            info!(
                                "WebSocket close frame received: {}",
                                frame
                                    .map(|f| format!("{}, {}", f.code, f.reason))
                                    .unwrap_or_else(|| "No close frame".into())
                            );
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        
        debug!("Receive task for connection {} completed", connection_id_for_recv);
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {
            debug!("Send task completed first");
        }
        _ = recv_task => {
            debug!("Receive task completed first");
        }
    }
    
    // Unregister the connection
    state.ws_manager.remove_connection(&connection_id).await;
    
    info!("WebSocket connection closed: {}", connection_id);
} 