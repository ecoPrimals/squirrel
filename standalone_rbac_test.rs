// A standalone test for RBAC implementations
// Run with: rustc standalone_rbac_test.rs && ./standalone_rbac_test

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::error::Error;
use std::fmt;

// A simple error type for our tests
#[derive(Debug)]
struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TestError {}

// Define our own Result type for the test
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

// Define a simplified Context type for permission checks
#[derive(Debug, Clone, Default)]
struct Context {
    attributes: HashMap<String, String>,
}

// ----- Role and Permission Definitions ----- //

#[derive(Debug, Clone)]
struct Role {
    id: String,
    name: String,
    description: String,
    permissions: HashSet<String>,
    is_system_role: bool,
}

impl Role {
    fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            permissions: HashSet::new(),
            is_system_role: false,
        }
    }

    fn add_permission(&mut self, permission: &str) {
        self.permissions.insert(permission.to_string());
    }

    fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }
}

// ----- RBAC Manager Trait ----- //

trait RBACManager: Send + Sync + 'static {
    // ---- Core Role Management ---- //
    
    /// Assign a role to a user
    fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Revoke a role from a user
    fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Get all roles assigned to a user
    fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    
    /// Check if a user has a specific role
    fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool>;
    
    // ---- Permission Management ---- //
    
    /// Check if a user has a specific permission (with optional context)
    fn has_permission(
        &self, 
        user_id: &str, 
        permission: &str, 
        context: Option<&Context>
    ) -> Result<bool>;
    
    // ---- Role Details ---- //
    
    /// Get details about a specific role
    fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>>;
    
    /// Get all permissions for a specific role
    fn get_permissions_for_role(&self, role_id: &str) -> Result<Vec<String>>;
    
    // ---- Role Creation and Management ---- //
    
    /// Create a new role
    fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()>;
    
    /// Add a permission to a role
    fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()>;
}

// A simplified response for role details
#[derive(Debug, Clone)]
struct RoleDetailsResponse {
    id: String,
    name: String,
    description: String,
    permissions: Vec<String>,
    parent_roles: Vec<String>,
    child_roles: Vec<String>,
}

// ----- Basic RBAC Manager Implementation ----- //

struct BasicRBACManager {
    // Map of role IDs to role objects
    roles: std::sync::Mutex<HashMap<String, Role>>,
    
    // Map of user IDs to the set of role IDs they have
    user_roles: std::sync::Mutex<HashMap<String, HashSet<String>>>,
}

impl BasicRBACManager {
    fn new() -> Self {
        Self {
            roles: std::sync::Mutex::new(HashMap::new()),
            user_roles: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    // Internal helper to get a role (not part of the trait)
    fn get_role(&self, role_id: &str) -> Option<Role> {
        let roles = self.roles.lock().unwrap();
        roles.get(role_id).cloned()
    }
}

impl RBACManager for BasicRBACManager {
    fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify the role exists
        if self.get_role(role_id).is_none() {
            return Err(Box::new(TestError(format!("Role {} does not exist", role_id))));
        }
        
        let mut user_roles = self.user_roles.lock().unwrap();
        
        // Get or create the set of roles for this user
        let roles = user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new);
        
        // Add the role
        roles.insert(role_id.to_string());
        
        Ok(())
    }
    
    fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.lock().unwrap();
        
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        
        Ok(())
    }
    
    fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_roles = self.user_roles.lock().unwrap();
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.iter().cloned().collect()),
            None => Ok(Vec::new()),
        }
    }
    
    fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.user_roles.lock().unwrap();
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.contains(role_id)),
            None => Ok(false),
        }
    }
    
    fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()> {
        let mut roles = self.roles.lock().unwrap();
        
        if roles.contains_key(role_id) {
            return Err(Box::new(TestError(format!("Role {} already exists", role_id))));
        }
        
        roles.insert(
            role_id.to_string(),
            Role::new(role_id, name, description),
        );
        
        Ok(())
    }
    
    fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()> {
        let mut roles = self.roles.lock().unwrap();
        
        match roles.get_mut(role_id) {
            Some(role) => {
                role.add_permission(permission);
                Ok(())
            }
            None => Err(Box::new(TestError(format!("Role {} does not exist", role_id)))),
        }
    }
    
    fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        let role = self.get_role(role_id);
        
        match role {
            Some(role) => {
                Ok(Some(RoleDetailsResponse {
                    id: role.id,
                    name: role.name,
                    description: role.description,
                    permissions: role.permissions.into_iter().collect(),
                    parent_roles: Vec::new(), // Not implemented in basic manager
                    child_roles: Vec::new(),  // Not implemented in basic manager
                }))
            }
            None => Ok(None),
        }
    }
    
    fn get_permissions_for_role(&self, role_id: &str) -> Result<Vec<String>> {
        let role = self.get_role(role_id);
        
        match role {
            Some(role) => Ok(role.permissions.into_iter().collect()),
            None => Err(Box::new(TestError(format!("Role {} does not exist", role_id)))),
        }
    }
    
    fn has_permission(
        &self, 
        user_id: &str, 
        permission: &str, 
        _context: Option<&Context>
    ) -> Result<bool> {
        let roles = self.get_user_roles(user_id)?;
        
        for role_id in roles {
            let role = self.get_role(&role_id);
            
            if let Some(role) = role {
                if role.has_permission(permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}

// ----- Mock RBAC Manager Implementation ----- //

#[derive(Clone)]
struct MockRBACManager {
    allow_all: bool,
    user_roles: Arc<std::sync::Mutex<HashMap<String, Vec<String>>>>,
}

impl MockRBACManager {
    fn new(allow_all: bool) -> Self {
        Self {
            allow_all,
            user_roles: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }
    
    // Set specific roles for a user
    fn with_user_roles(mut self, user_id: &str, roles: Vec<String>) -> Self {
        // Create the roles outside the borrow scope
        {
            let mut user_roles = self.user_roles.lock().unwrap();
            user_roles.insert(user_id.to_string(), roles);
        }
        // Return self after the lock is dropped
        self
    }
}

impl RBACManager for MockRBACManager {
    fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        if self.allow_all {
            return Ok(());
        }
        
        let mut user_roles = self.user_roles.lock().unwrap();
        
        let roles = user_roles
            .entry(user_id.to_string())
            .or_insert_with(Vec::new);
        
        if !roles.contains(&role_id.to_string()) {
            roles.push(role_id.to_string());
        }
        
        Ok(())
    }
    
    fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        if self.allow_all {
            return Ok(());
        }
        
        let mut user_roles = self.user_roles.lock().unwrap();
        
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.retain(|r| r != role_id);
        }
        
        Ok(())
    }
    
    fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        if self.allow_all {
            // Return a special wildcard role when all permissions are allowed
            return Ok(vec!["*".to_string()]);
        }
        
        let user_roles = self.user_roles.lock().unwrap();
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        if self.allow_all {
            return Ok(true);
        }
        
        let user_roles = self.user_roles.lock().unwrap();
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.contains(&role_id.to_string())),
            None => Ok(false),
        }
    }
    
    fn has_permission(
        &self, 
        user_id: &str, 
        _permission: &str, 
        _context: Option<&Context>
    ) -> Result<bool> {
        if self.allow_all {
            return Ok(true);
        }
        
        // In this mock, we'll just check if the user has any roles
        let user_roles = self.user_roles.lock().unwrap();
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(!roles.is_empty()),
            None => Ok(false),
        }
    }
    
    fn get_role_details(&self, _role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        // Simplified implementation for mock
        Ok(None)
    }
    
    fn get_permissions_for_role(&self, _role_id: &str) -> Result<Vec<String>> {
        // Simplified implementation for mock
        Ok(Vec::new())
    }
    
    fn create_role(&self, _role_id: &str, _name: &str, _description: &str) -> Result<()> {
        // Simplified implementation for mock
        Ok(())
    }
    
    fn add_permission_to_role(&self, _role_id: &str, _permission: &str) -> Result<()> {
        // Simplified implementation for mock
        Ok(())
    }
}

// ----- Main Test Function ----- //

fn main() -> Result<()> {
    println!("RBAC Independent Test");
    println!("=====================");
    
    // Test the BasicRBACManager
    test_basic_rbac_manager()?;
    
    // Test the MockRBACManager
    test_mock_rbac_manager()?;
    
    println!("\nAll tests passed successfully!");
    
    Ok(())
}

// ----- Test Basic RBAC Manager ----- //

