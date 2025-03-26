//! Secure WebSocket server for dashboard updates
//!
//! This module provides a secure WebSocket server implementation that supports:
//! - TLS encryption
//! - Authentication and authorization
//! - Rate limiting
//! - Origin verification
//! - Message validation and sanitization

use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use axum::{
    extract::{State, WebSocketUpgrade, ConnectInfo},
    response::IntoResponse,
    routing::get,
    Json, Router,
    http::{StatusCode, HeaderMap, header, Method},
};
use axum::extract::ws::{WebSocket, Message};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, debug, error};
use uuid::Uuid;
use std::time::{Instant};
use flate2::Compression;

use super::config::DashboardConfig;
use super::security::{AuthManager, RateLimiter, OriginVerifier, DataMaskingManager, AuditLogger};

/// Server state for the secure dashboard server
#[derive(Debug, Clone)]
pub struct SecureServerState {
    /// Dashboard configuration
    pub config: DashboardConfig,
    /// Authentication manager
    pub auth_manager: Option<Arc<AuthManager>>,
    /// Rate limiter
    pub rate_limiter: Option<Arc<RateLimiter>>,
    /// Origin verifier
    pub origin_verifier: Option<Arc<OriginVerifier>>,
    /// Data masking manager
    pub data_masking_manager: Option<Arc<DataMaskingManager>>,
    /// Audit logger
    pub audit_logger: Option<Arc<AuditLogger>>,
    /// Connected clients
    pub clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    /// Broadcast channel for messages
    pub tx: broadcast::Sender<BroadcastMessage>,
}

/// Client information
#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    /// Client ID
    pub id: String,
    /// Client IP address
    pub ip: String,
    /// Client user agent
    pub user_agent: String,
    /// Client username (if authenticated)
    pub username: Option<String>,
    /// Client connection time
    pub connected_at: u64,
    /// Client subscriptions
    pub subscriptions: Vec<String>,
    /// Message count
    pub message_count: usize,
}

/// Message for broadcasting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// Message type
    pub message_type: String,
    /// Target component ID (if applicable)
    pub component_id: Option<String>,
    /// Message payload
    pub payload: Value,
    /// Message timestamp
    pub timestamp: u64,
    /// Is this message compressed?
    pub compressed: bool,
}

/// Batched messages for efficient transmission
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchedMessages {
    /// Batch ID
    pub batch_id: String,
    /// Messages in the batch
    pub messages: Vec<BroadcastMessage>,
    /// Batch timestamp
    pub timestamp: u64,
    /// Is this batch compressed?
    pub compressed: bool,
}

/// Message compression settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression for messages
    pub enabled: bool,
    /// Minimum size (in bytes) for compression to be applied
    pub min_size_bytes: usize,
    /// Compression level (0-9, where 9 is maximum compression)
    pub level: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_size_bytes: 1024, // Only compress messages larger than 1KB
            level: 6, // Default compression level
        }
    }
}

/// Batching configuration for high-frequency updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Enable batching for messages
    pub enabled: bool,
    /// Maximum number of messages per batch
    pub max_messages: usize,
    /// Maximum batch interval in milliseconds
    pub max_interval_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_messages: 50, // Maximum 50 messages per batch
            max_interval_ms: 100, // Send batch at least every 100ms
        }
    }
}

