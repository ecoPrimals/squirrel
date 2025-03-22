// WebSocket server for dashboard updates
//
// This module provides a real-time WebSocket server for dashboard updates
// that allows clients to receive live updates about system metrics, health status,
// and other monitoring information.

use std::collections::HashSet;
use std::collections::HashMap;
use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;
use std::io::Write;

use axum::{
    extract::{State, WebSocketUpgrade, Path},
    response::IntoResponse,
    routing::get,
    Router,
    Json,
    http::StatusCode,
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{StreamExt, SinkExt};
use futures_util::stream::SplitSink;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use flate2::{Compression, write::GzEncoder};
use base64;
use base64::Engine;
use tokio::{net::TcpListener, time};
use tokio::sync::mpsc;
use tracing::{info, error};

use squirrel_core::error::SquirrelError;
use super::{Manager, Layout, Update};

/// Maximum number of messages to buffer in the broadcast channel
const MAX_BROADCAST_CAPACITY: usize = 1024;

/// Maximum number of updates to batch in a single WebSocket message
const MAX_BATCH_SIZE: usize = 50;

/// Maximum time to wait before sending a batch (milliseconds)
const MAX_BATCH_WAIT_MS: u64 = 100;

/// Minimum size (in bytes) for compressing a payload
const COMPRESSION_THRESHOLD: usize = 1024;

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

/// Batched update response sent to clients
#[derive(Debug, Serialize)]
struct BatchedUpdate {
    updates: Vec<Update>,
}

/// State shared between all connections
#[derive(Clone)]
struct ServerState {
    manager: Arc<Manager>,
    config: ServerConfig,
}

/// Configuration for the dashboard server
#[derive(Clone)]
struct ServerConfig {
    update_interval_ms: u64,
}

/// Client message types for WebSocket communication
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    #[serde(rename = "subscribe")]
    Subscribe {
        component_ids: HashSet<String>,
    },
    #[serde(rename = "unsubscribe")]
    Unsubscribe {
        component_ids: HashSet<String>,
    },
}

/// Query parameters for layout listing
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// Optional filter by name
    name: Option<String>,
    /// Optional limit on number of results
    limit: Option<usize>,
}

/// Starts a WebSocket server that provides dashboard data to clients
///
/// # Arguments
/// * `manager` - Dashboard manager instance
/// * `addr` - Socket address to bind the server to
///
/// # Returns
/// * `Result` - Ok if server starts successfully, Error otherwise
pub async fn start_server(
    manager: Arc<Manager>,
    addr: SocketAddr,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create router for handling different routes
    info!("Creating dashboard WebSocket server router");
    
    // Create state for the server
    let state = ServerState {
        manager: manager.clone(),
        config: ServerConfig { update_interval_ms: 1000 },
    };
    
    // Set up routes
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/components", get(list_available_components))
        .with_state(state);
    
    // Start the server
    info!("Starting dashboard WebSocket server on {}", addr);
    let listener = TcpListener::bind(addr).await
        .map_err(|e| Box::new(SquirrelError::other(format!("Failed to bind to address: {}", e))) as Box<dyn std::error::Error>)?;
    
    // Serve until shutdown
    axum::serve(listener, app)
        .await
        .map_err(|e| Box::new(SquirrelError::other(format!("Failed to start dashboard server: {}", e))) as Box<dyn std::error::Error>)
}

/// Handler for WebSocket connections
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        // Wrap handle_socket_connection in a future that returns ()
        async move {
            handle_socket_connection(socket, state).await;
            // Return () to satisfy the Output = () requirement
        }
    })
}

