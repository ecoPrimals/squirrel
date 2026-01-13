//! Security capabilities (authentication, authorization)

use crate::error::PrimalError;
// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};

/// Authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// Username or identifier
    pub identifier: String,

    /// Credential (password, token, etc.)
    pub credential: String,

    /// Authentication method (password, token, oauth, etc.)
    pub method: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    /// Whether authentication succeeded
    pub authenticated: bool,

    /// Access token (if applicable)
    pub token: Option<String>,

    /// User ID
    pub user_id: Option<String>,

    /// Expiration timestamp (Unix timestamp)
    pub expires_at: Option<u64>,
}

/// Authorization request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzRequest {
    /// User or entity requesting authorization
    pub subject: String,

    /// Resource being accessed
    pub resource: String,

    /// Action being performed
    pub action: String,

    /// Additional context
    pub context: std::collections::HashMap<String, serde_json::Value>,
}

/// Authorization response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzResponse {
    /// Whether the action is allowed
    pub allowed: bool,

    /// Reason for decision
    pub reason: Option<String>,
}

/// Capability for authentication

pub trait AuthenticationCapability: Send + Sync {
    /// Authenticate a user
    async fn authenticate(&self, request: AuthRequest) -> Result<AuthResponse, PrimalError>;

    /// Validate a token
    async fn validate_token(&self, token: String) -> Result<bool, PrimalError>;
}

/// Capability for authorization

pub trait AuthorizationCapability: Send + Sync {
    /// Check if an action is authorized
    async fn authorize(&self, request: AuthzRequest) -> Result<AuthzResponse, PrimalError>;
}
