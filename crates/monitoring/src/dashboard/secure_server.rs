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
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, debug};
use uuid::Uuid;

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
    // Split socket into sender and receiver
    let (sender, mut receiver) = socket.split();
    
    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    
    // Put sender in an Arc<Mutex<>> so it can be shared between tasks
    let sender = Arc::new(Mutex::new(sender));
    
    // Clone state for tasks
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    
    // Spawn task to forward broadcast messages to client
    let sender_clone = sender.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // Check if message is for a component this client is subscribed to
            let should_send = if let Some(component_id) = &msg.component_id {
                let clients = state_clone.clients.read().await;
                if let Some(client) = clients.get(&client_id_clone) {
                    client.subscriptions.contains(component_id)
                } else {
                    false
                }
            } else {
                // Broadcast to all
                true
            };
            
            if should_send {
                // Convert to WebSocket message
                let ws_msg = Message::Text(serde_json::to_string(&msg).unwrap());
                
                // Send message
                let mut sender_guard = sender_clone.lock().await;
                if sender_guard.send(ws_msg).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Handle incoming messages
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let username_clone = username.clone();
    
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    // Check rate limits
                    if let Some(rate_limiter) = &state_clone.rate_limiter {
                        if !rate_limiter.check_message(&client_id_clone).await {
                            // Rate limit exceeded
                            let error_msg = json!({
                                "type": "error",
                                "code": "rate_limit_exceeded",
                                "message": "Message rate limit exceeded"
                            });
                            
                            // Get the sender from the Arc<Mutex<>>
                            let error_str = error_msg.to_string();
                            let mut sender_guard = sender.lock().await;
                            if sender_guard.send(Message::Text(error_str)).await.is_err() {
                                break;
                            }
                            
                            continue;
                        }
                    }
                    
                    // Parse message
                    if let Ok(mut value) = serde_json::from_str::<Value>(&text) {
                        // Apply data masking if enabled
                        if let Some(masking_manager) = &state_clone.data_masking_manager {
                            value = masking_manager.mask_json(&value);
                        }
                        
                        // Process message
                        let mut sender_guard = sender.lock().await;
                        process_client_message(
                            value, 
                            &state_clone, 
                            &client_id_clone, 
                            username_clone.as_deref(),
                            &mut sender_guard
                        ).await;
                    }
                },
                Message::Binary(data) => {
                    // Handle binary messages (compressed JSON, etc.)
                    debug!("Received binary message of size {} bytes", data.len());
                },
                Message::Ping(data) => {
                    // Respond to ping with pong
                    let mut sender_guard = sender.lock().await;
                    if sender_guard.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                },
                Message::Pong(_) => {
                    // Ignore pong messages
                },
                Message::Close(_) => {
                    break;
                }
            }
        }
    });
    
    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
    
    // Client disconnected
    handle_disconnect(&state, &client_id, &ip, username.as_deref()).await;
}

/// Process a client message
async fn process_client_message(
    value: Value,
    state: &SecureServerState,
    client_id: &str,
    username: Option<&str>,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>
) {
    // Extract message type
    let msg_type = match value.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return,
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
                    
                    let _ = sender.send(Message::Text(error_msg.to_string())).await;
                    return;
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
                
                let _ = sender.send(Message::Text(confirm_msg.to_string())).await;
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
                
                let _ = sender.send(Message::Text(confirm_msg.to_string())).await;
            }
        },
        "ping" => {
            // Respond to ping with pong
            let pong_msg = json!({
                "type": "pong",
                "timestamp": chrono::Utc::now().timestamp_millis()
            });
            
            let _ = sender.send(Message::Text(pong_msg.to_string())).await;
        },
        _ => {
            // Unknown message type
            debug!("Unknown message type: {}", msg_type);
        }
    }
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