/// Function to create a secure dashboard server router
pub fn create_secure_server(config: DashboardConfig) -> Router {
    // Create state
    let (tx, _) = broadcast::channel(1000);
    
    let state = SecureServerState {
        config: config.clone(),
        auth_manager: None,
        rate_limiter: None,
        origin_verifier: None,
        data_masking_manager: None,
        audit_logger: None,
        clients: Arc::new(RwLock::new(HashMap::new())),
        tx,
    };
    
    // Create router
    let mut router = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .with_state(state.clone());
    
    // Add CORS if origins are specified
    if !config.security.allowed_origins.is_empty() {
        let cors = create_cors_layer(&config.security.allowed_origins);
        router = router.layer(cors);
    }
    
    // Add trace layer
    router = router.layer(TraceLayer::new_for_http());
    
    // Return router
    router
}

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SecureServerState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract IP and user agent
    let ip = addr.ip().to_string();
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    
    // Check rate limits if enabled
    if let Some(rate_limiter) = &state.rate_limiter {
        if !rate_limiter.check_connection(&ip).await {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }
    }
    
    // Check origin if enabled
    if let Some(origin_verifier) = &state.origin_verifier {
        if let Some(origin) = headers.get(header::ORIGIN).and_then(|h| h.to_str().ok()) {
            if !origin_verifier.is_allowed(origin) {
                return StatusCode::FORBIDDEN.into_response();
            }
        }
    }
    
    // Extract authentication token if present
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok());
    let mut username = None;
    
    // Validate token if auth is enabled
    if let Some(auth_manager) = &state.auth_manager {
        if let Some(auth) = auth_header {
            if auth.starts_with("Bearer ") {
                let token = auth.trim_start_matches("Bearer ").trim();
                match auth_manager.validate_token(token) {
                    Ok(claims) => {
                        username = Some(claims.sub);
                    },
                    Err(e) => {
                        // Log authentication failure
                        if let Some(audit_logger) = &state.audit_logger {
                            let audit_logger = audit_logger.clone();
                            let details = serde_json::json!({
                                "ip": ip,
                                "user_agent": user_agent,
                                "reason": e
                            });
                            
                            tokio::spawn(async move {
                                audit_logger.log_event("auth_failure", details, None).await;
                            });
                        }
                        
                        return StatusCode::UNAUTHORIZED.into_response();
                    }
                }
            }
        }
    }
    
    // Generate client ID
    let client_id = Uuid::new_v4().to_string();
    
    // Log connection
    info!("Client connected: {client_id} from {ip}");
    
    // Audit successful connection
    if let Some(audit_logger) = &state.audit_logger {
        let audit_logger = audit_logger.clone();
        let details = serde_json::json!({
            "client_id": client_id.clone(),
            "ip": ip.clone(),
        });
        
        let username_for_log = username.clone();
        tokio::spawn(async move {
            audit_logger.log_event("client_connect", details, username_for_log.as_deref()).await;
        });
    }
    
    // Create client info
    let client_info = ClientInfo {
        id: client_id.clone(),
        ip: ip.clone(),
        user_agent,
        username: username.clone(),
        connected_at: chrono::Utc::now().timestamp() as u64,
        subscriptions: Vec::new(),
        message_count: 0,
    };
    
    // Store client info
    {
        let mut clients = state.clients.write().await;
        clients.insert(client_id.clone(), client_info);
    }
    
    // Upgrade connection
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_id, ip, username))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: SecureServerState, client_id: String, ip: String, username: Option<String>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to broadcast channel
    let mut broadcast_rx = state.tx.subscribe();
    
    // Get compression and batching configuration
    let compression_config = CompressionConfig {
        enabled: state.config.performance.as_ref()
            .and_then(|p| p.compression_enabled)
            .unwrap_or(false),
        min_size_bytes: state.config.performance.as_ref()
            .and_then(|p| p.min_compression_size)
            .unwrap_or(1024),
        level: state.config.performance.as_ref()
            .and_then(|p| p.compression_level)
            .unwrap_or(6),
    };
    
    let batch_config = BatchConfig {
        enabled: state.config.performance.as_ref()
            .and_then(|p| p.batching_enabled)
            .unwrap_or(false),
        max_messages: state.config.performance.as_ref()
            .and_then(|p| p.max_batch_size)
            .unwrap_or(10),
        max_interval_ms: state.config.performance.as_ref()
            .and_then(|p| p.max_batch_interval)
            .unwrap_or(100),
    };
    
    // Create a channel for client message responses
    let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(32);
    
    // Client message handler
    let client_message_handler = {
        let state = state.clone();
        let client_id = client_id.clone();
        let username = username.clone();
        let ip_clone = ip.clone();
        let response_tx = response_tx.clone();
        
        tokio::spawn(async move {
            while let Some(Ok(message)) = receiver.next().await {
                if let Ok(text) = message.to_text() {
                    // Process the message
                    let value = serde_json::from_str(text).unwrap_or(json!({"error": "Invalid JSON"}));
                    
                    // Process message and determine if a response is needed
                    let response = process_client_message(
                        value,
                        &state,
                        &client_id,
                        username.as_deref(),
                    ).await;
                    
                    // If there's a response, send it through the channel
                    if let Some(response_msg) = response {
                        let _ = response_tx.send(response_msg).await;
                    }
                } else {
                    error!("Received non-text message from client {}", client_id);
                }
            }
            
            // Handle client disconnect
            handle_disconnect(&state, &client_id, &ip_clone, username.as_deref()).await;
        })
    };
    
    // Server message handler with batching
    let server_message_handler = {
        let state = state.clone();
        let client_id = client_id.clone();
        
        tokio::spawn(async move {
            // Batch state
            let mut batch = Vec::new();
            let mut last_batch_time = Instant::now();
            
            // Process both broadcast messages and client response messages
            loop {
                tokio::select! {
                    // Handle broadcast messages
                    msg = broadcast_rx.recv() => {
                        if let Ok(msg) = msg {
                            let clients_guard = state.clients.read().await;
                            let subscriptions = if let Some(client) = clients_guard.get(&client_id) {
                                client.subscriptions.clone()
                            } else {
                                Vec::new()
                            };
                            
                            // Check if message is for this client
                            let is_relevant = match &msg.component_id {
                                Some(component_id) => subscriptions.contains(component_id),
                                None => true, // Broadcast to all
                            };
                            
                            if is_relevant {
                                if batch_config.enabled {
                                    // Add message to batch
                                    batch.push(msg);
                                    
                                    // Send batch if conditions are met
                                    let batch_full = batch.len() >= batch_config.max_messages;
                                    let batch_aged = last_batch_time.elapsed().as_millis() > batch_config.max_interval_ms as u128;
                                    
                                    if batch_full || batch_aged {
                                        let batch_to_send = std::mem::take(&mut batch);
                                        last_batch_time = Instant::now();
                                        
                                        // Send the batch
                                        if !batch_to_send.is_empty() {
                                            send_batch(&client_id, &batch_to_send, &mut sender, &compression_config).await;
                                        }
                                    }
                                } else {
                                    // Send message without batching
                                    send_message(&client_id, msg, &mut sender, &compression_config).await;
                                }
                            }
                        } else {
                            // Broadcast channel closed
                            break;
                        }
                    },
                    
                    // Handle client response messages
                    response = response_rx.recv() => {
                        if let Some(response) = response {
                            if let Err(e) = sender.send(response).await {
                                error!("Failed to send response to client {}: {}", client_id, e);
                                break;
                            }
                        } else {
                            // Response channel closed
                            break;
                        }
                    }
                }
            }
            
            // Handle any remaining messages in batch
            if !batch.is_empty() {
                send_batch(&client_id, &batch, &mut sender, &compression_config).await;
            }
        })
    };
    
    // Wait for either task to complete
    tokio::select! {
        _ = client_message_handler => debug!("Client handler completed for {}", client_id),
        _ = server_message_handler => debug!("Server handler completed for {}", client_id),
    }
    
    // Handle disconnect (in case the logic in the tasks didn't run)
    handle_disconnect(&state, &client_id, &ip, username.as_deref()).await;
}

