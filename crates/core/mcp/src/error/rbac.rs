// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

use thiserror::Error;

/// Errors related to Role-Based Access Control (RBAC) operations
///
/// This enum represents errors that can occur when working with RBAC,
/// including role and permission management, assignment, and validation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RBACError {
    #[error("Role already exists: {0}")]
    RoleExists(String),
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    #[error("Failed to create role: {0}")]
    RoleCreationFailed(String),
    #[error("Permission already exists in role: {0}")]
    PermissionExists(String),
    #[error("Permission not found in role: {0}")]
    PermissionNotFound(String),
    #[error("Inheritance relationship already exists")]
    InheritanceExists,
    #[error("Inheritance relationship not found")]
    InheritanceNotFound,
    #[error("Adding inheritance would create a cycle: {0} -> {1}")]
    CycleDetected(String, String),
    #[error("Validation rule already exists: {0}")]
    ValidationRuleExists(String),
    #[error("Validation rule not found: {0}")]
    ValidationRuleNotFound(String),
    #[error("Failed to assign role: {0}")]
    AssignmentError(String),
    #[error("Failed to remove role assignment: {0}")]
    UnassignmentError(String),
    #[error("Inheritance operation failed: {0}")]
    InheritanceError(String),
    #[error("Delegation not allowed for role: {0}")]
    DelegationNotAllowed(String),
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
    #[error("General RBAC error: {0}")]
    General(String),
}
