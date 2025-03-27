use std::fmt::{Debug, Formatter, Result as FmtResult};
use tokio::sync::RwLock;

/// WebSocket connection handler
/// 
/// Manages a client connection's state, including topic subscriptions.
/// This struct uses async-aware locking to ensure proper concurrency.
pub struct WebSocketConnection {
    /// List of topics this client is subscribed to
    pub subscriptions: RwLock<Vec<String>>,
    /// Unique identifier for this client
    pub client_id: String,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new(client_id: String) -> Self {
        Self {
            subscriptions: RwLock::new(Vec::new()),
            client_id,
        }
    }

    /// Get the client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Get subscriptions
    pub async fn subscriptions(&self) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.clone()
    }

    /// Add a subscription
    pub async fn subscribe(&self, topic: String) {
        let mut subscriptions = self.subscriptions.write().await;
        if !subscriptions.contains(&topic) {
            subscriptions.push(topic);
        }
    }

    /// Remove a subscription
    pub async fn unsubscribe(&self, topic: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.retain(|t| t != topic);
    }

    /// Check if subscribed to a topic
    pub async fn is_subscribed(&self, topic: &str) -> bool {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.iter().any(|t| t == topic)
    }

    /// Get a copy of all subscriptions
    pub async fn get_subscriptions(&self) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.clone()
    }
}

impl Debug for WebSocketConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("WebSocketConnection")
            .field("client_id", &self.client_id)
            .field("subscriptions", &"[subscriptions]") // Can't display RwLock contents directly
            .finish()
    }
} 