//! Session management for MCP connections
//!
//! This module provides session management functionality for MCP connections.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::PrimalError;
use crate::protocol::types::SessionId;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub timeout: std::time::Duration,
    pub max_connections: u32,
    pub enable_logging: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        let timeout_secs = std::env::var("SESSION_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(300); // Default 5 minutes

        let max_connections = std::env::var("SESSION_MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(100);

        Self {
            timeout: std::time::Duration::from_secs(timeout_secs),
            max_connections,
            enable_logging: true,
        }
    }
}

/// Metadata for a user session
///
/// Contains essential information about a session including timing,
/// client information, and capabilities. This structure is serializable
/// for persistence and can be cloned efficiently.
///
/// # Fields
///
/// * `session_id` - Unique identifier for the session
/// * `created_at` - UTC timestamp when session was created
/// * `last_activity` - UTC timestamp of last session activity
/// * `client_info` - Optional information about the client
/// * `capabilities` - List of capabilities supported by the session
///
/// # Examples
///
/// ```rust
/// use squirrel::session::SessionMetadata;
/// use chrono::Utc;
///
/// let metadata = SessionMetadata {
///     session_id: "session_123".to_string(),
///     created_at: Utc::now(),
///     last_activity: Utc::now(),
///     client_info: Some("WebApp v1.0".to_string()),
///     capabilities: vec!["mcp".to_string(), "ai_intelligence".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub client_info: Option<String>,
    pub capabilities: Vec<String>,
}

/// Current state of a user session
///
/// Represents the lifecycle state of a session, used for session management
/// and cleanup operations. States transition in a predictable manner:
/// Active → Inactive → Terminated.
///
/// # Variants
///
/// * `Active` - Session is currently active and processing requests
/// * `Inactive` - Session is idle but can be reactivated
/// * `Terminated` - Session has been ended and cannot be reactivated
///
/// # Examples
///
/// ```rust
/// use squirrel::session::SessionState;
///
/// let state = SessionState::Active;
/// match state {
///     SessionState::Active => println!("Session is active"),
///     SessionState::Inactive => println!("Session is idle"),
///     SessionState::Terminated => println!("Session is terminated"),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is currently active and processing requests
    Active,
    /// Session is idle but can be reactivated
    Inactive,
    /// Session has been ended and cannot be reactivated
    Terminated,
}

/// Complete session information including metadata, state, and data
///
/// Represents a complete user session with all associated metadata, current state,
/// and arbitrary data storage. Sessions are typically shared across threads using
/// Arc wrapping for memory efficiency.
///
/// # Fields
///
/// * `metadata` - Session metadata including ID, timestamps, and capabilities
/// * `state` - Current lifecycle state of the session
/// * `data` - Key-value storage for session-specific data
///
/// # Thread Safety
///
/// Session instances are designed to be wrapped in Arc<Session> for efficient
/// sharing across threads. The internal HashMap is not synchronized, so external
/// synchronization is required for concurrent modifications.
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use squirrel::session::{Session, SessionMetadata, SessionState};
/// use chrono::Utc;
///
/// let metadata = SessionMetadata {
///     session_id: "user_session_123".to_string(),
///     created_at: Utc::now(),
///     last_activity: Utc::now(),
///     client_info: Some("Mobile App v2.1".to_string()),
///     capabilities: vec!["ai_chat".to_string()],
/// };
///
/// let mut data = HashMap::new();
/// data.insert("user_preferences".to_string(),
///             serde_json::json!({"theme": "dark", "language": "en"}));
///
/// let session = Arc::new(Session {
///     metadata,
///     state: SessionState::Active,
///     data,
/// });
/// ```
///
/// # Memory Efficiency
///
/// When stored in collections, sessions should be wrapped in Arc to avoid
/// expensive cloning operations:
///
/// ```rust
/// use std::collections::HashMap;
/// use std::sync::Arc;
///
/// // Efficient: Arc<Session> sharing
/// let sessions: HashMap<String, Arc<Session>> = HashMap::new();
/// // Inefficient: Direct Session cloning
/// // let sessions: HashMap<String, Session> = HashMap::new();
/// ```
#[derive(Debug, Clone)]
pub struct Session {
    pub metadata: SessionMetadata,
    pub state: SessionState,
    pub data: HashMap<String, serde_json::Value>,
}

