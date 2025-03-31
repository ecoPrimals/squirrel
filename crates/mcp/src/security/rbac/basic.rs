//! Basic RBAC Manager implementation.
//!
//! This module contains a simple implementation of the unified `RBACManager` trait
//! that provides core role-based access control functionality without advanced
//! features like caching, inheritance, or complex validation rules.

use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::error::{Result, SecurityError};
use crate::context_manager::Context;

// Import unified RBACManager directly
use super::unified::RBACManager;
// Import RoleDefinition directly
use super::unified::RoleDefinition;
// Import RolePermission directly if needed
use super::unified::RolePermission;
// Import RoleDetailsResponse directly if needed
use super::unified::RoleDetailsResponse;

/// Role structure containing role metadata and assigned permissions
#[derive(Debug, Clone)]
struct Role {
    /// Role definition containing basic information
    definition: RoleDefinition,
    /// Permissions assigned to this role
    permissions: HashSet<String>,
    /// Parent roles from which this role inherits permissions
    parent_roles: HashSet<String>,
}

/// Basic RBAC Manager implementation with core functionality.
///
/// This implementation provides the core RBAC operations without advanced
/// features like caching, inheritance, or complex validation rules.
/// It is designed to be simple, reliable, and easy to understand.
#[derive(Debug)]
pub struct BasicRBACManager {
    /// Roles managed by this RBAC manager
    roles: RwLock<HashMap<String, Role>>,
    /// User-to-role mappings
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl BasicRBACManager {
    /// Create a new basic RBAC manager.
    ///
    /// # Returns
    /// A new `BasicRBACManager` instance with empty roles and user-role mappings.
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }
    
    /// Internal helper to get user roles as a HashSet.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to get roles for
    ///
    /// # Returns
    /// A HashSet of role IDs assigned to the user. If the user has no roles,
    /// an empty HashSet is returned.
    async fn get_user_roles_set(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }
    
    /// Get a role by ID.
    ///
    /// # Arguments
    /// * `role_id` - The ID of the role to retrieve
    ///
    /// # Returns
    /// * `Ok(Role)` - The role with the specified ID
    /// * `Err(...)` - If the role does not exist or an error occurred
    async fn get_role(&self, role_id: &str) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id)
            .cloned()
            .ok_or_else(|| SecurityError::RoleNotFound(role_id.to_string()).into())
    }
}

#[async_trait]
impl RBACManager for BasicRBACManager {
    fn name(&self) -> &str {
        "BasicRBACManager"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn has_permission(&self, user_id: &str, permission: &str, _context: Option<&Context>) -> Result<bool> {
        // Get all roles for the user
        let user_roles = self.get_user_roles_set(user_id).await;
        
        // Check if any of the user's roles have the permission
        for role_id in user_roles {
            let role = self.get_role(&role_id).await?;
            if role.permissions.contains(permission) {
                return Ok(true);
            }
        }
        
        // If we get here, the user doesn't have the permission
        Ok(false)
    }
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify the role exists
        self.get_role(role_id).await?;
        
        // Get the current user_roles mapping
        let mut user_roles = self.user_roles.write().await;
        
        // Get or create the roles for this user
        user_roles.entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());
        
        Ok(())
    }
    
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        
        // If the user has roles, remove the specified one
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        
        Ok(())
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_roles = self.get_user_roles_set(user_id).await;
        Ok(user_roles.into_iter().collect())
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.get_user_roles_set(user_id).await;
        Ok(user_roles.contains(role_id))
    }
    
    async fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        // Try to get the role
        let roles = self.roles.read().await;
        
        if let Some(role) = roles.get(role_id) {
            // Convert the internal Role to a RoleDefinition
            let role_def = RoleDefinition {
                id: role.definition.id.clone(),
                name: role.definition.name.clone(),
                description: role.definition.description.clone(),
                is_system_role: false,
                created_at: role.definition.created_at,
                updated_at: role.definition.updated_at,
            };
            
            // Create the response
            let response = RoleDetailsResponse {
                role: role_def,
                permissions: role.permissions.iter()
                    .map(|p| RolePermission {
                        permission_id: p.clone(),
                        granted_at: chrono::Utc::now(), // Since we don't track this, use current time
                        expires_at: None,
                    })
                    .collect(),
                parent_roles: Vec::new(),
                child_roles: Vec::new(),
            };
            
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }
    
    async fn get_permissions_for_role(&self, role_id: &str) -> Result<Vec<String>> {
        // Get the role
        let role = self.get_role(role_id).await?;
        
        // Return the permissions
        Ok(role.permissions.iter().cloned().collect())
    }
    
    async fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        // Check if the role already exists
        if roles.contains_key(role_id) {
            return Err(crate::error::SecurityError::RoleExists(role_id.to_string()).into());
        }
        
        // Create the new role
        let role = Role {
            definition: RoleDefinition {
                id: role_id.to_string(),
                name: name.to_string(),
                description: description.to_string(),
                is_system_role: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        };
        
        // Store the role
        roles.insert(role_id.to_string(), role);
        
        Ok(())
    }
    
    async fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        // Check if the role exists
        if let Some(role) = roles.get_mut(role_id) {
            // Add the permission to the role
            role.permissions.insert(permission.to_string());
            Ok(())
        } else {
            Err(crate::error::SecurityError::RoleNotFound(role_id.to_string()).into())
        }
    }
}

impl Default for BasicRBACManager {
    fn default() -> Self {
        Self::new()
    }
} 