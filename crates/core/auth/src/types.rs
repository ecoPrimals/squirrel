//! Core authentication types and error definitions.
//!
//! This module defines the fundamental types used throughout the authentication
//! system, including user models, permissions, session data, and error types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Authentication error types
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("User not found")]
    UserNotFound,
    #[error("Session expired")]
    SessionExpired,
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("BearDog error: {0}")]
    BeardogError(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Authentication context containing user information and session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub username: String,
    pub permissions: Vec<Permission>,
    pub session_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub roles: Vec<String>,
}

impl AuthContext {
    /// Create a new authentication context
    pub fn new(
        user_id: Uuid,
        username: String,
        permissions: Vec<Permission>,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
        issued_at: DateTime<Utc>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            user_id,
            username,
            permissions,
            session_id,
            expires_at,
            issued_at,
            roles,
        }
    }

    /// Check if the context is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if the user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
}

/// Permission definition for resource-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: None,
        }
    }

    /// Create a new permission with scope
    pub fn with_scope(
        resource: impl Into<String>,
        action: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: Some(scope.into()),
        }
    }

    /// Check if this permission matches another permission
    pub fn matches(&self, other: &Permission) -> bool {
        self.resource == other.resource && self.action == other.action && self.scope == other.scope
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(scope) = &self.scope {
            write!(f, "{}:{}:{}", self.resource, self.action, scope)
        } else {
            write!(f, "{}:{}", self.resource, self.action)
        }
    }
}

/// User model with complete profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub active: bool,
}

impl User {
    /// Create a new user with minimal information
    pub fn new(username: impl Into<String>, email: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username: username.into(),
            email: email.into(),
            roles: vec![],
            permissions: vec![],
            created_at: now,
            updated_at: now,
            last_login: None,
            login_attempts: 0,
            locked_until: None,
            active: true,
        }
    }

    /// Check if the user is locked out
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Check if the user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if the user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Add a role to the user
    pub fn add_role(&mut self, role: impl Into<String>) {
        let role = role.into();
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }

    /// Add a permission to the user
    pub fn add_permission(&mut self, permission: Permission) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
    }

    /// Remove a permission from the user
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.retain(|p| p != permission);
    }

    /// Mark the user as having logged in
    pub fn mark_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.login_attempts = 0;
    }

    /// Increment login attempts
    pub fn increment_login_attempts(&mut self) {
        self.login_attempts += 1;
    }

    /// Lock the user for a specific duration
    pub fn lock_until(&mut self, until: DateTime<Utc>) {
        self.locked_until = Some(until);
    }

    /// Unlock the user
    pub fn unlock(&mut self) {
        self.locked_until = None;
        self.login_attempts = 0;
    }
}

/// Login request with credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
}

impl LoginRequest {
    /// Create a new login request
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            remember_me: false,
        }
    }

    /// Create a new login request with remember me option
    pub fn with_remember_me(
        username: impl Into<String>,
        password: impl Into<String>,
        remember_me: bool,
    ) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            remember_me,
        }
    }
}

/// Login response with tokens and user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub user: User,
}

impl LoginResponse {
    /// Create a new login response
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: u64,
        user: User,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            token_type: "Bearer".to_string(),
            user,
        }
    }
}

/// Session information for active user sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub active: bool,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Uuid, username: impl Into<String>, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            username: username.into(),
            expires_at,
            created_at: now,
            last_accessed: now,
            active: true,
        }
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        self.active && !self.is_expired()
    }

    /// Mark the session as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.active = false;
    }
}

/// Token response for refresh operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub user_id: Uuid,
}

impl TokenResponse {
    /// Create a new token response
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: u64,
        user_id: Uuid,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            user_id,
        }
    }
}

/// Audit event for compliance monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: String,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub success: bool,
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(
        event_type: impl Into<String>,
        user_id: Option<Uuid>,
        username: Option<String>,
        success: bool,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            user_id,
            username,
            success,
            ip_address: None,
            timestamp: Utc::now(),
            details: serde_json::json!({}),
        }
    }

    /// Set the IP address for the event
    pub fn with_ip_address(mut self, ip_address: impl Into<String>) -> Self {
        self.ip_address = Some(ip_address.into());
        self
    }

    /// Set additional details for the event
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

/// Compliance check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub user_id: Uuid,
    pub action: String,
    pub resource: String,
    pub timestamp: DateTime<Utc>,
}

impl ComplianceCheck {
    /// Create a new compliance check
    pub fn new(user_id: Uuid, action: impl Into<String>, resource: impl Into<String>) -> Self {
        Self {
            user_id,
            action: action.into(),
            resource: resource.into(),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_auth_context_creation() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::hours(1);

        let context = AuthContext::new(
            user_id,
            "test_user".to_string(),
            vec![],
            session_id,
            expires_at,
            now,
            vec!["user".to_string()],
        );

        assert_eq!(context.user_id, user_id);
        assert_eq!(context.username, "test_user");
        assert_eq!(context.session_id, session_id);
        assert!(!context.is_expired());
        assert!(context.has_role("user"));
        assert!(!context.has_role("admin"));
    }

    #[test]
    fn test_permission_equality() {
        let permission1 = Permission::new("mcp", "read");
        let permission2 = Permission::new("mcp", "read");
        let permission3 = Permission::with_scope("mcp", "read", "scope1");

        assert_eq!(permission1, permission2);
        assert_ne!(permission1, permission3);
        assert!(permission1.matches(&permission2));
        assert!(!permission1.matches(&permission3));
    }

    #[test]
    fn test_user_management() {
        let mut user = User::new("test_user", "test@example.com");

        assert!(!user.is_locked());
        assert!(user.has_role("user") == false);

        user.add_role("user");
        user.add_role("admin");
        assert!(user.has_role("user"));
        assert!(user.has_role("admin"));

        let permission = Permission::new("mcp", "read");
        user.add_permission(permission.clone());
        assert!(user.has_permission(&permission));

        user.remove_role("admin");
        assert!(!user.has_role("admin"));
        assert!(user.has_role("user"));

        user.mark_login();
        assert!(user.last_login.is_some());
        assert_eq!(user.login_attempts, 0);
    }

    #[test]
    fn test_session_management() {
        let user_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);
        let mut session = Session::new(user_id, "test_user", expires_at);

        assert!(!session.is_expired());
        assert!(session.is_active());

        session.mark_accessed();
        assert!(session.last_accessed > session.created_at);

        session.invalidate();
        assert!(!session.is_active());
    }

    #[test]
    fn test_login_request() {
        let login_request = LoginRequest::new("username", "password");
        assert_eq!(login_request.username, "username");
        assert_eq!(login_request.password, "password");
        assert!(!login_request.remember_me);

        let login_request_with_remember =
            LoginRequest::with_remember_me("username", "password", true);
        assert!(login_request_with_remember.remember_me);
    }

    #[test]
    fn test_audit_event() {
        let user_id = Uuid::new_v4();
        let event = AuditEvent::new("login", Some(user_id), Some("test_user".to_string()), true)
            .with_ip_address("192.168.1.1")
            .with_details(serde_json::json!({
                "source": "test",
                "action": "login"
            }));

        assert_eq!(event.event_type, "login");
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.username, Some("test_user".to_string()));
        assert!(event.success);
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert!(!event.details.is_null());
    }

    #[test]
    fn test_permission_display() {
        let permission1 = Permission::new("mcp", "read");
        let permission2 = Permission::with_scope("mcp", "read", "scope1");

        assert_eq!(permission1.to_string(), "mcp:read");
        assert_eq!(permission2.to_string(), "mcp:read:scope1");
    }
}
