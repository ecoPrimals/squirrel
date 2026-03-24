// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_new_and_debug() {
        let p = Permission::new("workflow".to_string(), "Read".to_string());
        assert_eq!(p.resource, "workflow");
        assert_eq!(p.action, "Read");
        let dbg = format!("{p:?}");
        assert!(dbg.contains("workflow"));
    }

    #[test]
    fn permission_serde_round_trip() {
        let p = Permission::new(String::new(), "Write".to_string());
        let json = serde_json::to_string(&p).expect("permission serializes");
        let back: Permission = serde_json::from_str(&json).expect("permission deserializes");
        assert_eq!(p, back);
    }

    #[test]
    fn role_new_serde_round_trip() {
        let mut role = Role::new("admin".to_string(), "desc".to_string());
        role.permissions
            .insert(Permission::new("r".to_string(), "a".to_string()));
        let id = role.id;
        let json = serde_json::to_string(&role).expect("role serializes");
        let back: Role = serde_json::from_str(&json).expect("role deserializes");
        assert_eq!(back.id, id);
        assert_eq!(back.name, "admin");
        assert_eq!(back.description, "desc");
        assert_eq!(back.permissions, role.permissions);
        assert!(back.parent_roles.is_empty());
    }

    #[test]
    fn user_role_assignment_serde_round_trip() {
        let a = UserRoleAssignment {
            user_id: Uuid::nil(),
            role_id: Uuid::max(),
            granted_by: Uuid::new_v4(),
            granted_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&a).expect("assignment serializes");
        let back: UserRoleAssignment =
            serde_json::from_str(&json).expect("assignment deserializes");
        assert_eq!(back.user_id, a.user_id);
        assert_eq!(back.role_id, a.role_id);
        assert_eq!(back.granted_by, a.granted_by);
        assert_eq!(back.granted_at, a.granted_at);
    }

    #[tokio::test]
    async fn basic_rbac_manager_new_default_and_role_crud() {
        let m = BasicRBACManager::new();
        assert!(!format!("{m:?}").is_empty());
        let d = BasicRBACManager::default();
        let role = d
            .create_role("r1".to_string(), "d1".to_string())
            .await
            .expect("create_role");
        assert_eq!(role.name, "r1");
        let fetched = d
            .get_role(&role.id)
            .await
            .expect("get_role")
            .expect("role exists");
        assert_eq!(fetched.id, role.id);
        assert_eq!(
            d.get_role_by_name("r1")
                .await
                .expect("get_role_by_name")
                .expect("role by name")
                .id,
            role.id
        );
        assert!(
            d.get_role(&Uuid::new_v4())
                .await
                .expect("get_role")
                .is_none()
        );
        assert!(
            d.get_role_by_name("nope")
                .await
                .expect("get_role_by_name")
                .is_none()
        );

        let mut updated = fetched.clone();
        updated.description = "updated".to_string();
        d.update_role(updated.clone()).await.expect("update_role");
        assert_eq!(
            d.get_role(&role.id)
                .await
                .expect("get_role")
                .expect("role exists")
                .description,
            "updated"
        );

        d.delete_role(&role.id).await.expect("delete_role");
        assert!(d.get_role(&role.id).await.expect("get_role").is_none());
    }

    #[tokio::test]
    async fn add_remove_permission_and_missing_role_noop() {
        let m = BasicRBACManager::new();
        let role = m
            .create_role("x".to_string(), String::new())
            .await
            .expect("create_role");
        let perm = Permission::new("res".to_string(), "act".to_string());
        m.add_permission_to_role(&role.id, perm.clone())
            .await
            .expect("add_permission_to_role");
        let r = m
            .get_role(&role.id)
            .await
            .expect("get_role")
            .expect("role exists");
        assert!(r.permissions.contains(&perm));

        let bogus = Uuid::new_v4();
        m.add_permission_to_role(&bogus, Permission::new("a".to_string(), "b".to_string()))
            .await
            .expect("add_permission noop for missing role");

        m.remove_permission_from_role(&role.id, &perm)
            .await
            .expect("remove_permission_from_role");
        assert!(
            !m.get_role(&role.id)
                .await
                .expect("get_role")
                .expect("role exists")
                .permissions
                .contains(&perm)
        );

        m.remove_permission_from_role(&bogus, &perm)
            .await
            .expect("remove_permission noop");
    }

    #[tokio::test]
    async fn assign_roles_and_check_permission_direct() {
        let m = BasicRBACManager::new();
        let role = m
            .create_role("editor".to_string(), String::new())
            .await
            .expect("create_role");
        let perm = Permission::new("doc".to_string(), "edit".to_string());
        m.add_permission_to_role(&role.id, perm.clone())
            .await
            .expect("add_permission_to_role");

        let user = Uuid::new_v4();
        let admin = Uuid::new_v4();
        assert!(
            m.get_user_roles(&user)
                .await
                .expect("get_user_roles")
                .is_empty()
        );

        m.assign_role_to_user(&user, &role.id, &admin)
            .await
            .expect("assign_role_to_user");
        assert!(
            m.get_user_roles(&user)
                .await
                .expect("get_user_roles")
                .contains(&role.id)
        );

        assert!(
            m.check_permission(&user, "doc", "edit")
                .await
                .expect("check_permission")
        );
        assert!(
            !m.check_permission(&user, "doc", "read")
                .await
                .expect("check_permission")
        );
        assert!(
            !m.check_permission(&Uuid::new_v4(), "doc", "edit")
                .await
                .expect("check_permission")
        );

        m.remove_role_from_user(&user, &role.id)
            .await
            .expect("remove_role_from_user");
        assert!(
            !m.check_permission(&user, "doc", "edit")
                .await
                .expect("check_permission")
        );
        m.remove_role_from_user(&user, &role.id)
            .await
            .expect("remove_role_from_user idempotent");
    }

    #[tokio::test]
    async fn check_permission_inherited_via_parent_role() {
        let m = BasicRBACManager::new();
        let parent = m
            .create_role("parent".to_string(), String::new())
            .await
            .expect("create_role");
        let perm = Permission::new("api".to_string(), "invoke".to_string());
        m.add_permission_to_role(&parent.id, perm)
            .await
            .expect("add_permission_to_role");

        let mut child = m
            .create_role("child".to_string(), String::new())
            .await
            .expect("create_role");
        child.parent_roles.insert(parent.id);
        m.update_role(child.clone()).await.expect("update_role");

        let user = Uuid::new_v4();
        m.assign_role_to_user(&user, &child.id, &Uuid::new_v4())
            .await
            .expect("assign_role_to_user");

        assert!(
            m.check_permission(&user, "api", "invoke")
                .await
                .expect("check_permission")
        );
    }

    #[tokio::test]
    async fn get_all_roles_and_user_assignments() {
        let m = BasicRBACManager::new();
        let u = Uuid::new_v4();
        let r = m
            .create_role("a".to_string(), String::new())
            .await
            .expect("create_role");
        m.assign_role_to_user(&u, &r.id, &Uuid::new_v4())
            .await
            .expect("assign_role_to_user");

        let all = m.get_all_roles().await.expect("get_all_roles");
        assert_eq!(all.len(), 1);
        let assigns = m
            .get_user_role_assignments(&u)
            .await
            .expect("get_user_role_assignments");
        assert_eq!(assigns.len(), 1);
        assert_eq!(assigns[0].user_id, u);
        assert_eq!(assigns[0].role_id, r.id);
    }
}
