// Tests for the enhanced RBAC system
#![allow(unused_imports)]

use super::*;
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::security::types::{
    Action, Permission, PermissionCondition, PermissionContext, PermissionScope, Role
};
use crate::types::SecurityLevel;
use crate::security::rbac::{
    EnhancedRBACManager, ValidationResult
};

/// Helper to create test permissions
fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
    Permission {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: Some(format!("Test permission for {}", name)),
        resource: resource.to_string(),
        action,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    }
}

/// Helper to create test roles
fn create_test_role(name: &str, permissions: Vec<Permission>) -> Role {
    Role {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: Some(format!("Test role for {}", name)),
        permissions: permissions.into_iter().collect(),
        parent_roles: HashSet::new(),
        security_level: crate::types::SecurityLevel::Normal,
        can_delegate: false,
        managed_roles: HashSet::new(),
    }
}

/// Helper to create test permission context
fn create_test_context(user_id: &str) -> PermissionContext {
    let mut attributes = HashMap::new();
    attributes.insert("department".to_string(), "Engineering".to_string());
    attributes.insert("location".to_string(), "HQ".to_string());
    
    PermissionContext {
        user_id: user_id.to_string(),
        current_time: Some(Utc::now()),
        network_address: Some("192.168.1.1".to_string()),
        security_level: crate::types::SecurityLevel::Normal,
        attributes,
        resource_owner_id: Some(user_id.to_string()),
        resource_group_id: Some("test-group".to_string()),
    }
}

#[tokio::test]
async fn test_inheritance_graph() -> Result<()> {
    // Create inheritance graph
    let mut graph = InheritanceGraph::new();
    
    // Add roles
    graph.add_role("role1")?;
    graph.add_role("role2")?;
    graph.add_role("role3")?;
    graph.add_role("role4")?;
    
    // Add inheritance relationships
    graph.add_inheritance("role1", "role2", InheritanceType::Direct)?;
    graph.add_inheritance("role2", "role3", InheritanceType::Direct)?;
    graph.add_inheritance("role1", "role4", InheritanceType::Direct)?;
    
    // Check inheritance
    assert!(graph.inherits_from("role2", "role1")?);
    assert!(graph.inherits_from("role3", "role2")?);
    assert!(graph.inherits_from("role3", "role1")?);
    assert!(graph.inherits_from("role4", "role1")?);
    assert!(!graph.inherits_from("role2", "role3")?);
    assert!(!graph.inherits_from("role1", "role2")?);
    
    // Check ancestry
    let ancestors = graph.get_ancestors("role3")?;
    assert_eq!(ancestors.len(), 2);
    assert!(ancestors.contains("role1"));
    assert!(ancestors.contains("role2"));
    
    // Check descendants
    let descendants = graph.get_descendants("role1")?;
    assert_eq!(descendants.len(), 3);
    assert!(descendants.contains("role2"));
    assert!(descendants.contains("role3"));
    assert!(descendants.contains("role4"));
    
    Ok(())
}

#[tokio::test]
async fn test_inheritance_cycle_detection() -> Result<()> {
    // Create inheritance graph
    let mut graph = InheritanceGraph::new();
    
    // Add roles
    graph.add_role("role1")?;
    graph.add_role("role2")?;
    graph.add_role("role3")?;
    
    // Add inheritance relationships
    graph.add_inheritance("role1", "role2", InheritanceType::Direct)?;
    graph.add_inheritance("role2", "role3", InheritanceType::Direct)?;
    
    // Adding role3->role1 would create a cycle
    let result = graph.add_inheritance("role3", "role1", InheritanceType::Direct);
    assert!(result.is_err(), "Cycle should be detected");
    
    Ok(())
}

