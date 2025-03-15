//! Role-Based Access Control (RBAC) implementation
//! 
//! This module provides role and permission management for the security system.

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::mcp::{MCPError, Result, SecurityError};

/// Represents a role in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

/// Represents a permission in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
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

/// Represents possible actions that can be performed on resources
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
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

/// Role-Based Access Control Manager
#[derive(Debug)]
pub struct RBACManager {
    /// Map of role names to Role objects
    roles: HashMap<String, Role>,
    /// Map of user IDs to their assigned role names
    user_roles: HashMap<String, HashSet<String>>,
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RBACManager {
    /// Creates a new RBAC manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    /// Gets a role by its name
    #[must_use]
    pub fn get_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Gets a role by its ID
    #[must_use]
    pub fn get_role_by_id(&self, id: &str) -> Option<&Role> {
        self.roles.values().find(|role| role.id == id)
    }

    /// Creates a new role with the given parameters.
    /// 
    /// # Arguments
    /// * `name` - The name of the role
    /// * `description` - Optional description of the role
    /// * `permissions` - Set of permissions granted to the role
    /// * `parent_roles` - Set of parent role IDs
    /// 
    /// # Errors
    /// Returns an error if any of the parent roles do not exist
    pub fn create_role(&mut self, name: String, description: Option<String>, permissions: HashSet<Permission>, parent_roles: HashSet<String>) -> Result<Role> {
        // Verify all parent roles exist by ID
        for parent_id in &parent_roles {
            if !self.roles.values().any(|role| role.id == *parent_id) {
                return Err(MCPError::Security(SecurityError::InvalidRole(
                    format!("Parent role {parent_id} does not exist")
                )));
            }
        }

        let role = Role {
            id: Uuid::new_v4().to_string(),
            name: name.clone(),
            description,
            permissions,
            parent_roles,
        };

        self.roles.insert(name, role.clone());
        Ok(role)
    }

    /// Assigns a role to a user.
    /// 
    /// # Arguments
    /// * `user_id` - The ID of the user
    /// * `role_id` - The ID of the role to assign
    /// 
    /// # Errors
    /// Returns an error if the role does not exist
    pub fn assign_role(&mut self, user_id: String, role_id: &str) -> Result<()> {
        // Find role by ID
        let role = self.roles.values()
            .find(|role| role.id == role_id)
            .ok_or_else(|| MCPError::Security(SecurityError::InvalidRole(
                format!("Role {role_id} does not exist")
            )))?;

        self.user_roles
            .entry(user_id)
            .or_default()
            .insert(role.name.clone());

        Ok(())
    }

    /// Gets all permissions assigned to a user through their roles.
    /// 
    /// # Arguments
    /// * `user_id` - The ID of the user
    #[must_use]
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        if let Some(role_names) = self.user_roles.get(user_id) {
            for role_name in role_names {
                if let Some(role) = self.roles.get(role_name) {
                    self.collect_role_permissions(role, &mut permissions);
                }
            }
        }

        permissions
    }

    /// Recursively collects all permissions from a role and its parent roles.
    fn collect_role_permissions(&self, role: &Role, permissions: &mut HashSet<Permission>) {
        permissions.extend(role.permissions.clone());

        for parent_id in &role.parent_roles {
            if let Some(parent_role) = self.roles.values().find(|r| r.id == *parent_id) {
                self.collect_role_permissions(parent_role, permissions);
            }
        }
    }

    /// Checks if a user has a specific permission.
    /// 
    /// # Arguments
    /// * `user_id` - The ID of the user
    /// * `permission` - The permission to check
    #[must_use]
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        user_permissions.iter().any(|p| 
            p.name == permission.name && 
            p.resource == permission.resource && 
            p.action == permission.action
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
        Permission {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            resource: resource.to_string(),
            action,
        }
    }

    #[test]
    fn test_role_creation() {
        let mut rbac = RBACManager::new();
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read_doc", "document", Action::Read));

        let role = rbac.create_role(
            "reader".to_string(),
            Some("Document reader".to_string()),
            permissions,
            HashSet::new(),
        ).unwrap();

        assert_eq!(role.name, "reader");
        assert_eq!(role.permissions.len(), 1);
    }

    #[test]
    fn test_role_inheritance() {
        let mut rbac = RBACManager::new();
        
        // Create base role
        let mut base_permissions = HashSet::new();
        base_permissions.insert(create_test_permission("read_doc", "document", Action::Read));
        let base_role = rbac.create_role(
            "reader".to_string(),
            None,
            base_permissions,
            HashSet::new(),
        ).unwrap();

        // Create admin role inheriting from base
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(create_test_permission("write_doc", "document", Action::Create));
        let mut parent_roles = HashSet::new();
        parent_roles.insert(base_role.id.clone());

        let admin_role = rbac.create_role(
            "admin".to_string(),
            None,
            admin_permissions,
            parent_roles,
        ).unwrap();

        // Assign admin role to user
        let user_id = "test_user";
        rbac.assign_role(user_id.to_string(), &admin_role.id).unwrap();

        // Check permissions
        let user_permissions = rbac.get_user_permissions(user_id);
        assert_eq!(user_permissions.len(), 2);
    }

    #[test]
    fn test_permission_check() {
        let mut rbac = RBACManager::new();
        
        // Create role with read permission
        let mut permissions = HashSet::new();
        let read_permission = create_test_permission("read_doc", "document", Action::Read);
        permissions.insert(read_permission.clone());

        let role = rbac.create_role(
            "reader".to_string(),
            None,
            permissions,
            HashSet::new(),
        ).unwrap();

        // Assign role to user
        let user_id = "test_user";
        rbac.assign_role(user_id.to_string(), &role.id).unwrap();

        // Test permission checks
        assert!(rbac.has_permission(user_id, &read_permission));
        
        let write_permission = create_test_permission("write_doc", "document", Action::Create);
        assert!(!rbac.has_permission(user_id, &write_permission));
    }
} 