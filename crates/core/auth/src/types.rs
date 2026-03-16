// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Modern authentication types for Squirrel MCP system
//!
//! Supports both standalone operation and capability-based beardog integration
//! through the universal adapter pattern.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Authentication request for login operations
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct LoginRequest {
    /// Username for authentication
    pub username: String,
    /// Password for authentication (will be securely handled)
    pub password: String,
    /// Optional additional authentication factors
    #[zeroize(skip)]
    pub additional_factors: Option<serde_json::Value>,
}

impl LoginRequest {
    /// Create a new login request
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            additional_factors: None,
        }
    }

    /// Add additional authentication factors (MFA, etc.)
    pub fn with_factors(mut self, factors: serde_json::Value) -> Self {
        self.additional_factors = Some(factors);
        self
    }
}

/// Authentication response containing user context and session info
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct LoginResponse {
    /// Authentication success status
    pub success: bool,
    /// User context if authentication succeeded
    #[zeroize(skip)]
    pub user_context: Option<AuthContext>,
    /// Session token for subsequent requests
    pub session_token: Option<String>,
    /// Token expiration time
    #[zeroize(skip)]
    pub expires_at: Option<DateTime<Utc>>,
    /// Error message if authentication failed
    pub error_message: Option<String>,
}

/// User information and authentication context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<Permission>,
    /// User creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last login timestamp
    pub last_login: Option<DateTime<Utc>>,
    /// Account active status
    pub is_active: bool,
}

impl User {
    /// Create a new user
    pub fn new(username: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.into(),
            email: email.into(),
            roles: Vec::new(),
            permissions: Vec::new(),
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(permission))
    }
}

/// Permission structure for fine-grained access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    /// Resource being accessed (e.g., "mcp", "api", "admin")
    pub resource: String,
    /// Action being performed (e.g., "read", "write", "delete")
    pub action: String,
    /// Optional scope for the permission (e.g., specific service or data)
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

    /// Create a scoped permission
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

/// Authentication context containing user session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    /// User ID
    pub user_id: Uuid,
    /// Username
    pub username: String,
    /// Session ID
    pub session_id: Uuid,
    /// User roles
    pub roles: Vec<String>,
    /// User permissions
    pub permissions: Vec<Permission>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
    /// Authentication provider used (standalone, beardog-capability, etc.)
    pub auth_provider: AuthProvider,
}

impl AuthContext {
    /// Create a new auth context
    pub fn new(
        user: &User,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
        auth_provider: AuthProvider,
    ) -> Self {
        Self {
            user_id: user.id,
            username: user.username.clone(),
            session_id,
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
            created_at: Utc::now(),
            expires_at,
            auth_provider,
        }
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(permission))
    }
}

/// Authentication provider type - supports universal capability discovery
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AuthProvider {
    /// Standalone squirrel authentication (failsafe fallback)
    #[default]
    Standalone,
    /// Any primal with security capabilities discovered through universal adapter
    SecurityCapability {
        /// Discovered security endpoint
        endpoint: String,
        /// Capability discovery method used
        discovery_method: String,
        /// Information about the discovered security capability
        capability_info: SecurityCapabilityInfo,
    },
    /// Test/development provider
    Development,
}

/// Information about discovered security capabilities from any primal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityCapabilityInfo {
    /// Type of primal providing the capability (discovered, not hardcoded)
    pub primal_type: String,
    /// Whether this primal supports authentication
    pub supports_auth: bool,
    /// Whether this primal supports session management
    pub supports_sessions: bool,
    /// API version supported
    pub api_version: String,
}

/// Session information for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
    /// Last accessed time
    pub last_accessed: DateTime<Utc>,
    /// Session active status
    pub is_active: bool,
    /// Authentication provider used for this session
    pub auth_provider: AuthProvider,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Uuid, duration: Duration, auth_provider: AuthProvider) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            created_at: now,
            expires_at: now + duration,
            last_accessed: now,
            is_active: true,
            auth_provider,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }
}

