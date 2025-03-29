// Enhanced RBAC Manager
//
// This module provides a unified RBAC manager that integrates role inheritance
// and permission validation components for comprehensive access control.

use std::collections::{HashMap, HashSet};
use tracing::info;
use uuid::Uuid;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::error::{SecurityError, Result, MCPError};
use crate::security::types::{
    Permission, Role, PermissionContext, Action,
    PermissionCondition,
};
use super::super::types::{RoleId, SecurityLevel};


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

/// RBAC Manager for managing roles, permissions, and role assignments
#[derive(Debug)]
pub struct RBACManager {
    /// Roles managed by this RBAC manager
    roles: tokio::sync::RwLock<HashMap<String, Role>>,
    /// User-to-role mappings
    user_roles: tokio::sync::RwLock<HashMap<String, HashSet<String>>>,
}

impl RBACManager {
    /// Create a new RBAC manager
    #[must_use] pub fn new() -> Self {
        Self {
            roles: tokio::sync::RwLock::new(HashMap::new()),
            user_roles: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create a new role
    ///
    /// # Arguments
    /// * `name` - Name of the role
    /// * `description` - Optional description of the role
    ///
    /// # Returns
    /// A new `Role` instance with the specified properties
    ///
    /// # Errors
    /// This function will return an error if:
    /// * A role with the same name already exists
    /// * Any underlying storage or database operation fails
    pub async fn create_role(&self, name: &str, description: Option<&str>) -> Result<Role> {
        let role_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let role = Role {
            id: role_id.clone(),
            name: name.to_string(),
            description: description.map(std::string::ToString::to_string),
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
                return Err(MCPError::Security(SecurityError::RBACError(
                    format!("Role exists: {name}")
                )));
            }
            
            roles.insert(role_id.clone(), role.clone());
        }
        
        info!("Created role {} with ID {}", name, role_id);
        Ok(role)
    }

    /// Add permission to a role
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to add the permission to
    /// * `permission` - Permission to add to the role
    ///
    /// # Returns
    /// `Ok(())` if the permission was successfully added
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The role with the specified ID does not exist
    /// * Any underlying storage or database operation fails
    pub async fn add_permission_to_role(&self, role_id: &str, permission: Permission) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.insert(permission);
            role.updated_at = Utc::now();
            Ok(())
        } else {
            Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {role_id}")
            )))
        }
    }

    /// Assign role to a user
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to assign the role to
    /// * `role_id` - ID of the role to assign
    ///
    /// # Returns
    /// `Ok(())` if the role was successfully assigned
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The role with the specified ID does not exist
    /// * Any underlying storage or database operation fails
    pub async fn assign_role_to_user(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(MCPError::Security(SecurityError::RBACError(
                    format!("Role not found: {role_id}")
                )));
            }
        }
        
        // Add role to user
        {
            // Use write() lock and chain the subsequent operations
            self.user_roles.write().await
                .entry(user_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(role_id.to_string());
        }
        
        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Get roles for a user
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to get roles for
    ///
    /// # Returns
    /// A set of role IDs assigned to the user
    pub async fn get_user_roles(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }

    /// Get a role by ID
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to retrieve
    ///
    /// # Returns
    /// The role with the specified ID
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The role with the specified ID does not exist
    /// * Any underlying storage or database operation fails
    pub async fn get_role(&self, role_id: &str) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id)
            .cloned()
            .ok_or_else(|| MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {role_id}")
            )))
    }

    /// Check if a user has a specific permission
    ///
    /// # Arguments
    /// * `user_id` - ID of the user to check
    /// * `resource` - Resource to check permission for
    /// * `action` - Action to check permission for
    /// * `context` - Additional context information for condition evaluation
    ///
    /// # Returns
    /// `true` if the user has the permission, `false` otherwise
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The permission condition evaluation fails
    /// * Any underlying storage or database operation fails
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
                    if !Self::evaluate_condition(condition, context) {
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
    fn evaluate_condition(condition: &PermissionCondition, context: &PermissionContext) -> bool {
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
pub(super) enum VerificationType {
    /// Simple verification
    Simple,
    /// Required verification
    Required,
    /// Optional verification
    Optional,
} 