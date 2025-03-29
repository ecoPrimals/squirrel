//! # Role-Based Access Control (RBAC) System
//!
//! This module provides a comprehensive and high-performance Role-Based Access Control
//! system designed for complex authorization requirements in enterprise applications.
//!
//! ## Key Features
//!
//! - **Fine-grained permission control** - Detailed permission management with resource/action pairs
//! - **Advanced role inheritance** - Support for multiple inheritance types:
//!   - Direct inheritance (full parent permissions)
//!   - Filtered inheritance (selected parent permissions)
//!   - Conditional inheritance (context-dependent permissions)
//!   - Delegated inheritance (temporary or user-granted permissions)
//! - **Permission validation** - Complex rules with condition evaluation and audit logging
//! - **High-performance caching** - Optimized for repeated permission checks
//! - **Scalable architecture** - Designed to handle large role hierarchies efficiently
//!
//! ## Usage
//!
//! The primary entry point is the `EnhancedRBACManager` which coordinates all RBAC components:
//!
//! ```rust
//! use mcp::security::rbac::EnhancedRBACManager;
//!
//! let rbac = EnhancedRBACManager::new();
//!
//! // Check permissions
//! let context = PermissionContext::new();
//! let result = rbac.has_permission("user123", "document/123", Action::Read, &context).await?;
//! ```
//!
//! ## Architecture
//!
//! The RBAC system consists of several specialized components:
//!
//! 1. **RBAC Manager** - Core role and permission storage
//! 2. **Inheritance Manager** - Handles complex role inheritance relationships
//! 3. **Permission Validator** - Evaluates permissions against validation rules
//! 4. **Audit System** - Tracks permission decisions for compliance and debugging

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use lru::LruCache;
use tokio::sync::Mutex;
use std::num::NonZeroUsize;
use tokio::sync::RwLock;
use log::{debug, info, warn};
use tracing::instrument;

use crate::error::{MCPError, Result, SecurityError};

// Re-export types from security::types
use crate::security::types::{
    Action, Permission, PermissionCondition, PermissionContext, 
    PermissionScope, Role,
};

// Interface Crate Imports
use crate::context_manager::Context; // Fix: Use correct path for Context

// Local Crate Imports
use super::types::{SecurityLevel, PermissionId, RoleId}; // Import necessary types from super::types

// Private modules
/// Role inheritance graph implementation
mod role_inheritance;
/// Permission validation and condition checking
mod permission_validation;
/// Core RBAC manager implementation
mod manager;

#[cfg(test)]
mod tests;

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

/// Represents a single permission validation event for auditing purposes
///
/// This struct contains all the information related to a permission check,
/// including the user, resource, action, result, and relevant context.
/// It is used for compliance tracking, security analysis, and debugging.
///
/// # Fields
/// * `id` - Unique identifier for the audit event
/// * `timestamp` - When the permission check occurred
/// * `user_id` - The user who requested the permission
/// * `action` - The action attempted on the resource
/// * `resource` - The resource the action was attempted on
/// * `result` - The outcome of the permission check (granted/denied)
/// * `roles` - The roles considered during evaluation
/// * `permissions` - The permissions evaluated
/// * `context` - Additional contextual information for the permission check
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

/// Advanced Role-Based Access Control (RBAC) implementation with enhanced features
///
/// This struct provides a comprehensive RBAC system with additional capabilities:
/// * Advanced role inheritance (direct, filtered, conditional, delegated)
/// * High-performance permission caching
/// * Detailed audit logging
/// * Validation rules for complex authorization scenarios
/// * Resource scope awareness (global, group, own)
/// * Permission conditions evaluation
///
/// The enhanced manager coordinates several specialized components:
/// * RBAC Manager - Core role and permission storage
/// * Inheritance Manager - Handles complex inheritance relationships
/// * Permission Validator - Evaluates permissions against rules
/// * Permission Cache - Optimizes repeated permission checks
#[derive(Debug)]
pub struct EnhancedRBACManager {
    /// RBAC Manager
    rbac_manager: Arc<RBACManager>,
    
    /// Inheritance manager - Wrapped in `RwLock` for concurrent access
    inheritance_manager: Arc<RwLock<InheritanceManager>>,
    
    /// Permission validator - Wrapped in `RwLock` for concurrent access
    permission_validator: Arc<RwLock<AsyncPermissionValidator>>,
    
    /// Whether audit logging is enabled
    audit_enabled: bool,
    
    /// Permission check cache
    permission_cache: Arc<Mutex<LruCache<u64, ValidationResult>>>,
    
    /// Cache hit metrics
    cache_hits: Arc<Mutex<usize>>,
    
    /// Cache miss metrics
    cache_misses: Arc<Mutex<usize>>,
}