#[tokio::test]
async fn test_filtered_inheritance() -> Result<()> {
    // Create inheritance manager
    let manager = InheritanceManager::new();
    
    // Create test roles
    let read_perm = create_test_permission("Read Data", "data", Action::Read);
    let write_perm = create_test_permission("Write Data", "data", Action::Update);
    let delete_perm = create_test_permission("Delete Data", "data", Action::Delete);
    
    let admin_role = create_test_role("Admin", vec![
        read_perm.clone(), 
        write_perm.clone(), 
        delete_perm.clone()
    ]);
    
    let user_role = create_test_role("User", vec![]);
    
    // Add roles to manager
    manager.add_role(&admin_role.id).await?;
    manager.add_role(&user_role.id).await?;
    
    // Create filtered inheritance (user inherits only read permission from admin)
    let mut included = HashSet::new();
    included.insert(read_perm.id.clone());
    
    manager.add_filtered_inheritance(
        &admin_role.id, 
        &user_role.id, 
        included, 
        HashSet::new()
    ).await?;
    
    // Create role map for testing
    let mut role_map = HashMap::new();
    role_map.insert(admin_role.id.clone(), admin_role.clone());
    role_map.insert(user_role.id.clone(), user_role.clone());
    
    // Test inherited permissions
    let context = create_test_context("test-user");
    let inherited = manager.get_inherited_permissions(
        &user_role.id, 
        &role_map, 
        Some(&context)
    ).await?;
    
    assert_eq!(inherited.len(), 1, "Should only inherit one permission");
    assert!(inherited.iter().any(|p| p.id == read_perm.id), "Should inherit read permission");
    assert!(!inherited.iter().any(|p| p.id == write_perm.id), "Should not inherit write permission");
    assert!(!inherited.iter().any(|p| p.id == delete_perm.id), "Should not inherit delete permission");
    
    Ok(())
}

#[tokio::test]
async fn test_conditional_inheritance() -> Result<()> {
    // Create inheritance manager
    let manager = InheritanceManager::new();
    
    // Create test roles
    let hq_perm = create_test_permission("HQ Access", "building", Action::Execute);
    let admin_role = create_test_role("Admin", vec![hq_perm.clone()]);
    let user_role = create_test_role("User", vec![]);
    
    // Add roles to manager
    manager.add_role(&admin_role.id).await?;
    manager.add_role(&user_role.id).await?;
    
    // Create conditional inheritance (user inherits admin permissions only in HQ)
    manager.add_conditional_inheritance(
        &admin_role.id,
        &user_role.id,
        "context.attributes.get('location') == Some(&String::from('HQ'))".to_string()
    ).await?;
    
    // Create role map for testing
    let mut role_map = HashMap::new();
    role_map.insert(admin_role.id.clone(), admin_role.clone());
    role_map.insert(user_role.id.clone(), user_role.clone());
    
    // Test with HQ context (condition satisfied)
    let hq_context = create_test_context("test-user");
    let inherited = manager.get_inherited_permissions(
        &user_role.id,
        &role_map,
        Some(&hq_context)
    ).await?;
    
    assert_eq!(inherited.len(), 1, "Should inherit permission when in HQ");
    
    // Test with different context (condition not satisfied)
    let mut remote_context = create_test_context("test-user");
    remote_context.attributes.insert("location".to_string(), "Remote".to_string());
    
    let inherited = manager.get_inherited_permissions(
        &user_role.id,
        &role_map,
        Some(&remote_context)
    ).await?;
    
    assert_eq!(inherited.len(), 0, "Should not inherit permission when not in HQ");
    
    Ok(())
}

#[tokio::test]
async fn test_delegated_inheritance() -> Result<()> {
    // Create inheritance manager
    let manager = InheritanceManager::new();
    
    // Create test roles
    let admin_perm = create_test_permission("Admin Access", "system", Action::Admin);
    let admin_role = create_test_role("Admin", vec![admin_perm.clone()]);
    let user_role = create_test_role("User", vec![]);
    
    // Add roles to manager
    manager.add_role(&admin_role.id).await?;
    manager.add_role(&user_role.id).await?;
    
    // Create delegated inheritance (temporary admin access)
    let expiration = Utc::now() + Duration::hours(1);
    manager.add_delegated_inheritance(
        &admin_role.id,
        &user_role.id,
        "delegator-admin".to_string(),
        Some(expiration)
    ).await?;
    
    // Create role map for testing
    let mut role_map = HashMap::new();
    role_map.insert(admin_role.id.clone(), admin_role.clone());
    role_map.insert(user_role.id.clone(), user_role.clone());
    
    // Test before expiration
    let context = create_test_context("test-user");
    let inherited = manager.get_inherited_permissions(
        &user_role.id,
        &role_map,
        Some(&context)
    ).await?;
    
    assert_eq!(inherited.len(), 1, "Should inherit permission before expiration");
    
    // Test with expired delegation
    let mut expired_inheritance = manager.inheritance_graph.write().await;
    if let Some(node) = expired_inheritance.nodes.get_mut(&user_role.id) {
        if let Some(parent) = node.parent_roles.get_mut(&admin_role.id) {
            if let InheritanceType::Delegated { delegator_id: _, ref mut expires_at } = parent {
                // Set expiration to the past
                *expires_at = Some(Utc::now() - Duration::hours(1));
            }
        }
    }
    drop(expired_inheritance);
    
    let inherited = manager.get_inherited_permissions(
        &user_role.id,
        &role_map,
        Some(&context)
    ).await?;
    
    assert_eq!(inherited.len(), 0, "Should not inherit permission after expiration");
    
    Ok(())
}

