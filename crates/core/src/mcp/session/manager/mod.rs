/// Module for session manager functionality.
///
/// Contains the persistence layer for sessions and handles
/// session creation, validation, and lifecycle operations.
pub mod persistence;

use std::sync::Arc;
use tokio::sync::RwLock;
use super::Session;

/// MCP Session Manager
#[derive(Debug)]
pub struct MCPSessionManager {
    sessions: RwLock<Vec<Arc<Session>>>,
}

impl MCPSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }
    
    /// Get all active sessions
    pub async fn get_sessions(&self) -> Vec<Arc<Session>> {
        self.sessions.read().await.clone()
    }
}

impl Default for MCPSessionManager {
    fn default() -> Self {
        Self::new()
    }
} 