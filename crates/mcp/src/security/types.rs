// Security types module
//
// This module defines the core types used in the security subsystem,
// particularly for Role-Based Access Control (RBAC).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::types::SecurityLevel;

/// Role in the RBAC system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Unique identifier for the role
    pub id: String,
    /// Name of the role
    pub name: String,
    /// Optional description of the role
    pub description: Option<String>,
    /// Set of permissions granted by this role
    pub permissions: HashSet<Permission>,
    /// Set of parent role IDs
    pub parent_roles: HashSet<String>,
    /// Security level required for this role
    pub security_level: SecurityLevel,
    /// Whether this role can be delegated
    pub can_delegate: bool,
    /// Roles that this role can manage
    pub managed_roles: HashSet<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Permission for a specific resource and action
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    /// Name of the permission
    pub name: String,
    /// Resource the permission applies to
    pub resource: String,
    /// Action allowed by this permission
    pub action: Action,
    /// Optional resource identifier for fine-grained control
    pub resource_id: Option<String>,
    /// Optional scope limitation
    pub scope: PermissionScope,
    /// Conditions under which this permission applies
    pub conditions: Vec<PermissionCondition>,
}

/// Action types for permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Create new resources
    Create,
    /// Read existing resources
    Read,
    /// Write to resources
    Write,
    /// Update existing resources
    Update,
    /// Delete existing resources
    Delete,
    /// Execute operations on resources
    Execute,
    /// Full administrative access
    Admin,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Read => write!(f, "read"),
            Action::Write => write!(f, "write"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::Execute => write!(f, "execute"),
            Action::Admin => write!(f, "admin"),
        }
    }
}

/// Scope of a permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[derive(Default)]
pub enum PermissionScope {
    /// Permission applies only to owned resources
    #[default]
    Own,
    /// Permission applies to resources in the same group
    Group,
    /// Permission applies to all resources
    All,
    /// Permission applies to resources matching a pattern
    Pattern(String),
}


/// Condition for permission application
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionCondition {
    /// Time-based condition (e.g., business hours)
    TimeRange {
        start_time: String,
        end_time: String,
        days: Vec<String>,
    },
    /// Network-based condition (e.g., specific IP range)
    NetworkRange { cidr: String },
    /// Security level requirement
    MinimumSecurityLevel(SecurityLevel),
    /// Custom attribute-based condition
    AttributeEquals { attribute: String, value: String },
}

/// Context for evaluating permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    /// Current user ID
    pub user_id: String,
    /// Current time
    pub current_time: Option<DateTime<Utc>>,
    /// Current network address
    pub network_address: Option<String>,
    /// Current security level
    pub security_level: SecurityLevel,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
    /// Resource owner ID
    pub resource_owner_id: Option<String>,
    /// Resource group ID
    pub resource_group_id: Option<String>,
}

impl PermissionContext {
    /// Create a new permission context with the given user ID
    pub fn new(user_id: &str) -> Self {
        PermissionContext {
            user_id: user_id.to_string(),
            current_time: Some(Utc::now()),
            network_address: None,
            security_level: SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        }
    }
}

impl Default for PermissionContext {
    fn default() -> Self {
        PermissionContext {
            user_id: String::from("system"),
            current_time: Some(Utc::now()),
            network_address: None,
            security_level: SecurityLevel::Standard,
            attributes: HashMap::new(),
            resource_owner_id: None,
            resource_group_id: None,
        }
    }
} 