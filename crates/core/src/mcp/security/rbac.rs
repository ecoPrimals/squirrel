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
    pub fn new() -> Self {
        Self {
            roles_by_id: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    /// Gets a role by name
    pub fn get_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles_by_name.get(name).and_then(|id| self.roles_by_id.get(id))
    }

    /// Gets a role by ID
    pub fn get_role_by_id(&self, id: &str) -> Option<&Role> {
        self.roles_by_id.get(id)
    }

    /// Gets a role by either ID or name
    pub fn get_role(&self, id_or_name: &str) -> Option<&Role> {
        // First try as ID
        if let Some(role) = self.get_role_by_id(id_or_name) {
            return Some(role);
        }
        
        // Then try as name
        self.get_role_by_name(id_or_name)
    }

    /// Creates a new role
    pub fn create_role(
        &mut self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if role name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(SquirrelError::Security(format!("Role with name '{}' already exists", name)));
        }
        
        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;
        
        // Create new role with UUID
        let id = Uuid::new_v4().to_string();
        
        // Create the role
        self.create_role_with_id(id, name, description, permissions, parent_roles)
    }
    
    /// Creates a new role with a specific ID (useful for testing)
    pub fn create_role_with_id(
        &mut self,
        id: String,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if role ID already exists
        if self.roles_by_id.contains_key(&id) {
            return Err(SquirrelError::Security(format!("Role with ID '{}' already exists", id)));
        }
        
        // Check if role name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(SquirrelError::Security(format!("Role with name '{}' already exists", name)));
        }
        
        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;
        
        let role = Role {
            id: id.clone(),
            name: name.clone(),
            description,
            permissions,
            parent_roles,
        };
        
        // Store role
        self.roles_by_id.insert(id.clone(), role.clone());
        self.roles_by_name.insert(name, id);
        
        Ok(role)
    }
    
    /// Verifies parent roles exist
    fn verify_parent_roles(&self, parent_roles: &HashSet<String>) -> Result<()> {
        for parent_id in parent_roles {
            if !self.roles_by_id.contains_key(parent_id) {
                return Err(SquirrelError::Security(format!("Role '{}' not found in system", parent_id)));
            }
        }
        Ok(())
    }

    /// Assigns a role to a user
    pub fn assign_role(&mut self, user_id: String, role_id: String) -> Result<()> {
        // Check if role exists
        if !self.roles_by_id.contains_key(&role_id) {
            return Err(SquirrelError::Security(format!("Role '{}' not found in system", role_id)));
        }
        
        // Get or create user roles set
        let user_roles = self.user_roles.entry(user_id).or_insert_with(HashSet::new);
        
        // Add role ID to user roles
        user_roles.insert(role_id);
        
        Ok(())
    }
    
    /// Assigns a role to a user by name
    pub fn assign_role_by_name(&mut self, user_id: String, role_name: String) -> Result<()> {
        // Check if role exists
        let role_id = self.roles_by_name.get(&role_name)
            .ok_or_else(|| SquirrelError::Security(format!("Role '{}' not found in system", role_name)))?
            .clone();
        
        // Assign role by ID
        self.assign_role(user_id, role_id)
    }

    /// Gets all permissions for a user
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
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
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        user_permissions.contains(permission)
    }
    
    /// Gets all roles assigned to a user
    pub fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
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
    pub fn get_role_users(&self, role_id: &str) -> Vec<String> {
        let mut users = Vec::new();
        
        for (user_id, role_ids) in &self.user_roles {
            if role_ids.contains(role_id) {
                users.push(user_id.clone());
            }
        }
        
        users
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
        
        // Create role with permissions
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read", "document", Action::Read));
        
        let role = rbac.create_role(
            "reader".to_string(),
            Some("Read-only role".to_string()),
            permissions,
            HashSet::new(),
        ).unwrap();
        
        // Verify role
        assert_eq!(role.name, "reader");
        assert_eq!(role.description, Some("Read-only role".to_string()));
        assert_eq!(role.permissions.len(), 1);
        assert_eq!(role.parent_roles.len(), 0);
    }
    
    #[test]
    fn test_role_inheritance() {
        let mut rbac = RBACManager::new();
        
        // Create base role with read permission
        let mut base_permissions = HashSet::new();
        base_permissions.insert(create_test_permission("read", "document", Action::Read));
        
        let base_role = rbac.create_role(
            "reader".to_string(),
            None,
            base_permissions,
            HashSet::new(),
        ).unwrap();
        
        // Create admin role with write permission, inheriting from base role
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(create_test_permission("write", "document", Action::Create));
        
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
        rbac.assign_role(user_id.to_string(), admin_role.id.clone()).unwrap();
        
        // Check permissions
        let user_permissions = rbac.get_user_permissions(user_id);
        assert_eq!(user_permissions.len(), 2);
    }
    
    #[test]
    fn test_role_lookup() {
        let mut rbac = RBACManager::new();
        
        // Create a role
        let role = rbac.create_role(
            "tester".to_string(),
            None,
            HashSet::new(),
            HashSet::new(),
        ).unwrap();
        
        // Look up by ID
        let by_id = rbac.get_role_by_id(&role.id);
        assert!(by_id.is_some());
        assert_eq!(by_id.unwrap().name, "tester");
        
        // Look up by name
        let by_name = rbac.get_role_by_name("tester");
        assert!(by_name.is_some());
        assert_eq!(by_name.unwrap().id, role.id);
        
        // Look up with generic method
        assert!(rbac.get_role(&role.id).is_some());
        assert!(rbac.get_role("tester").is_some());
        assert!(rbac.get_role("nonexistent").is_none());
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
        rbac.assign_role(user_id.to_string(), role.id.clone()).unwrap();
        
        // Test permission checks
        assert!(rbac.has_permission(user_id, &read_permission));
        
        let write_permission = create_test_permission("write_doc", "document", Action::Create);
        assert!(!rbac.has_permission(user_id, &write_permission));
    }
    
    #[test]
    fn test_assign_role_by_name() {
        let mut rbac = RBACManager::new();
        
        // Create role
        let role = rbac.create_role(
            "editor".to_string(),
            None,
            HashSet::new(),
            HashSet::new(),
        ).unwrap();
        
        // Assign by name
        let user_id = "test_user";
        rbac.assign_role_by_name(user_id.to_string(), "editor".to_string()).unwrap();
        
        // Verify assignment
        let user_roles = rbac.get_user_roles(user_id);
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles[0].id, role.id);
    }
    
    #[test]
    fn test_role_with_predefined_id() {
        let mut rbac = RBACManager::new();
        
        // Create role with predefined ID
        let test_id = "test-role-123";
        let role = rbac.create_role_with_id(
            test_id.to_string(),
            "tester".to_string(),
            None,
            HashSet::new(),
            HashSet::new(),
        ).unwrap();
        
        // Verify ID
        assert_eq!(role.id, test_id);
        
        // Look up by ID
        let found = rbac.get_role_by_id(test_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "tester");
    }
} 