/// JWT Claims Structure (used by both local and delegated JWT)
///
/// This struct is always available regardless of feature flags,
/// since it's needed by both BearDog JWT client and local JWT validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject claim: the user ID.
    pub sub: String,
    /// Username of the authenticated user.
    pub username: String,
    /// Roles assigned to the user.
    pub roles: Vec<String>,
    /// Session identifier for this token.
    pub session_id: String,
    /// Issued-at timestamp (Unix seconds).
    pub iat: i64,
    /// Expiration timestamp (Unix seconds).
    pub exp: i64,
    /// Not-before timestamp (Unix seconds).
    pub nbf: i64,
    /// Issuer of the token.
    pub iss: String,
    /// Intended audience for the token.
    pub aud: String,
    /// Unique JWT ID for this token.
    pub jti: String,
}

impl JwtClaims {
    /// Creates new JWT claims from user data and expiration.
    pub fn new(
        user_id: Uuid,
        username: String,
        roles: Vec<String>,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();

        Self {
            sub: user_id.to_string(),
            username,
            roles,
            session_id: session_id.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            nbf: now.timestamp(),
            iss: "squirrel-mcp".to_string(),
            aud: "squirrel-mcp-api".to_string(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Converts JWT claims into an [`AuthContext`] for authorization checks.
    pub fn to_auth_context(&self) -> Result<AuthContext, crate::AuthError> {
        let user_id = Uuid::parse_str(&self.sub).map_err(|_| crate::AuthError::InvalidToken)?;

        let session_id =
            Uuid::parse_str(&self.session_id).map_err(|_| crate::AuthError::InvalidToken)?;

        let created_at =
            DateTime::from_timestamp(self.iat, 0).ok_or(crate::AuthError::InvalidToken)?;

        let expires_at =
            DateTime::from_timestamp(self.exp, 0).ok_or(crate::AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id,
            username: self.username.clone(),
            permissions: vec![], // Permissions are fetched separately
            session_id,
            expires_at,
            created_at,
            roles: self.roles.clone(),
            auth_provider: AuthProvider::Standalone,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_request_new() {
        let req = LoginRequest::new("alice", "secret");
        assert_eq!(req.username, "alice");
        assert_eq!(req.password, "secret");
        assert!(req.additional_factors.is_none());
    }

    #[test]
    fn login_request_with_factors() {
        let factors = serde_json::json!({"totp": "123456"});
        let req = LoginRequest::new("alice", "secret").with_factors(factors.clone());
        assert_eq!(req.additional_factors, Some(factors));
    }

    #[test]
    fn user_new() {
        let user = User::new("bob", "bob@example.com");
        assert_eq!(user.username, "bob");
        assert_eq!(user.email, "bob@example.com");
        assert!(user.roles.is_empty());
        assert!(user.permissions.is_empty());
        assert!(user.is_active);
    }

    #[test]
    fn user_has_role() {
        let mut user = User::new("bob", "bob@example.com");
        user.roles = vec!["admin".into(), "editor".into()];
        assert!(user.has_role("admin"));
        assert!(user.has_role("editor"));
        assert!(!user.has_role("viewer"));
    }

    #[test]
    fn user_has_permission() {
        let mut user = User::new("bob", "bob@example.com");
        user.permissions = vec![
            Permission::new("mcp", "read"),
            Permission::with_scope("api", "write", "projects"),
        ];
        assert!(user.has_permission(&Permission::new("mcp", "read")));
        assert!(user.has_permission(&Permission::with_scope("api", "write", "projects")));
        assert!(!user.has_permission(&Permission::new("admin", "delete")));
    }

    #[test]
    fn permission_new() {
        let p = Permission::new("mcp", "read");
        assert_eq!(p.resource, "mcp");
        assert_eq!(p.action, "read");
        assert!(p.scope.is_none());
    }

    #[test]
    fn permission_with_scope() {
        let p = Permission::with_scope("api", "write", "projects");
        assert_eq!(p.resource, "api");
        assert_eq!(p.action, "write");
        assert_eq!(p.scope.as_deref(), Some("projects"));
    }

    #[test]
    fn permission_matches() {
        let p1 = Permission::new("mcp", "read");
        let p2 = Permission::new("mcp", "read");
        let p3 = Permission::with_scope("mcp", "read", "scope");
        assert!(p1.matches(&p2));
        assert!(!p1.matches(&p3));
        assert!(p3.matches(&Permission::with_scope("mcp", "read", "scope")));
    }

    #[test]
    fn auth_context_new() {
        let user = User::new("alice", "alice@example.com");
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);
        let ctx = AuthContext::new(&user, session_id, expires_at, AuthProvider::Standalone);
        assert_eq!(ctx.user_id, user.id);
        assert_eq!(ctx.username, "alice");
        assert_eq!(ctx.session_id, session_id);
        assert!(!ctx.is_expired());
    }

    #[test]
    fn auth_context_is_expired() {
        let user = User::new("alice", "alice@example.com");
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::hours(1);
        let ctx = AuthContext::new(&user, session_id, expires_at, AuthProvider::Standalone);
        assert!(ctx.is_expired());
    }

    #[test]
    fn auth_context_has_role() {
        let mut user = User::new("alice", "alice@example.com");
        user.roles = vec!["admin".into()];
        let ctx = AuthContext::new(
            &user,
            Uuid::new_v4(),
            Utc::now() + Duration::hours(1),
            AuthProvider::Standalone,
        );
        assert!(ctx.has_role("admin"));
        assert!(!ctx.has_role("viewer"));
    }

    #[test]
    fn auth_context_has_permission() {
        let mut user = User::new("alice", "alice@example.com");
        user.permissions = vec![Permission::new("mcp", "read")];
        let ctx = AuthContext::new(
            &user,
            Uuid::new_v4(),
            Utc::now() + Duration::hours(1),
            AuthProvider::Standalone,
        );
        assert!(ctx.has_permission(&Permission::new("mcp", "read")));
        assert!(!ctx.has_permission(&Permission::new("admin", "delete")));
    }

    #[test]
    fn session_new() {
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        assert_eq!(session.user_id, user_id);
        assert!(session.is_active);
        assert!(!session.is_expired());
    }

    #[test]
    fn session_is_expired() {
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        session.expires_at = Utc::now() - Duration::hours(1);
        assert!(session.is_expired());
    }

    #[test]
    fn session_touch() {
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let before = session.last_accessed;
        std::thread::sleep(std::time::Duration::from_millis(2));
        session.touch();
        assert!(session.last_accessed >= before);
    }

    #[test]
    fn session_invalidate() {
        let user_id = Uuid::new_v4();
        let mut session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        assert!(session.is_active);
        session.invalidate();
        assert!(!session.is_active);
    }

    #[test]
    fn login_request_serde_roundtrip() {
        let req = LoginRequest::new("alice", "secret")
            .with_factors(serde_json::json!({"mfa": true}));
        let json = serde_json::to_string(&req).unwrap();
        let restored: LoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.username, req.username);
        assert_eq!(restored.additional_factors, req.additional_factors);
    }

    #[test]
    fn auth_provider_serde_roundtrip() {
        let providers = [
            AuthProvider::Standalone,
            AuthProvider::Development,
            AuthProvider::SecurityCapability {
                endpoint: "http://localhost:8443".into(),
                discovery_method: "config".into(),
                capability_info: SecurityCapabilityInfo {
                    primal_type: "security".into(),
                    supports_auth: true,
                    supports_sessions: true,
                    api_version: "1.0".into(),
                },
            },
        ];
        for provider in providers {
            let json = serde_json::to_string(&provider).unwrap();
            let restored: AuthProvider = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, provider);
        }
    }

    #[test]
    fn session_serde_roundtrip() {
        let user_id = Uuid::new_v4();
        let session = Session::new(user_id, Duration::hours(1), AuthProvider::Standalone);
        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, session.id);
        assert_eq!(restored.user_id, session.user_id);
    }

    #[test]
    fn login_request_empty_strings() {
        let req = LoginRequest::new("", "");
        assert_eq!(req.username, "");
        assert_eq!(req.password, "");
    }

    #[test]
    fn auth_context_expired_timestamp_edge() {
        let user = User::new("alice", "alice@example.com");
        let expires_at = Utc::now() - Duration::seconds(1);
        let ctx = AuthContext::new(&user, Uuid::new_v4(), expires_at, AuthProvider::Standalone);
        assert!(ctx.is_expired());
    }
}
