// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Session management for Squirrel authentication system
//!
//! Provides in-memory session storage with cleanup and validation.
//! Supports both standalone and beardog-integrated sessions.

use crate::errors::AuthResult;
use crate::types::AuthProvider;

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

// Re-export Session from types for convenience
pub use crate::types::Session;

/// Session manager for handling user sessions
#[derive(Debug)]
pub struct SessionManager {
    /// In-memory session storage
    sessions: RwLock<HashMap<Uuid, Session>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new session
    pub async fn create_session(&self, session: Session) -> AuthResult<()> {
        debug!(
            "Creating session {} for user {}",
            session.id, session.user_id
        );

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id, session);

        // Clean up expired sessions while we have the write lock (best effort)
        if let Err(e) = self.cleanup_expired_sessions_internal(&mut sessions).await {
            tracing::warn!("Failed to cleanup expired sessions: {}", e);
        }

        Ok(())
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &Uuid) -> AuthResult<Option<Session>> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            if session.is_expired() || !session.is_active {
                debug!("Session {} is expired or inactive", session_id);
                return Ok(None);
            }
            Ok(Some(session.clone()))
        } else {
            Ok(None)
        }
    }

    /// Update session last accessed time
    pub async fn touch_session(&self, session_id: &Uuid) -> AuthResult<bool> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_expired() && session.is_active {
                session.touch();
                debug!("Updated last accessed time for session {}", session_id);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Invalidate a session
    pub async fn invalidate_session(&self, session_id: &Uuid) -> AuthResult<bool> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.invalidate();
            debug!("Invalidated session {}", session_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: &Uuid) -> AuthResult<Vec<Session>> {
        let sessions = self.sessions.read().await;
        let user_sessions: Vec<Session> = sessions
            .values()
            .filter(|session| {
                session.user_id == *user_id && session.is_active && !session.is_expired()
            })
            .cloned()
            .collect();

        Ok(user_sessions)
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&self, user_id: &Uuid) -> AuthResult<usize> {
        let mut sessions = self.sessions.write().await;
        let mut invalidated_count = 0;

        for session in sessions.values_mut() {
            if session.user_id == *user_id && session.is_active {
                session.invalidate();
                invalidated_count += 1;
            }
        }

        debug!(
            "Invalidated {} sessions for user {}",
            invalidated_count, user_id
        );
        Ok(invalidated_count)
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> AuthResult<usize> {
        let mut sessions = self.sessions.write().await;
        self.cleanup_expired_sessions_internal(&mut sessions).await
    }

    /// Internal cleanup method (requires write lock)
    async fn cleanup_expired_sessions_internal(
        &self,
        sessions: &mut HashMap<Uuid, Session>,
    ) -> AuthResult<usize> {
        let initial_count = sessions.len();

        sessions.retain(|_id, session| {
            let should_keep = session.is_active && !session.is_expired();
            if !should_keep {
                debug!("Removing expired/inactive session {}", session.id);
            }
            should_keep
        });

        let removed_count = initial_count - sessions.len();
        if removed_count > 0 {
            debug!("Cleaned up {} expired sessions", removed_count);
        }

        Ok(removed_count)
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> AuthResult<SessionStats> {
        let sessions = self.sessions.read().await;

        let total_sessions = sessions.len();
        let mut active_sessions = 0;
        let mut expired_sessions = 0;
        let mut security_capability_sessions = 0;
        let mut standalone_sessions = 0;

        for session in sessions.values() {
            if session.is_active && !session.is_expired() {
                active_sessions += 1;
            } else {
                expired_sessions += 1;
            }

            match &session.auth_provider {
                AuthProvider::SecurityCapability { .. } => security_capability_sessions += 1,
                AuthProvider::Standalone => standalone_sessions += 1,
                AuthProvider::Development => {} // Don't count dev sessions
            }
        }

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
            security_capability_sessions,
            standalone_sessions,
        })
    }

    /// Extend session expiration time
    pub async fn extend_session(
        &self,
        session_id: &Uuid,
        additional_duration: Duration,
    ) -> AuthResult<bool> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_active && !session.is_expired() {
                session.expires_at = session.expires_at + additional_duration;
                debug!(
                    "Extended session {} by {} minutes",
                    session_id,
                    additional_duration.num_minutes()
                );
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session statistics for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of sessions in memory
    pub total_sessions: usize,
    /// Number of active, non-expired sessions
    pub active_sessions: usize,
    /// Number of expired sessions
    pub expired_sessions: usize,
    /// Number of sessions using discovered security capabilities
    pub security_capability_sessions: usize,
    /// Number of sessions using standalone auth
    pub standalone_sessions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AuthProvider;

    #[tokio::test]
    async fn test_session_creation_and_retrieval() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();

        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let session_id = session.id;

        // Create session
        manager.create_session(session).await.unwrap();

        // Retrieve session
        let retrieved = manager.get_session(&session_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, user_id);
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();

        // Create expired session
        let mut session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        session.expires_at = Utc::now() - Duration::hours(1); // Already expired
        let session_id = session.id;

        manager.create_session(session).await.unwrap();

        // Should return None for expired session
        let retrieved = manager.get_session(&session_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_session_invalidation() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();

        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let session_id = session.id;

        manager.create_session(session).await.unwrap();

        // Invalidate session
        let result = manager.invalidate_session(&session_id).await.unwrap();
        assert!(result);

        // Should return None for invalidated session
        let retrieved = manager.get_session(&session_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let manager = SessionManager::new();
        let user_id = Uuid::new_v4();

        // Create expired session
        let mut expired_session =
            Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        expired_session.expires_at = Utc::now() - Duration::hours(1);

        // Create valid session
        let valid_session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);

        manager.create_session(expired_session).await.unwrap();
        manager.create_session(valid_session).await.unwrap();

        // Clean up expired sessions
        let removed = manager.cleanup_expired_sessions().await.unwrap();
        assert_eq!(removed, 1);

        // Verify stats
        let stats = manager.get_session_stats().await.unwrap();
        assert_eq!(stats.active_sessions, 1);
        assert_eq!(stats.total_sessions, 1);
    }
}
