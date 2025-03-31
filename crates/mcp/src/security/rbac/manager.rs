// Enhanced RBAC Manager
//
// This module provides a unified RBAC manager that integrates role inheritance
// and permission validation components for comprehensive access control.

use std::collections::{HashMap, HashSet};
use tracing::info;
use chrono::Utc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::error::{SecurityError, Result};
use crate::context_manager::Context;
use super::unified::RBACManager;
use super::unified::RoleDefinition;
use super::unified::RoleDetailsResponse;

/// Error types for RBAC operations
#[derive(Debug, thiserror::Error)]
pub(super) enum InternalRBACError {
    /// Role already exists
    #[error("Role already exists: {0}")]
    RoleExists(String),
    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// Permission error
    #[error("Permission error: {0}")]
    PermissionError(String),
    /// Internal error
    #[error("Internal RBAC error: {0}")]
    InternalError(String),
}

/// Inner role structure for RBACManagerImpl
#[derive(Debug, Clone)]
struct RoleInternal {
    /// Role definition
    definition: RoleDefinition,
    /// Permissions assigned to this role
    permissions: HashSet<String>,
    /// Parent roles from which this role inherits permissions
    parent_roles: HashSet<String>,
}

/// RBAC Manager for managing roles, permissions, and role assignments
#[derive(Debug)]
pub struct RBACManagerImpl {
    /// Roles managed by this RBAC manager
    roles: RwLock<HashMap<String, RoleInternal>>,
    /// User-to-role mappings
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl RBACManagerImpl {
    /// Create a new RBAC manager
    #[must_use] pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }

    /// Get user roles as a HashSet
    async fn get_user_roles_set(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }
}

#[async_trait]
impl RBACManager for RBACManagerImpl {
    fn name(&self) -> &str {
        "RBACManagerImpl"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }

    async fn has_permission(&self, user_id: &str, permission: &str, _context: Option<&Context>) -> Result<bool> {
        let user_roles = self.get_user_roles_set(user_id).await;
        
        // If the user has no roles, they have no permissions
        if user_roles.is_empty() {
            return Ok(false);
        }
        
        // Check each role for the permission
        let roles = self.roles.read().await;
        for role_id in user_roles {
            if let Some(role) = roles.get(&role_id) {
                if role.permissions.contains(permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(SecurityError::RoleNotFound(role_id.to_string()).into());
            }
        }
        
        // Add role to user
        {
            self.user_roles.write().await
                .entry(user_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(role_id.to_string());
        }
        
        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }
    
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut removed = false;
        {
            let mut user_roles = self.user_roles.write().await;
            if let Some(roles) = user_roles.get_mut(user_id) {
                removed = roles.remove(role_id);
            }
        }
        
        if removed {
            info!("Revoked role {} from user {}", role_id, user_id);
        } else {
            info!("User {} did not have role {}, nothing to revoke", user_id, role_id);
        }
        
        Ok(())
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let roles = self.get_user_roles_set(user_id).await;
        Ok(roles.into_iter().collect())
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.get_user_roles_set(user_id).await;
        Ok(user_roles.contains(role_id))
    }

    async fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        let roles = self.roles.read().await;
        
        if let Some(role) = roles.get(role_id) {
            Ok(Some(RoleDetailsResponse {
                role: role.definition.clone(),
                permissions: Vec::new(), // Simplified for now
                parent_roles: role.parent_roles.iter().cloned().collect(),
                child_roles: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()> {
        let now = Utc::now();
        
        let role = RoleInternal {
            definition: RoleDefinition {
                id: role_id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                is_system_role: false,
                created_at: now,
                updated_at: now,
            },
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        };
        
        let mut roles = self.roles.write().await;
        
        // Check if role with this ID already exists
        if roles.contains_key(role_id) {
            return Err(SecurityError::RoleExists(role_id.to_string()).into());
        }
        
        roles.insert(role_id.to_string(), role);
        
        info!("Created role {} with ID {}", name, role_id);
        Ok(())
    }

    async fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.insert(permission.to_string());
            role.definition.updated_at = Utc::now();
            Ok(())
        } else {
            Err(SecurityError::RoleNotFound(role_id.to_string()).into())
        }
    }
}

impl Default for RBACManagerImpl {
    fn default() -> Self {
        Self::new()
    }
} 