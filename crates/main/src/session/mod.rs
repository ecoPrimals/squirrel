// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Session management for MCP connections
//!
//! This module provides session management functionality for MCP connections.

use async_trait::async_trait; // KEEP: SessionManager used as trait object (Arc<dyn SessionManager>)
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::PrimalError;
use crate::protocol::types::SessionId;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session idle timeout before expiration
    pub timeout: std::time::Duration,
    /// Maximum number of concurrent sessions allowed
    pub max_connections: u32,
    /// Whether to enable session activity logging
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
    /// Unique identifier for the session
    pub session_id: SessionId,
    /// UTC timestamp when session was created
    pub created_at: DateTime<Utc>,
    /// UTC timestamp of last session activity
    pub last_activity: DateTime<Utc>,
    /// Optional information about the client
    pub client_info: Option<String>,
    /// List of capabilities supported by the session
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
/// Session instances are designed to be wrapped in `Arc<Session>` for efficient
/// sharing across threads. The internal `HashMap` is not synchronized, so external
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
/// ```ignore
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
    /// Session metadata including ID, timestamps, and capabilities
    pub metadata: SessionMetadata,
    /// Current lifecycle state of the session
    pub state: SessionState,
    /// Key-value storage for session-specific data
    pub data: HashMap<String, serde_json::Value>,
}

/// Session manager implementation  
#[derive(Debug)]
pub struct SessionManagerImpl {
    sessions: Arc<DashMap<SessionId, Arc<Session>>>,
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
    /// A new `SessionManagerImpl` instance ready to manage sessions
    #[must_use]
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Creates a new session with optional client information.
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

        self.sessions.insert(session_id.clone(), Arc::new(session));