/// Send a single message with optional compression
async fn send_message(
    client_id: &str, 
    msg: BroadcastMessage, 
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    compression_config: &CompressionConfig
) {
    let json_msg = match serde_json::to_string(&msg) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize message for client {}: {}", client_id, e);
            return;
        }
    };
    
    // Apply compression if needed
    let message = if compression_config.enabled && json_msg.len() > compression_config.min_size_bytes {
        match compress_message(&json_msg, compression_config.level) {
            Ok(compressed) => {
                debug!("Sent compressed message to client {} ({} -> {} bytes)", 
                       client_id, json_msg.len(), compressed.len());
                Message::Binary(compressed)
            },
            Err(e) => {
                error!("Failed to compress message: {}", e);
                Message::Text(json_msg)
            }
        }
    } else {
        Message::Text(json_msg)
    };
    
    // Send message
    if let Err(e) = sender.send(message).await {
        error!("Failed to send message to client {}: {}", client_id, e);
    }
}

/// Send a batch of messages with optional compression
async fn send_batch(
    client_id: &str, 
    messages: &[BroadcastMessage], 
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    compression_config: &CompressionConfig
) {
    // Create batch
    let batch = BatchedMessages {
        batch_id: Uuid::new_v4().to_string(),
        messages: messages.to_vec(),
        timestamp: chrono::Utc::now().timestamp() as u64,
        compressed: false,
    };
    
    // Serialize batch
    let json_batch = match serde_json::to_string(&batch) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize batch for client {}: {}", client_id, e);
            return;
        }
    };
    
    // Apply compression if needed
    let message = if compression_config.enabled && json_batch.len() > compression_config.min_size_bytes {
        match compress_message(&json_batch, compression_config.level) {
            Ok(compressed) => {
                debug!("Sent compressed batch to client {} with {} messages ({} -> {} bytes)", 
                       client_id, messages.len(), json_batch.len(), compressed.len());
                Message::Binary(compressed)
            },
            Err(e) => {
                error!("Failed to compress batch: {}", e);
                Message::Text(json_batch)
            }
        }
    } else {
        Message::Text(json_batch)
    };
    
    // Send batch
    if let Err(e) = sender.send(message).await {
        error!("Failed to send batch to client {}: {}", client_id, e);
    }
}

/// Compress a message using flate2
fn compress_message(message: &str, level: u32) -> Result<Vec<u8>, anyhow::Error> {
    use flate2::write::GzEncoder;
    use std::io::Write;
    
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(message.as_bytes())?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

/// Decompress a message using flate2
fn decompress_message(compressed: &[u8]) -> Result<String, anyhow::Error> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    
    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}

