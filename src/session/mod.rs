//! Session management for MCP
//! 
//! Basic session management functionality for MCP protocol.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session ID
    pub id: String,
    /// Session timeout in seconds
    pub timeout: u64,
    /// Maximum connections
    pub max_connections: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timeout: 3600, // 1 hour
            max_connections: 100,
        }
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID
    pub session_id: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// User ID
    pub user_id: Option<String>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            created_at: now,
            last_activity: now,
            user_id: None,
        }
    }
}

/// Session state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active
    Active,
    /// Session is idle
    Idle,
    /// Session is terminated
    Terminated,
}

/// Session manager trait
pub trait SessionManager: Send + Sync {
    /// Create a new session
    fn create_session(&self, config: SessionConfig) -> impl std::future::Future<Output = Result<String, crate::error::MCPError>> + Send;
    
    /// Get session metadata
    fn get_session(&self, session_id: &str) -> impl std::future::Future<Output = Result<SessionMetadata, crate::error::MCPError>> + Send;
    
    /// Update session state
    fn update_session_state(&self, session_id: &str, state: SessionState) -> impl std::future::Future<Output = Result<(), crate::error::MCPError>> + Send;
    
    /// Close session
    fn close_session(&self, session_id: &str) -> impl std::future::Future<Output = Result<(), crate::error::MCPError>> + Send;
} 