        Ok(session_id)
    }

    /// Retrieves a session by ID if it exists.
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Arc<Session>>, PrimalError> {
        Ok(self
            .sessions
            .get(session_id)
            .map(|entry| entry.value().clone()))
    }

    /// Retrieves metadata for a session without loading full session data.
    pub async fn get_session_metadata(
        &self,
        session_id: &str,
    ) -> Result<SessionMetadata, PrimalError> {
        self.sessions
            .get(session_id)
            .map(|entry| entry.value().metadata.clone())
            .ok_or_else(|| PrimalError::NotFoundError(format!("Session not found: {session_id}")))
    }

    /// Updates session data, merging with existing data and refreshing last activity.
    pub async fn update_session(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        if let Some(mut session_entry) = self.sessions.get_mut(session_id) {
            // Create a new updated session
            let session = session_entry.value().as_ref();
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
            *session_entry.value_mut() = updated_session;
        }
        Ok(())
    }

    /// Terminates a session, transitioning it to the Terminated state.
    pub async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError> {
        if let Some(mut session_entry) = self.sessions.get_mut(session_id) {
            // Create a new terminated session
            let session = session_entry.value().as_ref();
            let terminated_session = Arc::new(Session {
                metadata: session.metadata.clone(),
                data: session.data.clone(),
                state: SessionState::Terminated,
            });

            // Replace the session in the map
            *session_entry.value_mut() = terminated_session;
        }
        Ok(())
    }

    /// Removes expired sessions and returns the count of removed sessions.
    pub async fn cleanup_expired_sessions(&self) -> Result<u32, PrimalError> {
        let now = Utc::now();
        let timeout = chrono::Duration::from_std(self.config.timeout)
            .map_err(|e| PrimalError::Internal(format!("Invalid timeout duration: {e}")))?;

        let mut removed_count = 0;
        self.sessions.retain(|_, session| {
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

    /// Returns the number of sessions currently in the manager.
    pub async fn get_active_session_count(&self) -> u32 {
        self.sessions.len() as u32
    }
}

/// Session manager trait for dependency injection
///
/// NOTE: This trait uses `async_trait` because it is used as a trait object (`Arc<dyn SessionManager>`)
/// in `primal_provider/core.rs`. Native async traits are not compatible with trait objects.
#[async_trait]
pub trait SessionManager: Send + Sync {
    /// Creates a new session and returns its ID.
    async fn create_session(&self, client_info: Option<String>) -> Result<String, PrimalError>;

    /// Retrieves metadata for a session by ID.
    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError>;

    /// Updates session data, merging with existing data.
    async fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError>;

    /// Terminates a session by ID.
    async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError>;
}

#[async_trait]
impl SessionManager for SessionManagerImpl {
    async fn create_session(&self, client_info: Option<String>) -> Result<String, PrimalError> {
        Self::create_session(self, client_info).await
    }

    async fn get_session_metadata(&self, session_id: &str) -> Result<SessionMetadata, PrimalError> {
        Self::get_session_metadata(self, session_id).await
    }

    async fn update_session_data(
        &self,
        session_id: &str,
        data: HashMap<String, serde_json::Value>,
    ) -> Result<(), PrimalError> {
        Self::update_session(self, session_id, data).await
    }

    async fn terminate_session(&self, session_id: &str) -> Result<(), PrimalError> {
        Self::terminate_session(self, session_id).await
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

    #[tokio::test]
    async fn test_session_manager_initialization_with_custom_config() {
        let config = SessionConfig {
            timeout: std::time::Duration::from_secs(600),
            max_connections: 50,
            enable_logging: false,
        };
        let manager = SessionManagerImpl::new(config.clone());

        // Verify config is stored correctly
        assert_eq!(manager.config.timeout, config.timeout);
        assert_eq!(manager.config.max_connections, config.max_connections);
        assert_eq!(manager.config.enable_logging, config.enable_logging);

        // Verify initial state
        assert_eq!(manager.get_active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_manager_initialization_with_default_config() {
        let manager = SessionManagerImpl::new(SessionConfig::default());

        // Verify default config values
        assert_eq!(manager.config.timeout, std::time::Duration::from_secs(300));
        assert_eq!(manager.config.max_connections, 100);
        assert!(manager.config.enable_logging);

        // Verify initial state
        assert_eq!(manager.get_active_session_count().await, 0);
    }

    #[tokio::test]
    async fn test_session_lifecycle_create_get_update_delete() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Create session
        let session_id = manager
            .create_session(Some("lifecycle_test_client".to_string()))
            .await
            .unwrap();
        assert!(!session_id.is_empty());
        assert_eq!(manager.get_active_session_count().await, 1);

        // Get session
        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.metadata.session_id, session_id);
        assert!(matches!(session.state, SessionState::Active));
        assert_eq!(
            session.metadata.client_info,
            Some("lifecycle_test_client".to_string())
        );

        // Update session
        let mut update_data = HashMap::new();
        update_data.insert(
            "user_id".to_string(),
            serde_json::Value::String("user_123".to_string()),
        );
        update_data.insert(
            "preferences".to_string(),
            serde_json::json!({"theme": "dark"}),
        );

        manager
            .update_session(&session_id, update_data.clone())
            .await
            .unwrap();

        // Verify update
        let updated_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(
            updated_session.data.get("user_id"),
            Some(&serde_json::Value::String("user_123".to_string()))
        );
        assert!(updated_session.data.contains_key("preferences"));

        // Delete/terminate session
        manager.terminate_session(&session_id).await.unwrap();

        // Verify termination
        let terminated_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(matches!(terminated_session.state, SessionState::Terminated));
        // Session still exists in map but is terminated
        assert_eq!(manager.get_active_session_count().await, 1);
    }

    #[tokio::test]
    async fn test_session_timeout_cleanup() {
        let config = SessionConfig {
            timeout: std::time::Duration::from_millis(5), // Ultra-short timeout for fast tests
            max_connections: 100,
            enable_logging: true,
        };
        let manager = SessionManagerImpl::new(config);

        // Create multiple sessions
        let _session_id1 = manager.create_session(None).await.unwrap();
        let _session_id2 = manager.create_session(None).await.unwrap();
        let session_id3 = manager.create_session(None).await.unwrap();

        assert_eq!(manager.get_active_session_count().await, 3);

        // Update one session to keep it active
        let mut keep_alive_data = HashMap::new();
        keep_alive_data.insert("keep_alive".to_string(), serde_json::json!(true));
        manager
            .update_session(&session_id3, keep_alive_data)
            .await
            .unwrap();

        // Yield to let time pass for the ultra-short timeout (5ms)
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Cleanup expired sessions
        let removed_count = manager.cleanup_expired_sessions().await.unwrap();
        assert!(removed_count >= 2); // At least 2 sessions should be expired

        // Verify remaining session count
        let remaining_count = manager.get_active_session_count().await;
        assert!(remaining_count <= 1);
    }

    #[tokio::test]
    async fn test_concurrent_session_access() {
        let config = SessionConfig::default();
        let manager = Arc::new(SessionManagerImpl::new(config));

        // Create a session
        let session_id = manager.create_session(None).await.unwrap();

        // Spawn multiple concurrent tasks accessing the same session
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let session_id_clone = session_id.clone();
            let handle = tokio::spawn(async move {
                let mut update_data = HashMap::new();
                update_data.insert(
                    format!("concurrent_key_{}", i),
                    serde_json::Value::Number(i.into()),
                );
                manager_clone
                    .update_session(&session_id_clone, update_data)
                    .await
                    .unwrap();

                let session = manager_clone
                    .get_session(&session_id_clone)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(session.metadata.session_id, session_id_clone);
                i
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results: Vec<_> = futures::future::join_all(handles).await;
        for result in results {
            assert!(result.is_ok());
        }

        // Verify final session state
        let final_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(final_session.metadata.session_id, session_id);
        // Verify multiple updates were applied
        assert!(final_session.data.len() >= 10);
    }

    #[tokio::test]
    async fn test_concurrent_session_creation() {
        let config = SessionConfig::default();
        let manager = Arc::new(SessionManagerImpl::new(config));

        // Create multiple sessions concurrently
        let mut handles = vec![];
        for i in 0..20 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let client_info = Some(format!("concurrent_client_{}", i));
                manager_clone.create_session(client_info).await.unwrap()
            });
            handles.push(handle);
        }

        // Wait for all sessions to be created
        let session_ids: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // Verify all sessions are unique
        let unique_ids: std::collections::HashSet<_> = session_ids.iter().collect();
        assert_eq!(unique_ids.len(), 20);

        // Verify session count
        assert_eq!(manager.get_active_session_count().await, 20);
    }

    #[tokio::test]
    async fn test_session_metadata_operations() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Create session with client info
        let client_info = Some("metadata_test_client".to_string());
        let session_id = manager.create_session(client_info.clone()).await.unwrap();

        // Get session and verify metadata
        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.metadata.session_id, session_id);
        assert_eq!(session.metadata.client_info, client_info);
        assert!(session.metadata.capabilities.contains(&"mcp".to_string()));
        assert!(
            session
                .metadata
                .capabilities
                .contains(&"ai_intelligence".to_string())
        );
        assert!(matches!(session.state, SessionState::Active));

        // Verify timestamps
        let created_at = session.metadata.created_at;
        let last_activity = session.metadata.last_activity;
        assert!(created_at <= last_activity);

        // Update session and verify last_activity is updated
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        let mut update_data = HashMap::new();
        update_data.insert("test".to_string(), serde_json::json!("value"));
        manager
            .update_session(&session_id, update_data)
            .await
            .unwrap();

        let updated_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(updated_session.metadata.last_activity > last_activity);
        assert_eq!(updated_session.metadata.created_at, created_at); // Created at shouldn't change
    }

    #[tokio::test]
    async fn test_session_data_operations() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let session_id = manager.create_session(None).await.unwrap();

        // Test adding data
        let mut data1 = HashMap::new();
        data1.insert("key1".to_string(), serde_json::json!("value1"));
        data1.insert("key2".to_string(), serde_json::json!(42));
        manager.update_session(&session_id, data1).await.unwrap();

        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(session.data.len(), 2);
        assert_eq!(session.data.get("key1"), Some(&serde_json::json!("value1")));
        assert_eq!(session.data.get("key2"), Some(&serde_json::json!(42)));

        // Test updating existing data
        let mut data2 = HashMap::new();
        data2.insert("key1".to_string(), serde_json::json!("updated_value1"));
        data2.insert("key3".to_string(), serde_json::json!({"nested": "object"}));
        manager.update_session(&session_id, data2).await.unwrap();

        let updated_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert_eq!(updated_session.data.len(), 3);
        assert_eq!(
            updated_session.data.get("key1"),
            Some(&serde_json::json!("updated_value1"))
        );
        assert_eq!(
            updated_session.data.get("key3"),
            Some(&serde_json::json!({"nested": "object"}))
        );
    }

    #[tokio::test]
    async fn test_session_state_transitions() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Create session - should be Active
        let session_id = manager.create_session(None).await.unwrap();
        let session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(matches!(session.state, SessionState::Active));

        // Terminate session - should transition to Terminated
        manager.terminate_session(&session_id).await.unwrap();
        let terminated_session = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(matches!(terminated_session.state, SessionState::Terminated));

        // Verify terminated session can still be retrieved but state remains Terminated
        let still_terminated = manager.get_session(&session_id).await.unwrap().unwrap();
        assert!(matches!(still_terminated.state, SessionState::Terminated));
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Try to get a session that doesn't exist
        let result = manager.get_session("nonexistent_session_id").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_nonexistent_session() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Try to update a session that doesn't exist
        let mut data = HashMap::new();
        data.insert("test".to_string(), serde_json::json!("value"));
        let result = manager.update_session("nonexistent_session_id", data).await;
        // Should not error, just do nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_terminate_nonexistent_session() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        // Try to terminate a session that doesn't exist
        let result = manager.terminate_session("nonexistent_session_id").await;
        // Should not error, just do nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_session_metadata() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let session_id = manager
            .create_session(Some("metadata_test".to_string()))
            .await
            .unwrap();

        let metadata = manager.get_session_metadata(&session_id).await.unwrap();
        assert_eq!(metadata.session_id, session_id);
        assert_eq!(metadata.client_info, Some("metadata_test".to_string()));
        assert!(metadata.capabilities.contains(&"mcp".to_string()));
    }

    #[tokio::test]
    async fn test_get_session_metadata_nonexistent() {
        let config = SessionConfig::default();
        let manager = SessionManagerImpl::new(config);

        let result = manager.get_session_metadata("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_session_manager_trait_object() {
        use super::SessionManager;
        let manager: std::sync::Arc<dyn SessionManager> =
            std::sync::Arc::new(SessionManagerImpl::new(SessionConfig::default()));

        let session_id = manager.create_session(None).await.unwrap();
        let metadata = manager.get_session_metadata(&session_id).await.unwrap();
        assert_eq!(metadata.session_id, session_id);
    }
}
