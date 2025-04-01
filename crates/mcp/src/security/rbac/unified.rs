//! Unified RBAC Manager trait and supporting types.
//!
//! This module provides a consolidated trait for Role-Based Access Control (RBAC)
//! operations, replacing the previous multi-trait hierarchy with a single unified interface.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
// Remove unused imports
// use std::collections::{HashMap, HashSet};
// use uuid::Uuid;

use crate::error::Result;
use crate::context_manager::Context;

/// Represents a role definition in the RBAC system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDefinition {
    /// Unique identifier for the role
    pub id: String,
    
    /// Human-readable name for the role
    pub name: String,
    
    /// Description of what this role represents
    pub description: String,
    
    /// Whether this is a system role (cannot be modified)
    pub is_system_role: bool,
    
    /// When the role was created
    pub created_at: DateTime<Utc>,
    
    /// When the role was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a permission definition in the RBAC system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionDefinition {
    /// Unique identifier for the permission
    pub id: String,
    
    /// Human-readable name for the permission
    pub name: String,
    
    /// Description of what this permission represents
    pub description: String,
}

/// Represents a permission assigned to a role
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RolePermission {
    /// The permission identifier (e.g., "document:read")
    pub permission_id: String,
    
    /// When the permission was granted to the role
    pub granted_at: DateTime<Utc>,
    
    /// Optional expiration time for the permission
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response structure for role details queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDetailsResponse {
    /// The role definition
    pub role: RoleDefinition,
    
    /// Permissions assigned to this role
    pub permissions: Vec<RolePermission>,
    
    /// Parent roles from which this role inherits permissions
    pub parent_roles: Vec<String>,
    
    /// Child roles that inherit from this role
    pub child_roles: Vec<String>,
}

/// Unified RBAC Manager trait that consolidates core and enhanced functionality
///
/// This trait provides a single interface for all RBAC operations, simplifying
/// the implementation and usage of the RBAC system throughout the application.
#[async_trait]
pub trait RBACManager: Send + Sync + std::fmt::Debug {
    /// Get the manager's name
    fn name(&self) -> &str;
    
    /// Get the version of the manager
    fn version(&self) -> &str;
    
    /// Check if a user has a specific permission
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to check permissions for
    /// * `permission` - The permission to check (format: "action:resource")
    /// * `context` - Optional context for contextual permission evaluation
    ///
    /// # Returns
    /// * `Ok(true)` if the user has the permission
    /// * `Ok(false)` if the user does not have the permission
    /// * `Err` if an error occurred during the check
    async fn has_permission(&self, user_id: &str, _permission: &str, context: Option<&Context>) -> Result<bool>;
    
    /// Assign a role to a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to assign the role to
    /// * `role_id` - The ID of the role to assign
    ///
    /// # Returns
    /// * `Ok(())` if the role was assigned successfully
    /// * `Err` if an error occurred during assignment
    async fn assign_role(&self, user_id: &str, _role_id: &str) -> Result<()>;
    
    /// Revoke a role from a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to revoke the role from
    /// * `role_id` - The ID of the role to revoke
    ///
    /// # Returns
    /// * `Ok(())` if the role was revoked successfully
    /// * `Err` if an error occurred during revocation
    async fn revoke_role(&self, user_id: &str, _role_id: &str) -> Result<()>;
    
    /// Get all roles assigned to a user
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to get roles for
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` with the list of role IDs assigned to the user
    /// * `Err` if an error occurred during retrieval
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    
    /// Check if a user has a specific role
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to check
    /// * `role_id` - The ID of the role to check for
    ///
    /// # Returns
    /// * `Ok(true)` if the user has the role
    /// * `Ok(false)` if the user does not have the role
    /// * `Err` if an error occurred during the check
    async fn has_role(&self, user_id: &str, _role_id: &str) -> Result<bool>;
    
    /// Get detailed information about a role
    ///
    /// # Arguments
    /// * `role_id` - The ID of the role to get details for
    ///
    /// # Returns
    /// * `Ok(Some(RoleDetailsResponse))` with the role details if found
    /// * `Ok(None)` if the role does not exist
    /// * `Err` if an error occurred during retrieval
    async fn get_role_details(&self, _role_id: &str) -> Result<Option<RoleDetailsResponse>> {
        // Default implementation returns None
        Ok(None)
    }
    
    /// Get all permissions assigned to a role
    ///
    /// # Arguments
    /// * `role_id` - The ID of the role to get permissions for
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` with the list of permission IDs assigned to the role
    /// * `Err` if an error occurred during retrieval
    async fn get_permissions_for_role(&self, _role_id: &str) -> Result<Vec<String>> {
        // Default implementation returns an empty list
        Ok(Vec::new())
    }
    
    /// Create a new role
    ///
    /// # Arguments
    /// * `role_id` - The ID for the new role
    /// * `name` - The name for the new role
    /// * `description` - The description for the new role
    ///
    /// # Returns
    /// * `Ok(())` if the role was created successfully
    /// * `Err` if an error occurred during creation
    async fn create_role(&self, _role_id: &str, _name: &str, _description: &str) -> Result<()> {
        // Default implementation returns an error
        Err(crate::error::SecurityError::Unsupported("Role creation not supported by this RBAC manager".to_string()).into())
    }
    
    /// Add a permission to a role
    ///
    /// # Arguments
    /// * `role_id` - The ID of the role to add the permission to
    /// * `permission` - The permission to add (format: "action:resource")
    ///
    /// # Returns
    /// * `Ok(())` if the permission was added successfully
    /// * `Err` if an error occurred during addition
    async fn add_permission_to_role(&self, _role_id: &str, _permission: &str) -> Result<()> {
        // Default implementation returns an error
        Err(crate::error::SecurityError::Unsupported("Adding permissions not supported by this RBAC manager".to_string()).into())
    }
}