/// Session manager implementation  
#[derive(Debug)]
pub struct SessionManagerImpl {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    config: SessionConfig,
}

impl SessionManagerImpl {
    /// Create a new session manager with the provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for session management including timeouts and connection limits
    ///
    /// # Returns
    ///
    /// A new SessionManagerImpl instance ready to manage sessions
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn create_session(
        &self,
        client_info: Option<String>,
    ) -> Result<SessionId, PrimalError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = Session {
            metadata: SessionMetadata {
                session_id: session_id.clone(),
                created_at: now,
                last_activity: now,
                client_info,
                capabilities: vec!["mcp".to_string(), "ai_intelligence".to_string()],
            },
            state: SessionState::Active,
            data: HashMap::new(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), Arc::new(session));

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Arc<Session>>, PrimalError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned()) // Now cloning Arc instead of Session - much cheaper!
    }

    pub async fn update_session(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session_arc) = sessions.get(session_id) {
            // Create a new updated session
            let session = session_arc.as_ref();
            let mut updated_metadata = session.metadata.clone();
            updated_metadata.last_activity = Utc::now();

            let mut updated_data = session.data.clone();
            updated_data.extend(data);

            let updated_session = Arc::new(Session {
                metadata: updated_metadata,
                data: updated_data,
                state: session.state.clone(),
            });

            // Replace the session in the map
            sessions.insert(session_id.to_string(), updated_session);
        }
        Ok(())
    }

    pub async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session_arc) = sessions.get(session_id) {
            // Create a new terminated session
            let session = session_arc.as_ref();
            let terminated_session = Arc::new(Session {
                metadata: session.metadata.clone(),
                data: session.data.clone(),
                state: SessionState::Terminated,
            });

            // Replace the session in the map
            sessions.insert(session_id.to_string(), terminated_session);
        }
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<u32, PrimalError> {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        let timeout = chrono::Duration::from_std(self.config.timeout)
            .map_err(|e| PrimalError::Internal(format!("Invalid timeout duration: {e}")))?;

        let mut removed_count = 0;
        sessions.retain(|_, session| {
            let expired = now.signed_duration_since(session.metadata.last_activity) > timeout;
            if expired {
                removed_count += 1;
                false
            } else {
                true
            }
        });

        Ok(removed_count)
    }

    pub async fn get_active_session_count(&self) -> u32 {
        let sessions = self.sessions.read().await;
        sessions.len() as u32
    }
}

/// Session manager trait for dependency injection
/// 
/// NOTE: This trait uses async_trait because it is used as a trait object (Arc<dyn SessionManager>)
/// in primal_provider/core.rs. Native async traits are not compatible with trait objects.
#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn create_session(&self, client_info: Option<String>) -> Result<String, PrimalError>;

    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError>;

    async fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError>;

    async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError>;
}

#[async_trait]
impl SessionManager for SessionManagerImpl {
    async fn create_session(&self, client_info: Option<String>) -> Result<String, PrimalError> {
        self.create_session(client_info).await
    }

    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError> {
        self.get_session_metadata(session_id).await
    }

    async fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        self.update_session_data(session_id, data).await
    }

    async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError> {
        self.terminate_session(session_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let session_id = manager
            .create_session(Some("test_client".to_string()))
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id).await.unwrap();
        assert!(session.is_some());
    }

    #[tokio::test]
    async fn test_session_update() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let session_id = manager.create_session(None).await.unwrap();

        let mut data = HashMap::new();
        data.insert(
            "test_key".to_string(),
            serde_json::Value::String("test_value".to_string()),
        );

        manager.update_session(&session_id, data).await.unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(
            session.data.get("test_key"),
            Some(&serde_json::Value::String("test_value".to_string()))
        );
    }

    #[tokio::test]
    async fn test_session_termination() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let session_id = manager.create_session(None).await.unwrap();
        manager.terminate_session(&session_id).await.unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(matches!(session.state, SessionState::Terminated));
    }
}
