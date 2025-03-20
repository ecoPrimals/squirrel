//! Role-Based Access Control (RBAC) implementation
//! 
//! This module provides role and permission management for the security system.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::error::{Result, SquirrelError};

/// Role in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: String,
    /// Name of the role
    pub name: String,
    /// Optional description of the role
    pub description: Option<String>,
    /// Set of permissions granted by this role
    pub permissions: HashSet<Permission>,
    /// Set of parent role IDs
    pub parent_roles: HashSet<String>,
}

/// Permission for a specific resource and action
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    /// Name of the permission
    pub name: String,
    /// Resource the permission applies to
    pub resource: String,
    /// Action allowed by this permission
    pub action: Action,
}

/// Action types for permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Create new resources
    Create,
    /// Read existing resources
    Read,
    /// Update existing resources
    Update,
    /// Delete existing resources
    Delete,
    /// Execute operations on resources
    Execute,
    /// Full administrative access
    Admin,
}

/// Role-Based Access Control manager
#[derive(Debug, Clone)]
pub struct RBACManager {
    /// Map of role IDs to Role objects (primary lookup)
    roles_by_id: HashMap<String, Role>,
    /// Map of role names to role IDs (secondary lookup)
    roles_by_name: HashMap<String, String>,
    /// Map of user IDs to their assigned role IDs
    user_roles: HashMap<String, HashSet<String>>,
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RBACManager {
    /// Creates a new RBAC manager
    #[must_use] pub fn new() -> Self {
        Self {
            roles_by_id: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    /// Gets a role by name
    #[must_use] pub fn get_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles_by_name.get(name).and_then(|id| self.roles_by_id.get(id))
    }

    /// Gets a role by ID
    #[must_use] pub fn get_role_by_id(&self, id: &str) -> Option<&Role> {
        self.roles_by_id.get(id)
    }

    /// Gets a role by either ID or name
    #[must_use] pub fn get_role(&self, id_or_name: &str) -> Option<&Role> {
        // First try as ID
        if let Some(role) = self.get_role_by_id(id_or_name) {
            return Some(role);
        }
        
