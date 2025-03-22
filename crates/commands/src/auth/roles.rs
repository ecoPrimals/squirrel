//! Role-based access control (RBAC) system
//!
//! This module implements a role-based access control system that provides
//! fine-grained authorization for commands through roles and permissions.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthResult;
use crate::{Command, CommandError};

/// Permission to perform an action on a resource
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    
    /// Human-readable name for the permission
    pub name: String,
    
    /// Detailed description of what the permission allows
    pub description: String,
    
    /// Resource being accessed (e.g., "command", "user", "system")
    pub resource: String,
    
    /// Action being performed (e.g., "read", "write", "execute")
    pub action: String,
}

impl Permission {
    /// Creates a new permission
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Creates a new permission with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Returns a permission key in the format "resource:action"
    pub fn key(&self) -> String {
        format!("{}:{}", self.resource, self.action)
    }
}

/// Role containing a set of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: String,
    
    /// Human-readable name for the role
    pub name: String,
    
    /// Detailed description of the role
    pub description: String,
    
    /// Set of permission IDs granted by this role
    pub permissions: HashSet<String>,
    
    /// Set of parent role IDs that this role inherits from
    pub parent_roles: HashSet<String>,
}

impl Role {
    /// Creates a new role
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        }
    }

    /// Creates a new role with a specific ID
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        }
    }

    /// Adds a permission to the role
    pub fn with_permission(mut self, permission_id: impl Into<String>) -> Self {
        self.permissions.insert(permission_id.into());
        self
    }

    /// Adds multiple permissions to the role
    pub fn with_permissions(mut self, permission_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for permission_id in permission_ids {
            self.permissions.insert(permission_id.into());
        }
        self
    }

    /// Adds a parent role to inherit permissions from
    pub fn with_parent(mut self, parent_role_id: impl Into<String>) -> Self {
        self.parent_roles.insert(parent_role_id.into());
        self
    }

    /// Adds multiple parent roles to inherit permissions from
    pub fn with_parents(mut self, parent_role_ids: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for parent_role_id in parent_role_ids {
            self.parent_roles.insert(parent_role_id.into());
        }
        self
    }

    /// Checks if the role directly has a specific permission
    pub fn has_direct_permission(&self, permission_id: &str) -> bool {
        self.permissions.contains(permission_id)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.id)
    }
}

/// Manager for roles and permissions
#[derive(Debug, Clone)]
pub struct RoleManager {
    /// Mapping of role ID to role
    roles: Arc<RwLock<HashMap<String, Role>>>,
    
    /// Mapping of permission ID to permission
    permissions: Arc<RwLock<HashMap<String, Permission>>>,
    
    /// Mapping of user ID to set of role IDs
    user_roles: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    
    /// Mapping of command name to required permissions
    command_permissions: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl RoleManager {
    /// Creates a new role manager
    pub fn new() -> Self {
        Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            command_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new permission
    pub async fn create_permission(&self, permission: Permission) -> AuthResult<()> {
        let mut permissions = self.permissions.write().await;
        if permissions.contains_key(&permission.id) {
            return Err(CommandError::AuthorizationError(format!(
                "Permission with ID '{}' already exists",
                permission.id
            )));
        }
        permissions.insert(permission.id.clone(), permission);
        Ok(())
    }

    /// Gets a permission by ID
    pub async fn get_permission(&self, permission_id: &str) -> AuthResult<Permission> {
        let permissions = self.permissions.read().await;
        permissions
            .get(permission_id)
            .cloned()
            .ok_or_else(|| {
                CommandError::AuthorizationError(format!("Permission with ID '{}' not found", permission_id))
            })
    }

    /// Lists all permissions
    pub async fn list_permissions(&self) -> AuthResult<Vec<Permission>> {
        let permissions = self.permissions.read().await;
        Ok(permissions.values().cloned().collect())
    }

    /// Deletes a permission
    pub async fn delete_permission(&self, permission_id: &str) -> AuthResult<()> {
        // Check if permission is in use by any roles
        let roles = self.roles.read().await;
        for role in roles.values() {
            if role.permissions.contains(permission_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Cannot delete permission '{}' as it is used by role '{}'",
                    permission_id, role.name
                )));
            }
        }

