//! Session management for MCP
//!
//! Core session handling functionality for the MCP protocol.
//! Complex authentication and authorization moved to BearDog.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;
use crate::protocol::types::{MCPMessage, MessageId};

// Remove imports for types moved to other projects:
// SessionToken, UserId, RoleId, AuthToken, SessionId → Moved to BearDog

/// Basic session information for core MCP functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub metadata: HashMap<String, String>,
}

/// Session management interface
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Create a new session
    async fn create_session(&self, metadata: HashMap<String, String>) -> Result<Session>;
    
    /// Get session by ID
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
    
    /// Update session activity
    async fn update_activity(&self, id: &str) -> Result<()>;
    
    /// Remove session
    async fn remove_session(&self, id: &str) -> Result<()>;
}

/// Simple session manager implementation
pub struct CoreSessionManager {
    sessions: Arc<tokio::sync::RwLock<HashMap<String, Session>>>,
}

impl CoreSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionManager for CoreSessionManager {
    async fn create_session(&self, metadata: HashMap<String, String>) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            metadata,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        
        Ok(session)
    }
    
    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }
    
    async fn update_activity(&self, id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            session.last_activity = SystemTime::now();
        }
        Ok(())
    }
    
    async fn remove_session(&self, id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        Ok(())
    }
} 