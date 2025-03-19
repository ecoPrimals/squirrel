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
    /// Collection of active sessions managed by this manager
    /// 
    /// This field contains all active sessions in memory, stored in a thread-safe
    /// container that allows concurrent reads and exclusive writes. Sessions are
    /// stored as Arc<Session> to enable safe sharing across different parts of 
    /// the application without copying the entire session data.
    sessions: RwLock<Vec<Arc<Session>>>,
}

impl MCPSessionManager {
    /// Create a new session manager
    #[must_use] pub fn new() -> Self {
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