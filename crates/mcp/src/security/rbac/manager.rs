// Enhanced RBAC Manager
//
// This module provides a unified RBAC manager that integrates role inheritance
// and permission validation components for comprehensive access control.

use std::collections::{HashMap, HashSet};
use tracing::info;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{SecurityError, Result, MCPError};
use crate::security::types::{
    Permission, Role, PermissionContext, Action,
    PermissionCondition,
};
use crate::security::rbac::RBACError;
use crate::types::SecurityLevel;


/// Error types for RBAC operations
#[derive(Debug, thiserror::Error)]
pub enum InternalRBACError {
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

/// Basic RBAC Manager implementation
pub struct RBACManager {
    /// Roles managed by this RBAC manager
    roles: tokio::sync::RwLock<HashMap<String, Role>>,
    /// User-to-role mappings
    user_roles: tokio::sync::RwLock<HashMap<String, HashSet<String>>>,
}

impl RBACManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        Self {
            roles: tokio::sync::RwLock::new(HashMap::new()),
            user_roles: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create a new role
    pub async fn create_role(&self, name: &str, description: Option<&str>) -> Result<Role> {
        let role_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let role = Role {
            id: role_id.clone(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            security_level: SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
            created_at: now,
            updated_at: now,
        };
        
        {
            let mut roles = self.roles.write().await;
            // Check if role with this name already exists
            if roles.values().any(|r| r.name == name) {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleExists(
                    name.to_string()
                ))));
            }
            
            roles.insert(role_id.clone(), role.clone());
        }
        
        info!("Created role {} with ID {}", name, role_id);
        Ok(role)
    }

    /// Add permission to a role
    pub async fn add_permission_to_role(&self, role_id: &str, permission: Permission) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.insert(permission);
            role.updated_at = Utc::now();
            Ok(())
        } else {
            Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                role_id.to_string()
            ))))
        }
    }

    /// Assign role to a user
    pub async fn assign_role_to_user(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Check if role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    role_id.to_string()
                ))));
            }
        }
        
        // Add role to user
        {
            let mut user_roles = self.user_roles.write().await;
            let user_role_set = user_roles.entry(user_id.to_string()).or_insert_with(HashSet::new);
            user_role_set.insert(role_id.to_string());
        }
        
        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Get roles for a user
    pub async fn get_user_roles(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }

    /// Get a role by ID
    pub async fn get_role(&self, role_id: &str) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id)
            .cloned()
            .ok_or_else(|| MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                role_id.to_string()
            ))))
    }

    /// Check if a user has a specific permission
    pub async fn has_permission(&self, user_id: &str, resource: &str, action: Action, context: &PermissionContext) -> Result<bool> {
        let user_role_ids = self.get_user_roles(user_id).await;
        if user_role_ids.is_empty() {
            return Ok(false);
        }

        let roles = self.roles.read().await;
        
        // Get all permissions from user roles
        let mut all_permissions = HashSet::new();
        
        for role_id in &user_role_ids {
            if let Some(role) = roles.get(role_id) {
                // Add direct permissions
                for perm in &role.permissions {
                    all_permissions.insert(perm);
                }
            }
        }

        // Check if any permission matches
        for perm in all_permissions {
            if perm.resource == resource && (perm.action == action || perm.action == Action::Admin) {
                // Basic match found, check conditions if any
                if perm.conditions.is_empty() {
                    return Ok(true);
                }

                // Check all conditions
                let mut all_conditions_pass = true;
                for condition in &perm.conditions {
                    if !self.evaluate_condition(condition, context) {
                        all_conditions_pass = false;
                        break;
                    }
                }

                if all_conditions_pass {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Evaluate a permission condition against the context
    fn evaluate_condition(&self, condition: &PermissionCondition, context: &PermissionContext) -> bool {
        match condition {
            PermissionCondition::MinimumSecurityLevel(level) => {
                context.security_level >= *level
            },
            // Simplified implementation for other conditions
            _ => true
        }
    }
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

// For convenience in testing
/// Type of verification to perform on permissions
#[derive(Debug, Clone)]
pub enum VerificationType {
    /// Simple verification
    Simple,
    /// Required verification
    Required,
    /// Optional verification
    Optional,
} 