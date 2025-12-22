//! Modern authentication types for Squirrel MCP system
//!
//! Supports both standalone operation and capability-based beardog integration
//! through the universal adapter pattern.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authentication request for login operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username for authentication
    pub username: String,
    /// Password for authentication (will be securely handled)
    pub password: String,
    /// Optional additional authentication factors
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Authentication success status
    pub success: bool,
    /// User context if authentication succeeded
    pub user_context: Option<AuthContext>,
    /// Session token for subsequent requests
    pub session_token: Option<String>,
    /// Token expiration time
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthProvider {
    /// Standalone squirrel authentication (failsafe fallback)
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

impl Default for AuthProvider {
    fn default() -> Self {
        Self::Standalone
    }
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