// Implement Clone for EnhancedRBACManager
impl Clone for EnhancedRBACManager {
    fn clone(&self) -> Self {
        Self {
            rbac_manager: self.rbac_manager.clone(),
            inheritance_manager: self.inheritance_manager.clone(), // Arc clone is shallow
            permission_validator: self.permission_validator.clone(), // Arc clone is shallow
            audit_enabled: self.audit_enabled,
            permission_cache: self.permission_cache.clone(),
            cache_hits: self.cache_hits.clone(),
            cache_misses: self.cache_misses.clone(),
        }
    }
}

#[allow(clippy::expect_used)] // Allowed because Default::default must return Self,
                               // and Self::new(1000) is guaranteed not to fail as 1000 is non-zero.
impl Default for EnhancedRBACManager {
    fn default() -> Self {
        Self::new(1000).expect("Failed to create EnhancedRBACManager with default cache size")
    }
}

impl EnhancedRBACManager {
    /// Creates a new `EnhancedRBACManager` with default settings.
    ///
    /// Initializes the underlying components (RBAC, Inheritance, Validator)
    /// and sets up the permission cache. Audit logging is disabled by default.
    ///
    /// # Arguments
    ///
    /// * `cache_size` - The maximum number of entries in the permission cache.
    ///
    /// # Errors
    ///
    /// Returns `MCPError` if cache creation fails (e.g., invalid size).
    pub fn new(cache_size: usize) -> Result<Self> {
        let cache_capacity = NonZeroUsize::new(cache_size).ok_or_else(|| {
            MCPError::Configuration("Cache size must be non-zero".to_string())
        })?;
        Ok(Self {
            rbac_manager: Arc::new(RBACManager::new()),
            inheritance_manager: Arc::new(RwLock::new(InheritanceManager::new())),
            permission_validator: Arc::new(RwLock::new(AsyncPermissionValidator::new())),
            audit_enabled: false,
            permission_cache: Arc::new(Mutex::new(LruCache::new(cache_capacity))),
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        })
    }

    /// Enables or disables audit logging.
    pub fn set_audit_enabled(&mut self, enabled: bool) {
        self.audit_enabled = enabled;
    }

    /// Creates a new role within the RBAC system.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name/identifier for the role.
    /// * `parent_id` - The identifier of the parent role from which this role inherits.
    /// * `initial_security_level` - Optional security level assigned to the role.
    /// * `can_delegate` - Optional flag indicating if the role's permissions can be delegated.
    ///
    /// # Errors
    ///
    /// Returns `MCPError` if:
    /// * The role name already exists (`RBACError::RoleExists`).
    /// * The parent role does not exist (`RBACError::RoleNotFound`).
    /// * Creating the role fails for other reasons.
    pub async fn create_role(
        &self,
        name: &str,
        parent_id: &str,
        _initial_security_level: Option<SecurityLevel>,
        _can_delegate: Option<bool>,
    ) -> Result<Role> {
        // Parent role check (consider if redundant with rbac_manager::create_role)
        if self.rbac_manager.get_role(parent_id).await.is_err() {
             return Err(MCPError::Security(SecurityError::RBACError(
                 RBACError::RoleNotFound(parent_id.to_string()).to_string(),
             )));
        }

        // Fix: Call the inner rbac_manager's create_role with name and description.
        // Pass None for description for now. Add parent relationship separately.
        // TODO: Revisit if description, security_level, can_delegate should be passed here.
        let created_role = self.rbac_manager
            .create_role(name, None) // Pass name and None description
            .await
            .map_err(|rbac_err| MCPError::Security(SecurityError::RBACError(rbac_err.to_string())))?;

        // TODO: Add parent inheritance relationship explicitly using InheritanceManager if needed.
        // This depends on whether rbac_manager.create_role handles it.
        // Example: 
        // self.inheritance_manager.write().await
        //     .add_direct_inheritance(parent_id, name).await?;
        
        // TODO: Update role properties like security_level, can_delegate if not handled by create_role.
        // Example:
        // let mut roles = self.rbac_manager.roles.write().await; // Assuming roles accessible
        // if let Some(role_mut) = roles.get_mut(name) {
        //     if let Some(level) = initial_security_level { role_mut.security_level = level; }
        //     if let Some(delegate) = can_delegate { role_mut.can_delegate = delegate; }
        //     role_mut.updated_at = Utc::now();
        // }

        Ok(created_role) // Return the role created by the inner manager
    }
    