#[tokio::test]
async fn test_validation_framework() -> Result<()> {
    // Create permission validator
    let validator = AsyncPermissionValidator::new();
    
    // Create validation rules
    let rule1 = ValidationRule {
        id: "sensitive-data-rule".to_string(),
        name: "Sensitive Data Access".to_string(),
        description: Some("Requires additional verification for sensitive data".to_string()),
        resource_pattern: "sensitive/.*".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        verification: Some(VerificationType::MultiFactorAuth),
    };
    
    let rule2 = ValidationRule {
        id: "admin-only-rule".to_string(),
        name: "Admin Only".to_string(),
        description: Some("Only admins can delete".to_string()),
        resource_pattern: ".*".to_string(),
        action: Some(Action::Delete),
        validation_expression: "roles.iter().any(|r| r.name == 'Admin')".to_string(),
        priority: 200,
        verification: None,
    };
    
    // Add rules
    validator.add_rule(rule1).await?;
    validator.add_rule(rule2).await?;
    
    // Create test roles and permissions
    let read_perm = create_test_permission("Read Sensitive", "sensitive/data", Action::Read);
    let delete_perm = create_test_permission("Delete Data", "data", Action::Delete);
    
    let admin_role = create_test_role("Admin", vec![read_perm.clone(), delete_perm.clone()]);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    // Create context
    let context = create_test_context("test-user");
    
    // Test admin role with sensitive data (requires verification)
    let result = validator.validate(
        "admin-user",
        "sensitive/data",
        Action::Read,
        &vec![admin_role.clone()],
        &vec![read_perm.clone()],
        &context
    ).await;
    
    match result {
        ValidationResult::RequiresVerification { verification_type, .. } => {
            assert_eq!(verification_type, VerificationType::MultiFactorAuth);
        },
        _ => panic!("Expected verification requirement for sensitive data"),
    }
    
    // Test admin role with delete permission (should be granted)
    let result = validator.validate(
        "admin-user",
        "data",
        Action::Delete,
        &vec![admin_role.clone()],
        &vec![delete_perm.clone()],
        &context
    ).await;
    
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Expected admin to have delete permission"),
    }
    
    // Test user role with delete permission (should be denied)
    let result = validator.validate(
        "regular-user",
        "data",
        Action::Delete,
        &vec![user_role.clone()],
        &vec![delete_perm.clone()],
        &context
    ).await;
    
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Expected regular user to be denied delete permission"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_enhanced_rbac() {
    let rbac = EnhancedRBACManager::new();
    
    // Create roles
    let admin_role = rbac.create_role("Admin", Some("Administrator role")).await.unwrap();
    let user_role = rbac.create_role("User", Some("Regular user role")).await.unwrap();
    let manager_role = rbac.create_role("Manager", Some("Manager role")).await.unwrap();
    
    // Add permissions to roles
    for perm in create_admin_permissions() {
        rbac.add_permission_to_role(&admin_role.id, perm).await.unwrap();
    }
    
    for perm in create_user_permissions() {
        rbac.add_permission_to_role(&user_role.id, perm).await.unwrap();
    }
    
    for perm in create_manager_permissions() {
        rbac.add_permission_to_role(&manager_role.id, perm).await.unwrap();
    }
    
    // Assign roles to users
    rbac.assign_role_to_user("admin-user", &admin_role.id).await.unwrap();
    rbac.assign_role_to_user("regular-user", &user_role.id).await.unwrap();
    rbac.assign_role_to_user("manager-user", &manager_role.id).await.unwrap();
    
    // Test permissions
    let context = create_test_context("test-user");
    
    // Admin should have access to everything
    let result = rbac.check_permission(
        "admin-user",
        "reports",
        Action::Read,
        &context,
    ).await.unwrap();
    
    assert!(result, "Expected admin to have access to reports");
    
    // User should not have access to reports
    let result = rbac.check_permission(
        "regular-user",
        "reports",
        Action::Read,
        &context,
    ).await.unwrap();
    
    assert!(!result, "Expected user to be denied access to reports");
    
    // Manager should have access to user data in group scope
    let result = rbac.check_permission(
        "manager-user",
        "user_data",
        Action::Read,
        &context,
    ).await.unwrap();
    
    assert!(result, "Expected manager to have access to user data");
}

