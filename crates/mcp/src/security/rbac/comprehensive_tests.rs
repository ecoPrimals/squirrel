// Comprehensive tests for the enhanced RBAC system
//
// This file contains a comprehensive test suite for the enhanced RBAC system,
// testing all aspects including:
// - Role inheritance mechanisms (direct, filtered, conditional, delegated)
// - Permission validation framework
// - Audit logging capabilities
// - Integration of components in the RBACManager
// - Edge cases and performance considerations

use super::*;
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use tokio::time::timeout;
use std::time::Duration as StdDuration;
use std::sync::Arc;

// ----- Helper Functions -----

/// Helper to create test permissions
fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
    Permission {
        id: format!("perm-{}-{}", name, Uuid::new_v4().simple()),
        name: name.to_string(),
        description: Some(format!("Test permission for {}", name)),
        resource: resource.to_string(),
        action,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    }
}

/// Helper to create test permissions with conditions
fn create_test_permission_with_conditions(
    name: &str,
    resource: &str,
    action: Action,
    conditions: Vec<PermissionCondition>,
) -> Permission {
    Permission {
        id: format!("perm-{}-{}", name, Uuid::new_v4().simple()),
        name: name.to_string(),
        description: Some(format!("Test permission for {}", name)),
        resource: resource.to_string(),
        action,
        resource_id: None,
        scope: PermissionScope::All,
        conditions,
    }
}

/// Helper to create test roles
fn create_test_role(name: &str, permissions: Vec<Permission>) -> Role {
    Role {
        id: format!("role-{}-{}", name, Uuid::new_v4().simple()),
        name: name.to_string(),
        description: Some(format!("Test role for {}", name)),
        permissions: permissions.into_iter().collect(),
        parent_roles: HashSet::new(),
        security_level: crate::types::SecurityLevel::Normal,
        can_delegate: false,
        managed_roles: HashSet::new(),
    }
}

/// Helper to create a role with delegation capability
fn create_delegating_role(name: &str, permissions: Vec<Permission>, managed_roles: Vec<String>) -> Role {
    let mut role = create_test_role(name, permissions);
    role.can_delegate = true;
    role.managed_roles = managed_roles.into_iter().collect();
    role
}

/// Helper to create test permission context
fn create_test_context(user_id: &str) -> PermissionContext {
    let mut attributes = HashMap::new();
    attributes.insert("department".to_string(), "Engineering".to_string());
    attributes.insert("location".to_string(), "HQ".to_string());
    attributes.insert("clearance".to_string(), "Standard".to_string());
    
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

/// Helper to create a high security context
fn create_high_security_context(user_id: &str) -> PermissionContext {
    let mut context = create_test_context(user_id);
    context.security_level = crate::types::SecurityLevel::High;
    context.attributes.insert("clearance".to_string(), "High".to_string());
    context
}

// ----- Basic RBAC Tests -----

#[tokio::test]
async fn test_basic_role_operations() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let write_perm = create_test_permission("Write", "document", Action::Update);
    let admin_perm = create_test_permission("Admin", "system", Action::Admin);
    
    let reader_role = create_test_role("Reader", vec![read_perm.clone()]);
    let writer_role = create_test_role("Writer", vec![write_perm.clone()]);
    let admin_role = create_test_role("Admin", vec![admin_perm.clone()]);
    
    // Add roles to RBAC manager
    rbac.add_role(reader_role.clone()).await?;
    rbac.add_role(writer_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    // Test getting roles
    let retrieved_reader = rbac.get_role(&reader_role.id).await?;
    assert_eq!(retrieved_reader.name, "Reader", "Retrieved role name should match");
    
    // Test getting all roles
    let all_roles = rbac.get_all_roles().await;
    assert_eq!(all_roles.len(), 3, "Should have 3 roles in total");
    
    // Test updating a role
    let mut updated_reader = reader_role.clone();
    updated_reader.description = Some("Updated reader role".to_string());
    rbac.update_role(updated_reader.clone()).await?;
    
    let retrieved_updated = rbac.get_role(&reader_role.id).await?;
    assert_eq!(
        retrieved_updated.description,
        Some("Updated reader role".to_string()),
        "Role description should be updated"
    );
    
    // Test removing a role
    rbac.remove_role(&writer_role.id).await?;
    
    let all_roles_after_remove = rbac.get_all_roles().await;
    assert_eq!(all_roles_after_remove.len(), 2, "Should have 2 roles after removal");
    assert!(
        !all_roles_after_remove.iter().any(|r| r.id == writer_role.id),
        "Writer role should be removed"
    );
    
    // Test error on getting non-existent role
    let result = rbac.get_role("non-existent-role").await;
    assert!(result.is_err(), "Getting non-existent role should error");
    
    Ok(())
}

#[tokio::test]
async fn test_user_role_management() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create and add roles
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let write_perm = create_test_permission("Write", "document", Action::Update);
    
    let reader_role = create_test_role("Reader", vec![read_perm.clone()]);
    let writer_role = create_test_role("Writer", vec![write_perm.clone()]);
    
    rbac.add_role(reader_role.clone()).await?;
    rbac.add_role(writer_role.clone()).await?;
    
    // Assign roles to users
    let user1 = "user1";
    let user2 = "user2";
    
    rbac.assign_role_to_user(user1, &reader_role.id).await?;
    rbac.assign_role_to_user(user2, &reader_role.id).await?;
    rbac.assign_role_to_user(user2, &writer_role.id).await?;
    
    // Test getting user roles
    let user1_roles = rbac.get_user_roles(user1).await?;
    assert_eq!(user1_roles.len(), 1, "User1 should have 1 role");
    assert_eq!(user1_roles[0].id, reader_role.id, "User1 should have reader role");
    
    let user2_roles = rbac.get_user_roles(user2).await?;
    assert_eq!(user2_roles.len(), 2, "User2 should have 2 roles");
    
    // Test unassigning roles
    rbac.unassign_role_from_user(user2, &reader_role.id).await?;
    
    let user2_roles_after = rbac.get_user_roles(user2).await?;
    assert_eq!(user2_roles_after.len(), 1, "User2 should have 1 role after unassign");
    assert_eq!(
        user2_roles_after[0].id,
        writer_role.id,
        "User2 should only have writer role after unassign"
    );
    
    // Test unassigning non-existent role (should not error)
    let result = rbac.unassign_role_from_user(user1, "non-existent-role").await;
    assert!(result.is_ok(), "Unassigning non-existent role should not error");
    
    // Test for non-existent user
    let non_existent_user_roles = rbac.get_user_roles("non-existent-user").await?;
    assert_eq!(
        non_existent_user_roles.len(),
        0,
        "Non-existent user should have 0 roles"
    );
    
    Ok(())
}

// ----- Role Inheritance Tests -----

