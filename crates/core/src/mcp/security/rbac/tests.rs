use super::*;

#[test]
fn test_rbac_manager_creation() {
    let rbac = RBACManager::new();
    assert!(rbac.roles_by_id.is_empty());
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
    
    // Create HashSet for permissions
    let mut permissions = HashSet::new();
    permissions.insert(permission);
    
    // Create role with permission
    let role = rbac.create_role(
        "test-role".to_string(), 
        Some("Test Role".to_string()), 
        permissions,
        HashSet::new()
    ).unwrap();
    
    // Verify role was created
    assert!(!role.id.is_empty());
    
    // Verify role can be retrieved
    let found_role = rbac.get_role_by_name("test-role");
    assert!(found_role.is_some(), "Role should be found by name");
    
    let role = found_role.unwrap();
    assert_eq!(role.name, "test-role");
    assert_eq!(role.description, Some("Test Role".to_string()));
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
        action: Action::Update,
    };
    
    // Create role with read permission only
    let mut permissions = HashSet::new();
    permissions.insert(read_permission);
    
    let role_id = rbac.create_role(
        "reader".to_string(),
        Some("Reader".to_string()),
        permissions,
        HashSet::new()
    ).unwrap().id;
    
    // Verify role has read permission but not write
    let role = rbac.get_role_by_id(&role_id).unwrap();
    assert!(rbac.has_permission_for_role(&role, "Document", Action::Read));
    assert!(!rbac.has_permission_for_role(&role, "Document", Action::Update));
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
        action: Action::Update,
    };
    
    let admin_permission = Permission {
        id: "perm-admin".to_string(),
        name: "Admin".to_string(),
        resource: "System".to_string(),
        action: Action::Admin,
    };
    
    // Create reader role with read permission
    let mut read_permissions = HashSet::new();
    read_permissions.insert(read_permission);
    
    let reader_role = rbac.create_role(
        "reader".to_string(),
        Some("Reader".to_string()),
        read_permissions,
        HashSet::new()
    ).unwrap();
    
    // Create editor role that inherits from reader
    let mut write_permissions = HashSet::new();
    write_permissions.insert(write_permission);
    
    let mut editor_parent_roles = HashSet::new();
    editor_parent_roles.insert(reader_role.id.clone());
    
    let editor_role = rbac.create_role(
        "editor".to_string(),
        Some("Editor".to_string()),
        write_permissions,
        editor_parent_roles
    ).unwrap();
    
    // Create admin role that inherits from editor
    let mut admin_permissions = HashSet::new();
    admin_permissions.insert(admin_permission);
    
    let mut admin_parent_roles = HashSet::new();
    admin_parent_roles.insert(editor_role.id.clone());
    
    let admin_role = rbac.create_role(
        "admin".to_string(),
        Some("Admin".to_string()),
        admin_permissions,
        admin_parent_roles
    ).unwrap();
    
    // Admin should have all permissions
    assert!(rbac.has_permission_for_role(&admin_role, "Document", Action::Read));
    assert!(rbac.has_permission_for_role(&admin_role, "Document", Action::Update));
    assert!(rbac.has_permission_for_role(&admin_role, "System", Action::Admin));
    
    // Editor should have read and write but not admin
    assert!(rbac.has_permission_for_role(&editor_role, "Document", Action::Read));
    assert!(rbac.has_permission_for_role(&editor_role, "Document", Action::Update));
    assert!(!rbac.has_permission_for_role(&editor_role, "System", Action::Admin));
    
    // Reader should only have read
    assert!(rbac.has_permission_for_role(&reader_role, "Document", Action::Read));
    assert!(!rbac.has_permission_for_role(&reader_role, "Document", Action::Update));
    assert!(!rbac.has_permission_for_role(&reader_role, "System", Action::Admin));
} 