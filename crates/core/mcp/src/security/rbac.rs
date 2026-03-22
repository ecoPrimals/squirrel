// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Role-Based Access Control (RBAC) for MCP security
//!
//! This module provides RBAC functionality for the MCP system.
//! Actual RBAC operations are delegated to the BearDog framework.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Permission definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Resource identifier (e.g. `workflow`, `message`).
    pub resource: String,
    /// Action name (e.g. `Read`, `Write`).
    pub action: String,
}

impl Permission {
    /// Creates a permission for the given resource and action.
    #[must_use]
    pub const fn new(resource: String, action: String) -> Self {
        Self { resource, action }
    }
}

/// Role definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Stable role id.
    pub id: Uuid,
    /// Short role name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Direct permissions granted to this role.
    pub permissions: HashSet<Permission>,
    /// Parent role ids for permission inheritance.
    pub parent_roles: HashSet<Uuid>,
}

impl Role {
    /// Creates an empty role with a new id and the given name and description.
    #[must_use]
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
        }
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    /// User receiving the role.
    pub user_id: Uuid,
    /// Role being granted.
    pub role_id: Uuid,
    /// Administrator or system id that granted the role.
    pub granted_by: Uuid,
    /// When the grant was recorded.
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

/// Basic RBAC manager implementation
///
/// This provides basic RBAC functionality that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct BasicRBACManager {
    roles: Arc<RwLock<HashMap<Uuid, Role>>>,
    user_roles: Arc<RwLock<HashMap<Uuid, HashSet<Uuid>>>>,
    role_assignments: Arc<RwLock<Vec<UserRoleAssignment>>>,
}

impl BasicRBACManager {
    /// Create a new RBAC manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            roles: Arc::new(RwLock::new(HashMap::new())),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            role_assignments: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Inserts a new role into the registry.
    pub async fn create_role(&self, name: String, description: String) -> Result<Role> {
        let role = Role::new(name, description);
        let mut roles = self.roles.write().await;
        roles.insert(role.id, role.clone());
        Ok(role)
    }

    /// Returns the role by id, if present.
    pub async fn get_role(&self, id: &Uuid) -> Result<Option<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.get(id).cloned())
    }

    /// Looks up a role by its name.
    pub async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().find(|r| r.name == name).cloned())
    }

    /// Replaces the stored role definition.
    pub async fn update_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.id, role);
        Ok(())
    }

    /// Deletes a role from the registry.
    pub async fn delete_role(&self, id: &Uuid) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.remove(id);
        Ok(())
    }

    /// Adds a permission to a role's direct set.
    pub async fn add_permission_to_role(
        &self,
        role_id: &Uuid,
        permission: Permission,
    ) -> Result<()> {
        let mut roles = self.roles.write().await;
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.insert(permission);
        }
        Ok(())
    }

    /// Removes a permission from a role.
    pub async fn remove_permission_from_role(
        &self,
        role_id: &Uuid,
        permission: &Permission,
    ) -> Result<()> {
        let mut roles = self.roles.write().await;
        if let Some(role) = roles.get_mut(role_id) {
            role.permissions.remove(permission);
        }
        Ok(())
    }

    /// Grants a role to a user and appends an audit row.
    pub async fn assign_role_to_user(
        &self,
        user_id: &Uuid,
        role_id: &Uuid,
        granted_by: &Uuid,
    ) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        user_roles
            .entry(*user_id)
            .or_insert_with(HashSet::new)
            .insert(*role_id);

        let mut assignments = self.role_assignments.write().await;
        assignments.push(UserRoleAssignment {
            user_id: *user_id,
            role_id: *role_id,
            granted_by: *granted_by,
            granted_at: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Revokes a role from a user.
    pub async fn remove_role_from_user(&self, user_id: &Uuid, role_id: &Uuid) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        Ok(())
    }

    /// Returns role ids currently assigned to the user.
    pub async fn get_user_roles(&self, user_id: &Uuid) -> Result<HashSet<Uuid>> {
        let user_roles = self.user_roles.read().await;
        Ok(user_roles.get(user_id).cloned().unwrap_or_default())
    }

    /// Returns true if any of the user's roles (including parents) grant the permission.
    pub async fn check_permission(
        &self,
        user_id: &Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool> {
        let user_role_ids = self.get_user_roles(user_id).await?;
        let roles = self.roles.read().await;

        let permission = Permission::new(resource.to_string(), action.to_string());

        for role_id in user_role_ids {
            if let Some(role) = roles.get(&role_id) {
                if role.permissions.contains(&permission) {
                    return Ok(true);
                }

                // Check parent roles recursively
                if self
                    .check_permission_in_parent_roles(&role.parent_roles, &permission, &roles)
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Recursively checks parent roles for a permission match.
    async fn check_permission_in_parent_roles(
        &self,
        parent_roles: &HashSet<Uuid>,
        permission: &Permission,
        roles: &HashMap<Uuid, Role>,
    ) -> Result<bool> {
        for parent_role_id in parent_roles {
            if let Some(parent_role) = roles.get(parent_role_id) {
                if parent_role.permissions.contains(permission) {
                    return Ok(true);
                }

                // Check parent's parent roles recursively (boxed to avoid infinite future size)
                if Box::pin(self.check_permission_in_parent_roles(
                    &parent_role.parent_roles,
                    permission,
                    roles,
                ))
                .await?
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Returns every role definition in the registry.
    pub async fn get_all_roles(&self) -> Result<Vec<Role>> {
        let roles = self.roles.read().await;
        Ok(roles.values().cloned().collect())
    }

    /// Returns assignment records for the given user.
    pub async fn get_user_role_assignments(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<UserRoleAssignment>> {
        let assignments = self.role_assignments.read().await;
        Ok(assignments
            .iter()
            .filter(|a| a.user_id == *user_id)
            .cloned()
            .collect())
    }
}

impl Default for BasicRBACManager {
    fn default() -> Self {
        Self::new()
    }
}