/// Handler for WebSocket connections providing dashboard updates to clients
/// and receiving client interactions
async fn handle_socket_connection(socket: WebSocket, state: ServerState) {
    // Split the socket into sender and receiver parts
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    
    // Each connection has a unique set of component subscriptions
    let subscriptions: Arc<tokio::sync::RwLock<Option<HashSet<String>>>> = Arc::new(tokio::sync::RwLock::new(None));
    
    // Create a channel for signaling exit of background tasks
    let (exit_tx, mut exit_rx) = mpsc::channel::<()>(1);
    
    // Client UUID for identifying the connection
    let client_id = uuid::Uuid::new_v4().to_string();
    
    // Spawn a task to send periodic updates
    let update_task = {
        let sender = sender.clone();
        let subscriptions_guard = subscriptions.clone();
        let state = state.clone();
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(state.config.update_interval_ms));
            let mut pending_updates: Vec<Update> = Vec::new();
            
            loop {
                tokio::select! {
                    // Handle interval-based updates
                    _ = interval.tick() => {
                        if pending_updates.is_empty() {
                            continue;
                        }
                        
                        let mut sender_lock = sender.lock().await;
                        if let Err(e) = send_batched_updates(&mut sender_lock, pending_updates.clone(), false).await {
                            error!("Error sending batched updates: {}", e);
                            break;
                        }
                        pending_updates.clear();
                    }
                    
                    // Handle exit signal
                    _ = exit_rx.recv() => {
                        break;
                    }
                    
                    // Handle new updates
                    else => {
                        // Check for any new updates from components the client is subscribed to
                        if let Some(subscriptions) = &*subscriptions_guard.read().await {
                            pending_updates.clear();
                            
                            // Collect updates from all subscribed components
                            for component_id in subscriptions.iter() {
                                if let Some(update_vec) = state.manager.get_component_data(component_id) {
                                    // Add all updates from this component to our pending updates
                                    pending_updates.extend(update_vec.clone());
                                }
                            }
                            
                            // If we have updates, send them immediately
                            if !pending_updates.is_empty() {
                                let mut sender_lock = sender.lock().await;
                                if let Err(e) = send_batched_updates(&mut sender_lock, pending_updates.clone(), false).await {
                                    error!("Error sending immediate updates: {}", e);
                                    break;
                                }
                                pending_updates.clear();
                            }
                        }
                        
                        // Sleep a bit to avoid tight loop
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                }
            }
        })
    };
    
    // Process incoming messages from the client
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            ClientMessage::Subscribe { component_ids } => {
                                info!("Client {} subscribing to components: {:?}", client_id, component_ids);
                                let mut subs = subscriptions.write().await;
                                *subs = Some(component_ids);
                                
                                // Send initial data for all components
                                let mut updates = Vec::new();
                                if let Some(component_ids) = &*subs {
                                    for component_id in component_ids.iter() {
                                        if let Some(update_vec) = state.manager.get_component_data(component_id) {
                                            // Add all updates from this component
                                            updates.extend(update_vec.clone());
                                        }
                                    }
                                }
                                
                                if !updates.is_empty() {
                                    let mut sender_lock = sender.lock().await;
                                    if let Err(e) = send_batched_updates(&mut sender_lock, updates, false).await {
                                        error!("Error sending initial updates: {}", e);
                                        break;
                                    }
                                }
                            }
                            ClientMessage::Unsubscribe { component_ids } => {
                                info!("Client {} unsubscribing from components: {:?}", client_id, component_ids);
                                let mut subs = subscriptions.write().await;
                                
                                if let Some(current_subs) = &mut *subs {
                                    current_subs.retain(|id| !component_ids.contains(id));
                                }
                            }
                            // Handle other client message types here
                        }
                    }
                    Err(e) => {
                        error!("Error parsing client message: {}", e);
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client {} connection closed", client_id);
                break;
            }
            Err(e) => {
                error!("Error from client {}: {}", client_id, e);
                break;
            }
            _ => {} // Ignore other message types
        }
    }
    
    // Client disconnected, clean up tasks
    let _ = exit_tx.send(()).await;
    update_task.abort();
    
    info!("Client {} disconnected, cleaned up resources", client_id);
}

