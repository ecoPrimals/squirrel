//! Test RBAC role functionality independently

use std::collections::HashSet;
use crate::mcp::security::{
    RBACManager,
    Role,
    Permission,
    Action,
};

#[test]
fn test_simple_role_creation() {
    // Create RBAC manager
    let mut rbac = RBACManager::new();
    
    // Create a simple permission
    let permission = Permission {
        id: "test-permission".to_string(),
        name: "test".to_string(),
        resource: "document".to_string(),
        action: Action::Read,
    };
    
    // Create a role with the permission
    let mut permissions = HashSet::new();
    permissions.insert(permission);
    
    let result = rbac.create_role(
        "test-role".to_string(),
        None,
        permissions,
        HashSet::new(),
    );
    
    assert!(result.is_ok(), "Role creation should succeed");
    
    // Verify the role exists in the manager
    let role = rbac.get_role_by_name("test-role");
    assert!(role.is_some(), "Role should exist in the manager");
} 