//! WebSocket connection manager for handling client connections and broadcasting messages.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_stream::Stream;
use uuid::Uuid;
use serde_json::Value;
use tracing::{debug, error, info, warn};

use super::error::WebSocketError;
use super::models::{ChannelCategory, Subscription, WebSocketEvent, WebSocketResponse};

/// Size of the channel for broadcasting messages to all connected clients
const BROADCAST_CHANNEL_SIZE: usize = 1000;

/// Maximum number of subscribers allowed per channel
const MAX_SUBSCRIBERS_PER_CHANNEL: usize = 10000;

/// Connection ID
pub type ConnectionId = String;

/// Channel identifier (category:name)
pub type ChannelId = String;

/// Connection information
#[derive(Debug)]
pub struct Connection {
    /// Unique connection identifier
    pub id: ConnectionId,
    
    /// User ID associated with this connection, if authenticated
    pub user_id: Option<String>,
    
    /// Roles assigned to the user
    pub roles: Vec<String>,
    
    /// Sender to send messages to this specific client
    pub sender: mpsc::Sender<Result<String, WebSocketError>>,
    
    /// Channels this connection is subscribed to
    pub subscriptions: HashSet<ChannelId>,
}

/// Connection manager for handling WebSocket connections
#[derive(Debug, Clone)]
pub struct ConnectionManager {
    /// Active connections
    connections: Arc<RwLock<HashMap<ConnectionId, Connection>>>,
    
    /// Channel subscriptions - maps channel_id -> set of connection_ids
    channel_subscribers: Arc<RwLock<HashMap<ChannelId, HashSet<ConnectionId>>>>,
    
    /// User subscriptions - maps user_id -> set of connection_ids
    user_connections: Arc<RwLock<HashMap<String, HashSet<ConnectionId>>>>,
    
