//! RBAC Module
//!
//! This module provides a comprehensive Role-Based Access Control system with:
//! - Fine-grained permission control
//! - Role inheritance (direct, filtered, and conditional)
//! - Permission validation with audit logging
//! - Complex authorization rules 

use tracing::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::error::{MCPError, Result, SecurityError};

// Re-export types from security::types
use crate::security::types::{
    Action, Permission, PermissionCondition, PermissionContext, 
    PermissionScope, Role,
};

// Local modules
mod role_inheritance;
mod permission_validation;
mod manager;

// Re-export components for use throughout the application
pub use self::role_inheritance::{InheritanceManager, InheritanceType};
pub use self::permission_validation::{
    AsyncPermissionValidator, ValidationAuditRecord, ValidationResult, ValidationRule,
};
pub use self::manager::RBACManager;

/// RBAC-specific error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum RBACError {
    /// Role already exists
    #[error("Role already exists: {0}")]
    RoleExists(String),
    
    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    /// Role dependency cycle detected
    #[error("Role dependency cycle detected: {0}")]
    DependencyCycle(String),
    
    /// Inheritance already exists
    #[error("Inheritance already exists: {0} -> {1}")]
    InheritanceExists(String, String),
    
    /// Validation rule already exists
    #[error("Validation rule already exists: {0}")]
    ValidationRuleExists(String),
    
    /// Validation rule not found
    #[error("Validation rule not found: {0}")]
    ValidationRuleNotFound(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Expression error
    #[error("Expression error: {0}")]
    ExpressionError(String),
    
    /// Inheritance error
    #[error("Inheritance error: {0}")]
    InheritanceError(String),
    
    /// General error
    #[error("General error: {0}")]
    General(String),
}

/// Permission audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditEvent {
    /// Audit event ID
    pub id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// User ID
    pub user_id: String,
    
    /// Action performed
    pub action: String,
    
    /// Target resource
    pub resource: String,
    
    /// Result of the permission check
    pub result: String,
    
    /// Roles used
    pub roles: Vec<String>,
    
    /// Permissions evaluated
    pub permissions: Vec<String>,
    
    /// Context information
    pub context: HashMap<String, String>,
}

impl EnhancedRBACManager {
    /// Create a new enhanced RBAC manager
    pub fn new() -> Self {
        Self {
            rbac_manager: Arc::new(RwLock::new(RBACManager::new())),
            inheritance_manager: Arc::new(InheritanceManager::new()),
            permission_validator: Arc::new(AsyncPermissionValidator::new()),
            audit_enabled: true,
        }
    }
    
    /// Create from existing RBAC manager
    pub fn from_existing(rbac_manager: RBACManager) -> Self {
        Self {
            rbac_manager: Arc::new(RwLock::new(rbac_manager)),
            inheritance_manager: Arc::new(InheritanceManager::new()),
            permission_validator: Arc::new(AsyncPermissionValidator::new()),
            audit_enabled: true,
        }
    }
    
    /// Create a new role with enhanced inheritance
    pub async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Create role in base RBAC manager
        let manager = self.rbac_manager.write().await;
        
        // Create a new Role struct with a unique ID
        let role = Role {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            permissions: permissions.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            parent_roles: HashSet::new(),
            security_level: crate::types::SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
        };
        
        // Add the role to the RBAC manager
        manager.add_role(role.clone()).await?;
        
        // Add role to inheritance manager
        self.inheritance_manager.add_role(&role.id).await?;
        
        // Add direct inheritance relationships
        for parent_id in parent_roles {
            self.inheritance_manager
                .add_direct_inheritance(&parent_id, &role.id)
                .await?;
        }
        
