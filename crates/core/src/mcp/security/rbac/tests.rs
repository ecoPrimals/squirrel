use super::*;

#[test]
fn test_rbac_manager_creation() {
    let rbac = RBACManager::new();
    assert!(rbac.roles.is_empty());
}

#[test]
fn test_rbac_role_creation() {
    let mut rbac = RBACManager::new();
    
    // Create permission
    let permission = Permission {
        id: "perm-1".to_string(),
        name: "Read".to_string(),
        resource: "Document".to_string(),
        action: Action::Read,
    };
    
    // Create role with permission
    let role_id = rbac.create_role("test-role", "Test Role", vec![permission]);
    
    // Verify role was created
    assert!(!role_id.is_empty());
    
    // Verify role can be retrieved
    let found_role = rbac.get_role_by_name("test-role");
    assert!(found_role.is_some());
    
    let role = found_role.unwrap();
    assert_eq!(role.name, "test-role");
    assert_eq!(role.display_name, "Test Role");
    assert_eq!(role.permissions.len(), 1);
}

#[test]
fn test_permission_check() {
    let mut rbac = RBACManager::new();
    
    // Create read permission
    let read_permission = Permission {
        id: "perm-read".to_string(),
        name: "Read".to_string(),
        resource: "Document".to_string(),
        action: Action::Read,
    };
    
    // Create write permission
    let write_permission = Permission {
        id: "perm-write".to_string(),
        name: "Write".to_string(),
        resource: "Document".to_string(),
        action: Action::Write,
    };
    
    // Create role with read permission only
    let role_id = rbac.create_role("reader", "Reader", vec![read_permission]);
    
    // Verify role has read permission but not write
    let role = rbac.get_role_by_id(&role_id).unwrap();
    assert!(rbac.role_has_permission(&role, "Document", Action::Read));
    assert!(!rbac.role_has_permission(&role, "Document", Action::Write));
}

#[test]
fn test_role_inheritance() {
    let mut rbac = RBACManager::new();
    
    // Create permissions
    let read_permission = Permission {
        id: "perm-read".to_string(),
        name: "Read".to_string(),
        resource: "Document".to_string(),
        action: Action::Read,
    };
    
    let write_permission = Permission {
        id: "perm-write".to_string(),
        name: "Write".to_string(),
        resource: "Document".to_string(),
        action: Action::Write,
    };
    
    let admin_permission = Permission {
        id: "perm-admin".to_string(),
        name: "Admin".to_string(),
        resource: "System".to_string(),
        action: Action::Admin,
    };
    
    // Create reader role
    let reader_id = rbac.create_role("reader", "Reader", vec![read_permission]);
    
    // Create editor role that inherits from reader
    let editor_id = rbac.create_role_with_parent(
        "editor", 
        "Editor", 
        vec![write_permission], 
        vec![reader_id.clone()]
    );
    
    // Create admin role that inherits from editor
    let admin_id = rbac.create_role_with_parent(
        "admin", 
        "Admin", 
        vec![admin_permission], 
        vec![editor_id.clone()]
    );
    
    // Get admin role
    let admin_role = rbac.get_role_by_id(&admin_id).unwrap();
    
    // Admin should have all permissions
    assert!(rbac.role_has_permission(&admin_role, "Document", Action::Read));
    assert!(rbac.role_has_permission(&admin_role, "Document", Action::Write));
    assert!(rbac.role_has_permission(&admin_role, "System", Action::Admin));
    
    // Editor should have read and write but not admin
    let editor_role = rbac.get_role_by_id(&editor_id).unwrap();
    assert!(rbac.role_has_permission(&editor_role, "Document", Action::Read));
    assert!(rbac.role_has_permission(&editor_role, "Document", Action::Write));
    assert!(!rbac.role_has_permission(&editor_role, "System", Action::Admin));
    
    // Reader should only have read
    let reader_role = rbac.get_role_by_id(&reader_id).unwrap();
    assert!(rbac.role_has_permission(&reader_role, "Document", Action::Read));
    assert!(!rbac.role_has_permission(&reader_role, "Document", Action::Write));
    assert!(!rbac.role_has_permission(&reader_role, "System", Action::Admin));
} 