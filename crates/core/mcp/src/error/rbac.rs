// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

/// Errors related to Role-Based Access Control (RBAC) operations
///
/// This enum represents errors that can occur when working with RBAC,
/// including role and permission management, assignment, and validation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RBACError {
    /// Returned when attempting to create a role that already exists
    #[error("Role already exists: {0}")]
    RoleExists(String),
    /// Returned when the specified role was not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    /// Returned when role creation failed
    #[error("Failed to create role: {0}")]
    RoleCreationFailed(String),
    /// Returned when permission already exists in the role
    #[error("Permission already exists in role: {0}")]
    PermissionExists(String),
    /// Returned when permission was not found in the role
    #[error("Permission not found in role: {0}")]
    PermissionNotFound(String),
    /// Returned when the inheritance relationship already exists
    #[error("Inheritance relationship already exists")]
    InheritanceExists,
    /// Returned when the inheritance relationship was not found
    #[error("Inheritance relationship not found")]
    InheritanceNotFound,
    /// Returned when adding inheritance would create a cycle
    #[error("Adding inheritance would create a cycle: {0} -> {1}")]
    CycleDetected(String, String),
    /// Returned when the validation rule already exists
    #[error("Validation rule already exists: {0}")]
    ValidationRuleExists(String),
    /// Returned when the validation rule was not found
    #[error("Validation rule not found: {0}")]
    ValidationRuleNotFound(String),
    /// Returned when role assignment failed
    #[error("Failed to assign role: {0}")]
    AssignmentError(String),
    /// Returned when removing role assignment failed
    #[error("Failed to remove role assignment: {0}")]
    UnassignmentError(String),
    /// Returned when an inheritance operation failed
    #[error("Inheritance operation failed: {0}")]
    InheritanceError(String),
    /// Returned when delegation is not allowed for the role
    #[error("Delegation not allowed for role: {0}")]
    DelegationNotAllowed(String),
    /// Returned when the permission format is invalid
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    /// General RBAC error for uncategorized failures
    #[error("General RBAC error: {0}")]
    General(String),
}
