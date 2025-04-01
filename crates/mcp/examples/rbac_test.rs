use std::sync::Arc;
use squirrel_mcp::security::rbac::{BasicRBACManager, MockRBACManager, RBACManager};
use tokio::runtime::Runtime;

// Define our own Result type for cleaner error handling
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    println!("RBAC Implementation Test");
    println!("=======================");
    
    // Create a new tokio runtime for our tests
    let rt = Runtime::new()?;
    
    // Run the tests
    rt.block_on(async {
        test_basic_rbac_manager().await;
        test_mock_rbac_manager().await;
    });
    
    println!("\nAll tests completed successfully!");
    Ok(())
}

async fn test_basic_rbac_manager() {
    println!("\nTesting BasicRBACManager...");
    
    // Create a new BasicRBACManager
    let rbac_manager = Arc::new(BasicRBACManager::new());
    
    // Test assigning and checking roles
    let user_id = "test_user_1";
    let role_id = "admin";
    
    // Create the role first
    rbac_manager.create_role(role_id, "Administrator", "System Administrator")
        .await
        .expect("Failed to create role");
    
    // Assign a role to the user
    rbac_manager.assign_role(user_id, role_id)
        .await
        .expect("Failed to assign role");
    
    // Check if the user has the role
    let has_role = rbac_manager.has_role(user_id, role_id)
        .await
        .expect("Failed to check role");
    assert!(has_role, "User should have the assigned role");
    println!("✓ Role assignment and checking works");
    
    // Get all roles for the user
    let roles = rbac_manager.get_user_roles(user_id)
        .await
        .expect("Failed to get user roles");
    assert_eq!(roles.len(), 1, "User should have exactly one role");
    assert_eq!(roles[0], role_id, "The role should be the one we assigned");
    println!("✓ Get user roles works");
    
    // Revoke the role
    rbac_manager.revoke_role(user_id, role_id)
        .await
        .expect("Failed to revoke role");
    
    // Check that the role was revoked
    let has_role = rbac_manager.has_role(user_id, role_id)
        .await
        .expect("Failed to check role after revocation");
    assert!(!has_role, "User should not have the role after it was revoked");
    println!("✓ Role revocation works");
    
    // Test with multiple roles
    let roles = vec!["user", "moderator", "admin"];
    
    // Create and assign multiple roles
    for role in &roles {
        if *role != "admin" { // admin role was already created
            rbac_manager.create_role(role, &format!("{} Role", role), &format!("Description for {}", role))
                .await
                .expect(&format!("Failed to create role {}", role));
        }
        
        rbac_manager.assign_role(user_id, role)
            .await
            .expect(&format!("Failed to assign role {}", role));
    }
    
    // Check that all roles were assigned
    for role in &roles {
        let has_role = rbac_manager.has_role(user_id, role)
            .await
            .expect(&format!("Failed to check role {}", role));
        assert!(has_role, "User should have the assigned role: {}", role);
    }
    
    // Get all roles and verify
    let user_roles = rbac_manager.get_user_roles(user_id)
        .await
        .expect("Failed to get user roles");
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
    let has_role = mock_rbac_manager.has_role("any_user", "any_role")
        .await
        .expect("Failed to check role with allow_all=true");
    assert!(has_role, "With allow_all=true, any user should have any role");
    println!("✓ allow_all=true works correctly");
    
    // Test with specific roles
    let mock_rbac_manager = Arc::new(MockRBACManager::new(false));
    
    // Setup test data
    let user_id = "test_user_2";
    let role_id = "editor";
    
    // Initially, the user should not have any roles
    let has_role = mock_rbac_manager.has_role(user_id, role_id)
        .await
        .expect("Failed to check role initially");
    assert!(!has_role, "User should not have any roles initially");
    
    // Assign a role - for MockRBACManager we don't need to create it first
    mock_rbac_manager.assign_role(user_id, role_id)
        .await
        .expect("Failed to assign role");
    
    // Check if the user has the role
    let has_role = mock_rbac_manager.has_role(user_id, role_id)
        .await
        .expect("Failed to check role after assignment");
    assert!(has_role, "User should have the assigned role");
    
    // Check a role the user doesn't have
    let has_role = mock_rbac_manager.has_role(user_id, "non_existent_role")
        .await
        .expect("Failed to check non-existent role");
    assert!(!has_role, "User should not have unassigned roles");
    println!("✓ Role assignment and checking works");
    
    // Test with_user_roles method
    let test_roles = vec!["role1".to_string(), "role2".to_string()];
    mock_rbac_manager.with_user_roles(user_id, test_roles.clone())
        .await
        .expect("Failed to call with_user_roles");
    
    // Check if the user has the roles set with with_user_roles
    for role in &test_roles {
        let has_role = mock_rbac_manager.has_role(user_id, role)
            .await
            .expect(&format!("Failed to check role {} after with_user_roles", role));
        assert!(has_role, "User should have the role set with with_user_roles: {}", role);
    }
    println!("✓ with_user_roles method works");
    
    // Test cloning
    let cloned_manager = mock_rbac_manager.clone();
    
    // The cloned manager should have the same user/role data
    for role in &test_roles {
        let has_role = cloned_manager.has_role(user_id, role)
            .await
            .expect(&format!("Failed to check role {} in cloned manager", role));
        assert!(has_role, "Cloned manager should have the same role data: {}", role);
    }
    println!("✓ Cloning works correctly");
    
    println!("MockRBACManager tests passed!");
} 