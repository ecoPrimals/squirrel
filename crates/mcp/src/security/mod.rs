//! Security module for the MCP system
//!
//! This module provides security-related functionality, including
//! authentication, authorization, and encryption.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

pub mod crypto;
pub mod rbac;
pub mod types;

// Re-export types from types.rs
pub use types::{Role, Permission, PermissionCondition, PermissionContext, PermissionScope, Action};

// Re-export enhanced RBAC components
pub use rbac::{
    EnhancedRBACManager, ValidationResult, ValidationRule, InheritanceType, ValidationAuditRecord
};

use crate::error::{MCPError, Result, SecurityError};
use crate::types::{SecurityLevel, UserId};

/// Security manager interface
#[async_trait]
pub trait SecurityManager: Send + Sync {
    /// Check if a user has a specific permission
    async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool;
    
    /// Get all permissions for a user
    async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission>;
    
    /// Assign a role to a user
    async fn assign_role(&self, user_id: String, role_id: String) -> Result<()>;
    
    /// Create a new role
    async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role>;
    
    /// Get roles for a user
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>>;
}

/// Implementation of the security manager
pub struct SecurityManagerImpl {
    /// RBAC manager
    rbac_manager: Arc<EnhancedRBACManager>,
}

impl SecurityManagerImpl {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            rbac_manager: Arc::new(EnhancedRBACManager::new()),
        }
    }
}

#[async_trait]
impl SecurityManager for SecurityManagerImpl {
    async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        self.rbac_manager.has_permission(
            user_id,
            &permission.resource,
            permission.action,
        ).await
    }
    
    async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        // Get the user's roles
        let user_role_ids = self.rbac_manager.get_user_roles(user_id).await;
        let mut all_permissions = HashSet::new();
        
        // Build a simple permission set for now
        let roles = self.rbac_manager.roles.read().await;
        for role_id in user_role_ids {
            if let Some(role) = roles.get(&role_id) {
                for perm in &role.permissions {
                    all_permissions.insert(perm.clone());
                }
            }
        }
        
        all_permissions
    }
    
    async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        match self.rbac_manager.assign_role_to_user(&user_id, &role_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(MCPError::Security(SecurityError::RBACError(e))),
        }
    }
    
    async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        let role = match self.rbac_manager.create_role(&name, description.as_deref()).await {
            Ok(role) => role,
            Err(e) => return Err(MCPError::Security(SecurityError::RBACError(e))),
        };
        
        // Add permissions
        for permission in permissions {
            if let Err(e) = self.rbac_manager.add_permission_to_role(&role.id, permission).await {
                return Err(MCPError::Security(SecurityError::RBACError(e)));
            }
        }
        
        // Add parent roles (inheritance)
        for parent_id in parent_roles {
            // We would set up inheritance here in a full implementation
            // This is simplified for now
        }
        
        Ok(role)
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>> {
        let role_ids = self.rbac_manager.get_user_roles(user_id).await;
        let mut roles = Vec::new();
        
        let roles_map = self.rbac_manager.roles.read().await;
        for role_id in role_ids {
            if let Some(role) = roles_map.get(&role_id) {
                roles.push(role.clone());
            }
        }
        
        Ok(roles)
    }
}

impl Default for SecurityManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    
    #[tokio::test]
    async fn test_security_manager_basic() {
        let manager = SecurityManagerImpl::new();
        
        // Create a test role
        let mut permissions = HashSet::new();
        permissions.insert(Permission {
            id: "test-permission".to_string(),
            name: "Test Permission".to_string(),
            resource: "test-resource".to_string(),
            action: Action::Read,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        });
        
        let role = manager.create_role(
            "TestRole".to_string(),
            Some("Test role".to_string()),
            permissions.clone(),
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        manager.assign_role("test-user".to_string(), role.id.clone()).await.unwrap();
        
        // Check permission
        let permission = Permission {
            id: "test-permission".to_string(),
            name: "Test Permission".to_string(),
            resource: "test-resource".to_string(),
            action: Action::Read,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        };
        
        assert!(manager.has_permission("test-user", &permission).await);
        
        // Get user roles
        let user_roles = manager.get_user_roles("test-user").await.unwrap();
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0].name, "TestRole");
    }
}