        Ok(role)
    }
    
    /// Create filtered inheritance relationship
    pub async fn create_filtered_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        included_permissions: HashSet<String>,
        excluded_permissions: HashSet<String>,
    ) -> Result<()> {
        // Verify roles exist
        {
            let manager = self.rbac_manager.read().await;
            if manager.get_role(parent_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    parent_id.to_string()
                ))));
            }
            
            if manager.get_role(child_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    child_id.to_string()
                ))));
            }
        }
        
        // Create filtered inheritance
        self.inheritance_manager
            .add_filtered_inheritance(
                parent_id,
                child_id,
                included_permissions,
                excluded_permissions,
            )
            .await
    }
    
    /// Create conditional inheritance relationship
    pub async fn create_conditional_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        condition: String,
    ) -> Result<()> {
        // Verify roles exist
        {
            let manager = self.rbac_manager.read().await;
            if manager.get_role(parent_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    parent_id.to_string()
                ))));
            }
            
            if manager.get_role(child_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    child_id.to_string()
                ))));
            }
        }
        
        // Create conditional inheritance
        self.inheritance_manager
            .add_conditional_inheritance(parent_id, child_id, condition)
            .await
    }
    
    /// Create delegated inheritance relationship
    pub async fn create_delegated_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        delegator_id: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Verify roles exist
        {
            let manager = self.rbac_manager.read().await;
            if manager.get_role(parent_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    parent_id.to_string()
                ))));
            }
            
            if manager.get_role(child_id).await.is_err() {
                return Err(MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound(
                    child_id.to_string()
                ))));
            }
        }
        
        // Create delegated inheritance
        self.inheritance_manager
            .add_delegated_inheritance(parent_id, child_id, delegator_id, expires_at)
            .await
    }
    
    /// Assign role to user
    pub async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        let manager = self.rbac_manager.write().await;
        manager.assign_role_to_user(&user_id, &role_id).await
    }
    
    /// Add validation rule
    pub async fn add_validation_rule(&self, rule: ValidationRule) -> Result<()> {
        self.permission_validator.add_rule(rule.clone()).await?;
        
        info!("Added validation rule: {}", rule.id);
        Ok(())
    }
    
    /// Check if user has permission
    pub async fn has_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: Action,
        context: &PermissionContext,
    ) -> Result<ValidationResult> {
        // Get roles for user
        let roles = {
            let manager = self.rbac_manager.read().await;
            let user_role_ids = manager.get_user_roles(user_id).await?;
            
            user_role_ids
        };
        
        // Get permissions for user, including inherited permissions
        let mut all_permissions = HashSet::new();
        {
            let manager = self.rbac_manager.read().await;
            
            // Add base permissions from roles
            for role in &roles {
                all_permissions.extend(role.permissions.clone());
                
                // Add inherited permissions
                let role_map: HashMap<String, Role> = roles
                    .iter()
                    .map(|r| (r.id.clone(), r.clone()))
                    .collect();
                
                let inherited = self.inheritance_manager
                    .get_inherited_permissions(&role.id, &role_map, Some(context))
                    .await?;
                
                all_permissions.extend(inherited);
            }
        }
        
        // Validate permission
        let result = self.permission_validator
            .validate(user_id, resource, action, &roles, &all_permissions, context)
            .await;
        
        Ok(result)
    }
    
    /// Enable audit logging
    pub fn enable_audit(&mut self, enabled: bool) {
        self.audit_enabled = enabled;
    }
    
    /// Get audit records for a user
    pub async fn get_user_audit(&self, user_id: &str) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_user_audit(user_id).await
    }
    
    /// Get audit records for a resource
    pub async fn get_resource_audit(&self, resource: &str) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_resource_audit(resource).await
    }
    
    /// Get all audit records
    pub async fn get_all_audit(&self) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_all_audit().await
    }
    
    /// Clear audit records
    pub async fn clear_audit(&self) {
        self.permission_validator.clear_audit().await;
    }
    
    /// Set maximum audit log size
    pub async fn set_max_audit_size(&self, size: usize) {
        self.permission_validator.set_max_audit_size(size).await;
    }
    
    /// Get role inheritance diagram as DOT format
    pub async fn get_inheritance_diagram(&self) -> String {
        self.inheritance_manager.to_dot().await
    }
} 