/// Send batched updates to a client
async fn send_batched_updates(
    sender: &mut SplitSink<WebSocket, Message>, 
    updates: Vec<Update>, 
    force_compression: bool
) -> std::result::Result<(), String> {
    if updates.is_empty() {
        return Ok(());
    }
    
    // Create the JSON message containing all updates
    let json_data = match serde_json::to_string(&BatchedUpdate { updates }) {
        Ok(json) => json,
        Err(e) => return Err(format!("Failed to serialize updates: {}", e)),
    };
    
    // Check if compression is needed (for large payloads or if forced)
    let compress = force_compression || json_data.len() > 8192;
    
    if compress {
        // Compress the data using gzip
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        if encoder.write_all(json_data.as_bytes()).is_err() {
            return Err("Failed to compress data".to_string());
        }
        
        let compressed_data = match encoder.finish() {
            Ok(data) => data,
            Err(e) => return Err(format!("Failed to finalize compression: {}", e)),
        };
        
        // Encode the compressed data as base64
        let encoded = base64::engine::general_purpose::STANDARD.encode(&compressed_data);
        
        // Send as a compressed message
        let message = format!("{{\"compressed\":true,\"data\":\"{}\"}}", encoded);
        if let Err(e) = sender.send(Message::Text(message)).await {
            return Err(format!("Failed to send compressed message: {}", e));
        }
    } else {
        // Send uncompressed JSON directly
        if let Err(e) = sender.send(Message::Text(json_data)).await {
            return Err(format!("Failed to send message: {}", e));
        }
    }
    
    Ok(())
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

/// Handler for retrieving all current metrics
async fn get_all_metrics(
    State(state): State<ServerState>,
) -> std::result::Result<Json<HashMap<String, Value>>, (StatusCode, String)> {
    // Get all component data from the manager
    let mut metrics = HashMap::new();
    let components = state.manager.get_available_components();
    
    for component_id in components {
        if let Some(data) = state.manager.get_component_data(&component_id) {
            // Only include the latest update for each component
            if let Some(latest) = data.last() {
                metrics.insert(component_id, latest.data.clone());
            }
        }
    }
    
    Ok(Json(metrics))
}

/// Handler for retrieving a specific metric by ID
async fn get_metric_by_id(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get component data from the manager
    match state.manager.get_component_data(&id) {
        Some(data) if !data.is_empty() => {
            // Return the latest data point
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, format!("Metric '{}' not found", id)))
    }
}

/// Handler for retrieving CPU metrics
async fn get_cpu_metrics(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get CPU metrics from the manager
    match state.manager.get_component_data("system_cpu") {
        Some(data) if !data.is_empty() => {
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, "CPU metrics not available".to_string()))
    }
}

/// Handler for retrieving memory metrics
async fn get_memory_metrics(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get memory metrics from the manager
    match state.manager.get_component_data("system_memory") {
        Some(data) if !data.is_empty() => {
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, "Memory metrics not available".to_string()))
    }
}

/// Handler for retrieving disk metrics
async fn get_disk_metrics(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get disk metrics from the manager
    match state.manager.get_component_data("disk_usage") {
        Some(data) if !data.is_empty() => {
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, "Disk metrics not available".to_string()))
    }
}

/// Handler for retrieving network metrics
async fn get_network_metrics(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get network metrics from the manager
    match state.manager.get_component_data("network_traffic") {
        Some(data) if !data.is_empty() => {
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, "Network metrics not available".to_string()))
    }
}

/// Handler for retrieving health status
async fn get_health_status(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Value>, (StatusCode, String)> {
    // Get health status from the manager
    match state.manager.get_component_data("health_status") {
        Some(data) if !data.is_empty() => {
            Ok(Json(data.last().unwrap().data.clone()))
        },
        _ => Err((StatusCode::NOT_FOUND, "Health status not available".to_string()))
    }
}

/// Handler for listing all available components
async fn list_available_components(
    State(state): State<ServerState>,
) -> std::result::Result<Json<Vec<String>>, (StatusCode, String)> {
    // Get list of available components from the manager
    let components = state.manager.get_available_components();
    Ok(Json(components))
} 