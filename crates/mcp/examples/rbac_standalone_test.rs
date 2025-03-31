use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Simplified error type for testing
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// The unified RBAC trait that matches our implementation
#[async_trait::async_trait]
trait RBACManager: Send + Sync + 'static {
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool>;
}

// A simplified BasicRBACManager implementation
struct BasicRBACManager {
    // Map of user IDs to the roles they have
    user_roles: RwLock<HashMap<String, Vec<String>>>,
}

impl BasicRBACManager {
    fn new() -> Self {
        Self {
            user_roles: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl RBACManager for BasicRBACManager {
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        
        let roles = user_roles.entry(user_id.to_string())
            .or_insert_with(Vec::new);
        
        if !roles.contains(&role_id.to_string()) {
            roles.push(role_id.to_string());
        }
        
        Ok(())
    }

    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.retain(|r| r != role_id);
        }
        
        Ok(())
    }

    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.clone()),
            None => Ok(Vec::new()),
        }
    }

    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.user_roles.read().await;
        
        match user_roles.get(user_id) {
            Some(roles) => Ok(roles.contains(&role_id.to_string())),
            None => Ok(false),
        }
    }
}

// A simplified MockRBACManager implementation
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
        
        let roles = user_roles.entry(user_id.to_string())
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
            // Return a dummy role list when allow_all is true
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
}

// Main function with tests
#[tokio::main]
async fn main() {
    println!("RBAC Standalone Implementation Test");
    println!("==================================");
    
    // Run the tests
    test_basic_rbac_manager().await;
    test_mock_rbac_manager().await;
    
    println!("\nAll tests completed successfully!");
}

async fn test_basic_rbac_manager() {
    println!("\nTesting BasicRBACManager...");
    
    // Create a new BasicRBACManager
    let rbac_manager = Arc::new(BasicRBACManager::new());
    
    // Test assigning and checking roles
    let user_id = "test_user_1";
    let role_id = "admin";
    
    // Assign a role to the user
    rbac_manager.assign_role(user_id, role_id).await.unwrap();
    
    // Check if the user has the role
    let has_role = rbac_manager.has_role(user_id, role_id).await.unwrap();
    assert!(has_role, "User should have the assigned role");
    println!("✓ Role assignment and checking works");
    
    // Get all roles for the user
    let roles = rbac_manager.get_user_roles(user_id).await.unwrap();
    assert_eq!(roles.len(), 1, "User should have exactly one role");
    assert_eq!(roles[0], role_id, "The role should be the one we assigned");
    println!("✓ Get user roles works");
    
    // Revoke the role
    rbac_manager.revoke_role(user_id, role_id).await.unwrap();
    
    // Check that the role was revoked
    let has_role = rbac_manager.has_role(user_id, role_id).await.unwrap();
    assert!(!has_role, "User should not have the role after it was revoked");
    println!("✓ Role revocation works");
    
    // Test with multiple roles
    let roles = vec!["user", "moderator", "admin"];
    
    // Assign multiple roles
    for role in &roles {
        rbac_manager.assign_role(user_id, role).await.unwrap();
    }
    
    // Check that all roles were assigned
    for role in &roles {
        let has_role = rbac_manager.has_role(user_id, role).await.unwrap();
        assert!(has_role, "User should have the assigned role: {}", role);
    }
    
    // Get all roles and verify
    let user_roles = rbac_manager.get_user_roles(user_id).await.unwrap();
    assert_eq!(user_roles.len(), roles.len(), "User should have all assigned roles");
    for role in &roles {
        assert!(user_roles.contains(&role.to_string()), "User roles should contain: {}", role);
    }
    println!("✓ Multiple role assignment works");
    
    println!("BasicRBACManager tests passed!");
}

async fn test_mock_rbac_manager() {
    println!("\nTesting MockRBACManager...");
    
    // Test with allow_all = true
    let mock_rbac_manager = Arc::new(MockRBACManager::new(true));
    
    // Even without assigning any roles, the user should have all roles
    let has_role = mock_rbac_manager.has_role("any_user", "any_role").await.unwrap();
    assert!(has_role, "With allow_all=true, any user should have any role");
    println!("✓ allow_all=true works correctly");
    
    // Test with specific roles
    let mock_rbac_manager = Arc::new(MockRBACManager::new(false));
    
    // Setup test data
    let user_id = "test_user_2";
    let role_id = "editor";
    
    // Initially, the user should not have any roles
    let has_role = mock_rbac_manager.has_role(user_id, role_id).await.unwrap();
    assert!(!has_role, "User should not have any roles initially");
    
    // Assign a role
    mock_rbac_manager.assign_role(user_id, role_id).await.unwrap();
    
    // Check if the user has the role
    let has_role = mock_rbac_manager.has_role(user_id, role_id).await.unwrap();
    assert!(has_role, "User should have the assigned role");
    
    // Check a role the user doesn't have
    let has_role = mock_rbac_manager.has_role(user_id, "non_existent_role").await.unwrap();
    assert!(!has_role, "User should not have unassigned roles");
    println!("✓ Role assignment and checking works");
    
    // Test with_user_roles method
    let test_roles = vec!["role1".to_string(), "role2".to_string()];
    let mock_manager = MockRBACManager::new(false);
    let mock_manager = mock_manager.with_user_roles(user_id, test_roles.clone()).await;
    let mock_rbac_manager = Arc::new(mock_manager);
    
    // Check if the user has the roles set with with_user_roles
    for role in &test_roles {
        let has_role = mock_rbac_manager.has_role(user_id, role).await.unwrap();
        assert!(has_role, "User should have the role set with with_user_roles: {}", role);
    }
    println!("✓ with_user_roles method works");
    
    // Test cloning
    let mock_manager = mock_rbac_manager.as_ref().clone();
    let cloned_manager = Arc::new(mock_manager);
    
    // The cloned manager should have the same user/role data
    for role in &test_roles {
        let has_role = cloned_manager.has_role(user_id, role).await.unwrap();
        assert!(has_role, "Cloned manager should have the same role data: {}", role);
    }
    println!("✓ Cloning works correctly");
    
    println!("MockRBACManager tests passed!");
} 