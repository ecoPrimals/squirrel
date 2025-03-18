// MCP Session Manager
pub mod persistence;

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;
use super::MCPSession;

/// MCP Session Manager
#[derive(Debug)]
pub struct MCPSessionManager {
    sessions: RwLock<Vec<Arc<dyn MCPSession>>>,
}

impl MCPSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }
    
    /// Get all active sessions
    pub async fn get_sessions(&self) -> Vec<Arc<dyn MCPSession>> {
        self.sessions.read().await.clone()
    }
}

impl Default for MCPSessionManager {
    fn default() -> Self {
        Self::new()
    }
} 