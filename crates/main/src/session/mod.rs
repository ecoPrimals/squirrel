// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Session management for MCP connections
//!
//! This module provides session management functionality for MCP connections.

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
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
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
#[path = "session_tests.rs"]
mod tests;