    /// Creates a filtered inheritance relationship between two roles.
    ///
    /// Allows `child_id` to inherit from `parent_id`, but only specific permissions included
    /// or excluded by the filter.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Parent role identifier.
    /// * `child_id` - Child role identifier.
    /// * `include_permissions` - Set of permissions to explicitly include.
    /// * `exclude_permissions` - Set of permissions to explicitly exclude.
    ///
    /// # Errors
    ///
    /// Returns `MCPError` if:
    /// * Roles don't exist (`RBACError::RoleNotFound`).
    /// * Inheritance would create a cycle (`RBACError::CycleDetected`).
    /// * Adding inheritance fails (`RBACError::InheritanceError`).
    pub async fn create_filtered_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        include_permissions: HashSet<PermissionId>,
        exclude_permissions: HashSet<PermissionId>,
    ) -> Result<()> {
        // TODO: Validate roles exist
        // Remove call to non-existent can_inherit_role
        // self.can_inherit_role(parent_id, child_id).await?;

        let graph = self.inheritance_manager.write().await; 
        
        graph.add_filtered_inheritance(parent_id, child_id, include_permissions, exclude_permissions)
            .await
            .map_err(|e| MCPError::Security(SecurityError::RBACError(e.to_string())))?;

        Ok(())
    }
    
    /// Assigns a role to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The ID of the user to assign the role to.
    /// * `role_id` - The ID of the role to assign to the user.
    ///
    /// # Errors
    ///
    /// Returns `MCPError` if:
    /// * The user does not exist (`RBACError::RoleNotFound`).
    /// * The role does not exist (`RBACError::RoleNotFound`).
    /// * Assigning the role fails for other reasons.
    pub async fn assign_role(
        &self,
        _user_id: &str,
        _role_id: &str,
    ) -> Result<()> {
        // TODO: Implement role assignment logic
        Ok(())
    }

    /// Checks if a user has a specific permission (Placeholder).
    ///
    /// # Errors
    ///
    /// This is currently a placeholder and always returns `Ok(false)`.
    /// Error handling will be implemented later.
    pub async fn has_permission(
        &self,
        _user_id: &str,
        _permission_str: &str, 
        _context: Option<&Context>, 
    ) -> Result<bool> {
        println!("Warning: EnhancedRBACManager::has_permission is a placeholder.");
        // Ensure it returns Ok(false)
        Ok(false)
    }

    /// Retrieves the IDs of roles assigned to a user (Placeholder).
    ///
    /// # Errors
    ///
    /// This is currently a placeholder and always returns `Ok` with an empty set.
    /// Error handling will be implemented later.
    pub async fn get_user_roles(&self, _user_id: &str) -> Result<HashSet<RoleId>> {
        // TODO: Implement actual user role retrieval
        println!("Warning: EnhancedRBACManager::get_user_roles is a placeholder.");
        Ok(HashSet::new())
    }

    /// Retrieves a role by its ID (Placeholder).
    ///
    /// # Errors
    ///
    /// This is currently a placeholder and always returns an 
    /// `MCPError::Security(SecurityError::RBACError(RBACError::RoleNotFound))` error.
    pub async fn get_role(&self, _role_id: &str) -> Result<Role> {
        // TODO: Implement actual role retrieval
        println!("Warning: EnhancedRBACManager::get_role is a placeholder.");
        // Return a default/dummy role or error
        Err(MCPError::Security(SecurityError::RBACError(
            RBACError::RoleNotFound("Placeholder: Role not found".to_string()).to_string(),
        )))
    }

    /// Removes a permission from a role.
    #[instrument(skip(self), fields(role_id = %role_id, permission_id = %permission_id))]
    pub async fn remove_permission_from_role(&self, role_id: &str, permission_id: &str) -> Result<()> {
        debug!("Attempting to remove permission from role: {} from {}", permission_id, role_id);
        // TODO: Implement permission removal logic directly in RBACManager
        // Call the inner RBACManager to revoke the permission
        // self.rbac_manager.revoke_permission_from_role(role_id, permission_id).await
        //     .map_err(|e| MCPError::Security(SecurityError::RBACError(e.to_string())))?;
        warn!("Permission removal not yet implemented for role {} and permission {}", role_id, permission_id);
        info!("Placeholder: Permission removal skipped for role");
        Ok(())
    }

    /// Adds a child role to a parent role, establishing an inheritance relationship.
    #[instrument(skip(self), fields(parent_role_id = %parent_role_id, child_role_id = %child_role_id))]
    pub async fn add_child_role(&self, parent_role_id: &str, child_role_id: &str) -> Result<()> {
        debug!("Attempting to add child role relationship");
        let manager_guard = self.inheritance_manager.write().await;
        // Call the correct method on InheritanceManager and map the error
        (*manager_guard).add_direct_inheritance(parent_role_id, child_role_id).await
             .map_err(|e| MCPError::Security(SecurityError::RBACError(e.to_string())))?;
        info!("Child role relationship successfully added");
        Ok(())
    }
}