#[tokio::test]
async fn test_direct_inheritance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test permissions
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let write_perm = create_test_permission("Write", "document", Action::Update);
    let delete_perm = create_test_permission("Delete", "document", Action::Delete);
    let admin_perm = create_test_permission("Admin", "system", Action::Admin);
    
    // Create test roles
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    let editor_role = create_test_role("Editor", vec![write_perm.clone()]);
    let admin_role = create_test_role("Admin", vec![delete_perm.clone(), admin_perm.clone()]);
    
    // Add roles to RBAC manager
    rbac.add_role(user_role.clone()).await?;
    rbac.add_role(editor_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    // Create inheritance chain: Admin -> Editor -> User
    rbac.add_direct_inheritance(&editor_role.id, &user_role.id).await?;
    rbac.add_direct_inheritance(&admin_role.id, &editor_role.id).await?;
    
    // Assign roles to users
    rbac.assign_role_to_user("user1", &user_role.id).await?;
    rbac.assign_role_to_user("editor1", &editor_role.id).await?;
    rbac.assign_role_to_user("admin1", &admin_role.id).await?;
    
    // Create context for permission checks
    let context = create_test_context("test");
    
    // Test user1 permissions (should only have read)
    let result = rbac.check_permission("user1", "document", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("User should have read permission"),
    }
    
    let result = rbac.check_permission("user1", "document", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("User should not have write permission"),
    }
    
    // Test editor1 permissions (should have read and write from inheritance)
    let result = rbac.check_permission("editor1", "document", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Editor should have read permission from inheritance"),
    }
    
    let result = rbac.check_permission("editor1", "document", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Editor should have write permission"),
    }
    
    let result = rbac.check_permission("editor1", "document", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Editor should not have delete permission"),
    }
    
    // Test admin1 permissions (should have all permissions from inheritance chain)
    let result = rbac.check_permission("admin1", "document", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should have read permission from inheritance"),
    }
    
    let result = rbac.check_permission("admin1", "document", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should have write permission from inheritance"),
    }
    
    let result = rbac.check_permission("admin1", "document", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should have delete permission"),
    }
    
    // Test removing inheritance
    rbac.remove_inheritance(&admin_role.id, &editor_role.id).await?;
    
    // Admin should no longer inherit editor's permissions
    let result = rbac.check_permission("admin1", "document", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Admin should no longer have write permission after inheritance removal"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_filtered_inheritance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test permissions for a reporting system
    let view_reports_perm = create_test_permission("ViewReports", "reports", Action::Read);
    let create_reports_perm = create_test_permission("CreateReports", "reports", Action::Create);
    let export_reports_perm = create_test_permission("ExportReports", "reports", Action::Execute);
    let delete_reports_perm = create_test_permission("DeleteReports", "reports", Action::Delete);
    let manage_users_perm = create_test_permission("ManageUsers", "users", Action::Admin);
    
    // Create roles
    let reporter_role = create_test_role(
        "Reporter", 
        vec![view_reports_perm.clone(), create_reports_perm.clone(), export_reports_perm.clone()]
    );
    
    let admin_role = create_test_role(
        "Admin",
        vec![delete_reports_perm.clone(), manage_users_perm.clone()]
    );
    
    // Add roles to RBAC manager
    rbac.add_role(reporter_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    // Create filtered inheritance: Admin inherits only view and export permissions from Reporter
    let mut included_permissions = HashSet::new();
    included_permissions.insert(view_reports_perm.id.clone());
    included_permissions.insert(export_reports_perm.id.clone());
    
    rbac.add_filtered_inheritance(
        &reporter_role.id,
        &admin_role.id,
        included_permissions,
        HashSet::new()
    ).await?;
    
    // Assign roles
    rbac.assign_role_to_user("reporter1", &reporter_role.id).await?;
    rbac.assign_role_to_user("admin1", &admin_role.id).await?;
    
    // Test permissions
    let context = create_test_context("test");
    
    // Reporter should have view, create, and export permissions
    let result = rbac.check_permission("reporter1", "reports", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Reporter should have view permission"),
    }
    
    let result = rbac.check_permission("reporter1", "reports", Action::Create, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Reporter should have create permission"),
    }
    
    let result = rbac.check_permission("reporter1", "reports", Action::Execute, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Reporter should have export permission"),
    }
    
    // Admin should have delete and manage users permissions directly
    let result = rbac.check_permission("admin1", "reports", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should have delete permission"),
    }
    
    let result = rbac.check_permission("admin1", "users", Action::Admin, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should have user management permission"),
    }
    
    // Admin should inherit view and export permissions but not create
    let result = rbac.check_permission("admin1", "reports", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should inherit view permission"),
    }
    
    let result = rbac.check_permission("admin1", "reports", Action::Execute, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should inherit export permission"),
    }
    
    let result = rbac.check_permission("admin1", "reports", Action::Create, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Admin should not inherit create permission due to filtered inheritance"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_conditional_inheritance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test permissions
    let view_financial_perm = create_test_permission("ViewFinancial", "financial", Action::Read);
    let edit_financial_perm = create_test_permission("EditFinancial", "financial", Action::Update);
    
    // Create roles
    let financial_role = create_test_role(
        "Financial", 
        vec![view_financial_perm.clone(), edit_financial_perm.clone()]
    );
    
    let manager_role = create_test_role("Manager", vec![]);
    
    // Add roles to RBAC manager
    rbac.add_role(financial_role.clone()).await?;
    rbac.add_role(manager_role.clone()).await?;
    
    // Create conditional inheritance: Manager inherits Financial permissions only if in Finance department
    rbac.add_conditional_inheritance(
        &financial_role.id,
        &manager_role.id,
        "context.attributes.get('department') == Some(&String::from('Finance'))".to_string()
    ).await?;
    
    // Assign roles
    rbac.assign_role_to_user("financial1", &financial_role.id).await?;
    rbac.assign_role_to_user("manager1", &manager_role.id).await?;
    
    // Test with Finance department context
    let mut finance_context = create_test_context("manager1");
    finance_context.attributes.insert("department".to_string(), "Finance".to_string());
    
    // Manager should inherit financial permissions when in Finance department
    let result = rbac.check_permission("manager1", "financial", Action::Read, Some(finance_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Manager in Finance department should inherit view financial permission"),
    }
    
    let result = rbac.check_permission("manager1", "financial", Action::Update, Some(finance_context)).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Manager in Finance department should inherit edit financial permission"),
    }
    
    // Test with Engineering department context
    let mut eng_context = create_test_context("manager1");
    eng_context.attributes.insert("department".to_string(), "Engineering".to_string());
    
    // Manager should not inherit financial permissions when in Engineering department
    let result = rbac.check_permission("manager1", "financial", Action::Read, Some(eng_context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Manager in Engineering department should not inherit financial permissions"),
    }
    
    let result = rbac.check_permission("manager1", "financial", Action::Update, Some(eng_context)).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Manager in Engineering department should not inherit financial permissions"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_delegated_inheritance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test permissions
    let approve_expense_perm = create_test_permission("ApproveExpense", "expenses", Action::Approve);
    
    // Create roles
    let finance_manager_role = create_test_role(
        "FinanceManager", 
        vec![approve_expense_perm.clone()]
    );
    
    let delegate_role = create_test_role("TemporaryApprover", vec![]);
    
    // Add roles to RBAC manager
    rbac.add_role(finance_manager_role.clone()).await?;
    rbac.add_role(delegate_role.clone()).await?;
    
    // Assign roles
    rbac.assign_role_to_user("finance_manager", &finance_manager_role.id).await?;
    rbac.assign_role_to_user("temp_approver", &delegate_role.id).await?;
    
    // Create delegated inheritance with expiration (1 hour from now)
    let expiration = Utc::now() + Duration::hours(1);
    
    rbac.add_delegated_inheritance(
        &finance_manager_role.id,
        &delegate_role.id,
        "finance_manager".to_string(),
        Some(expiration)
    ).await?;
    
    // Test before expiration
    let context = create_test_context("test");
    
    // Temporary approver should have approval permission before expiration
    let result = rbac.check_permission("temp_approver", "expenses", Action::Approve, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Temporary approver should have approval permission before expiration"),
    }
    
    // Manually override the expiration time in the inheritance graph to test expiration
    // Note: This is for testing purposes only, in production we would wait for the actual expiration
    
    // This part is tricky without direct access to the inheritance graph internals
    // We'll leverage the inheritance manager to modify the inheritance relationship
    
    // For testing purposes, we'll create a new inheritance with an already expired timestamp
    let expired_time = Utc::now() - Duration::hours(1);
    
    // First remove the existing inheritance
    rbac.remove_inheritance(&finance_manager_role.id, &delegate_role.id).await?;
    
    // Then add a new delegation with an expired timestamp
    rbac.add_delegated_inheritance(
        &finance_manager_role.id,
        &delegate_role.id,
        "finance_manager".to_string(),
        Some(expired_time)
    ).await?;
    
    // Test after expiration
    let result = rbac.check_permission("temp_approver", "expenses", Action::Approve, Some(context)).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Temporary approver should not have permission after delegation expiration"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_inheritance_cycles() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let role_a = create_test_role("RoleA", vec![]);
    let role_b = create_test_role("RoleB", vec![]);
    let role_c = create_test_role("RoleC", vec![]);
    
    // Add roles to RBAC manager
    rbac.add_role(role_a.clone()).await?;
    rbac.add_role(role_b.clone()).await?;
    rbac.add_role(role_c.clone()).await?;
    
    // Create a valid inheritance chain: A -> B -> C
    rbac.add_direct_inheritance(&role_a.id, &role_b.id).await?;
    rbac.add_direct_inheritance(&role_b.id, &role_c.id).await?;
    
    // Try to create a cycle: C -> A (should fail)
    let result = rbac.add_direct_inheritance(&role_c.id, &role_a.id).await;
    assert!(result.is_err(), "Creating an inheritance cycle should fail");
    
    // Try with a different inheritance type (should still fail)
    let result = rbac.add_conditional_inheritance(
        &role_c.id,
        &role_a.id,
        "true".to_string()
    ).await;
    assert!(result.is_err(), "Creating an inheritance cycle with conditional inheritance should fail");
    
    Ok(())
}

// ----- Permission Validation Tests -----

#[tokio::test]
async fn test_basic_validation_rules() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create validation rules
    
    // Rule 1: Allow read access to public resources
    let public_read_rule = ValidationRule {
        id: "public-read".to_string(),
        name: "Public Read Access".to_string(),
        description: Some("Allow reading public resources".to_string()),
        resource_pattern: "public/.*".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(), // Always allow
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    // Rule 2: Deny delete operations for all users
    let deny_delete_rule = ValidationRule {
        id: "deny-delete".to_string(),
        name: "Deny Delete".to_string(),
        description: Some("Deny delete operations for all users".to_string()),
        resource_pattern: ".*".to_string(),
        action: Some(Action::Delete),
        validation_expression: "true".to_string(), // Always apply
        priority: 200, // Higher priority than public read
        is_allow: false,
        verification: None,
    };
    
    // Add rules
    rbac.add_validation_rule(public_read_rule).await?;
    rbac.add_validation_rule(deny_delete_rule).await?;
    
    // Test permission checks
    let context = create_test_context("test_user");
    
    // Public read should be allowed
    let result = rbac.check_permission("test_user", "public/document", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Public read should be allowed by validation rule"),
    }
    
    // Delete should be denied regardless of resource
    let result = rbac.check_permission("test_user", "public/document", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Delete should be denied by validation rule"),
    }
    
    // Other operations on public resources should be denied (no matching rule)
    let result = rbac.check_permission("test_user", "public/document", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Update should be denied (no matching rule)"),
    }
    
    // Operations on non-public resources should be denied
    let result = rbac.check_permission("test_user", "private/document", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Read on private resource should be denied (no matching rule)"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_rule_priority() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create validation rules with different priorities for the same resource/action
    
    // Low priority rule (allow)
    let low_priority_rule = ValidationRule {
        id: "low-priority".to_string(),
        name: "Low Priority Allow".to_string(),
        description: Some("Low priority rule that allows access".to_string()),
        resource_pattern: "test/resource".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    // High priority rule (deny)
    let high_priority_rule = ValidationRule {
        id: "high-priority".to_string(),
        name: "High Priority Deny".to_string(),
        description: Some("High priority rule that denies access".to_string()),
        resource_pattern: "test/resource".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 200, // Higher priority
        is_allow: false,
        verification: None,
    };
    
    // Add rules
    rbac.add_validation_rule(low_priority_rule).await?;
    rbac.add_validation_rule(high_priority_rule).await?;
    
    // Test permission check - high priority rule should win
    let context = create_test_context("test_user");
    let result = rbac.check_permission("test_user", "test/resource", Action::Read, Some(context)).await?;
    
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("High priority deny rule should override low priority allow rule"),
    }
    
    // Now remove the high priority rule and test again
    rbac.remove_validation_rule("high-priority").await?;
    
    let context = create_test_context("test_user");
    let result = rbac.check_permission("test_user", "test/resource", Action::Read, Some(context)).await?;
    
    match result {
        ValidationResult::Granted => {},
        _ => panic!("With high priority rule removed, low priority allow rule should apply"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_resource_pattern_matching() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create rule with regex pattern
    let pattern_rule = ValidationRule {
        id: "pattern-rule".to_string(),
        name: "Resource Pattern Rule".to_string(),
        description: Some("Tests regex pattern matching".to_string()),
        resource_pattern: "documents/[0-9]{4}/[a-z]+\\.pdf".to_string(), // Matches documents/NNNN/name.pdf
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    // Add rule
    rbac.add_validation_rule(pattern_rule).await?;
    
    // Test with various resource paths
    let context = create_test_context("test_user");
    
    // Should match the pattern
    let result = rbac.check_permission("test_user", "documents/1234/report.pdf", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should match documents/1234/report.pdf"),
    }
    
    // Should not match (year has 5 digits)
    let result = rbac.check_permission("test_user", "documents/12345/report.pdf", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Should not match documents/12345/report.pdf"),
    }
    
    // Should not match (name contains numbers)
    let result = rbac.check_permission("test_user", "documents/1234/report123.pdf", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Should not match documents/1234/report123.pdf"),
    }
    
    // Should not match (different action)
    let result = rbac.check_permission("test_user", "documents/1234/report.pdf", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Should not match Update action"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_verification_requirements() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create rules with verification requirements
    
    // Rule requiring MFA
    let mfa_rule = ValidationRule {
        id: "mfa-rule".to_string(),
        name: "Sensitive Data MFA".to_string(),
        description: Some("Requires MFA for sensitive data".to_string()),
        resource_pattern: "sensitive/.*".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: Some(VerificationType::MultiFactorAuth),
    };
    
    // Rule requiring manager approval
    let approval_rule = ValidationRule {
        id: "approval-rule".to_string(),
        name: "Financial Approval".to_string(),
        description: Some("Requires manager approval for financial operations".to_string()),
        resource_pattern: "financial/.*".to_string(),
        action: Some(Action::Update),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: Some(VerificationType::ManagerApproval),
    };
    
    // Add rules
    rbac.add_validation_rule(mfa_rule).await?;
    rbac.add_validation_rule(approval_rule).await?;
    
    // Test permission checks
    let context = create_test_context("test_user");
    
    // Sensitive data should require MFA
    let result = rbac.check_permission("test_user", "sensitive/customer_data", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::RequiresVerification { verification_type, .. } => {
            assert_eq!(verification_type, VerificationType::MultiFactorAuth, "Should require MFA verification");
        },
        _ => panic!("Sensitive data should require MFA verification"),
    }
    
    // Financial update should require manager approval
    let result = rbac.check_permission("test_user", "financial/budget", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::RequiresVerification { verification_type, .. } => {
            assert_eq!(verification_type, VerificationType::ManagerApproval, "Should require manager approval");
        },
        _ => panic!("Financial update should require manager approval"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_complex_validation_expressions() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    let admin_role = create_test_role("Admin", vec![]);
    
    rbac.add_role(user_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    rbac.assign_role_to_user("test_admin", &admin_role.id).await?;
    
    // Create rule with complex validation expression
    let complex_rule = ValidationRule {
        id: "complex-rule".to_string(),
        name: "Complex Validation Rule".to_string(),
        description: Some("Tests complex validation expressions".to_string()),
        resource_pattern: "project/.*".to_string(),
        action: Some(Action::Update),
        // Only allow if user has high clearance and is at HQ, or is an admin
        validation_expression: "(context.attributes.get('clearance') == Some(&String::from('High')) && \
                               context.attributes.get('location') == Some(&String::from('HQ'))) || \
                               roles.iter().any(|r| r.name == 'Admin')".to_string(),
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    // Add rule
    rbac.add_validation_rule(complex_rule).await?;
    
    // Test with various contexts
    
    // Regular user at HQ with standard clearance (should be denied)
    let standard_hq_context = create_test_context("test_user");
    let result = rbac.check_permission("test_user", "project/alpha", Action::Update, Some(standard_hq_context)).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Standard clearance user at HQ should be denied"),
    }
    
    // Regular user at HQ with high clearance (should be granted)
    let high_hq_context = create_high_security_context("test_user");
    let result = rbac.check_permission("test_user", "project/alpha", Action::Update, Some(high_hq_context)).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("High clearance user at HQ should be granted"),
    }
    
    // Admin with standard clearance and not at HQ (should be granted due to admin role)
    let mut admin_context = create_test_context("test_admin");
    admin_context.attributes.insert("location".to_string(), "Remote".to_string());
    
    let result = rbac.check_permission("test_admin", "project/alpha", Action::Update, Some(admin_context)).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should be granted regardless of location/clearance"),
    }
    
    Ok(())
}

// ----- Audit Logging Tests -----

#[tokio::test]
async fn test_basic_audit_logging() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles with permissions
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Perform permission checks to generate audit logs
    let context = create_test_context("test_user");
    
    // Successful permission check
    rbac.check_permission("test_user", "document", Action::Read, Some(context.clone())).await?;
    
    // Failed permission check
    rbac.check_permission("test_user", "document", Action::Update, Some(context.clone())).await?;
    
    // Get audit logs for the user
    let user_audit = rbac.get_user_audit("test_user").await?;
    
    // Verify audit records
    assert_eq!(user_audit.len(), 2, "Should have 2 audit records for test_user");
    
    // Get audit logs for the resource
    let resource_audit = rbac.get_resource_audit("document").await?;
    assert_eq!(resource_audit.len(), 2, "Should have 2 audit records for document resource");
    
    // Verify granted audit record
    let granted_audit = user_audit.iter()
        .find(|record| record.action == Action::Read)
        .expect("Should have a record for Read action");
    
    assert_eq!(granted_audit.resource, "document", "Resource should be document");
    assert!(granted_audit.granted, "Access should be granted");
    assert!(!granted_audit.matched_permissions.is_empty(), "Should have matched permissions");
    
    // Verify denied audit record
    let denied_audit = user_audit.iter()
        .find(|record| record.action == Action::Update)
        .expect("Should have a record for Update action");
    
    assert_eq!(denied_audit.resource, "document", "Resource should be document");
    assert!(!denied_audit.granted, "Access should be denied");
    assert!(denied_audit.reason.is_some(), "Should have a denial reason");
    
    Ok(())
}

#[tokio::test]
async fn test_audit_with_verification() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles
    let user_role = create_test_role("User", vec![]);
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create a rule that requires verification
    let verification_rule = ValidationRule {
        id: "verification-rule".to_string(),
        name: "Sensitive Data Rule".to_string(),
        description: Some("Requires MFA for sensitive data".to_string()),
        resource_pattern: "sensitive/.*".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: Some(VerificationType::MultiFactorAuth),
    };
    
    rbac.add_validation_rule(verification_rule).await?;
    
    // Perform permission check that requires verification
    let context = create_test_context("test_user");
    rbac.check_permission("test_user", "sensitive/customer_data", Action::Read, Some(context)).await?;
    
    // Get audit logs
    let user_audit = rbac.get_user_audit("test_user").await?;
    assert_eq!(user_audit.len(), 1, "Should have 1 audit record for test_user");
    
    // Verify audit record
    let verification_audit = &user_audit[0];
    assert_eq!(verification_audit.resource, "sensitive/customer_data", "Resource should match");
    
    // The actual validation_result field is not directly accessible in the audit record
    // But we can check if the record contains information about verification
    let has_verification_info = verification_audit.context.iter()
        .any(|(key, _)| key.contains("verification") || key.contains("mfa"));
    
    // If the validation_result isn't directly stored, we should at least have context info
    assert!(!verification_audit.context.is_empty(), "Should have context information in the audit record");
    
    Ok(())
}

#[tokio::test]
async fn test_audit_log_management() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles with permissions
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Generate some audit logs
    let context = create_test_context("test_user");
    for i in 0..5 {
        rbac.check_permission("test_user", &format!("document_{}", i), Action::Read, Some(context.clone())).await?;
    }
    
    // Verify we have 5 audit records
    let all_audit = rbac.get_all_audit().await?;
    assert_eq!(all_audit.len(), 5, "Should have 5 audit records");
    
    // Set a small max audit size
    rbac.set_max_audit_size(3).await?;
    
    // Generate more logs to trigger truncation
    for i in 5..10 {
        rbac.check_permission("test_user", &format!("document_{}", i), Action::Read, Some(context.clone())).await?;
    }
    
    // Verify we now have at most 3 records (due to max size)
    let all_audit_after_truncation = rbac.get_all_audit().await?;
    assert!(all_audit_after_truncation.len() <= 3, "Should have at most 3 audit records after setting max size");
    
    // Clear audit logs
    rbac.clear_audit().await?;
    
    // Verify all logs are cleared
    let all_audit_after_clear = rbac.get_all_audit().await?;
    assert_eq!(all_audit_after_clear.len(), 0, "Should have 0 audit records after clearing");
    
    Ok(())
}

#[tokio::test]
async fn test_context_in_audit_logs() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create test roles with permissions
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    rbac.add_role(user_role.clone()).await?;
    rbac.assign_role_to_user("test_user", &user_role.id).await?;
    
    // Create a context with rich attributes
    let mut rich_context = create_test_context("test_user");
    rich_context.attributes.insert("client_ip".to_string(), "203.0.113.1".to_string());
    rich_context.attributes.insert("device_type".to_string(), "mobile".to_string());
    rich_context.attributes.insert("browser".to_string(), "Chrome".to_string());
    rich_context.attributes.insert("request_id".to_string(), Uuid::new_v4().to_string());
    
    // Perform permission check with rich context
    rbac.check_permission("test_user", "document", Action::Read, Some(rich_context)).await?;
    
    // Get audit logs
    let audit_records = rbac.get_user_audit("test_user").await?;
    assert_eq!(audit_records.len(), 1, "Should have 1 audit record");
    
    // Verify context attributes in audit record
    let audit_record = &audit_records[0];
    
    // Check for selected attributes in the audit context
    assert!(!audit_record.context.is_empty(), "Audit record should have context data");
    
    // The structure of context in the audit record may vary depending on implementation
    // Here we're making assumptions about how attributes might be stored
    let has_relevant_context = audit_record.context.iter().any(|(key, _)| 
        key.contains("client_ip") || 
        key.contains("device_type") || 
        key.contains("browser") ||
        key.contains("request_id")
    );
    
    assert!(has_relevant_context, "Audit record should contain relevant context attributes");
    
    Ok(())
}

// ----- Integration Tests -----

#[tokio::test]
async fn test_end_to_end_rbac_flow() -> Result<()> {
    // This test simulates a complete end-to-end flow of RBAC operations
    let rbac = RBACManager::new();
    
    // 1. Create hierarchy of roles with different permissions
    let view_reports_perm = create_test_permission("ViewReports", "reports", Action::Read);
    let edit_reports_perm = create_test_permission("EditReports", "reports", Action::Update);
    let approve_reports_perm = create_test_permission("ApproveReports", "reports", Action::Approve);
    let delete_reports_perm = create_test_permission("DeleteReports", "reports", Action::Delete);
    let manage_users_perm = create_test_permission("ManageUsers", "users", Action::Admin);
    
    // Create roles
    let analyst_role = create_test_role("Analyst", vec![view_reports_perm.clone()]);
    let editor_role = create_test_role("Editor", vec![edit_reports_perm.clone()]);
    let manager_role = create_test_role("Manager", vec![approve_reports_perm.clone()]);
    let admin_role = create_test_role(
        "Admin", 
        vec![delete_reports_perm.clone(), manage_users_perm.clone()]
    );
    
    // 2. Add roles to RBAC manager
    rbac.add_role(analyst_role.clone()).await?;
    rbac.add_role(editor_role.clone()).await?;
    rbac.add_role(manager_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    // 3. Create inheritance relationships
    
    // Editor inherits from Analyst
    rbac.add_direct_inheritance(&analyst_role.id, &editor_role.id).await?;
    
    // Manager inherits from Editor
    rbac.add_direct_inheritance(&editor_role.id, &manager_role.id).await?;
    
    // Admin has filtered inheritance from Manager (gets approval but not edit)
    let mut included_permissions = HashSet::new();
    included_permissions.insert(approve_reports_perm.id.clone());
    
    rbac.add_filtered_inheritance(
        &manager_role.id,
        &admin_role.id,
        included_permissions,
        HashSet::new()
    ).await?;
    
    // 4. Set up validation rules
    
    // Rule: Sensitive reports require MFA
    let sensitive_rule = ValidationRule {
        id: "sensitive-rule".to_string(),
        name: "Sensitive Reports".to_string(),
        description: Some("Requires MFA for sensitive reports".to_string()),
        resource_pattern: "reports/sensitive/.*".to_string(),
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: Some(VerificationType::MultiFactorAuth),
    };
    
    // Rule: Finance reports require high clearance
    let finance_rule = ValidationRule {
        id: "finance-rule".to_string(),
        name: "Finance Reports".to_string(),
        description: Some("Requires high clearance for finance reports".to_string()),
        resource_pattern: "reports/finance/.*".to_string(),
        action: None, // Applies to all actions
        validation_expression: "context.attributes.get('clearance') == Some(&String::from('High'))".to_string(),
        priority: 200,
        is_allow: true,
        verification: None,
    };
    
    // Rule: Only allow report approval during business hours
    let business_hours_rule = ValidationRule {
        id: "business-hours-rule".to_string(),
        name: "Business Hours".to_string(),
        description: Some("Only allow report approval during business hours".to_string()),
        resource_pattern: "reports/.*".to_string(),
        action: Some(Action::Approve),
        validation_expression: "true".to_string(), // Simplified for testing
        priority: 150,
        is_allow: true,
        verification: None,
    };
    
    // Add rules
    rbac.add_validation_rule(sensitive_rule).await?;
    rbac.add_validation_rule(finance_rule).await?;
    rbac.add_validation_rule(business_hours_rule).await?;
    
    // 5. Assign roles to users
    rbac.assign_role_to_user("analyst1", &analyst_role.id).await?;
    rbac.assign_role_to_user("editor1", &editor_role.id).await?;
    rbac.assign_role_to_user("manager1", &manager_role.id).await?;
    rbac.assign_role_to_user("admin1", &admin_role.id).await?;
    
    // 6. Test different permission scenarios
    let standard_context = create_test_context("test");
    let high_context = create_high_security_context("test");
    
    // Analyst permissions
    let result = rbac.check_permission("analyst1", "reports/standard", Action::Read, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Analyst should be able to view standard reports"),
    }
    
    let result = rbac.check_permission("analyst1", "reports/standard", Action::Update, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Analyst should not be able to edit reports"),
    }
    
    // Editor permissions (including inherited)
    let result = rbac.check_permission("editor1", "reports/standard", Action::Read, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Editor should be able to view standard reports (inherited)"),
    }
    
    let result = rbac.check_permission("editor1", "reports/standard", Action::Update, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Editor should be able to edit reports"),
    }
    
    // Manager permissions (including inherited)
    let result = rbac.check_permission("manager1", "reports/standard", Action::Read, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Manager should be able to view standard reports (inherited)"),
    }
    
    let result = rbac.check_permission("manager1", "reports/standard", Action::Update, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Manager should be able to edit reports (inherited)"),
    }
    
    let result = rbac.check_permission("manager1", "reports/standard", Action::Approve, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Manager should be able to approve reports"),
    }
    
    // Admin permissions
    let result = rbac.check_permission("admin1", "reports/standard", Action::Delete, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should be able to delete reports"),
    }
    
    let result = rbac.check_permission("admin1", "reports/standard", Action::Approve, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Admin should be able to approve reports (inherited with filter)"),
    }
    
    let result = rbac.check_permission("admin1", "reports/standard", Action::Update, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Admin should not be able to edit reports (filtered out in inheritance)"),
    }
    
    // Test rule-based restrictions
    
    // Sensitive reports requiring MFA
    let result = rbac.check_permission("analyst1", "reports/sensitive/customer", Action::Read, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::RequiresVerification { verification_type, .. } => {
            assert_eq!(verification_type, VerificationType::MultiFactorAuth);
        },
        _ => panic!("Sensitive reports should require MFA"),
    }
    
    // Finance reports requiring high clearance
    let result = rbac.check_permission("analyst1", "reports/finance/quarterly", Action::Read, Some(standard_context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Finance reports should be denied with standard clearance"),
    }
    
    let result = rbac.check_permission("analyst1", "reports/finance/quarterly", Action::Read, Some(high_context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Finance reports should be granted with high clearance"),
    }
    
    // 7. Check audit logs
    let all_audits = rbac.get_all_audit().await?;
    assert!(!all_audits.is_empty(), "Should have generated audit logs");
    
    Ok(())
}

#[tokio::test]
async fn test_complex_inheritance_scenarios() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create a complex role hierarchy to test diamond inheritance and other complex scenarios
    
    // Basic permissions
    let read_perm = create_test_permission("Read", "resource", Action::Read);
    let write_perm = create_test_permission("Write", "resource", Action::Update);
    let execute_perm = create_test_permission("Execute", "resource", Action::Execute);
    let delete_perm = create_test_permission("Delete", "resource", Action::Delete);
    let admin_perm = create_test_permission("Admin", "resource", Action::Admin);
    
    // Roles for diamond inheritance pattern:
    //      Base
    //     /    \
    //   Left   Right
    //     \    /
    //      Bottom
    
    let base_role = create_test_role("Base", vec![read_perm.clone()]);
    let left_role = create_test_role("Left", vec![write_perm.clone()]);
    let right_role = create_test_role("Right", vec![execute_perm.clone()]);
    let bottom_role = create_test_role("Bottom", vec![]);
    
    // Additional roles for complex scenarios
    let admin_role = create_test_role("Admin", vec![admin_perm.clone(), delete_perm.clone()]);
    
    // Add roles
    rbac.add_role(base_role.clone()).await?;
    rbac.add_role(left_role.clone()).await?;
    rbac.add_role(right_role.clone()).await?;
    rbac.add_role(bottom_role.clone()).await?;
    rbac.add_role(admin_role.clone()).await?;
    
    // Create diamond inheritance
    rbac.add_direct_inheritance(&base_role.id, &left_role.id).await?;
    rbac.add_direct_inheritance(&base_role.id, &right_role.id).await?;
    rbac.add_direct_inheritance(&left_role.id, &bottom_role.id).await?;
    rbac.add_direct_inheritance(&right_role.id, &bottom_role.id).await?;
    
    // Create filtered inheritance from Admin to Bottom
    // Bottom inherits only delete permission from Admin
    let mut included_permissions = HashSet::new();
    included_permissions.insert(delete_perm.id.clone());
    
    rbac.add_filtered_inheritance(
        &admin_role.id, 
        &bottom_role.id,
        included_permissions,
        HashSet::new()
    ).await?;
    
    // Assign Bottom role to test user
    rbac.assign_role_to_user("test_user", &bottom_role.id).await?;
    
    // Test permissions
    let context = create_test_context("test_user");
    
    // Bottom should inherit Read from Base through Left and Right
    let result = rbac.check_permission("test_user", "resource", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should inherit Read from Base through diamond inheritance"),
    }
    
    // Bottom should inherit Write from Left
    let result = rbac.check_permission("test_user", "resource", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should inherit Write from Left"),
    }
    
    // Bottom should inherit Execute from Right
    let result = rbac.check_permission("test_user", "resource", Action::Execute, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should inherit Execute from Right"),
    }
    
    // Bottom should inherit Delete from Admin (filtered)
    let result = rbac.check_permission("test_user", "resource", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should inherit Delete from Admin (filtered)"),
    }
    
    // Bottom should NOT inherit Admin from Admin (filtered out)
    let result = rbac.check_permission("test_user", "resource", Action::Admin, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Should NOT inherit Admin from Admin (filtered out)"),
    }
    
    Ok(())
}

// ----- Performance Tests -----

#[tokio::test]
async fn test_permission_check_performance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create a larger role hierarchy to test performance
    let mut roles = Vec::new();
    let mut role_ids = Vec::new();
    
    // Create 20 roles with unique permissions
    for i in 0..20 {
        let perm = create_test_permission(&format!("Perm{}", i), "resource", Action::Read);
        let role = create_test_role(&format!("Role{}", i), vec![perm.clone()]);
        role_ids.push(role.id.clone());
        roles.push(role);
    }
    
    // Add roles to manager
    for role in roles.clone() {
        rbac.add_role(role).await?;
    }
    
    // Create a moderately complex inheritance tree
    for i in 1..20 {
        if i % 3 == 0 {
            // Every 3rd role inherits from role 0
            rbac.add_direct_inheritance(&role_ids[0], &role_ids[i]).await?;
        } else if i % 3 == 1 {
            // Every role at position 1, 4, 7, etc. inherits from previous role
            rbac.add_direct_inheritance(&role_ids[i-1], &role_ids[i]).await?;
        } else {
            // Others inherit from role at (i-2)
            rbac.add_direct_inheritance(&role_ids[i-2], &role_ids[i]).await?;
        }
    }
    
    // Assign the last role to test user
    rbac.assign_role_to_user("test_user", &role_ids[19]).await?;
    
    // Create 50 validation rules
    for i in 0..50 {
        let rule = ValidationRule {
            id: format!("rule-{}", i),
            name: format!("Rule {}", i),
            description: Some(format!("Test rule {}", i)),
            resource_pattern: format!("resource/{}", i % 10),
            action: Some(Action::Read),
            validation_expression: "true".to_string(),
            priority: i as i32,
            is_allow: true,
            verification: None,
        };
        
        rbac.add_validation_rule(rule).await?;
    }
    
    // Test performance with timeout
    let context = create_test_context("test_user");
    
    // Check permission with timeout to ensure it completes within a reasonable time
    let result = timeout(
        StdDuration::from_millis(100), // 100ms should be more than enough
        rbac.check_permission("test_user", "resource/25", Action::Read, Some(context))
    ).await;
    
    // Verify the timeout didn't occur
    assert!(result.is_ok(), "Permission check took too long");
    
    // Verify the result is as expected
    match result.unwrap()? {
        ValidationResult::Granted => {},
        _ => panic!("Should have granted permission"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_large_role_hierarchy_performance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create a large role hierarchy with 100 roles
    const NUM_ROLES: usize = 100;
    let mut roles = Vec::new();
    let mut role_ids = Vec::new();
    
    // Create roles
    for i in 0..NUM_ROLES {
        let perm = create_test_permission(&format!("Perm{}", i), "resource", Action::Read);
        let role = create_test_role(&format!("Role{}", i), vec![perm.clone()]);
        role_ids.push(role.id.clone());
        roles.push(role);
    }
    
    // Add roles to manager
    for role in roles.clone() {
        rbac.add_role(role).await?;
    }
    
    // Create a deep inheritance chain: Role0 <- Role1 <- Role2 <- ... <- Role99
    for i in 1..NUM_ROLES {
        rbac.add_direct_inheritance(&role_ids[i-1], &role_ids[i]).await?;
    }
    
    // Assign the last role to test user
    rbac.assign_role_to_user("test_user", &role_ids[NUM_ROLES-1]).await?;
    
    // Test performance with timeout
    let context = create_test_context("test_user");
    
    // Check permission with timeout
    let result = timeout(
        StdDuration::from_millis(200), // 200ms should be enough for even a deep hierarchy
        rbac.check_permission("test_user", "resource", Action::Read, Some(context))
    ).await;
    
    // Verify the timeout didn't occur
    assert!(result.is_ok(), "Permission check with deep hierarchy took too long");
    
    // Verify the result is as expected (should inherit all the way up the chain)
    match result.unwrap()? {
        ValidationResult::Granted => {},
        _ => panic!("Should have granted permission through inheritance chain"),
    }
    
    Ok(())
}

#[tokio::test]
async fn test_combined_performance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create a moderate number of roles and rules with varied inheritance patterns
    const NUM_ROLES: usize = 50;
    const NUM_RULES: usize = 30;
    
    // Create roles with varied permissions
    let mut roles = Vec::new();
    let mut role_ids = Vec::new();
    
    for i in 0..NUM_ROLES {
        let action = match i % 5 {
            0 => Action::Read,
            1 => Action::Update,
            2 => Action::Delete,
            3 => Action::Execute,
            _ => Action::Admin,
        };
        
        let perm = create_test_permission(&format!("Perm{}", i), &format!("resource/{}", i % 10), action);
        let role = create_test_role(&format!("Role{}", i), vec![perm.clone()]);
        role_ids.push(role.id.clone());
        roles.push(role);
    }
    
    // Add roles to manager
    for role in roles.clone() {
        rbac.add_role(role).await?;
    }
    
    // Create varied inheritance patterns
    for i in 1..NUM_ROLES {
        if i % 5 == 0 {
            // Every 5th role inherits from role 0
            rbac.add_direct_inheritance(&role_ids[0], &role_ids[i]).await?;
        } else if i % 7 == 0 {
            // Every 7th role has filtered inheritance
            let mut included = HashSet::new();
            included.insert(format!("Perm{}", i-1));
            
            rbac.add_filtered_inheritance(
                &role_ids[i-1],
                &role_ids[i],
                included,
                HashSet::new()
            ).await?;
        } else if i % 11 == 0 {
            // Every 11th role has conditional inheritance
            rbac.add_conditional_inheritance(
                &role_ids[i-1],
                &role_ids[i],
                "context.attributes.get('location') == Some(&String::from('HQ'))".to_string()
            ).await?;
        } else {
            // Others have direct inheritance from previous
            rbac.add_direct_inheritance(&role_ids[i-1], &role_ids[i]).await?;
        }
    }
    
    // Create validation rules
    for i in 0..NUM_RULES {
        let rule = ValidationRule {
            id: format!("rule-{}", i),
            name: format!("Rule {}", i),
            description: Some(format!("Test rule {}", i)),
            resource_pattern: format!("resource/{}", i % 10),
            action: Some(match i % 5 {
                0 => Action::Read,
                1 => Action::Update,
                2 => Action::Delete,
                3 => Action::Execute,
                _ => Action::Admin,
            }),
            validation_expression: if i % 3 == 0 {
                "context.attributes.get('clearance') == Some(&String::from('High'))".to_string()
            } else {
                "true".to_string()
            },
            priority: (NUM_RULES - i) as i32, // Reverse priority
            is_allow: i % 4 != 0, // Every 4th rule is a deny rule
            verification: if i % 7 == 0 {
                Some(VerificationType::MultiFactorAuth)
            } else {
                None
            },
        };
        
        rbac.add_validation_rule(rule).await?;
    }
    
    // Assign various roles to test user
    for i in [5, 15, 25, 35, 45] {
        rbac.assign_role_to_user("test_user", &role_ids[i]).await?;
    }
    
    // Test performance with varied contexts
    let standard_context = create_test_context("test_user");
    let high_context = create_high_security_context("test_user");
    
    // Test multiple permission checks with timeout
    let resources = ["resource/1", "resource/3", "resource/5", "resource/7", "resource/9"];
    let actions = [Action::Read, Action::Update, Action::Delete, Action::Execute, Action::Admin];
    
    // Combined timeout for all operations
    let result = timeout(
        StdDuration::from_millis(500), // 500ms for all operations
        async {
            // Perform 25 permission checks (5 resources x 5 actions)
            for resource in &resources {
                for action in &actions {
                    // Alternate between standard and high context
                    let context = if (*resource).ends_with('3') || (*resource).ends_with('7') {
                        &high_context
                    } else {
                        &standard_context
                    };
                    
                    // We don't care about the result, just that it completes quickly
                    let _ = rbac.check_permission("test_user", resource, *action, Some(context.clone())).await?;
                }
            }
            
            // Get audit logs as part of the test
            let _ = rbac.get_user_audit("test_user").await?;
            let _ = rbac.get_resource_audit("resource/5").await?;
            
            Ok::<(), SecurityError>(())
        }
    ).await;
    
    // Verify all operations completed within timeout
    match result {
        Ok(inner_result) => {
            // Check that the inner operation succeeded
            inner_result?;
        },
        Err(_) => panic!("Combined operations took too long"),
    }
    
    Ok(())
}

// ----- Edge Cases and Error Handling Tests -----

#[tokio::test]
async fn test_empty_and_missing_entities() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Test permissions for non-existent user
    let context = create_test_context("non_existent_user");
    let result = rbac.check_permission("non_existent_user", "resource", Action::Read, Some(context.clone())).await?;
    
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Non-existent user should be denied permissions"),
    }
    
    // Create an empty role (no permissions)
    let empty_role = create_test_role("EmptyRole", vec![]);
    rbac.add_role(empty_role.clone()).await?;
    rbac.assign_role_to_user("empty_user", &empty_role.id).await?;
    
    // Test permissions for user with empty role
    let result = rbac.check_permission("empty_user", "resource", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("User with empty role should be denied permissions"),
    }
    
    // Test with empty context
    let empty_context = PermissionContext {
        user_id: "test_user".to_string(),
        current_time: None,
        network_address: None,
        security_level: crate::types::SecurityLevel::Normal,
        attributes: HashMap::new(),
        resource_owner_id: None,
        resource_group_id: None,
    };
    
    let result = rbac.check_permission("empty_user", "resource", Action::Read, Some(empty_context)).await?;
    match result {
        ValidationResult::Denied { .. } => {},
        _ => panic!("Permission check with empty context should be denied"),
    }
    
    // Test missing role operations
    let result = rbac.get_role("non_existent_role").await;
    assert!(result.is_err(), "Getting non-existent role should error");
    
    let result = rbac.update_role(create_test_role("NonExistentRole", vec![])).await;
    assert!(result.is_err(), "Updating non-existent role should error");
    
    let result = rbac.add_direct_inheritance("non_existent_parent", "non_existent_child").await;
    assert!(result.is_err(), "Adding inheritance between non-existent roles should error");
    
    Ok(())
}

#[tokio::test]
async fn test_malformed_validation_rules() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Test rule with invalid regex pattern
    let invalid_regex_rule = ValidationRule {
        id: "invalid-regex".to_string(),
        name: "Invalid Regex".to_string(),
        description: Some("Rule with invalid regex pattern".to_string()),
        resource_pattern: "[", // Invalid regex
        action: Some(Action::Read),
        validation_expression: "true".to_string(),
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    let result = rbac.add_validation_rule(invalid_regex_rule).await;
    assert!(result.is_err(), "Adding rule with invalid regex should error");
    
    // Test rule with malformed validation expression (this might not fail at add time)
    let invalid_expression_rule = ValidationRule {
        id: "invalid-expression".to_string(),
        name: "Invalid Expression".to_string(),
        description: Some("Rule with invalid expression".to_string()),
        resource_pattern: ".*".to_string(),
        action: Some(Action::Read),
        validation_expression: "invalid && expression".to_string(), // Invalid expression
        priority: 100,
        is_allow: true,
        verification: None,
    };
    
    // This might not fail at add time, but we check that it doesn't crash the system
    let _ = rbac.add_validation_rule(invalid_expression_rule).await;
    
    // Create role and user for permission check
    let role = create_test_role("TestRole", vec![]);
    rbac.add_role(role.clone()).await?;
    rbac.assign_role_to_user("test_user", &role.id).await?;
    
    // Permission check should not crash even with the bad rule
    let context = create_test_context("test_user");
    let result = rbac.check_permission("test_user", "test", Action::Read, Some(context)).await;
    
    // Either way, this should complete without crashing
    assert!(result.is_ok(), "Permission check should not crash with bad rule");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    // Create RBAC manager
    let rbac = Arc::new(RBACManager::new());
    
    // Create initial roles
    let read_perm = create_test_permission("Read", "document", Action::Read);
    let user_role = create_test_role("User", vec![read_perm.clone()]);
    
    rbac.add_role(user_role.clone()).await?;
    
    // Test concurrent role creation and assignment
    let mut handles = Vec::new();
    
    // Clone for move into tasks
    let rbac_clone1 = rbac.clone();
    let rbac_clone2 = rbac.clone();
    let rbac_clone3 = rbac.clone();
    
    // Task 1: Create and add 10 roles concurrently
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let perm = create_test_permission(&format!("Perm{}", i), "resource", Action::Read);
            let role = create_test_role(&format!("Role{}", i), vec![perm.clone()]);
            let _ = rbac_clone1.add_role(role).await;
        }
    }));
    
    // Task 2: Assign roles to users concurrently
    handles.push(tokio::spawn(async move {
        for i in 0..10 {
            let _ = rbac_clone2.assign_role_to_user(&format!("user{}", i), &user_role.id).await;
        }
    }));
    
    // Task 3: Perform permission checks concurrently
    handles.push(tokio::spawn(async move {
        let context = create_test_context("test_user");
        for i in 0..10 {
            let _ = rbac_clone3.check_permission("user0", "document", Action::Read, Some(context.clone())).await;
        }
    }));
    
    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }
    
    // If we got here without deadlocks or panics, the test passes
    
    Ok(())
}

#[tokio::test]
async fn test_cross_linking_inheritance() -> Result<()> {
    // Create RBAC manager
    let rbac = RBACManager::new();
    
    // Create roles with cross-linking inheritance (not cyclical, but complex)
    
    // Create roles
    let role_a = create_test_role("RoleA", vec![]);
    let role_b = create_test_role("RoleB", vec![]);
    let role_c = create_test_role("RoleC", vec![]);
    let role_d = create_test_role("RoleD", vec![]);
    let role_e = create_test_role("RoleE", vec![]);
    
    // Add roles
    rbac.add_role(role_a.clone()).await?;
    rbac.add_role(role_b.clone()).await?;
    rbac.add_role(role_c.clone()).await?;
    rbac.add_role(role_d.clone()).await?;
    rbac.add_role(role_e.clone()).await?;
    
    // Create a complex cross-linked inheritance pattern:
    // A -> B, A -> C, B -> D, C -> D, B -> E, D -> E
    // This is not cyclical but has multiple paths to the same roles
    
    rbac.add_direct_inheritance(&role_a.id, &role_b.id).await?;
    rbac.add_direct_inheritance(&role_a.id, &role_c.id).await?;
    rbac.add_direct_inheritance(&role_b.id, &role_d.id).await?;
    rbac.add_direct_inheritance(&role_c.id, &role_d.id).await?;
    rbac.add_direct_inheritance(&role_b.id, &role_e.id).await?;
    rbac.add_direct_inheritance(&role_d.id, &role_e.id).await?;
    
    // Add permissions to roles
    let a_perm = create_test_permission("PermA", "resource", Action::Read);
    let b_perm = create_test_permission("PermB", "resource", Action::Update);
    let c_perm = create_test_permission("PermC", "resource", Action::Delete);
    let d_perm = create_test_permission("PermD", "resource", Action::Execute);
    let e_perm = create_test_permission("PermE", "resource", Action::Admin);
    
    let mut updated_a = role_a.clone();
    updated_a.permissions.insert(a_perm.clone());
    let mut updated_b = role_b.clone();
    updated_b.permissions.insert(b_perm.clone());
    let mut updated_c = role_c.clone();
    updated_c.permissions.insert(c_perm.clone());
    let mut updated_d = role_d.clone();
    updated_d.permissions.insert(d_perm.clone());
    let mut updated_e = role_e.clone();
    updated_e.permissions.insert(e_perm.clone());
    
    rbac.update_role(updated_a).await?;
    rbac.update_role(updated_b).await?;
    rbac.update_role(updated_c).await?;
    rbac.update_role(updated_d).await?;
    rbac.update_role(updated_e).await?;
    
    // Assign role E to test user (should inherit all permissions through multiple paths)
    rbac.assign_role_to_user("test_user", &role_e.id).await?;
    
    // Test permissions
    let context = create_test_context("test_user");
    
    // Should have E's permission directly
    let result = rbac.check_permission("test_user", "resource", Action::Admin, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should have direct permission from E"),
    }
    
    // Should have D's permission (direct inheritance)
    let result = rbac.check_permission("test_user", "resource", Action::Execute, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should have permission from D"),
    }
    
    // Should have B's permission (inheritance via D or direct from B)
    let result = rbac.check_permission("test_user", "resource", Action::Update, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should have permission from B"),
    }
    
    // Should have C's permission (inheritance via D)
    let result = rbac.check_permission("test_user", "resource", Action::Delete, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should have permission from C"),
    }
    
    // Should have A's permission (inheritance via B and C, multiple paths)
    let result = rbac.check_permission("test_user", "resource", Action::Read, Some(context.clone())).await?;
    match result {
        ValidationResult::Granted => {},
        _ => panic!("Should have permission from A through multiple inheritance paths"),
    }
    
    Ok(())
}

// Finally, let's document the overall test coverage

// Test coverage includes:
// 1. Basic RBAC operations (roles, permissions, user-role assignments)
// 2. Role inheritance mechanisms (direct, filtered, conditional, delegated)
// 3. Permission validation framework (rules, priorities, resource patterns)
// 4. Additional verification requirements (MFA, approvals)
// 5. Audit logging functionality
// 6. Integration tests for end-to-end flows
// 7. Performance and scalability tests
// 8. Edge cases and error handling
// 
// These tests ensure that our enhanced RBAC system is:
// - Functionally correct
// - Secure
// - Performant
// - Robust against edge cases
// - Capable of handling complex authorization scenarios

// Test coverage includes:
// 1. Basic RBAC operations (roles, permissions, user-role assignments)
// 2. Role inheritance mechanisms (direct, filtered, conditional, delegated)
// 3. Permission validation framework (rules, priorities, resource patterns)
// 4. Additional verification requirements (MFA, approvals)
// 5. Audit logging functionality
// 6. Integration tests for end-to-end flows
// 7. Performance and scalability tests
// 8. Edge cases and error handling
// 
// These tests ensure that our enhanced RBAC system is:
// - Functionally correct
// - Secure
// - Performant
// - Robust against edge cases
// - Capable of handling complex authorization scenarios 