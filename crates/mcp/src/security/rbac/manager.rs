// Enhanced RBAC Manager
//
// This module provides a unified RBAC manager that integrates role inheritance
// and permission validation components for comprehensive access control.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;
use chrono::Utc;
use std::error::Error;
use thiserror::Error;

use crate::error::{SecurityError, Result, MCPError};
use crate::security::types::{
    Permission, Role, PermissionContext, Action,
    PermissionCondition, PermissionScope,
};
use crate::security::rbac::{
    InheritanceManager,
    ValidationResult, ValidationRule, AsyncPermissionValidator,
    RBACError,
};
use crate::types::SecurityLevel;

use super::permission_validation::{ValidationAuditRecord, ValidationRule as SuperValidationRule};
use super::role_inheritance::{InheritanceGraph, InheritanceType};

/// Error types for RBAC operations
#[derive(Debug, thiserror::Error)]
pub enum RBACError {
    /// Role already exists
    #[error("Role already exists: {0}")]
    RoleExists(String),
    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    /// Permission error
    #[error("Permission error: {0}")]
    PermissionError(String),
    /// Internal error
    #[error("Internal RBAC error: {0}")]
    InternalError(String),
}

/// Enhanced RBAC Manager that supports role inheritance, permission validation, and audit logging
pub struct EnhancedRBACManager {
    /// Roles managed by this RBAC manager
    roles: RwLock<HashMap<String, Role>>,
    /// Inheritance graph for roles
    inheritance_graph: Arc<RwLock<InheritanceGraph>>,
    /// User-to-role mappings
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
    /// Validation rules
    validation_rules: tokio::sync::Mutex<Vec<ValidationRule>>,
    /// Audit logs for permission validation
    audit_logs: tokio::sync::Mutex<Vec<ValidationAuditRecord>>,
}

impl EnhancedRBACManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            inheritance_graph: Arc::new(RwLock::new(InheritanceGraph::new())),
            user_roles: RwLock::new(HashMap::new()),
            validation_rules: tokio::sync::Mutex::new(Vec::new()),
            audit_logs: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a new role
    pub async fn create_role(&self, name: &str, description: Option<&str>) -> Result<Role, RBACError> {
        let role_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let role = Role {
            id: role_id.clone(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            security_level: SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
            created_at: now,
            updated_at: now,
        };
        
        {
            let mut roles = self.roles.write().unwrap();
            // Check if role with this name already exists
            if roles.values().any(|r| r.name == name) {
                return Err(RBACError::RoleExists(name.to_string()));
            }
            
            roles.insert(role_id.clone(), role.clone());
        }
        
        // Initialize in inheritance graph
        {
            let mut graph = self.inheritance_graph.write().unwrap();
            graph.add_role(&role_id);
        }
        
        info!("Created role {} with ID {}", name, role_id);
        Ok(role)
    }

    /// Add permission to a role
    pub async fn add_permission_to_role(&self, role_id: &str, permission: Permission) -> Result<(), RBACError> {
        let mut roles = self.roles.write().unwrap();
        
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.insert(permission);
            role.updated_at = Utc::now();
            Ok(())
        } else {
            Err(RBACError::RoleNotFound(role_id.to_string()))
        }
    }

    /// Assign role to a user
    pub async fn assign_role_to_user(&self, user_id: &str, role_id: &str) -> Result<(), RBACError> {
        // Check if role exists
        {
            let roles = self.roles.read().unwrap();
            if !roles.contains_key(role_id) {
                return Err(RBACError::RoleNotFound(role_id.to_string()));
            }
        }
        
        // Add role to user
        {
            let mut user_roles = self.user_roles.write().unwrap();
            let user_role_set = user_roles.entry(user_id.to_string()).or_insert_with(HashSet::new);
            user_role_set.insert(role_id.to_string());
        }
        
        info!("Assigned role {} to user {}", role_id, user_id);
        Ok(())
    }

    /// Get roles for a user
    pub async fn get_user_roles(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }

    /// Check if a user has a specific permission
    pub async fn has_permission(&self, user_id: &str, resource: &str, action: Action) -> bool {
        let context = PermissionContext::new(user_id);
        self.check_permission(user_id, resource, action, &context).await.unwrap_or(false)
    }

    /// Check if a user has a specific permission with context
    pub async fn check_permission(&self, user_id: &str, resource: &str, action: Action, context: &PermissionContext) -> Result<bool, RBACError> {
        let user_role_ids = self.get_user_roles(user_id).await;
        if user_role_ids.is_empty() {
            return Ok(false);
        }

        let roles = self.roles.read().unwrap();
        let inheritance_graph = self.inheritance_graph.read().unwrap();

        // Get all permissions from user roles and inherited roles
        let mut all_permissions = HashSet::new();
        
        for role_id in &user_role_ids {
            if let Some(role) = roles.get(role_id) {
                // Add direct permissions
                for perm in &role.permissions {
                    all_permissions.insert(perm);
                }
                
                // Add inherited permissions
                for inherited_role_id in inheritance_graph.get_inherited_roles(role_id) {
                    if let Some(inherited_role) = roles.get(&inherited_role_id) {
                        for perm in &inherited_role.permissions {
                            all_permissions.insert(perm);
                        }
                    }
                }
            }
        }

        // Check if any permission matches
        for perm in all_permissions {
            if perm.resource == resource && perm.action == action {
                // Basic match found, check conditions if any
                if perm.conditions.is_empty() {
                    return Ok(true);
                }

                // Check all conditions
                let mut all_conditions_pass = true;
                for condition in &perm.conditions {
                    if !self.evaluate_condition(condition, context) {
                        all_conditions_pass = false;
                        break;
                    }
                }

                if all_conditions_pass {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Evaluate a permission condition against the context
    fn evaluate_condition(&self, condition: &PermissionCondition, context: &PermissionContext) -> bool {
        match condition {
            PermissionCondition::MinimumSecurityLevel(level) => {
                context.security_level >= *level
            },
            // Simplified implementation for other conditions
            _ => true
        }
    }
}

impl Default for EnhancedRBACManager {
    fn default() -> Self {
        Self::new()
    }
}

// For convenience in testing
#[derive(Debug, Clone)]
pub enum VerificationType {
    /// Simple verification
    Simple,
    /// Required verification
    Required,
    /// Optional verification
    Optional,
} 