    /// Broadcast channel for all events
    event_tx: broadcast::Sender<WebSocketEvent>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            channel_subscribers: Arc::new(RwLock::new(HashMap::new())),
            user_connections: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }
    
    /// Register a new connection
    pub async fn register_connection(
        &self,
        user_id: Option<String>,
        roles: Vec<String>,
        sender: mpsc::Sender<Result<String, WebSocketError>>,
    ) -> ConnectionId {
        let connection_id = Uuid::new_v4().to_string();
        
        let connection = Connection {
            id: connection_id.clone(),
            user_id: user_id.clone(),
            roles,
            sender,
            subscriptions: HashSet::new(),
        };
        
        // Store the connection
        self.connections.write().await.insert(connection_id.clone(), connection);
        
        // Associate the connection with the user if authenticated
        if let Some(uid) = user_id {
            let mut user_conns = self.user_connections.write().await;
            user_conns
                .entry(uid)
                .or_insert_with(HashSet::new)
                .insert(connection_id.clone());
        }
        
        info!("New WebSocket connection registered: {}", connection_id);
        connection_id
    }
    
    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &str) {
        let mut remove_user_id = None;
        let mut channels_to_check = Vec::new();
        
        // Get connection data
        if let Some(conn) = self.connections.write().await.remove(connection_id) {
            remove_user_id = conn.user_id;
            channels_to_check = conn.subscriptions.into_iter().collect();
        }
        
        // Remove from user connections
        if let Some(user_id) = remove_user_id {
            let mut user_conns = self.user_connections.write().await;
            if let Some(conns) = user_conns.get_mut(&user_id) {
                conns.remove(connection_id);
                if conns.is_empty() {
                    user_conns.remove(&user_id);
                }
            }
        }
        
        // Remove from channel subscribers
        let mut channel_subs = self.channel_subscribers.write().await;
        for channel in channels_to_check {
            if let Some(conns) = channel_subs.get_mut(&channel) {
                conns.remove(connection_id);
                if conns.is_empty() {
                    channel_subs.remove(&channel);
                }
            }
        }
        
        info!("WebSocket connection removed: {}", connection_id);
    }
    
    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        connection_id: &str,
        category: ChannelCategory,
        channel: &str,
    ) -> Result<(), WebSocketError> {
        let channel_id = format!("{}:{}", category.as_str(), channel);
        
        // Get connection
        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(connection_id)
            .ok_or(WebSocketError::Internal("Connection not found".into()))?;
        
        // Check if already subscribed
        if connection.subscriptions.contains(&channel_id) {
            return Ok(());
        }
        
        // Add to connection's subscriptions
        connection.subscriptions.insert(channel_id.clone());
        
        // Add to channel subscribers
        let mut channel_subs = self.channel_subscribers.write().await;
        let subscribers = channel_subs
            .entry(channel_id.clone())
            .or_insert_with(HashSet::new);
        
        // Check max subscribers limit
        if subscribers.len() >= MAX_SUBSCRIBERS_PER_CHANNEL {
            return Err(WebSocketError::SubscriptionError(
                "Channel has reached maximum number of subscribers".into(),
            ));
        }
        
        subscribers.insert(connection_id.to_string());
        
        debug!(
            "Connection {} subscribed to channel {}:{}",
            connection_id,
            category.as_str(),
            channel
        );
        
        Ok(())
    }
    
    /// Unsubscribe from a channel
    pub async fn unsubscribe(
        &self,
        connection_id: &str,
        category: ChannelCategory,
        channel: &str,
    ) -> Result<(), WebSocketError> {
        let channel_id = format!("{}:{}", category.as_str(), channel);
        
        // Get connection
        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(connection_id)
            .ok_or(WebSocketError::Internal("Connection not found".into()))?;
        
        // Remove from connection's subscriptions
        connection.subscriptions.remove(&channel_id);
        
        // Remove from channel subscribers
        let mut channel_subs = self.channel_subscribers.write().await;
        if let Some(subscribers) = channel_subs.get_mut(&channel_id) {
            subscribers.remove(connection_id);
            if subscribers.is_empty() {
                channel_subs.remove(&channel_id);
            }
        }
        
        debug!(
            "Connection {} unsubscribed from channel {}:{}",
            connection_id,
            category.as_str(),
            channel
        );
        
        Ok(())
    }
    
    /// Send a message to a specific connection
    pub async fn send_to_connection(
        &self,
        connection_id: &str,
        message: WebSocketResponse,
    ) -> Result<(), WebSocketError> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(connection_id) {
            let json = serde_json::to_string(&message)
                .map_err(WebSocketError::JsonError)?;
            
            connection
                .sender
                .send(Ok(json))
                .await
                .map_err(|_| WebSocketError::SendError("Failed to send message".into()))?;
            
            Ok(())
        } else {
            Err(WebSocketError::Internal("Connection not found".into()))
        }
    }
    
    /// Send a message to a specific user (all connections)
    pub async fn send_to_user(
        &self,
        user_id: &str,
        message: WebSocketResponse,
    ) -> Result<(), WebSocketError> {
        let user_conns = self.user_connections.read().await;
        
        if let Some(connections) = user_conns.get(user_id) {
            let json = serde_json::to_string(&message)
                .map_err(WebSocketError::JsonError)?;
            
            let mut sent_count = 0;
            let connections_lock = self.connections.read().await;
            
            for conn_id in connections {
                if let Some(connection) = connections_lock.get(conn_id) {
                    if let Err(e) = connection.sender.send(Ok(json.clone())).await {
                        warn!("Failed to send message to connection {}: {}", conn_id, e);
                    } else {
                        sent_count += 1;
                    }
                }
            }
            
            if sent_count == 0 {
                return Err(WebSocketError::SendError("No active connections found".into()));
            }
            
            Ok(())
        } else {
            Err(WebSocketError::Internal("User has no active connections".into()))
        }
    }
    
    /// Broadcast an event to a specific channel
    pub async fn broadcast_to_channel(
        &self,
        category: ChannelCategory,
        channel: &str,
        event: &str,
        data: Value,
    ) -> Result<usize, WebSocketError> {
        let channel_id = format!("{}:{}", category.as_str(), channel);
        
        // Create WebSocket event
        let ws_event = WebSocketEvent {
            event: event.to_string(),
            category: category.clone(),
            channel: channel.to_string(),
            data: data.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Send to broadcast channel
        if let Err(e) = self.event_tx.send(ws_event) {
            error!("Failed to broadcast event: {}", e);
        }
        
        // Get subscribers for this channel
        let channel_subs = self.channel_subscribers.read().await;
        let subscribers = if let Some(subs) = channel_subs.get(&channel_id) {
            subs.clone()
        } else {
            return Ok(0); // No subscribers
        };
        
        // Create response message
        let response = WebSocketResponse {
            success: true,
            event: event.to_string(),
            data,
            error: None,
            id: None,
        };
        
        let json = serde_json::to_string(&response)
            .map_err(WebSocketError::JsonError)?;
        
        // Send to all subscribers
        let connections_lock = self.connections.read().await;
        let mut sent_count = 0;
        
        for conn_id in subscribers {
            if let Some(connection) = connections_lock.get(&conn_id) {
                if let Err(e) = connection.sender.send(Ok(json.clone())).await {
                    warn!("Failed to send message to connection {}: {}", conn_id, e);
                } else {
                    sent_count += 1;
                }
            }
        }
        
        Ok(sent_count)
    }
    
    /// Subscribe to all events (used internally for integration with MCP)
    pub fn subscribe_to_events(&self) -> impl Stream<Item = WebSocketEvent> {
        struct EventStream {
            rx: broadcast::Receiver<WebSocketEvent>,
        }
        
        impl Stream for EventStream {
            type Item = WebSocketEvent;
            
            fn poll_next(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                use std::task::Poll;
                
                // Use try_recv in a non-blocking way
                match self.rx.try_recv() {
                    Ok(event) => Poll::Ready(Some(event)),
                    Err(broadcast::error::TryRecvError::Empty) => {
                        // No data available yet, register waker and return pending
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                    Err(broadcast::error::TryRecvError::Lagged(_)) => {
                        // We fell behind, just continue with next message
                        warn!("Broadcast channel lagged behind, skipping messages");
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                    Err(broadcast::error::TryRecvError::Closed) => {
                        // Channel is closed, end the stream
                        Poll::Ready(None)
                    },
                }
            }
        }
        
        EventStream {
            rx: self.event_tx.subscribe(),
        }
    }
    
    /// Get active connections count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
    
    /// Get subscription count for a specific channel
    pub async fn subscription_count(&self, category: ChannelCategory, channel: &str) -> usize {
        let channel_id = format!("{}:{}", category.as_str(), channel);
        let channel_subs = self.channel_subscribers.read().await;
        
        channel_subs
            .get(&channel_id)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }
    
    /// Get all active subscriptions
    pub async fn get_active_subscriptions(&self) -> Vec<Subscription> {
        let mut result = Vec::new();
        let connections = self.connections.read().await;
        let channel_subs = self.channel_subscribers.read().await;
        
        for (channel_id, conn_ids) in channel_subs.iter() {
            if let Some((category_str, channel_name)) = channel_id.split_once(':') {
                if let Some(category) = ChannelCategory::from_str(category_str) {
                    for conn_id in conn_ids {
                        if let Some(conn) = connections.get(conn_id) {
                            if let Some(user_id) = &conn.user_id {
                                result.push(Subscription {
                                    user_id: user_id.clone(),
                                    category: category.clone(),
                                    channel: channel_name.to_string(),
                                    filter: Value::Null,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        result
    }
} 