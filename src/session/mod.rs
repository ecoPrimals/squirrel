//! Session management for MCP connections
//!
//! This module provides session management functionality for MCP connections.

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
        Self {
            timeout: std::time::Duration::from_secs(300), // 5 minutes
            max_connections: 100,
            enable_logging: true,
        }
    }
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub client_info: Option<String>,
    pub capabilities: Vec<String>,
}

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    Inactive,
    Terminated,
}

/// Session data
#[derive(Debug, Clone)]
pub struct Session {
    pub metadata: SessionMetadata,
    pub state: SessionState,
    pub data: HashMap<String, serde_json::Value>,
}

/// Session manager implementation
#[derive(Debug)]
pub struct SessionManagerImpl {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    config: SessionConfig,
}

impl SessionManagerImpl {
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
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, PrimalError> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn update_session(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.metadata.last_activity = Utc::now();
            session.data.extend(data);
        }
        Ok(())
    }

    pub async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.state = SessionState::Terminated;
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
pub trait SessionManager: Send + Sync {
    fn create_session(
        &self,
        client_info: Option<String>,
    ) -> impl std::future::Future<Output = Result<String, PrimalError>> + Send;

    fn get_session_metadata(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<SessionMetadata, PrimalError>> + Send;

    fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;

    fn terminate_session(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<(), PrimalError>> + Send;
}

impl SessionManager for SessionManagerImpl {
    async fn create_session(&self, client_info: Option<String>) -> Result<String, PrimalError> {
        self.create_session(client_info).await
    }

    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError> {
        if let Some(session) = self.get_session(session_id).await? {
            Ok(session.metadata)
        } else {
            Err(PrimalError::Internal(format!(
                "Session not found: {session_id}"
            )))
        }
    }

    async fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        self.update_session(session_id, data).await
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
