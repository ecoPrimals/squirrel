// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Access control and audit logging for sovereign data.

use super::super::{DataId, DataPermissions, FederationResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Data lifecycle stage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataLifecycleStage {
    /// Data has been created
    Created,
    /// Data is actively being accessed
    Active,
    /// Data has been archived
    Archived,
    /// Data is pending deletion
    PendingDeletion,
    /// Data has been deleted
    Deleted,
}

/// Data access audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessLog {
    /// Timestamp of the access
    pub timestamp: DateTime<Utc>,
    /// User who accessed the data
    pub user: String,
    /// Type of access
    pub access_type: DataAccessType,
    /// IP address of the accessor
    pub source_ip: Option<String>,
    /// Whether the access was successful
    pub success: bool,
    /// Error message if access failed
    pub error: Option<String>,
}

/// Type of data access operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataAccessType {
    /// Read access
    Read,
    /// Write access
    Write,
    /// Delete access
    Delete,
    /// Admin access
    Admin,
}

/// Access control manager trait
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait AccessControlManager: Send + Sync {
    /// Check if user has permission to access data
    async fn check_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<bool>;

    /// Grant permission to user
    async fn grant_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()>;

    /// Revoke permission from user
    async fn revoke_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()>;

    /// List permissions for data
    async fn list_permissions(&self, data_id: DataId) -> FederationResult<DataPermissions>;
}

/// Default access control manager
pub struct DefaultAccessControlManager {
    /// Permission storage
    permissions: Arc<RwLock<HashMap<DataId, DataPermissions>>>,
    /// Access logs
    access_logs: Arc<RwLock<HashMap<DataId, Vec<DataAccessLog>>>>,
}

impl DefaultAccessControlManager {
    /// Create a new access control manager
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            access_logs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Log data access
    async fn log_access(
        &self,
        data_id: DataId,
        user: &str,
        access_type: DataAccessType,
        success: bool,
        error: Option<String>,
    ) {
        let log_entry = DataAccessLog {
            timestamp: Utc::now(),
            user: user.to_string(),
            access_type,
            source_ip: None,
            success,
            error,
        };

        let mut logs = self.access_logs.write().await;
        logs.entry(data_id).or_insert_with(Vec::new).push(log_entry);
    }
}

impl Default for DefaultAccessControlManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessControlManager for DefaultAccessControlManager {
    async fn check_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<bool> {
        let permissions = self.permissions.read().await;

        if let Some(data_permissions) = permissions.get(&data_id) {
            let has_permission = match access_type {
                DataAccessType::Read => {
                    data_permissions.public_read
                        || data_permissions.read_users.contains(&user.to_string())
                        || data_permissions.admin_users.contains(&user.to_string())
                }
                DataAccessType::Write => {
                    data_permissions.public_write
                        || data_permissions.write_users.contains(&user.to_string())
                        || data_permissions.admin_users.contains(&user.to_string())
                }
                DataAccessType::Delete | DataAccessType::Admin => {
                    data_permissions.admin_users.contains(&user.to_string())
                }
            };

            self.log_access(data_id, user, access_type, has_permission, None)
                .await;

            Ok(has_permission)
        } else {
            self.log_access(
                data_id,
                user,
                access_type,
                false,
                Some("Data not found".to_string()),
            )
            .await;
            Ok(false)
        }
    }

    async fn grant_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()> {
        let mut permissions = self.permissions.write().await;
        let data_permissions = permissions
            .entry(data_id)
            .or_insert_with(DataPermissions::default);

        match access_type {
            DataAccessType::Read => {
                if !data_permissions.read_users.contains(&user.to_string()) {
                    data_permissions.read_users.push(user.to_string());
                }
            }
            DataAccessType::Write => {
                if !data_permissions.write_users.contains(&user.to_string()) {
                    data_permissions.write_users.push(user.to_string());
                }
            }
            DataAccessType::Admin | DataAccessType::Delete => {
                if !data_permissions.admin_users.contains(&user.to_string()) {
                    data_permissions.admin_users.push(user.to_string());
                }
            }
        }

        Ok(())
    }

    async fn revoke_permission(
        &self,
        user: &str,
        data_id: DataId,
        access_type: DataAccessType,
    ) -> FederationResult<()> {
        let mut permissions = self.permissions.write().await;

        if let Some(data_permissions) = permissions.get_mut(&data_id) {
            match access_type {
                DataAccessType::Read => {
                    data_permissions.read_users.retain(|u| u != user);
                }
                DataAccessType::Write => {
                    data_permissions.write_users.retain(|u| u != user);
                }
                DataAccessType::Admin | DataAccessType::Delete => {
                    data_permissions.admin_users.retain(|u| u != user);
                }
            }
        }

        Ok(())
    }

    async fn list_permissions(&self, data_id: DataId) -> FederationResult<DataPermissions> {
        let permissions = self.permissions.read().await;

        if let Some(data_permissions) = permissions.get(&data_id) {
            Ok(data_permissions.clone())
        } else {
            Ok(DataPermissions::default())
        }
    }
}
