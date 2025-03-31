use async_trait::async_trait;
use crate::error::Result;
use crate::security::types::{UserId, RoleId};
use tracing::{info, warn};
use uuid::Uuid;

/// Trait for audit service operations
#[async_trait]
pub trait AuditService: Send + Sync {
    /// Log a successful authentication
    async fn log_authentication_success(&self, user_id: &UserId);
    
    /// Log a successful authorization
    async fn log_authorization_success(&self, user_id: &UserId, resource: &str, action: &str);
    
    /// Log a failed authorization
    async fn log_authorization_failure(&self, user_id: &UserId, resource: &str, action: &str, reason: &str);
    
    /// Log a role assignment
    async fn log_role_assigned(&self, user_id: &UserId, role_id: &RoleId);
}

/// Default implementation of the AuditService that logs to tracing
pub struct DefaultAuditService;

impl DefaultAuditService {
    /// Create a new instance of the default audit service
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl AuditService for DefaultAuditService {
    async fn log_authentication_success(&self, user_id: &UserId) {
        info!(user_id = %user_id.0, "User authenticated successfully");
    }
    
    async fn log_authorization_success(&self, user_id: &UserId, resource: &str, action: &str) {
        info!(
            user_id = %user_id.0,
            resource = %resource,
            action = %action,
            "Authorization successful"
        );
    }
    
    async fn log_authorization_failure(&self, user_id: &UserId, resource: &str, action: &str, reason: &str) {
        warn!(
            user_id = %user_id.0,
            resource = %resource,
            action = %action,
            reason = %reason,
            "Authorization failed"
        );
    }
    
    async fn log_role_assigned(&self, user_id: &UserId, role_id: &RoleId) {
        info!(
            user_id = %user_id.0,
            role_id = %role_id.0,
            "Role assigned to user"
        );
    }
} 