fn create_admin_permissions() -> HashSet<Permission> {
    let mut permissions = HashSet::new();
    
    permissions.insert(Permission {
        id: "admin-read-all".to_string(),
        name: "Read All".to_string(),
        resource: "*".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    });
    
    permissions.insert(Permission {
        id: "admin-write-all".to_string(),
        name: "Write All".to_string(),
        resource: "*".to_string(),
        action: Action::Update,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    });
    
    permissions.insert(Permission {
        id: "admin-reports".to_string(),
        name: "Access Reports".to_string(),
        resource: "reports".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    });
    
    permissions
}

fn create_user_permissions() -> HashSet<Permission> {
    let mut permissions = HashSet::new();
    
    permissions.insert(Permission {
        id: "user-read-own".to_string(),
        name: "Read Own".to_string(),
        resource: "user_data".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::Own,
        conditions: Vec::new(),
    });
    
    permissions.insert(Permission {
        id: "user-write-own".to_string(),
        name: "Write Own".to_string(),
        resource: "user_data".to_string(),
        action: Action::Update,
        resource_id: None,
        scope: PermissionScope::Own,
        conditions: Vec::new(),
    });
    
    permissions
}

fn create_manager_permissions() -> HashSet<Permission> {
    let mut permissions = HashSet::new();
    
    permissions.insert(Permission {
        id: "manager-read-group".to_string(),
        name: "Read Group".to_string(),
        resource: "user_data".to_string(),
        action: Action::Read,
        resource_id: None,
        scope: PermissionScope::Group,
        conditions: Vec::new(),
    });
    
    permissions
}

#[tokio::test]
async fn test_audit_logging() -> Result<()> {
    // Create permission validator with audit
    let validator = AsyncPermissionValidator::new();
    
    // Create test roles and permissions
    let read_perm = create_test_permission("Read Data", "data", Action::Read);
    let write_perm = create_test_permission("Write Data", "data", Action::Update);
    
    let admin_role = create_test_role("Admin", vec![read_perm.clone(), write_perm.clone()]);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    // Create context
    let context = create_test_context("test-user");
    
    // Perform some validation operations to generate audit records
    validator.validate(
        "admin-user",
        "data",
        Action::Read,
        &vec![admin_role.clone()],
        &vec![read_perm.clone()],
        &context
    ).await;
    
    validator.validate(
        "admin-user",
        "data",
        Action::Update,
        &vec![admin_role.clone()],
        &vec![write_perm.clone()],
        &context
    ).await;
    
    validator.validate(
        "regular-user",
        "data",
        Action::Read,
        &vec![user_role.clone()],
        &vec![read_perm.clone()],
        &context
    ).await;
    
    validator.validate(
        "regular-user",
        "data",
        Action::Update,
        &vec![user_role.clone()],
        &vec![],
        &context
    ).await;
    
    // Check user audit records
    let admin_audit = validator.get_user_audit("admin-user").await;
    assert_eq!(admin_audit.len(), 2, "Admin should have two audit records");
    
    let user_audit = validator.get_user_audit("regular-user").await;
    assert_eq!(user_audit.len(), 2, "Regular user should have two audit records");
    
    // Check resource audit records
    let data_audit = validator.get_resource_audit("data").await;
    assert_eq!(data_audit.len(), 4, "Data resource should have four audit records");
    
    // Check all audit records
    let all_audit = validator.get_all_audit().await;
    assert_eq!(all_audit.len(), 4, "Should have four audit records total");
    
    // Check audit record details
    let admin_read = admin_audit.iter().find(|r| r.action == Action::Read).unwrap();
    assert_eq!(admin_read.user_id, "admin-user");
    assert_eq!(admin_read.resource, "data");
    assert!(matches!(admin_read.result, ValidationResult::Granted));
    
    let user_update = user_audit.iter().find(|r| r.action == Action::Update).unwrap();
    assert_eq!(user_update.user_id, "regular-user");
    assert_eq!(user_update.resource, "data");
    assert!(matches!(user_update.result, ValidationResult::Denied { .. }));
    
    Ok(())
}
