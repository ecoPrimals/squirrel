use super::*;

#[test]
fn test_role_creation() {
    // ARRANGE: Create RBAC manager and test data
    let mut rbac = create_test_rbac_manager();
    
    // Create permissions
    let mut permissions = HashSet::new();
    permissions.insert(create_test_permission("read", "document", Action::Read));
    
    // ACT: Create a role
    let result = rbac.create_role(
        "reader".to_string(),
        Some("Read-only role".to_string()),
        permissions.clone(),
        HashSet::new(),
    );
    
    // ASSERT: Verify role creation
    assert!(result.is_ok(), "Role creation should succeed");
    let role = result.unwrap();
    
    // Verify role properties
    assert_eq!(role.name, "reader", "Role should have correct name");
    assert_eq!(role.description, Some("Read-only role".to_string()), "Role should have correct description");
    assert_eq!(role.permissions.len(), 1, "Role should have one permission");
    assert_eq!(role.parent_roles.len(), 0, "Role should have no parent roles");
    
    // Verify permission is correctly added
    let perm = role.permissions.iter().next().unwrap();
    assert_eq!(perm.action, Action::Read, "Permission should have Read action");
    assert_eq!(perm.resource, "document", "Permission should be for document resource");
}

#[test]
fn test_role_inheritance() {
    // ARRANGE: Create RBAC manager
    let mut rbac = create_test_rbac_manager();
    
    // Create base role with read permission
    let mut base_permissions = HashSet::new();
    base_permissions.insert(create_test_permission("read", "document", Action::Read));
    
    let base_role = rbac.create_role(
        "reader".to_string(),
        None,
        base_permissions,
        HashSet::new(),
    ).expect("Failed to create base role");
    
    // Create admin permissions
    let mut admin_permissions = HashSet::new();
    admin_permissions.insert(create_test_permission("write", "document", Action::Create));
    
    // Set up parent roles
    let mut parent_roles = HashSet::new();
    parent_roles.insert(base_role.id.clone());
    
    // ACT: Create admin role inheriting from base role
    let admin_role = rbac.create_role(
        "admin".to_string(),
        None,
        admin_permissions,
        parent_roles,
    ).expect("Failed to create admin role");
    
    // Assign admin role to user
    let user_id = "test_user";
    rbac.assign_role(user_id.to_string(), admin_role.id.clone())
        .expect("Failed to assign role");
    
    // ACT: Get user permissions
    let user_permissions = rbac.get_user_permissions(user_id);
    
    // ASSERT: Verify permissions are inherited
    assert_eq!(user_permissions.len(), 2, "User should have permissions from both roles");
    
    // Verify specific permissions
    let has_read = user_permissions.iter().any(|p| p.action == Action::Read && p.resource == "document");
    let has_write = user_permissions.iter().any(|p| p.action == Action::Create && p.resource == "document");
    
    assert!(has_read, "User should have read permission from parent role");
    assert!(has_write, "User should have write permission from assigned role");
}

#[test]
fn test_role_lookup() {
    // ARRANGE: Create RBAC manager and a role
    let mut rbac = create_test_rbac_manager();
    
    let role = rbac.create_role(
        "tester".to_string(),
        None,
        HashSet::new(),
        HashSet::new(),
    ).expect("Failed to create role");
    
    // ACT & ASSERT: Look up by ID
    let by_id = rbac.get_role_by_id(&role.id);
    assert!(by_id.is_some(), "Should find role by ID");
    assert_eq!(by_id.unwrap().name, "tester", "Role should have correct name");
    
    // ACT & ASSERT: Look up by name
    let by_name = rbac.get_role_by_name("tester");
    assert!(by_name.is_some(), "Should find role by name");
    assert_eq!(by_name.unwrap().id, role.id, "Role should have correct ID");
    
    // ACT & ASSERT: Look up with generic method
    assert!(rbac.get_role(&role.id).is_some(), "Should find role by ID using generic method");
    assert!(rbac.get_role("tester").is_some(), "Should find role by name using generic method");
    assert!(rbac.get_role("nonexistent").is_none(), "Should not find nonexistent role");
}

#[test]
fn test_permission_check() {
    // ARRANGE: Create RBAC manager with a role and permission
    let mut rbac = create_test_rbac_manager();
    
    // Create role with read permission
    let mut permissions = HashSet::new();
    let read_permission = create_test_permission("read_doc", "document", Action::Read);
    permissions.insert(read_permission.clone());
    
    let role = rbac.create_role(
        "reader".to_string(),
        None,
        permissions,
        HashSet::new(),
    ).expect("Failed to create role");
    
    // Assign role to user
    let user_id = "test_user";
    rbac.assign_role(user_id.to_string(), role.id.clone())
        .expect("Failed to assign role");
    
    // ACT & ASSERT: Test permission checks
    assert!(rbac.has_permission(user_id, &read_permission), 
        "User should have explicitly assigned permission");
    
    // Test for permission user doesn't have
    let write_permission = create_test_permission("write_doc", "document", Action::Create);
    assert!(!rbac.has_permission(user_id, &write_permission), 
        "User should not have unassigned permission");
}

#[test]
fn test_assign_role_by_name() {
    // ARRANGE: Create RBAC manager and a role
    let mut rbac = create_test_rbac_manager();
    
    let role = rbac.create_role(
        "editor".to_string(),
        None,
        HashSet::new(),
        HashSet::new(),
    ).expect("Failed to create role");
    
    // ACT: Assign role by name
    let user_id = "test_user";
    let result = rbac.assign_role_by_name(user_id.to_string(), "editor".to_string());
    
    // ASSERT: Verify assignment
    assert!(result.is_ok(), "Role assignment by name should succeed");
    
    let user_roles = rbac.get_user_roles(user_id);
    assert_eq!(user_roles.len(), 1, "User should have one role");
    assert_eq!(user_roles[0].id, role.id, "User should have the correct role");
}

#[test]
fn test_role_with_predefined_id() {
    // ARRANGE: Create RBAC manager
    let mut rbac = create_test_rbac_manager();
    
    // ACT: Create role with predefined ID
    let test_id = "test-role-123";
    let result = rbac.create_role_with_id(
        test_id.to_string(),
        "tester".to_string(),
        None,
        HashSet::new(),
        HashSet::new(),
    );
    
    // ASSERT: Verify role creation
    assert!(result.is_ok(), "Role creation with predefined ID should succeed");
    let role = result.unwrap();
    
    assert_eq!(role.id, test_id, "Role should have the specified ID");
    
    // Verify lookup works
    let found_role = rbac.get_role_by_id(test_id);
    assert!(found_role.is_some(), "Should find role by predefined ID");
} 