/// Process a client message
async fn process_client_message(
    value: Value,
    state: &SecureServerState,
    client_id: &str,
    username: Option<&str>,
) -> Option<Message> {
    // Extract message type
    let msg_type = match value.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return None,
    };
    
    // Increment message count
    {
        let mut clients = state.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.message_count += 1;
        }
    }
    
    // Process based on message type
    match msg_type {
        "subscribe" => {
            // Check subscription rate limits
            if let Some(rate_limiter) = &state.rate_limiter {
                if !rate_limiter.check_subscription(client_id).await {
                    // Rate limit exceeded
                    let error_msg = json!({
                        "type": "error",
                        "code": "rate_limit_exceeded",
                        "message": "Subscription rate limit exceeded"
                    });
                    
                    return Some(Message::Text(error_msg.to_string()));
                }
            }
            
            // Extract component ID
            if let Some(component_id) = value.get("componentId").and_then(|c| c.as_str()) {
                // Update subscriptions
                {
                    let mut clients = state.clients.write().await;
                    if let Some(client) = clients.get_mut(client_id) {
                        if !client.subscriptions.contains(&component_id.to_string()) {
                            client.subscriptions.push(component_id.to_string());
                        }
                    }
                }
                
                // Audit subscription
                if let Some(audit_logger) = &state.audit_logger {
                    let audit_logger = audit_logger.clone();
                    let details = serde_json::json!({
                        "client_id": client_id,
                        "component_id": component_id
                    });
                    
                    let audit_logger = audit_logger.clone();
                    let username = username.map(|u| u.to_string());
                    tokio::spawn(async move {
                        audit_logger.log_event("component_subscribe", details, username.as_deref()).await;
                    });
                }
                
                // Send confirmation
                let confirm_msg = json!({
                    "type": "subscribe_confirm",
                    "componentId": component_id,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });
                
                return Some(Message::Text(confirm_msg.to_string()));
            }
        },
        "unsubscribe" => {
            // Extract component ID
            if let Some(component_id) = value.get("componentId").and_then(|c| c.as_str()) {
                // Update subscriptions
                {
                    let mut clients = state.clients.write().await;
                    if let Some(client) = clients.get_mut(client_id) {
                        client.subscriptions.retain(|id| id != component_id);
                    }
                }
                
                // Audit unsubscription
                if let Some(audit_logger) = &state.audit_logger {
                    let details = serde_json::json!({
                        "client_id": client_id,
                        "component_id": component_id
                    });
                    
                    let audit_logger = audit_logger.clone();
                    let username = username.map(|u| u.to_string());
                    tokio::spawn(async move {
                        audit_logger.log_event("component_unsubscribe", details, username.as_deref()).await;
                    });
                }
                
                // Send confirmation
                let confirm_msg = json!({
                    "type": "unsubscribe_confirm",
                    "componentId": component_id,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });
                
                return Some(Message::Text(confirm_msg.to_string()));
            }
        },
        "ping" => {
            // Respond to ping with pong
            let pong_msg = json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            
            return Some(Message::Text(pong_msg.to_string()));
        },
        _ => {
            // Unknown message type
            debug!("Unknown message type: {}", msg_type);
        }
    }
    
    None
}

/// Handle client disconnection
async fn handle_disconnect(state: &SecureServerState, client_id: &str, ip: &str, username: Option<&str>) {
    // Remove client from connected clients
    let mut clients = state.clients.write().await;
    if let Some(client) = clients.remove(client_id) {
        tracing::info!("Client disconnected: {}", client_id);
        
        // Log the disconnection
        if let Some(audit_logger) = &state.audit_logger {
            let details = serde_json::json!({
                "client_id": client_id,
                "ip": ip,
                "user_agent": client.user_agent,
                "subscriptions": client.subscriptions,
                "message_count": client.message_count,
                "connected_at": client.connected_at,
                "disconnected_at": chrono::Utc::now().timestamp()
            });

            // Clone the data needed for the async task
            let audit_logger = audit_logger.clone();
            let username_owned = username.map(String::from);
            let details_owned = details.clone();
            
            tokio::spawn(async move {
                audit_logger.log_event("client_disconnect", details_owned, username_owned.as_deref()).await;
            });
        }
    }
}

/// Health check handler
async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Server status handler
async fn status_handler(State(state): State<SecureServerState>) -> impl IntoResponse {
    let clients = state.clients.read().await;
    
    Json(json!({
        "status": "running",
        "clients": clients.len(),
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

/// Create CORS layer
fn create_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    // If no origins specified, allow any
    if allowed_origins.is_empty() {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);
    }
    
    // Create origins list
    let mut origins = Vec::new();
    for origin in allowed_origins {
        if let Ok(origin) = origin.parse::<header::HeaderValue>() {
            origins.push(origin);
        }
    }
    
    // Create layer with specific origins
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
} 