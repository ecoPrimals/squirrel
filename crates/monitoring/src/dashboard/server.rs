// WebSocket server for dashboard updates
//
// This module provides a real-time WebSocket server for dashboard updates
// that allows clients to receive live updates about system metrics, health status,
// and other monitoring information.

use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use axum::{
    extract::{State, WebSocketUpgrade, Path},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum::extract::ws::{WebSocket, Message};
use axum::http::{StatusCode, Method};
use futures_util::{SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use time::OffsetDateTime;
use tracing::{info, error, debug};
use uuid;

use squirrel_core::error::{Result, SquirrelError};
use super::{Manager, Layout, Component, Update};

/// Maximum number of messages to buffer in the broadcast channel
const MAX_BROADCAST_CAPACITY: usize = 1024;

/// Subscription message received from a client
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubscriptionMessage {
    /// Type of message
    #[serde(rename = "type")]
    message_type: String,
    /// Component ID to subscribe to
    #[serde(rename = "componentId")]
    component_id: Option<String>,
    /// Additional message data
    data: Option<Value>,
}

/// Shared server state
#[derive(Debug, Clone)]
struct ServerState {
    /// Dashboard manager instance
    manager: Arc<Manager>,
    /// Broadcast channel for sending updates to connected clients
    update_tx: broadcast::Sender<Update>,
}

/// Query parameters for layout listing
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// Optional filter by name
    name: Option<String>,
    /// Optional limit on number of results
    limit: Option<usize>,
}

/// Function to start the dashboard server
pub async fn start_server(addr: SocketAddr, manager: Arc<Manager>) -> Result<()> {
    info!("Starting dashboard server on {}", addr);
    
    // Create router with routes
    let router = create_router(manager);
    
    // Start the server
    axum::serve(tokio::net::TcpListener::bind(addr).await?, router)
        .await
        .map_err(|e| SquirrelError::other(format!("Failed to start dashboard server: {}", e)))
}

/// Handler for WebSocket connections
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handles an individual WebSocket connection
async fn handle_websocket(socket: WebSocket, state: ServerState) {
    // Split socket into sender and receiver
    let (sender, receiver) = socket.split();
    
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    let _client_id = uuid::Uuid::new_v4().to_string();
    
    // Subscribe to broadcast channel
    let mut update_rx = state.update_tx.subscribe();
    
    // Create subscription map for this client
    let subscriptions = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    
    // Clone for the send task
    let sender_clone = sender.clone();
    let subscriptions_clone = subscriptions.clone();
    
    // Spawn a task to send updates to the client
    let send_task_handle = tokio::spawn(async move {
        while let Ok(update) = update_rx.recv().await {
            let should_send = {
                let subs = subscriptions_clone.lock().await;
                subs.contains(&update.component_id)
            };
            
            if should_send {
                if let Ok(json) = serde_json::to_string(&update) {
                    let mut sender = sender_clone.lock().await;
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    });
    
    // Process incoming messages
    let recv_task_handle = tokio::spawn(async move {
        let mut receiver_stream = receiver;
        
        while let Some(Ok(msg)) = receiver_stream.next().await {
            if let Message::Text(text) = msg {
                debug!("Received message: {}", text);
                
                // Try to parse as a subscription message
                if let Ok(sub_msg) = serde_json::from_str::<SubscriptionMessage>(&text) {
                    if let Some(component_id) = sub_msg.component_id {
                        // Add to subscriptions
                        {
                            let mut subs = subscriptions.lock().await;
                            if !subs.contains(&component_id) {
                                subs.push(component_id.clone());
                            }
                        }
                        
                        // Send acknowledgment
                        let response = json!({
                            "type": "subscription_ack",
                            "component_id": component_id,
                            "timestamp": OffsetDateTime::now_utc(),
                        });
                        
                        let mut sender = sender.lock().await;
                        if let Ok(json) = serde_json::to_string(&response) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });
    
    // Wait for either task to complete
    let send_result = tokio::spawn(send_task_handle);

    let recv_result = tokio::spawn(recv_task_handle);

    tokio::select! {
        _ = send_result => {
            debug!("Send task completed");
        }
        _ = recv_result => {
            debug!("Receive task completed");
        }
    }
    
    debug!("WebSocket connection closed");
}

/// Update components periodically
async fn start_update_task(state: ServerState) {
    // Get config once
    let config = state.manager.config.read().await;
    let update_interval = Duration::from_secs(config.update_interval);
    drop(config);
    
    let update_tx = state.update_tx.clone();
    
    loop {
        // Get all layouts and components
        match state.manager.get_layouts().await {
            Ok(layouts) => {
                for layout in layouts {
                    for component in layout.components {
                        // Get data for this component
                        match state.manager.get_widget_data(&component).await {
                            Ok(widget_data) => {
                                // Extract component ID based on component type
                                let component_id = match &component {
                                    Component::PerformanceGraph { id, .. } => id.clone(),
                                    Component::AlertList { id, .. } => id.clone(),
                                    Component::HealthStatus { id, .. } => id.clone(),
                                    Component::Custom { id, .. } => id.clone(),
                                };
                                
                                // Create update
                                let update = Update {
                                    component_id: component_id.clone(),
                                    timestamp: OffsetDateTime::now_utc(),
                                    data: widget_data.clone(),
                                };
                                
                                // Broadcast update to WebSocket clients
                                if update_tx.send(update.clone()).is_err() {
                                    // No receivers, which is fine if no clients are connected
                                    debug!("No WebSocket clients connected to receive updates");
                                }
                                
                                // Also store the update in the manager's data store
                                let mut store = HashMap::new();
                                store.insert(component_id.clone(), widget_data);
                                if let Err(e) = state.manager.update_data(store).await {
                                    error!("Failed to update component data: {}", e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to get data for component: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get layouts: {}", e);
            }
        }
        
        // Wait for next update
        tokio::time::sleep(update_interval).await;
    }
}

/// Handler for getting all layouts
async fn get_layouts(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Vec<Layout>>, (StatusCode, String)> {
    // Get all layouts from the manager
    state.manager.get_layouts().await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Handler for getting a specific layout by ID
async fn get_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Layout>, (StatusCode, String)> {
    // Get layouts from the manager
    let layouts = state.manager.get_layouts().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Find the layout with the matching ID
    let layout = layouts.into_iter()
        .find(|l| l.id == id)
        .ok_or((StatusCode::NOT_FOUND, format!("Layout not found: {}", id)))?;
    
    Ok(Json(layout))
}

/// Handler for creating a new layout
async fn create_layout(
    State(state): State<ServerState>,
    Json(layout): Json<Layout>,
) -> std::result::Result<Json<Layout>, (StatusCode, String)> {
    // Add the layout to the manager
    state.manager.add_layout(layout.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(layout))
}

/// Handler for updating a layout
async fn update_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(layout): Json<Layout>,
) -> std::result::Result<Json<Layout>, (StatusCode, String)> {
    // Ensure the ID matches
    if layout.id != id {
        return Err((StatusCode::BAD_REQUEST, "Layout ID in path does not match body".to_string()));
    }
    
    // Get layouts
    let mut layouts = state.manager.get_layouts().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Check if the layout exists
    if !layouts.iter().any(|l| l.id == id) {
        return Err((StatusCode::NOT_FOUND, format!("Layout not found: {}", id)));
    }
    
    // Remove old layout and add updated one
    layouts.retain(|l| l.id != id);
    layouts.push(layout.clone());
    
    // Update layouts
    state.manager.add_layout(layout.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(layout))
}

/// Handler for deleting a layout
async fn delete_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> std::result::Result<StatusCode, (StatusCode, String)> {
    // Get layouts
    let layouts = state.manager.get_layouts().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Check if the layout exists
    if !layouts.iter().any(|l| l.id == id) {
        return Err((StatusCode::NOT_FOUND, format!("Layout not found: {}", id)));
    }
    
    // Convert String to &str for remove_layout
    match state.manager.remove_layout(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// Handler for getting component data
async fn get_component_data(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Vec<Update>>, (StatusCode, String)> {
    // Get data for the component
    let data = state.manager.get_data(&id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(data))
}

/// Function to create the router with routes
fn create_router(manager: Arc<Manager>) -> Router {
    // Create broadcast channel for updates
    let (update_tx, _) = broadcast::channel(MAX_BROADCAST_CAPACITY);
    
    // Create shared server state
    let state = ServerState { 
        manager,
        update_tx: update_tx.clone(),
    };
    
    // Set up CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);
    
    // Start update task in background
    let state_clone = state.clone();
    tokio::spawn(async move {
        start_update_task(state_clone).await;
    });
    
    // Create router with routes
    Router::new()
        // Dashboard data routes
        .route("/api/layouts", get(get_layouts).post(create_layout))
        .route("/api/layouts/:id", get(get_layout).put(update_layout).delete(delete_layout))
        .route("/api/components/:id/data", get(get_component_data))
        
        // WebSocket route
        .route("/ws", get(websocket_handler))
        
        // Apply middleware
        .layer(cors)
        .with_state(state)
} 