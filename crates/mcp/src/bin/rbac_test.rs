use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

// Define our own Result type for the test
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

#[async_trait::async_trait]
trait RBACManager: Send + Sync + 'static {
    // ---- Core Role Management ---- //
    
    /// Assign a role to a user
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Revoke a role from a user
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Get all roles assigned to a user
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    
    /// Check if a user has a specific role
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool>;
    
    // ---- Permission Management ---- //
    
    /// Check if a user has a specific permission (with optional context)
    async fn has_permission(
        &self, 
        user_id: &str, 
        permission: &str, 
        context: Option<&Context>
    ) -> Result<bool> {
        // Default implementation - check if any of the user's roles have this permission
        let roles = self.get_user_roles(user_id).await?;
        
        for role_id in roles {
            let role_details = self.get_role_details(&role_id).await?;
            
            if let Some(role) = role_details {
                if role.permissions.contains(permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    // ---- Role Details ---- //
    
    /// Get details about a specific role
    async fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>>;
    
    /// Get all permissions for a specific role
    async fn get_permissions_for_role(&self, role_id: &str) -> Result<Vec<String>>;
    
    // ---- Role Creation and Management ---- //
    
    /// Create a new role
    async fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()>;
    
    /// Add a permission to a role
    async fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()>;
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
    roles: RwLock<HashMap<String, Role>>,
    
    // Map of user IDs to the set of role IDs they have
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl BasicRBACManager {
    fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }
    
    // Internal helper to get a role (not part of the trait)
    async fn get_role(&self, role_id: &str) -> Option<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id).cloned()
    }
}

#[async_trait::async_trait]
impl RBACManager for BasicRBACManager {
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify the role exists
        if self.get_role(role_id).await.is_none() {
            return Err(format!("Role {} does not exist", role_id).into());
        }
        
        let mut user_roles = self.user_roles.write().await;
        
        // Get or create the set of roles for this user
        let roles = user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new);
        
        // Add the role
        roles.insert(role_id.to_string());
        
        Ok(())
    }
    
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        
        Ok(())
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.iter().cloned().collect()),
            None => Ok(Vec::new()),
        }
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.contains(role_id)),
            None => Ok(false),
        }
    }
    
    async fn create_role(&self, role_id: &str, name: &str, description: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if roles.contains_key(role_id) {
            return Err(format!("Role {} already exists", role_id).into());
        }
        
        roles.insert(
            role_id.to_string(),
            Role::new(role_id, name, description),
        );
        
        Ok(())
    }
    
    async fn add_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        match roles.get_mut(role_id) {
            Some(role) => {
                role.add_permission(permission);
                Ok(())
            }
            None => Err(format!("Role {} does not exist", role_id).into()),
        }
    }
    
    async fn get_role_details(&self, role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        let role = self.get_role(role_id).await;
        
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
    
    async fn get_permissions_for_role(&self, role_id: &str) -> Result<Vec<String>> {
        let role = self.get_role(role_id).await;
        
        match role {
            Some(role) => Ok(role.permissions.into_iter().collect()),
            None => Err(format!("Role {} does not exist", role_id).into()),
        }
    }
    
    async fn has_permission(
        &self, 
        user_id: &str, 
        permission: &str, 
        _context: Option<&Context>
    ) -> Result<bool> {
        let roles = self.get_user_roles(user_id).await?;
        
        for role_id in roles {
            let role = self.get_role(&role_id).await;
            
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
    user_roles: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MockRBACManager {
    fn new(allow_all: bool) -> Self {
        Self {
            allow_all,
            user_roles: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // Set specific roles for a user
    async fn with_user_roles(self, user_id: &str, roles: Vec<String>) -> Self {
        let mut user_roles = self.user_roles.write().await;
        user_roles.insert(user_id.to_string(), roles);
        self
    }
}

#[async_trait::async_trait]
impl RBACManager for MockRBACManager {
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        if self.allow_all {
            return Ok(());
        }
        
        let mut user_roles = self.user_roles.write().await;
        
        let roles = user_roles
            .entry(user_id.to_string())
            .or_insert_with(Vec::new);
        
        if !roles.contains(&role_id.to_string()) {
            roles.push(role_id.to_string());
        }
        
        Ok(())
    }
    
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        if self.allow_all {
            return Ok(());
        }
        
        let mut user_roles = self.user_roles.write().await;
        
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.retain(|r| r != role_id);
        }
        
        Ok(())
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        if self.allow_all {
            // Return a special wildcard role when all permissions are allowed
            return Ok(vec!["*".to_string()]);
        }
        
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.clone()),
            None => Ok(Vec::new()),
        }
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        if self.allow_all {
            return Ok(true);
        }
        
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.contains(&role_id.to_string())),
            None => Ok(false),
        }
    }
    
    async fn has_permission(
        &self, 
        user_id: &str, 
        _permission: &str, 
        _context: Option<&Context>
    ) -> Result<bool> {
        if self.allow_all {
            return Ok(true);
        }
        
        // In this mock, we'll just check if the user has any roles
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(!roles.is_empty()),
            None => Ok(false),
        }
    }
}

// ----- Main Test Function ----- //

#[tokio::main]
async fn main() -> Result<()> {
    println!("RBAC Independent Test");
    println!("=====================");
    
    // Test the BasicRBACManager
    test_basic_rbac_manager().await?;
    
    // Test the MockRBACManager
    test_mock_rbac_manager().await?;
    
    println!("\nAll tests passed successfully!");
    
    Ok(())
}

