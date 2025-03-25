//! Comprehensive tests for the RBAC system
//!
//! This module consolidates tests from various RBAC components to provide
//! a unified testing approach and reduce duplicate test module declarations.

#[cfg(test)]
mod role_inheritance_tests {
    use super::super::*;
    use super::super::role_inheritance::*;
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    use uuid::Uuid;

    // Helper function to create a test inheritance graph
    fn create_test_graph() -> InheritanceGraph {
        let mut graph = InheritanceGraph::new();
        
        // Add roles
        graph.add_role("admin").unwrap();
        graph.add_role("manager").unwrap();
        graph.add_role("user").unwrap();
        
        // Add inheritance relationships
        graph.add_inheritance("admin", "manager", InheritanceType::Direct).unwrap();
        graph.add_inheritance("manager", "user", InheritanceType::Direct).unwrap();
        
        graph
    }

    #[test]
    fn test_inheritance_graph_creation() {
        let graph = create_test_graph();
        
        // Verify roles exist
        assert!(graph.has_role("admin"));
        assert!(graph.has_role("manager"));
        assert!(graph.has_role("user"));
        
        // Verify inheritance relationships
        assert!(graph.has_inheritance("admin", "manager"));
        assert!(graph.has_inheritance("manager", "user"));
        assert!(!graph.has_inheritance("admin", "user")); // No direct inheritance
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = create_test_graph();
        
        // Attempt to create a cycle
        let result = graph.add_inheritance("user", "admin", InheritanceType::Direct);
        
        // Verify cycle is detected
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            crate::error::MCPError::Security(crate::error::SecurityError::RBACError(RBACError::General(msg))) => {
                assert!(msg.contains("cycle"));
            },
            _ => panic!("Expected cycle detection error, got: {:?}", error),
        }
    }

    #[test]
    fn test_filtered_inheritance() {
        let mut graph = InheritanceGraph::new();
        
        // Add roles
        graph.add_role("admin").unwrap();
        graph.add_role("restricted").unwrap();
        
        // Create filtered inheritance
        let mut included = HashSet::new();
        included.insert("permission1".to_string());
        
        let mut excluded = HashSet::new();
        excluded.insert("permission2".to_string());
        
        let inheritance_type = InheritanceType::Filtered {
            include: included,
            exclude: excluded,
        };
        
        // Add filtered inheritance
        graph.add_inheritance("admin", "restricted", inheritance_type).unwrap();
        
        // Verify inheritance type
        let relationship = graph.get_inheritance_type("admin", "restricted").unwrap();
        match relationship {
            InheritanceType::Filtered { include, exclude } => {
                assert!(include.contains("permission1"));
                assert!(exclude.contains("permission2"));
            },
            _ => panic!("Expected filtered inheritance, got: {:?}", relationship),
        }
    }

    #[test]
    fn test_conditional_inheritance() {
        let mut graph = InheritanceGraph::new();
        
        // Add roles
        graph.add_role("admin").unwrap();
        graph.add_role("temporary").unwrap();
        
        // Create conditional inheritance
        let condition = "context.attributes.get('department') == 'IT'".to_string();
        let inheritance_type = InheritanceType::Conditional {
            condition: condition.clone(),
        };
        
        // Add conditional inheritance
        graph.add_inheritance("admin", "temporary", inheritance_type).unwrap();
        
        // Verify inheritance type
        let relationship = graph.get_inheritance_type("admin", "temporary").unwrap();
        match relationship {
            InheritanceType::Conditional { condition: cond } => {
                assert_eq!(cond, condition);
            },
            _ => panic!("Expected conditional inheritance, got: {:?}", relationship),
        }
    }
}

#[cfg(test)]
mod permission_validation_tests {
    use super::super::*;
    use super::super::permission_validation::*;
    use crate::security::types::{Action, Permission, PermissionContext, PermissionScope};
    use uuid::Uuid;
    use std::collections::{HashMap, HashSet};

    // Helper function to create a test validator
    fn create_test_validator() -> AsyncPermissionValidator {
        AsyncPermissionValidator::new()
    }