        let mut permissions = self.permissions.write().await;
        if !permissions.contains_key(permission_id) {
            return Err(CommandError::AuthorizationError(format!(
                "Permission with ID '{}' not found",
                permission_id
            )));
        }
        permissions.remove(permission_id);
        Ok(())
    }

    /// Creates a new role
    pub async fn create_role(&self, role: Role) -> AuthResult<()> {
        // Validate that all parent roles exist
        let roles = self.roles.read().await;
        for parent_id in &role.parent_roles {
            if !roles.contains_key(parent_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Parent role with ID '{}' not found",
                    parent_id
                )));
            }
        }

        // Validate that all permissions exist
        let permissions = self.permissions.read().await;
        for permission_id in &role.permissions {
            if !permissions.contains_key(permission_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Permission with ID '{}' not found",
                    permission_id
                )));
            }
        }

        // Check for circular references in parent roles
        if role.parent_roles.contains(&role.id) {
            return Err(CommandError::AuthorizationError(
                "Role cannot be its own parent".to_string(),
            ));
        }
        
        // Check for indirect circular references
        Self::validate_role_hierarchy(&roles, &role)?;

        // Add the role
        drop(roles);
        let mut roles = self.roles.write().await;
        roles.insert(role.id.clone(), role);
        Ok(())
    }

    /// Updates an existing role
    pub async fn update_role(&self, role: Role) -> AuthResult<()> {
        // First delete the old role
        {
            let mut roles = self.roles.write().await;
            if !roles.contains_key(&role.id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Role with ID '{}' not found",
                    role.id
                )));
            }
            roles.remove(&role.id);
        }

        // Then create it again with validation
        self.create_role(role).await
    }

    /// Gets a role by ID
    pub async fn get_role(&self, role_id: &str) -> AuthResult<Role> {
        let roles = self.roles.read().await;
        roles
            .get(role_id)
            .cloned()
            .ok_or_else(|| {
                CommandError::AuthorizationError(format!("Role with ID '{}' not found", role_id))
            })
    }

    /// Lists all roles
    pub async fn list_roles(&self) -> AuthResult<Vec<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }

    /// Deletes a role
    pub async fn delete_role(&self, role_id: &str) -> AuthResult<()> {
        // Check if role is a parent of any other roles
        let roles = self.roles.read().await;
        for role in roles.values() {
            if role.parent_roles.contains(role_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Cannot delete role '{}' as it is a parent of role '{}'",
                    role_id, role.name
                )));
            }
        }

        // Check if role is assigned to any users
        let user_roles = self.user_roles.read().await;
        for (user_id, assigned_roles) in user_roles.iter() {
            if assigned_roles.contains(role_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Cannot delete role '{}' as it is assigned to user '{}'",
                    role_id, user_id
                )));
            }
        }

        let mut roles = self.roles.write().await;
        if !roles.contains_key(role_id) {
            return Err(CommandError::AuthorizationError(format!(
                "Role with ID '{}' not found",
                role_id
            )));
        }
        roles.remove(role_id);
        Ok(())
    }

    /// Assigns a role to a user
    pub async fn assign_role_to_user(&self, user_id: &str, role_id: &str) -> AuthResult<()> {
        // Validate role exists
        let roles = self.roles.read().await;
        if !roles.contains_key(role_id) {
            return Err(CommandError::AuthorizationError(format!(
                "Role with ID '{}' not found",
                role_id
            )));
        }

        let mut user_roles = self.user_roles.write().await;
        let user_role_set = user_roles.entry(user_id.to_string()).or_insert_with(HashSet::new);
        user_role_set.insert(role_id.to_string());
        Ok(())
    }

    /// Revokes a role from a user
    pub async fn revoke_role_from_user(&self, user_id: &str, role_id: &str) -> AuthResult<()> {
        let mut user_roles = self.user_roles.write().await;
        
        if let Some(role_set) = user_roles.get_mut(user_id) {
            if !role_set.remove(role_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "User '{}' does not have role '{}'",
                    user_id, role_id
                )));
            }
            Ok(())
        } else {
            Err(CommandError::AuthorizationError(format!(
                "User '{}' has no assigned roles",
                user_id
            )))
        }
    }

    /// Gets all roles assigned to a user
    pub async fn get_user_roles(&self, user_id: &str) -> AuthResult<HashSet<String>> {
        let user_roles = self.user_roles.read().await;
        Ok(user_roles
            .get(user_id)
            .cloned()
            .unwrap_or_else(HashSet::new))
    }

    /// Gets all roles assigned to a user, including inherited roles
    pub async fn get_effective_user_roles(&self, user_id: &str) -> AuthResult<HashSet<String>> {
        let direct_roles = self.get_user_roles(user_id).await?;
        let roles = self.roles.read().await;
        
        let mut effective_roles = HashSet::new();
        let mut queue = direct_roles.iter().cloned().collect::<Vec<_>>();
        
        while let Some(role_id) = queue.pop() {
            if effective_roles.contains(&role_id) {
                continue;
            }
            
            effective_roles.insert(role_id.clone());
            
            if let Some(role) = roles.get(&role_id) {
                for parent_id in &role.parent_roles {
                    if !effective_roles.contains(parent_id) {
                        queue.push(parent_id.clone());
                    }
                }
            }
        }
        
        Ok(effective_roles)
    }

    /// Gets all permissions granted to a user through their roles
    pub async fn get_user_permissions(&self, user_id: &str) -> AuthResult<HashSet<String>> {
        let effective_roles = self.get_effective_user_roles(user_id).await?;
        let roles = self.roles.read().await;
        
        let mut permissions = HashSet::new();
        for role_id in effective_roles {
            if let Some(role) = roles.get(&role_id) {
                permissions.extend(role.permissions.iter().cloned());
            }
        }
        
        Ok(permissions)
    }

    /// Checks if a user has a specific permission
    pub async fn user_has_permission(&self, user_id: &str, permission_id: &str) -> AuthResult<bool> {
        let user_permissions = self.get_user_permissions(user_id).await?;
        Ok(user_permissions.contains(permission_id))
    }

    /// Sets the permissions required to execute a command
    pub async fn set_command_permissions(&self, command_name: &str, permission_ids: HashSet<String>) -> AuthResult<()> {
        // Validate all permissions exist
        let permissions = self.permissions.read().await;
        for permission_id in &permission_ids {
            if !permissions.contains_key(permission_id) {
                return Err(CommandError::AuthorizationError(format!(
                    "Permission with ID '{}' not found",
                    permission_id
                )));
            }
        }
        
        let mut command_permissions = self.command_permissions.write().await;
        command_permissions.insert(command_name.to_string(), permission_ids);
        Ok(())
    }

    /// Gets the permissions required to execute a command
    pub async fn get_command_permissions(&self, command_name: &str) -> AuthResult<HashSet<String>> {
        let command_permissions = self.command_permissions.read().await;
        Ok(command_permissions
            .get(command_name)
            .cloned()
            .unwrap_or_else(HashSet::new))
    }

    /// Authorizes a user to execute a command
    pub async fn authorize_command(&self, user_id: &str, command: &dyn Command) -> AuthResult<bool> {
        let required_permissions = self.get_command_permissions(command.name()).await?;
        
        // If no specific permissions are required, the command is accessible
        if required_permissions.is_empty() {
            return Ok(true);
        }
        
        let user_permissions = self.get_user_permissions(user_id).await?;
        
        // Check if user has any of the required permissions
        for permission_id in &required_permissions {
            if user_permissions.contains(permission_id) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Validates that a role hierarchy does not contain circular references
    fn validate_role_hierarchy(
        roles: &HashMap<String, Role>,
        role: &Role,
    ) -> AuthResult<()> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // DFS to detect cycles
        stack.push(role.id.clone());
        
        while let Some(current_id) = stack.pop() {
            if visited.contains(&current_id) {
                continue;
            }
            
            visited.insert(current_id.clone());
            
            // Get current role
            let current_role = if current_id == role.id {
                role
            } else {
                match roles.get(&current_id) {
                    Some(r) => r,
                    None => continue, // Role not found, skip
                }
            };
            
            // Add parent roles to stack
            for parent_id in &current_role.parent_roles {
                // Check for cycle
                if parent_id == &role.id {
                    return Err(CommandError::AuthorizationError(
                        "Circular role reference detected".to_string(),
                    ));
                }
                
                if !visited.contains(parent_id) {
                    stack.push(parent_id.clone());
                }
            }
        }
        
        Ok(())
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a standard set of permissions
pub fn create_standard_permissions() -> Vec<Permission> {
    vec![
        // Command permissions
        Permission::new(
            "command:execute",
            "Execute any command",
            "command",
            "execute",
        ),
        Permission::new(
            "command:list",
            "List available commands",
            "command",
            "list",
        ),
        
        // User management permissions
        Permission::new(
            "user:create",
            "Create new users",
            "user",
            "create",
        ),
        Permission::new(
            "user:read",
            "View user information",
            "user",
            "read",
        ),
        Permission::new(
            "user:update",
            "Update user information",
            "user",
            "update",
        ),
        Permission::new(
            "user:delete",
            "Delete users",
            "user",
            "delete",
        ),
        
        // Role management permissions
        Permission::new(
            "role:create",
            "Create new roles",
            "role",
            "create",
        ),
        Permission::new(
            "role:read",
            "View role information",
            "role",
            "read",
        ),
        Permission::new(
            "role:update",
            "Update role information",
            "role",
            "update",
        ),
        Permission::new(
            "role:delete",
            "Delete roles",
            "role",
            "delete",
        ),
        Permission::new(
            "role:assign",
            "Assign roles to users",
            "role",
            "assign",
        ),
        
        // System permissions
        Permission::new(
            "system:configure",
            "Configure system settings",
            "system",
            "configure",
        ),
        Permission::new(
            "system:monitor",
            "Monitor system status",
            "system",
            "monitor",
        ),
    ]
}

/// Helper function to create standard roles
pub fn create_standard_roles(permissions: &[Permission]) -> Vec<Role> {
    // Create a map of permission names to IDs
    let perm_map: HashMap<String, String> = permissions
        .iter()
        .map(|p| (p.key(), p.id.clone()))
        .collect();
    
    // Create roles
    vec![
        // Admin role with all permissions
        Role::new("Administrator", "Full system access")
            .with_permissions(perm_map.values().cloned()),
        
        // User Manager role
        Role::new("User Manager", "Manage users and roles")
            .with_permission(perm_map.get("user:create").unwrap())
            .with_permission(perm_map.get("user:read").unwrap())
            .with_permission(perm_map.get("user:update").unwrap())
            .with_permission(perm_map.get("user:delete").unwrap())
            .with_permission(perm_map.get("role:assign").unwrap()),
        
        // Standard role
        Role::new("User", "Standard user access")
            .with_permission(perm_map.get("command:execute").unwrap())
            .with_permission(perm_map.get("command:list").unwrap())
            .with_permission(perm_map.get("user:read").unwrap()),
        
        // Readonly role
        Role::new("ReadOnly", "Read-only access")
            .with_permission(perm_map.get("command:list").unwrap())
            .with_permission(perm_map.get("user:read").unwrap())
            .with_permission(perm_map.get("role:read").unwrap())
            .with_permission(perm_map.get("system:monitor").unwrap()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }
        
        fn description(&self) -> &str {
            "Test command"
        }
        
        fn execute(&self, _args: &[String]) -> crate::CommandResult<String> {
            Ok("test".to_string())
        }
        
        fn parser(&self) -> clap::Command {
            clap::Command::new("test")
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(Self)
        }
    }

    #[tokio::test]
    async fn test_role_manager() {
        let manager = RoleManager::new();
        
        // Create permissions
        let perm1 = Permission::new("Test Permission", "For testing", "test", "read");
        let perm2 = Permission::new("Command Execute", "Execute commands", "command", "execute");
        
        manager.create_permission(perm1.clone()).await.unwrap();
        manager.create_permission(perm2.clone()).await.unwrap();
        
        // Create roles
        let role1 = Role::new("Test Role", "For testing")
            .with_permission(perm1.id.clone());
        
        let role2 = Role::new("Admin Role", "Administrator")
            .with_permission(perm1.id.clone())
            .with_permission(perm2.id.clone());
        
        manager.create_role(role1.clone()).await.unwrap();
        manager.create_role(role2.clone()).await.unwrap();
        
        // Assign roles to user
        let user_id = "test_user";
        manager.assign_role_to_user(user_id, &role1.id).await.unwrap();
        
        // Test permission checks
        assert!(manager.user_has_permission(user_id, &perm1.id).await.unwrap());
        assert!(!manager.user_has_permission(user_id, &perm2.id).await.unwrap());
        
        // Assign admin role
        manager.assign_role_to_user(user_id, &role2.id).await.unwrap();
        assert!(manager.user_has_permission(user_id, &perm2.id).await.unwrap());
        
        // Set command permissions
        let mut command_perms = HashSet::new();
        command_perms.insert(perm2.id.clone());
        manager.set_command_permissions("test", command_perms).await.unwrap();
        
        // Test command authorization
        let command = TestCommand;
        assert!(manager.authorize_command(user_id, &command).await.unwrap());
        
        // Revoke admin role
        manager.revoke_role_from_user(user_id, &role2.id).await.unwrap();
        assert!(!manager.user_has_permission(user_id, &perm2.id).await.unwrap());
        assert!(!manager.authorize_command(user_id, &command).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_role_hierarchy() {
        let manager = RoleManager::new();
        
        // Create permission
        let perm = Permission::new("Test Permission", "For testing", "test", "read");
        manager.create_permission(perm.clone()).await.unwrap();
        
        // Create roles with hierarchy
        let role1 = Role::new("Base Role", "Base role with permission")
            .with_permission(perm.id.clone());
        
        manager.create_role(role1.clone()).await.unwrap();
        
        let role2 = Role::new("Child Role", "Inherits from base role")
            .with_parent(role1.id.clone());
        
        manager.create_role(role2.clone()).await.unwrap();
        
        // Test permission inheritance
        let user_id = "test_user";
        manager.assign_role_to_user(user_id, &role2.id).await.unwrap();
        
        assert!(manager.user_has_permission(user_id, &perm.id).await.unwrap());
        
        // Test circular reference detection
        let role3 = Role::new("Circular Role", "Creates a circular reference")
            .with_parent(role2.id.clone());
        
        manager.create_role(role3.clone()).await.unwrap();
        
        // Try to update role2 to reference role3, creating a cycle
        let updated_role2 = Role {
            id: role2.id.clone(),
            name: role2.name.clone(),
            description: role2.description.clone(),
            permissions: role2.permissions.clone(),
            parent_roles: {
                let mut parents = HashSet::new();
                parents.insert(role1.id.clone());
                parents.insert(role3.id.clone());
                parents
            },
        };
        
        assert!(manager.update_role(updated_role2).await.is_err());
    }
    
    #[tokio::test]
    async fn test_standard_roles_and_permissions() {
        let permissions = create_standard_permissions();
        let roles = create_standard_roles(&permissions);
        
        // Verify permissions
        assert_eq!(permissions.len(), 13);
        
        // Verify roles
        assert_eq!(roles.len(), 4);
        
        // Verify admin role has all permissions
        let admin_role = roles.iter().find(|r| r.name == "Administrator").unwrap();
        assert_eq!(admin_role.permissions.len(), permissions.len());
        
        // Verify user manager role has user management permissions
        let user_manager = roles.iter().find(|r| r.name == "User Manager").unwrap();
        assert_eq!(user_manager.permissions.len(), 5);
        
        // Verify standard user role
        let user_role = roles.iter().find(|r| r.name == "User").unwrap();
        assert_eq!(user_role.permissions.len(), 3);
    }
} 