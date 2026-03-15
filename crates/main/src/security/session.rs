// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security Session Management
//!
//! This module contains all session-related types and functionality
//! for managing security sessions and user authentication state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::AuthorizationLevel;

/// Active security session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySession {
    /// Session identifier
    pub session_id: String,
    /// User identifier
    pub user_id: Option<String>,
    /// User identifier (string form for compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id_string: Option<String>,
    /// User roles
    pub roles: Vec<String>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Session type
    pub session_type: String,
    /// Authentication status
    pub authenticated: bool,
    /// Authorization level
    pub authorization_level: AuthorizationLevel,
    /// Session data
    pub session_data: HashMap<String, serde_json::Value>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: DateTime<Utc>,
    /// Last accessed
    pub last_accessed: DateTime<Utc>,
}

impl SecuritySession {
    /// Create a new security session
    #[must_use]
    pub fn new(session_id: String, user_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id_string: user_id.clone(),
            user_id,
            roles: Vec::new(),
            metadata: HashMap::new(),
            session_type: "standard".to_string(),
            authenticated: false,
            authorization_level: AuthorizationLevel::None,
            session_data: HashMap::new(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(24), // Default 24 hour expiry
            last_accessed: now,
        }
    }

    /// Create an authenticated session
    #[must_use]
    pub fn authenticated(session_id: String, user_id: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            user_id_string: Some(user_id.clone()),
            user_id: Some(user_id),
            roles: vec!["user".to_string()],
            metadata: HashMap::new(),
            session_type: "authenticated".to_string(),
            authenticated: true,
            authorization_level: AuthorizationLevel::User,
            session_data: HashMap::new(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(24),
            last_accessed: now,
        }
    }

    /// Check if the session is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update last accessed time
    pub fn update_last_accessed(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Set authorization level
    #[must_use]
    pub fn with_authorization_level(mut self, level: AuthorizationLevel) -> Self {
        self.authorization_level = level;
        self
    }

    /// Set session data
    #[must_use]
    pub fn with_session_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.session_data.insert(key, value);
        self
    }

    /// Get session data
    #[must_use]
    pub fn get_session_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.session_data.get(key)
    }

    /// Set session type
    #[must_use]
    pub fn with_session_type(mut self, session_type: String) -> Self {
        self.session_type = session_type;
        self
    }

    /// Extend session expiry
    pub fn extend_expiry(&mut self, duration: chrono::Duration) {
        self.expires_at += duration;
    }

    /// Check if session has required authorization level
    #[must_use]
    pub fn has_authorization_level(&self, required_level: &AuthorizationLevel) -> bool {
        use AuthorizationLevel::{Admin, Elevated, None, System, User};

        matches!(
            (&self.authorization_level, required_level),
            (System, _)
                | (Admin, Admin | Elevated | User | None)
                | (Elevated, Elevated | User | None)
                | (User, User | None)
                | (None, None)
        )
    }

    /// Get session duration
    #[must_use]
    pub fn get_duration(&self) -> chrono::Duration {
        self.last_accessed - self.created_at
    }

    /// Get time until expiry
    #[must_use]
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }
}

impl Default for SecuritySession {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_session_creation() {
        let session = SecuritySession::new("test-session".to_string(), Some("user123".to_string()));

        assert_eq!(session.session_id, "test-session");
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert!(!session.authenticated);
        assert_eq!(session.authorization_level, AuthorizationLevel::None);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_authenticated_session() {
        let session =
            SecuritySession::authenticated("test-session".to_string(), "user123".to_string());

        assert_eq!(session.session_id, "test-session");
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert!(session.authenticated);
        assert_eq!(session.authorization_level, AuthorizationLevel::User);
    }

    #[test]
    fn test_authorization_levels() {
        let session = SecuritySession::authenticated("test".to_string(), "user".to_string())
            .with_authorization_level(AuthorizationLevel::Admin);

        assert!(session.has_authorization_level(&AuthorizationLevel::User));
        assert!(session.has_authorization_level(&AuthorizationLevel::Admin));
        assert!(!session.has_authorization_level(&AuthorizationLevel::System));
    }

    #[test]
    fn test_session_expiry() {
        let mut session = SecuritySession::new("test".to_string(), None);

        // Set expiry to past
        session.expires_at = Utc::now() - Duration::hours(1);
        assert!(session.is_expired());

        // Extend expiry
        session.extend_expiry(Duration::hours(2));
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_data() {
        let session = SecuritySession::new("test".to_string(), None)
            .with_session_data("key1".to_string(), serde_json::json!("value1"))
            .with_session_data("key2".to_string(), serde_json::json!(42));

        assert_eq!(
            session.get_session_data("key1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(
            session.get_session_data("key2"),
            Some(&serde_json::json!(42))
        );
        assert_eq!(session.get_session_data("nonexistent"), None);
    }
}
