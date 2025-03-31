//! Unit tests for the RBAC subsystem.

use tokio::test;
use std::sync::Arc;

use crate::error::Result;
use crate::context_manager::Context;
use crate::security::rbac::{
    RBACManager,
    BasicRBACManager,
    MockRBACManager,
};

#[test]
async fn test_basic_rbac_manager() -> Result<()> {
    // Create a new basic RBAC manager
    let rbac = BasicRBACManager::new();
    
    // Create a role with permissions
    let role_id = "admin".to_string();
    let role_name = "Administrator".to_string();
    let role_description = "System administrator role".to_string();
    let permissions = vec![
        "document:read".to_string(),
        "document:write".to_string(),
        "user:manage".to_string()
    ];
    
    rbac.create_role(&role_id, role_name, role_description).await?;
    
    // Add permissions to the role
    for permission in permissions.iter() {
        rbac.add_permission_to_role(&role_id, permission).await?;
    }
    
    // Assign role to a user
    rbac.assign_role("user123", &role_id).await?;
    
    // Test has_role
    let has_role = rbac.has_role("user123", &role_id).await?;
    assert!(has_role, "User should have the assigned role");
    
    // Test get_user_roles
    let user_roles = rbac.get_user_roles("user123").await?;
    assert_eq!(user_roles.len(), 1, "User should have exactly one role");
    assert_eq!(user_roles[0], role_id, "User's role should match the assigned role");
    
    // Test has_permission
    let has_read_permission = rbac.has_permission("user123", "document:read", None).await?;
    assert!(has_read_permission, "User should have read permission");
    
    let has_write_permission = rbac.has_permission("user123", "document:write", None).await?;
    assert!(has_write_permission, "User should have write permission");
    
    let has_admin_permission = rbac.has_permission("user123", "user:manage", None).await?;
    assert!(has_admin_permission, "User should have user management permission");
    
    let has_delete_permission = rbac.has_permission("user123", "document:delete", None).await?;
    assert!(!has_delete_permission, "User should not have delete permission");
    
    // Test permission for a user without any role
    let has_permission = rbac.has_permission("user456", "document:read", None).await?;
    assert!(!has_permission, "User without a role should not have permission");
    
    // Test revoke_role
    rbac.revoke_role("user123", &role_id).await?;
    let has_role_after_revoke = rbac.has_role("user123", &role_id).await?;
    assert!(!has_role_after_revoke, "User should not have the role after it's revoked");
    
    // Test permission after revoking role
    let has_permission_after_revoke = rbac.has_permission("user123", "document:read", None).await?;
    assert!(!has_permission_after_revoke, "User should not have permission after role is revoked");
    
    Ok(())
}

#[test]
async fn test_mock_rbac_manager() -> Result<()> {
    // Create a new mock RBAC manager with allow_all=true
    let rbac = MockRBACManager::new(true);
    
    // Test manager metadata
    assert_eq!(rbac.name(), "MockRBACManager");
    assert_eq!(rbac.version(), "1.0.0");
    
    // Test has_permission (should always return true)
    let has_permission = rbac.has_permission("user123", "document:read", None).await?;
    assert!(has_permission);
    
    // Create a mock RBAC manager with allow_all=false
    let rbac = MockRBACManager::new(false);
    
    // Test has_permission (should always return false)
    let has_permission = rbac.has_permission("user123", "document:read", None).await?;
    assert!(!has_permission);
    
    // Test with_user_roles configuration
    let rbac = MockRBACManager::new(false)
        .with_user_roles("user123", vec!["admin".to_string()]).await;
    
    // Test has_role
    let has_role = rbac.has_role("user123", "admin").await?;
    assert!(has_role);
    
    // Test get_user_roles
    let roles = rbac.get_user_roles("user123").await?;
    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0], "admin");
    
    Ok(())
}

#[test]
async fn test_role_management() -> Result<()> {
    // Create a new basic RBAC manager
    let rbac = BasicRBACManager::new();
    
    // Create roles with permissions
    rbac.create_role("user", "Basic User".to_string(), "Basic user role".to_string()).await?;
    rbac.create_role("editor", "Content Editor".to_string(), "Can edit content".to_string()).await?;
    rbac.create_role("viewer", "Content Viewer".to_string(), "Can view content".to_string()).await?;
    
    // Add permissions to roles
    rbac.add_permission_to_role("user", "account:manage").await?;
    rbac.add_permission_to_role("editor", "content:edit").await?;
    rbac.add_permission_to_role("editor", "content:view").await?;
    rbac.add_permission_to_role("viewer", "content:view").await?;
    
    // Test multiple role assignments
    let user_id = "user123";
    
    // Assign multiple roles
    rbac.assign_role(user_id, "user").await?;
    rbac.assign_role(user_id, "editor").await?;
    rbac.assign_role(user_id, "viewer").await?;
    
    // Check all roles are assigned
    let roles = rbac.get_user_roles(user_id).await?;
    assert_eq!(roles.len(), 3);
    assert!(roles.contains(&"user".to_string()));
    assert!(roles.contains(&"editor".to_string()));
    assert!(roles.contains(&"viewer".to_string()));
    
    // Test has_permission
    assert!(rbac.has_permission(user_id, "account:manage", None).await?);
    assert!(rbac.has_permission(user_id, "content:edit", None).await?);
    assert!(rbac.has_permission(user_id, "content:view", None).await?);
    
    // Revoke one role
    rbac.revoke_role(user_id, "editor").await?;
    
    // Check the remaining roles
    let roles = rbac.get_user_roles(user_id).await?;
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"user".to_string()));
    assert!(roles.contains(&"viewer".to_string()));
    assert!(!roles.contains(&"editor".to_string()));
    
    // Test has_role for each
    assert!(rbac.has_role(user_id, "user").await?);
    assert!(rbac.has_role(user_id, "viewer").await?);
    assert!(!rbac.has_role(user_id, "editor").await?);
    
    // Permissions after role revocation
    assert!(rbac.has_permission(user_id, "account:manage", None).await?);
    assert!(rbac.has_permission(user_id, "content:view", None).await?);
    assert!(!rbac.has_permission(user_id, "content:edit", None).await?);
    
    Ok(())
} 