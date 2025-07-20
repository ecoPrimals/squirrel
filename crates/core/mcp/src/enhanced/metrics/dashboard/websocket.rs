//! WebSocket connection management for dashboard

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// WebSocket connection
#[derive(Debug)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: String,
    
    /// Client address
    pub client_address: String,
    
    /// Connection timestamp
    pub connected_at: Instant,
    
    /// Last activity timestamp
    pub last_activity: Instant,
    
    /// Send channel
    pub sender: Option<mpsc::UnboundedSender<String>>,
    
    /// Connection state
    pub state: ConnectionState,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connecting
    Connecting,
    /// Connected and active
    Connected,
    /// Disconnected
    Disconnected,
    /// Error state
    Error,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new(id: String, client_address: String) -> Self {
        Self {
            id,
            client_address,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            sender: None,
            state: ConnectionState::Connecting,
        }
    }
    
    /// Check if connection is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ConnectionState::Connected) &&
        self.last_activity.elapsed() < Duration::from_secs(300) // 5 minutes timeout
    }
    
    /// Send message to client
    pub fn send_message(&self, message: String) -> Result<(), String> {
        if let Some(sender) = &self.sender {
            sender.send(message).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    
    /// Update last activity
    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
    
    /// Close connection
    pub async fn close(&self) {
        // Implementation would close the WebSocket connection
    }
} 