        // Then try as name
        self.get_role_by_name(id_or_name)
    }

    /// Creates a new role with the given properties
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the role
    /// * `description` - Optional description of the role
    /// * `permissions` - Set of permissions granted by this role
    /// * `parent_roles` - Set of parent role IDs that this role inherits from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A role with the given name already exists
    /// - Any of the parent roles don't exist
    pub fn create_role(
        &mut self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if a role with this name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(SquirrelError::Security(format!("Role with name '{name}' already exists")));
        }
        
        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;
        
        // Create a unique ID for the role
        let id = Uuid::new_v4().to_string();
        
        // Create the role
        let role = Role {
            id: id.clone(),
            name: name.clone(),
            description,
            permissions,
            parent_roles,
        };
        
        // Store the role
        self.roles_by_id.insert(id.clone(), role.clone());
        self.roles_by_name.insert(name, id);
        
        Ok(role)
    }
    
    /// Creates a new role with the specified ID
    ///
    /// # Parameters
    ///
    /// * `id` - ID to use for the role
    /// * `name` - Name of the role
    /// * `description` - Optional description of the role
    /// * `permissions` - Set of permissions granted by this role
    /// * `parent_roles` - Set of parent role IDs that this role inherits from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A role with the given ID already exists
    /// - A role with the given name already exists
    /// - Any of the parent roles don't exist
    pub fn create_role_with_id(
        &mut self,
        id: String,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if a role with this ID already exists
        if self.roles_by_id.contains_key(&id) {
            return Err(SquirrelError::Security(format!("Role with ID '{id}' already exists")));
        }
        
        // Check if a role with this name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(SquirrelError::Security(format!("Role with name '{name}' already exists")));
        }
        
        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;
        
        // Create the role
        let role = Role {
            id: id.clone(),
            name: name.clone(),
            description,
            permissions,
            parent_roles,
        };
        
        // Store the role
        self.roles_by_id.insert(id.clone(), role.clone());
        self.roles_by_name.insert(name, id);
        
        Ok(role)
    }
    
    /// Verifies parent roles exist
    fn verify_parent_roles(&self, parent_roles: &HashSet<String>) -> Result<()> {
        for parent_id in parent_roles {
            if !self.roles_by_id.contains_key(parent_id) {
                return Err(SquirrelError::Security(format!("Role '{parent_id}' not found in system")));
            }
        }
        Ok(())
    }

    /// Assigns a role to a user
    ///
    /// # Parameters
    ///
    /// * `user_id` - ID of the user to assign the role to
    /// * `role_id` - ID of the role to assign
    ///
    /// # Errors
    ///
    /// Returns a `SquirrelError::Security` error if:
    /// - The role with the specified ID doesn't exist in the system
    pub fn assign_role(&mut self, user_id: String, role_id: String) -> Result<()> {
        // Check if role exists
        if !self.roles_by_id.contains_key(&role_id) {
            return Err(SquirrelError::Security(format!("Role '{role_id}' not found in system")));
        }
        
        // Get or create user roles set
        let user_roles = self.user_roles.entry(user_id).or_default();
        
        // Add role ID to user roles
        user_roles.insert(role_id);
        
        Ok(())
    }
    
    /// Assigns a role to a user by role name
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    /// * `role_name` - Name of the role to assign
    ///
    /// # Errors
    ///
    /// Returns a `SquirrelError::Security` error if:
    /// - The role with the specified name doesn't exist in the system
    /// - The underlying `assign_role` operation fails
    pub fn assign_role_by_name(&mut self, user_id: String, role_name: &str) -> Result<()> {
        // Check if role exists
        let role_id = self.roles_by_name.get(role_name)
            .ok_or_else(|| SquirrelError::Security(format!("Role '{role_name}' not found in system")))?
            .clone();
        
        // Assign role by ID
        self.assign_role(user_id, role_id)
    }

    /// Gets all permissions for a user
    #[must_use] pub fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        
        // Get user's role IDs
        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Some(role) = self.roles_by_id.get(role_id) {
                    // Collect permissions from this role and its parents
                    self.collect_role_permissions(role, &mut permissions);
                }
            }
        }
        
        permissions
    }

    /// Collects permissions from a role and its parents recursively
    fn collect_role_permissions(&self, role: &Role, permissions: &mut HashSet<Permission>) {
        // Add this role's permissions
        for permission in &role.permissions {
            permissions.insert(permission.clone());
        }
        
        // Add parent role permissions recursively
        for parent_id in &role.parent_roles {
            if let Some(parent_role) = self.roles_by_id.get(parent_id) {
                self.collect_role_permissions(parent_role, permissions);
            }
        }
    }

    /// Checks if a user has a specific permission
    #[must_use] pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        user_permissions.contains(permission)
    }
    
    /// Gets all roles assigned to a user
    #[must_use] pub fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
        let mut roles = Vec::new();
        
        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Some(role) = self.roles_by_id.get(role_id) {
                    roles.push(role.clone());
                }
            }
        }
        
        roles
    }
    
    /// Gets all users assigned to a role
    #[must_use] pub fn get_role_users(&self, role_id: &str) -> Vec<String> {
        let mut users = Vec::new();
        
        for (user_id, role_ids) in &self.user_roles {
            if role_ids.contains(role_id) {
                users.push(user_id.clone());
            }
        }
        
        users
    }

    /// Checks if a role has permission for a resource and action
    /// This replaces the old `role_has_permission` method
    #[must_use] pub fn has_permission_for_role(&self, role: &Role, resource: &str, action: Action) -> bool {
        // Check if the role has the permission directly
        let has_direct_permission = role.permissions.iter().any(|p| {
            p.resource == resource && p.action == action
        });
        
        if has_direct_permission {
            return true;
        }
        
        // Check parent roles recursively
        for parent_id in &role.parent_roles {
            if let Some(parent_role) = self.get_role_by_id(parent_id) {
                if self.has_permission_for_role(parent_role, resource, action) {
                    return true;
                }
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests;

// Remove the entire test module at the end of the file 