    // Helper function to create a test permission
    fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
        Permission {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            resource: resource.to_string(),
            action,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_validation_rule_addition() {
        let validator = create_test_validator();
        
        // Create validation rule
        let rule = ValidationRule {
            id: Uuid::new_v4().to_string(),
            name: "Test Rule".to_string(),
            description: Some("Test validation rule".to_string()),
            resource_pattern: "document.*".to_string(),
            action: Action::Read,
            validation_expr: "true".to_string(),
            verification: None,
            priority: 100,
            is_allow: true,
            enabled: true,
        };
        
        // Add rule
        let result = validator.add_rule(rule.clone()).await;
        
        // Verify rule is added
        assert!(result.is_ok());
        
        // Verify rule can be retrieved
        let retrieved = validator.get_rule(&rule.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Rule");
    }

    #[tokio::test]
    async fn test_permission_validation() {
        let validator = create_test_validator();
        
        // Create user and role
        let user_id = "test_user";
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read_doc", "document", Action::Read));
        
        // Create context
        let mut context = PermissionContext::new(user_id);
        context.security_level = crate::types::SecurityLevel::Standard;
        
        // Perform validation
        let result = validator.validate(
            user_id,
            "document",
            Action::Read,
            &Vec::new(), // No roles for simplicity
            &permissions,
            &context,
        ).await;
        
        // Verify validation result
        assert_eq!(result, ValidationResult::Granted);
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let validator = create_test_validator();
        
        // Enable audit logging
        validator.set_audit_enabled(true).await;
        
        // Create permission
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read_doc", "document", Action::Read));
        
        // Perform validation that will be logged
        let _ = validator.validate(
            "test_user",
            "document",
            Action::Read,
            &Vec::new(),
            &permissions,
            &PermissionContext::new("test_user"),
        ).await;
        
        // Verify audit record was created
        let logs = validator.get_user_audit("test_user").await;
        assert!(!logs.is_empty());
        
        // Verify record details
        let record = &logs[0];
        assert_eq!(record.user_id, "test_user");
        assert_eq!(record.resource, "document");
        assert_eq!(record.action, Action::Read);
        assert_eq!(record.result, ValidationResult::Granted);
    }
}

#[cfg(test)]
mod enhanced_rbac_tests {
    use super::super::*;
    use crate::security::types::{Action, Permission, PermissionContext, PermissionScope, Role};
    use chrono::Utc;
    use std::collections::{HashMap, HashSet};
    use std::time::{Duration, Instant};
    use uuid::Uuid;

    // Helper function to create a test RBAC manager
    fn create_test_manager() -> EnhancedRBACManager {
        EnhancedRBACManager::new()
    }

    // Helper function to create a test permission
    fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
        Permission {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            resource: resource.to_string(),
            action,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        }
    }

    #[tokio::test]
    async fn test_end_to_end_rbac() {
        let rbac = create_test_manager();
        
        // Create roles with permissions
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(create_test_permission("admin_read", "document", Action::Read));
        admin_permissions.insert(create_test_permission("admin_write", "document", Action::Write));
        
        let mut user_permissions = HashSet::new();
        user_permissions.insert(create_test_permission("user_read", "document", Action::Read));
        
        // Create roles
        let admin_role = rbac.create_role(
            "Admin".to_string(),
            Some("Administrator with full access".to_string()),
            admin_permissions,
            HashSet::new(),
        ).await.unwrap();
        
        let user_role = rbac.create_role(
            "User".to_string(),
            Some("Regular user with limited access".to_string()),
            user_permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign roles to users
        rbac.assign_role("admin@example.com".to_string(), admin_role.id.clone()).await.unwrap();
        rbac.assign_role("user@example.com".to_string(), user_role.id.clone()).await.unwrap();
        
        // Test permissions
        let context = PermissionContext::new("user@example.com");
        
        // User should have read permission
        let user_read = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // User should not have write permission
        let user_write = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Write,
            &context,
        ).await.unwrap();
        
        // Admin should have both permissions
        let admin_read = rbac.has_permission(
            "admin@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        
        let admin_write = rbac.has_permission(
            "admin@example.com",
            "document",
            Action::Write,
            &context,
        ).await.unwrap();
        
        // Verify results
        assert_eq!(user_read, ValidationResult::Granted);
        assert_eq!(user_write, ValidationResult::Denied);
        assert_eq!(admin_read, ValidationResult::Granted);
        assert_eq!(admin_write, ValidationResult::Granted);
    }

    #[tokio::test]
    async fn test_caching_performance() {
        let rbac = create_test_manager();
        
        // Create roles with permissions
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(create_test_permission("admin_read", "document", Action::Read));
        
        // Create role
        let admin_role = rbac.create_role(
            "Admin".to_string(),
            Some("Administrator with full access".to_string()),
            admin_permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        rbac.assign_role("admin@example.com".to_string(), admin_role.id.clone()).await.unwrap();
        
        // Create context
        let context = PermissionContext::new("admin@example.com");
        
        // First permission check (cache miss)
        let start_first = Instant::now();
        let _ = rbac.has_permission(
            "admin@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        let first_duration = start_first.elapsed();
        
        // Second permission check (should be a cache hit)
        let start_second = Instant::now();
        let _ = rbac.has_permission(
            "admin@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        let second_duration = start_second.elapsed();
        
        // Get cache stats
        let (hits, misses) = rbac.get_cache_stats().await;
        
        // Verify cache is working
        assert_eq!(misses, 1); // First check was a miss
        assert_eq!(hits, 1);   // Second check was a hit
        assert!(second_duration < first_duration); // Cached check should be faster
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        let rbac = create_test_manager();
        
        // Create roles with permissions
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read", "document", Action::Read));
        
        // Create role
        let role = rbac.create_role(
            "Role".to_string(),
            Some("Test role".to_string()),
            permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        rbac.assign_role("user@example.com".to_string(), role.id.clone()).await.unwrap();
        
        // Create context
        let context = PermissionContext::new("user@example.com");
        
        // First permission check
        let _result1 = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Clear cache
        rbac.clear_cache().await;
        
        // Second permission check (should be a cache miss)
        let _ = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Get cache stats
        let (hits, misses) = rbac.get_cache_stats().await;
        
        // Verify cache was cleared
        assert_eq!(misses, 2); // Both checks were misses
        assert_eq!(hits, 0);   // No hits
    }
    
    #[tokio::test]
    async fn test_cache_capacity() {
        let rbac = create_test_manager();
        
        // Set a small cache capacity
        rbac.set_cache_capacity(2).await;
        
        // Create roles with permissions
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read", "document", Action::Read));
        
        // Create role
        let role = rbac.create_role(
            "Role".to_string(),
            Some("Test role".to_string()),
            permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        rbac.assign_role("user@example.com".to_string(), role.id.clone()).await.unwrap();
        
        // Create contexts with different resources
        let context = PermissionContext::new("user@example.com");
        
        // First permission check
        let _ = rbac.has_permission(
            "user@example.com",
            "document1",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Second permission check
        let _ = rbac.has_permission(
            "user@example.com",
            "document2",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Third permission check (should evict the first one)
        let _ = rbac.has_permission(
            "user@example.com",
            "document3",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Check first document again (should be a cache miss)
        let _ = rbac.has_permission(
            "user@example.com",
            "document1",
            Action::Read,
            &context,
        ).await.unwrap();
        
        // Get cache stats
        let (hits, misses) = rbac.get_cache_stats().await;
        
        // Verify cache capacity is working
        assert_eq!(misses, 4); // All 4 checks should be misses
        assert_eq!(hits, 0);   // No hits
    }
    
    #[tokio::test]
    async fn test_context_dependent_caching() {
        let rbac = create_test_manager();
        
        // Create roles with permissions
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read", "document", Action::Read));
        
        // Create role
        let role = rbac.create_role(
            "Role".to_string(),
            Some("Test role".to_string()),
            permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        rbac.assign_role("user@example.com".to_string(), role.id.clone()).await.unwrap();
        
        // Create two different contexts
        let mut context1 = PermissionContext::new("user@example.com");
        context1.security_level = crate::types::SecurityLevel::Standard;
        
        let mut context2 = PermissionContext::new("user@example.com");
        context2.security_level = crate::types::SecurityLevel::High;
        
        // First permission check with context1
        let _ = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context1,
        ).await.unwrap();
        
        // Second permission check with context2 (should be a cache miss due to different context)
        let _ = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context2,
        ).await.unwrap();
        
        // Third permission check with context1 again (should be a cache hit)
        let _ = rbac.has_permission(
            "user@example.com",
            "document",
            Action::Read,
            &context1,
        ).await.unwrap();
        
        // Get cache stats
        let (hits, misses) = rbac.get_cache_stats().await;
        
        // Verify context-dependent caching is working
        assert_eq!(misses, 2); // First two checks were misses
        assert_eq!(hits, 1);   // Third check was a hit
    }
    
    #[tokio::test]
    async fn test_parallel_permission_checks() {
        let rbac = EnhancedRBACManager::new();
        
        // Create a set of permissions
        let mut permissions = HashSet::new();
        permissions.insert(create_test_permission("read", "document", Action::Read));
        
        // Update the wildcard permission to use the exact format used in the check
        let mut wildcard_perm = create_test_permission("read_all", "document", Action::Read);
        wildcard_perm.scope = PermissionScope::All; // Use All scope instead of Pattern
        wildcard_perm.resource = "document0".to_string(); // Match first document exactly
        permissions.insert(wildcard_perm);
        
        // Add permissions for the other document formats
        let doc1_perm = create_test_permission("read_doc1", "document1", Action::Read);
        permissions.insert(doc1_perm);
        
        let doc2_perm = create_test_permission("read_doc2", "document2", Action::Read);
        permissions.insert(doc2_perm);
        
        // Create role
        let role = rbac.create_role(
            "Role".to_string(),
            Some("Test role".to_string()),
            permissions,
            HashSet::new(),
        ).await.unwrap();
        
        // Assign role to user
        rbac.assign_role("user@example.com".to_string(), role.id.clone()).await.unwrap();
        
        // Create context
        let context = PermissionContext::new("user@example.com");
        
        // Run 10 permission checks in parallel
        let mut handles = Vec::new();
        for i in 0..10 {
            let rbac_clone = rbac.clone();
            let context_clone = context.clone();
            let handle = tokio::spawn(async move {
                rbac_clone.has_permission(
                    "user@example.com",
                    &format!("document{}", i % 3), // Use 3 different documents
                    Action::Read,
                    &context_clone,
                ).await.unwrap()
            });
            handles.push(handle);
        }
        
        // Wait for all permission checks to complete
        let results = futures::future::join_all(handles).await;
        
        // Verify all permission checks succeeded
        for result in results {
            assert_eq!(result.unwrap(), ValidationResult::Granted);
        }
        
        // Get cache stats
        let (hits, misses) = rbac.get_cache_stats().await;
        
        // Verify caching worked in parallel (should have 3 misses for the 3 unique documents)
        assert_eq!(misses, 3);
        assert_eq!(hits, 7);
    }
}