fn test_basic_rbac_manager() -> Result<()> {
    println!("\nTesting BasicRBACManager...");
    
    let rbac = Arc::new(BasicRBACManager::new());
    
    // Create roles
    rbac.create_role("admin", "Administrator", "System administrator")?;
    rbac.create_role("user", "Regular User", "Basic user permissions")?;
    rbac.create_role("editor", "Editor", "Can edit content")?;
    
    // Add permissions to roles
    rbac.add_permission_to_role("admin", "system:access")?;
    rbac.add_permission_to_role("admin", "user:manage")?;
    rbac.add_permission_to_role("user", "content:read")?;
    rbac.add_permission_to_role("editor", "content:read")?;
    rbac.add_permission_to_role("editor", "content:write")?;
    
    // Test assigning roles
    rbac.assign_role("alice", "admin")?;
    rbac.assign_role("bob", "user")?;
    rbac.assign_role("charlie", "editor")?;
    rbac.assign_role("charlie", "user")?;
    
    // Test role checks
    assert!(rbac.has_role("alice", "admin")?, "Alice should be an admin");
    assert!(!rbac.has_role("alice", "user")?, "Alice should not be a user");
    assert!(rbac.has_role("bob", "user")?, "Bob should be a user");
    assert!(rbac.has_role("charlie", "editor")?, "Charlie should be an editor");
    assert!(rbac.has_role("charlie", "user")?, "Charlie should also be a user");
    
    // Test role listing
    let alice_roles = rbac.get_user_roles("alice")?;
    assert_eq!(alice_roles.len(), 1, "Alice should have 1 role");
    assert!(alice_roles.contains(&"admin".to_string()), "Alice should be an admin");
    
    let charlie_roles = rbac.get_user_roles("charlie")?;
    assert_eq!(charlie_roles.len(), 2, "Charlie should have 2 roles");
    assert!(charlie_roles.contains(&"editor".to_string()), "Charlie should be an editor");
    assert!(charlie_roles.contains(&"user".to_string()), "Charlie should be a user");
    
    // Test permission checks
    assert!(rbac.has_permission("alice", "system:access", None)?, "Alice should have system access");
    assert!(rbac.has_permission("alice", "user:manage", None)?, "Alice should be able to manage users");
    assert!(!rbac.has_permission("alice", "content:write", None)?, "Alice should not be able to write content");
    
    assert!(rbac.has_permission("bob", "content:read", None)?, "Bob should be able to read content");
    assert!(!rbac.has_permission("bob", "content:write", None)?, "Bob should not be able to write content");
    
    assert!(rbac.has_permission("charlie", "content:read", None)?, "Charlie should be able to read content");
    assert!(rbac.has_permission("charlie", "content:write", None)?, "Charlie should be able to write content");
    assert!(!rbac.has_permission("charlie", "user:manage", None)?, "Charlie should not be able to manage users");
    
    // Test role revocation
    rbac.revoke_role("charlie", "editor")?;
    assert!(!rbac.has_role("charlie", "editor")?, "Charlie should no longer be an editor");
    assert!(!rbac.has_permission("charlie", "content:write", None)?, "Charlie should no longer be able to write content");
    assert!(rbac.has_permission("charlie", "content:read", None)?, "Charlie should still be able to read content");
    
    println!("✓ BasicRBACManager tests passed");
    
    Ok(())
}

// ----- Test Mock RBAC Manager ----- //

fn test_mock_rbac_manager() -> Result<()> {
    println!("\nTesting MockRBACManager...");
    
    // Test with allow_all = true
    let rbac = Arc::new(MockRBACManager::new(true));
    
    assert!(rbac.has_role("anyone", "anything")?, "Should allow any role");
    assert!(rbac.has_permission("anyone", "anything", None)?, "Should allow any permission");
    
    // Test with allow_all = false and explicit roles
    let rbac = Arc::new(MockRBACManager::new(false));
    
    assert!(!rbac.has_role("dave", "admin")?, "Dave should not have any roles initially");
    
    rbac.assign_role("dave", "admin")?;
    assert!(rbac.has_role("dave", "admin")?, "Dave should now be an admin");
    
    let dave_roles = rbac.get_user_roles("dave")?;
    assert_eq!(dave_roles.len(), 1, "Dave should have 1 role");
    assert!(dave_roles.contains(&"admin".to_string()), "Dave should be an admin");
    
    // Test with_user_roles method
    let mock_rbac = MockRBACManager::new(false);
    let mock_with_roles = mock_rbac.with_user_roles("eve", vec!["user".to_string(), "manager".to_string()]);
    let rbac = Arc::new(mock_with_roles);
    
    assert!(rbac.has_role("eve", "user")?, "Eve should be a user");
    assert!(rbac.has_role("eve", "manager")?, "Eve should be a manager");
    assert!(!rbac.has_role("eve", "admin")?, "Eve should not be an admin");
    
    let eve_roles = rbac.get_user_roles("eve")?;
    assert_eq!(eve_roles.len(), 2, "Eve should have 2 roles");
    
    // Test cloning
    let rbac_clone = rbac.clone();
    assert!(rbac_clone.has_role("eve", "user")?, "Eve should still be a user after cloning");
    assert!(rbac_clone.has_role("eve", "manager")?, "Eve should still be a manager after cloning");
    
    println!("✓ MockRBACManager tests passed");
    
    Ok(())
} 