// ----- Test Basic RBAC Manager ----- //

async fn test_basic_rbac_manager() -> Result<()> {
    println!("\nTesting BasicRBACManager...");
    
    let rbac = Arc::new(BasicRBACManager::new());
    
    // Create roles
    rbac.create_role("admin", "Administrator", "System administrator").await?;
    rbac.create_role("user", "Regular User", "Basic user permissions").await?;
    rbac.create_role("editor", "Editor", "Can edit content").await?;
    
    // Add permissions to roles
    rbac.add_permission_to_role("admin", "system:access").await?;
    rbac.add_permission_to_role("admin", "user:manage").await?;
    rbac.add_permission_to_role("user", "content:read").await?;
    rbac.add_permission_to_role("editor", "content:read").await?;
    rbac.add_permission_to_role("editor", "content:write").await?;
    
    // Test assigning roles
    rbac.assign_role("alice", "admin").await?;
    rbac.assign_role("bob", "user").await?;
    rbac.assign_role("charlie", "editor").await?;
    rbac.assign_role("charlie", "user").await?;
    
    // Test role checks
    assert!(rbac.has_role("alice", "admin").await?, "Alice should be an admin");
    assert!(!rbac.has_role("alice", "user").await?, "Alice should not be a user");
    assert!(rbac.has_role("bob", "user").await?, "Bob should be a user");
    assert!(rbac.has_role("charlie", "editor").await?, "Charlie should be an editor");
    assert!(rbac.has_role("charlie", "user").await?, "Charlie should also be a user");
    
    // Test role listing
    let alice_roles = rbac.get_user_roles("alice").await?;
    assert_eq!(alice_roles.len(), 1, "Alice should have 1 role");
    assert!(alice_roles.contains(&"admin".to_string()), "Alice should be an admin");
    
    let charlie_roles = rbac.get_user_roles("charlie").await?;
    assert_eq!(charlie_roles.len(), 2, "Charlie should have 2 roles");
    assert!(charlie_roles.contains(&"editor".to_string()), "Charlie should be an editor");
    assert!(charlie_roles.contains(&"user".to_string()), "Charlie should be a user");
    
    // Test permission checks
    assert!(rbac.has_permission("alice", "system:access", None).await?, "Alice should have system access");
    assert!(rbac.has_permission("alice", "user:manage", None).await?, "Alice should be able to manage users");
    assert!(!rbac.has_permission("alice", "content:write", None).await?, "Alice should not be able to write content");
    
    assert!(rbac.has_permission("bob", "content:read", None).await?, "Bob should be able to read content");
    assert!(!rbac.has_permission("bob", "content:write", None).await?, "Bob should not be able to write content");
    
    assert!(rbac.has_permission("charlie", "content:read", None).await?, "Charlie should be able to read content");
    assert!(rbac.has_permission("charlie", "content:write", None).await?, "Charlie should be able to write content");
    assert!(!rbac.has_permission("charlie", "user:manage", None).await?, "Charlie should not be able to manage users");
    
    // Test role revocation
    rbac.revoke_role("charlie", "editor").await?;
    assert!(!rbac.has_role("charlie", "editor").await?, "Charlie should no longer be an editor");
    assert!(!rbac.has_permission("charlie", "content:write", None).await?, "Charlie should no longer be able to write content");
    assert!(rbac.has_permission("charlie", "content:read", None).await?, "Charlie should still be able to read content");
    
    println!("✓ BasicRBACManager tests passed");
    
    Ok(())
}

// ----- Test Mock RBAC Manager ----- //

async fn test_mock_rbac_manager() -> Result<()> {
    println!("\nTesting MockRBACManager...");
    
    // Test with allow_all = true
    let rbac = Arc::new(MockRBACManager::new(true));
    
    assert!(rbac.has_role("anyone", "anything").await?, "Should allow any role");
    assert!(rbac.has_permission("anyone", "anything", None).await?, "Should allow any permission");
    
    // Test with allow_all = false and explicit roles
    let rbac = Arc::new(MockRBACManager::new(false));
    
    assert!(!rbac.has_role("dave", "admin").await?, "Dave should not have any roles initially");
    
    rbac.assign_role("dave", "admin").await?;
    assert!(rbac.has_role("dave", "admin").await?, "Dave should now be an admin");
    
    let dave_roles = rbac.get_user_roles("dave").await?;
    assert_eq!(dave_roles.len(), 1, "Dave should have 1 role");
    assert!(dave_roles.contains(&"admin".to_string()), "Dave should be an admin");
    
    // Test with_user_roles method
    let mock_rbac = MockRBACManager::new(false);
    let mock_with_roles = mock_rbac.with_user_roles("eve", vec!["user".to_string(), "manager".to_string()]).await;
    let rbac = Arc::new(mock_with_roles);
    
    assert!(rbac.has_role("eve", "user").await?, "Eve should be a user");
    assert!(rbac.has_role("eve", "manager").await?, "Eve should be a manager");
    assert!(!rbac.has_role("eve", "admin").await?, "Eve should not be an admin");
    
    let eve_roles = rbac.get_user_roles("eve").await?;
    assert_eq!(eve_roles.len(), 2, "Eve should have 2 roles");
    
    // Test cloning
    let rbac_clone = rbac.clone();
    assert!(rbac_clone.has_role("eve", "user").await?, "Eve should still be a user after cloning");
    assert!(rbac_clone.has_role("eve", "manager").await?, "Eve should still be a manager after cloning");
    
    println!("✓ MockRBACManager tests passed");
    
    Ok(())
} 