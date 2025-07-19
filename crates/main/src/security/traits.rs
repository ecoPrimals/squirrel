//! Security Traits
//!
//! This module contains the main SecurityAdapter trait and related async interfaces
//! for implementing security providers.

use async_trait::async_trait;

use super::{
    health::SecurityHealthStatus, SecurityContext, SecurityRequest, SecurityResponse,
    SecuritySession,
};
use crate::error::PrimalError;

/// Universal Security Adapter trait
#[async_trait]
pub trait SecurityAdapter: Send + Sync {
    /// Handle security request
    async fn handle_request(
        &self,
        request: SecurityRequest,
    ) -> Result<SecurityResponse, PrimalError>;

    /// Authenticate user
    async fn authenticate(
        &self,
        credentials: serde_json::Value,
    ) -> Result<SecuritySession, PrimalError>;

    /// Authorize operation
    async fn authorize(
        &self,
        session: &SecuritySession,
        operation: &str,
    ) -> Result<bool, PrimalError>;

    /// Validate token
    async fn validate_token(&self, token: &str) -> Result<SecuritySession, PrimalError>;

    /// Create session
    async fn create_session(&self, user_id: &str) -> Result<SecuritySession, PrimalError>;

    /// Destroy session
    async fn destroy_session(&self, session_id: &str) -> Result<(), PrimalError>;

    /// Encrypt data
    async fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, PrimalError>;

    /// Decrypt data
    async fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>, PrimalError>;

    /// Audit event
    async fn audit_event(&self, event: serde_json::Value) -> Result<(), PrimalError>;

    /// Check policy
    async fn check_policy(
        &self,
        policy_id: &str,
        context: &SecurityContext,
    ) -> Result<bool, PrimalError>;

    /// Health check
    async fn health_check(&self) -> Result<SecurityHealthStatus, PrimalError>;
}

/// Encryption provider trait
#[async_trait]
pub trait EncryptionProvider: Send + Sync {
    /// Encrypt data with key
    async fn encrypt_with_key(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>, PrimalError>;

    /// Decrypt data with key
    async fn decrypt_with_key(
        &self,
        encrypted_data: &[u8],
        key: &[u8],
    ) -> Result<Vec<u8>, PrimalError>;

    /// Generate encryption key
    async fn generate_key(&self) -> Result<Vec<u8>, PrimalError>;

    /// Key derivation
    async fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>, PrimalError>;
}

/// Authentication provider trait
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    /// Authenticate with credentials
    async fn authenticate_credentials(
        &self,
        credentials: serde_json::Value,
    ) -> Result<SecuritySession, PrimalError>;

    /// Refresh authentication
    async fn refresh_authentication(
        &self,
        session: &SecuritySession,
    ) -> Result<SecuritySession, PrimalError>;

    /// Validate authentication
    async fn validate_authentication(&self, token: &str) -> Result<bool, PrimalError>;
}

/// Authorization provider trait
#[async_trait]
pub trait AuthorizationProvider: Send + Sync {
    /// Check authorization
    async fn check_authorization(
        &self,
        session: &SecuritySession,
        resource: &str,
        action: &str,
    ) -> Result<bool, PrimalError>;

    /// Get user permissions
    async fn get_permissions(&self, user_id: &str) -> Result<Vec<String>, PrimalError>;

    /// Check permission
    async fn check_permission(&self, user_id: &str, permission: &str) -> Result<bool, PrimalError>;
}

/// Audit provider trait
#[async_trait]
pub trait AuditProvider: Send + Sync {
    /// Log security event
    async fn log_security_event(
        &self,
        event_type: &str,
        details: serde_json::Value,
    ) -> Result<(), PrimalError>;

    /// Log authentication event
    async fn log_authentication_event(
        &self,
        user_id: &str,
        success: bool,
        details: serde_json::Value,
    ) -> Result<(), PrimalError>;

    /// Log authorization event
    async fn log_authorization_event(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        granted: bool,
    ) -> Result<(), PrimalError>;

    /// Get audit log
    async fn get_audit_log(
        &self,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<serde_json::Value>, PrimalError>;
}
