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
        id: "test:read".to_string(),
        name: "Test Read".to_string(),
        resource: "test".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    // Create HashSet for permissions
    let mut permissions = HashSet::new();
    permissions.insert(permission);

    // Create role with permission
    let role = rbac
        .create_role(
            "test-role".to_string(),
            Some("Test Role".to_string()),
            permissions,
            HashSet::new(),
        )
        .unwrap();

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
        id: "document:read".to_string(),
        name: "Read Document".to_string(),
        resource: "document".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    // Create write permission
    let _write_permission = Permission {
        id: "document:update".to_string(),
        name: "Update Document".to_string(),
        resource: "document".to_string(),
        action: Action::Update,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    // Create role with read permission only
    let mut permissions = HashSet::new();
    permissions.insert(read_permission);

    let role_id = rbac
        .create_role(
            "reader".to_string(),
            Some("Reader".to_string()),
            permissions,
            HashSet::new(),
        )
        .unwrap()
        .id;

    // Verify role has read permission but not write
    let role = rbac.get_role_by_id(&role_id).unwrap();
    assert!(rbac.has_permission_for_role(role, "document", Action::Read));
    assert!(!rbac.has_permission_for_role(role, "document", Action::Update));
}

#[test]
fn test_role_inheritance() {
    let mut rbac = RBACManager::new();

    // Create permissions
    let read_permission = Permission {
        id: "document:read".to_string(),
        name: "Read Document".to_string(),
        resource: "document".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    let write_permission = Permission {
        id: "document:update".to_string(),
        name: "Update Document".to_string(),
        resource: "document".to_string(),
        action: Action::Update,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    let admin_permission = Permission {
        id: "document:admin".to_string(),
        name: "Admin Document".to_string(),
        resource: "document".to_string(),
        action: Action::Admin,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    };

    // Create reader role with read permission
    let mut read_permissions = HashSet::new();
    read_permissions.insert(read_permission);

    let reader_role = rbac
        .create_role(
            "reader".to_string(),
            Some("Reader".to_string()),
            read_permissions,
            HashSet::new(),
        )
        .unwrap();

    // Create editor role that inherits from reader
    let mut write_permissions = HashSet::new();
    write_permissions.insert(write_permission);

    let mut editor_parent_roles = HashSet::new();
    editor_parent_roles.insert(reader_role.id.clone());

    let editor_role = rbac
        .create_role(
            "editor".to_string(),
            Some("Editor".to_string()),
            write_permissions,
            editor_parent_roles,
        )
        .unwrap();

    // Create admin role that inherits from editor
    let mut admin_permissions = HashSet::new();
    admin_permissions.insert(admin_permission);

    let mut admin_parent_roles = HashSet::new();
    admin_parent_roles.insert(editor_role.id.clone());

    let admin_role = rbac
        .create_role(
            "admin".to_string(),
            Some("Admin".to_string()),
            admin_permissions,
            admin_parent_roles,
        )
        .unwrap();

    // Admin should have all permissions
    assert!(rbac.has_permission_for_role(&admin_role, "document", Action::Read));
    assert!(rbac.has_permission_for_role(&admin_role, "document", Action::Update));
    assert!(rbac.has_permission_for_role(&admin_role, "document", Action::Admin));

    // Editor should have read and write but not admin
    assert!(rbac.has_permission_for_role(&editor_role, "document", Action::Read));
    assert!(rbac.has_permission_for_role(&editor_role, "document", Action::Update));
    assert!(!rbac.has_permission_for_role(&editor_role, "document", Action::Admin));

    // Reader should only have read
    assert!(rbac.has_permission_for_role(&reader_role, "document", Action::Read));
    assert!(!rbac.has_permission_for_role(&reader_role, "document", Action::Update));
    assert!(!rbac.has_permission_for_role(&reader_role, "document", Action::Admin));
}
