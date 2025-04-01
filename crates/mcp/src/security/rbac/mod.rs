//! # Role-Based Access Control (RBAC) System
//!
//! This module provides a comprehensive Role-Based Access Control
//! system designed for authorization requirements in MCP applications.
//!
//! ## New Unified Architecture
//! 
//! The RBAC system has been restructured to reduce complexity and improve maintainability.
//! It now uses a unified `RBACManager` trait that consolidates functionality previously 
//! spread across multiple traits.
//!
//! ## Key Features
//!
//! - **Fine-grained permission control** - Detailed permission management
//! - **Role-based authorization** - Assign permissions through roles
//! - **Flexible implementation options** - Basic, cached, and advanced implementations
//! - **Testing support** - Mock implementation for unit testing
//!
//! ## Usage
//!
//! The primary entry point is the `RBACManager` trait which defines the interface
//! for all RBAC operations:
//!
//! ```rust
//! use mcp::security::rbac::{RBACManager, BasicRBACManager};
//!
//! let rbac = BasicRBACManager::new();
//!
//! // Check permissions
//! let has_permission = rbac.has_permission("user123", "document:read", None).await?;
//! ```

// Public modules
pub mod unified;
mod basic;
mod mock;
pub mod manager;
pub mod permission_validation;
mod role_inheritance;
mod tests;

// Re-export main components
pub use unified::RBACManager;
pub use basic::BasicRBACManager;
pub use mock::MockRBACManager;
pub use manager::RBACManagerImpl;

// Re-export types from the unified module
pub use unified::{
    RoleDefinition,
    PermissionDefinition,
    RolePermission,
    RoleDetailsResponse,
};

// Re-export types from permission_validation
pub use permission_validation::{
    ValidationResult,
    ValidationRule,
    ValidationAuditRecord,
};

// Re-export types from role_inheritance
pub use role_inheritance::InheritanceType;

// Error types

// Re-export types from other modules
// This ensures RBAC users can access these types through the rbac module
pub use crate::security::types::{Action, Resource};

// Permission type definition - can be moved to a separate file if needed
/// Represents a permission in the RBAC system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    
    /// Human-readable name for the permission
    pub name: String,
    
    /// Resource type this permission applies to
    pub resource: String,
    
    /// Action this permission applies to
    pub action: Action,
    
    /// Specific resource ID this permission applies to, if any
    pub resource_id: Option<String>,
}