//! Security module for the MCP system
//!
//! This module provides security-related functionality, including
//! authentication, authorization, and encryption.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;

pub mod crypto;
pub mod rbac;
pub mod types;

// Re-export types from types.rs
pub use types::{Role, Permission, PermissionCondition, PermissionContext, PermissionScope, Action};

// Import types from crate::types
use crate::types::{SessionToken, UserId};

// Re-export enhanced RBAC components
pub use rbac::{
    EnhancedRBACManager, ValidationResult, ValidationRule, InheritanceType, ValidationAuditRecord
};

use crate::error::Result;
use chrono::DateTime;
use chrono::Utc;

/// Authentication credentials for security operations
#[derive(Debug, Clone)]
pub struct Credentials {
    /// Username for authentication
    pub username: String,
    
    /// Password for authentication (optional)
    pub password: Option<String>,
    
    /// Token for authentication (optional)
    pub token: Option<String>,
}

/// Session information for authenticated users
#[derive(Debug, Clone)]
pub struct Session {
    /// Session token
    pub token: SessionToken,
    
    /// User ID
    pub user_id: UserId,
    
    /// Creation time
    pub created_at: DateTime<Utc>,
    
    /// Last access time
    pub updated_at: DateTime<Utc>,
    
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

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
        // Create a default context
        let _context = PermissionContext {
            user_id: user_id.to_string(),
            current_time: Some(chrono::Utc::now()),
            network_address: None,
            security_level: crate::types::SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        };
        
        // Check permission with context
        match self.rbac_manager.has_permission(
            user_id,
            &permission.resource,
            permission.action,
            &_context,
        ).await {
            Ok(result) => matches!(result, ValidationResult::Granted),
            Err(_) => false,
        }
    }
    
    async fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        // Create a default context
        let _context = PermissionContext {
            user_id: user_id.to_string(),
            current_time: Some(chrono::Utc::now()),
            network_address: None,
            security_level: crate::types::SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        };
        
        // Get user role IDs
        let user_role_ids = self.rbac_manager.get_user_roles(user_id).await;
        let mut roles = Vec::new();
        
        // Get role objects from IDs
        for role_id in &user_role_ids {
            if let Ok(role) = self.rbac_manager.get_role(role_id).await {
                roles.push(role);
            }
        }
        
        // Collect all permissions, including inherited ones
        let mut all_permissions = HashSet::new();
        
        // Create role map for inheritance
        let _role_map: HashMap<String, Role> = roles
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();
            
        // Add direct permissions from roles and gather inherited ones
        for role in &roles {
            // Add direct permissions
            all_permissions.extend(role.permissions.clone());
            
            // Add inherited permissions through has_permission checks
            for permission in &role.permissions {
                if self.has_permission(user_id, permission).await {
                    all_permissions.insert(permission.clone());
                }
            }
        }
        
        all_permissions
    }
    
    async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        self.rbac_manager.assign_role(user_id, role_id).await
    }
    
    async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        self.rbac_manager.create_role(name, description, permissions, parent_roles).await
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<Role>> {
        // Get the role IDs from the RBAC manager
        let role_ids = self.rbac_manager.get_user_roles(user_id).await;
        
        // Retrieve each role from the RBAC manager
        let mut roles = Vec::new();
        for role_id in &role_ids {
            if let Ok(role) = self.rbac_manager.get_role(role_id).await {
                roles.push(role);
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
