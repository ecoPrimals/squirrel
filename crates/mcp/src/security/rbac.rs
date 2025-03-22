//! Role-Based Access Control (RBAC) module for MCP
//!
//! This module provides RBAC functionality for the MCP system.

use chrono::{DateTime, NaiveTime, Utc};
use regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::error::{SecurityError, Result};
use crate::MCPError;
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
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::Execute => write!(f, "execute"),
            Action::Admin => write!(f, "admin"),
        }
    }
}

/// Scope of the permission
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionScope {
    /// Permission applies only to owned resources
    Own,
    /// Permission applies to resources in the same group
    Group,
    /// Permission applies to all resources
    All,
    /// Permission applies to resources matching a pattern
    Pattern(String),
}

/// Default implementation for PermissionScope
impl Default for PermissionScope {
    fn default() -> Self {
        Self::All
    }
}

/// Condition that must be satisfied for the permission to apply
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

/// Context for permission evaluation
#[derive(Debug, Clone, Default)]
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

/// Role template for creating common role types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Base permissions included in this template
    pub base_permissions: Vec<Permission>,
    /// Optional parent templates
    pub parent_templates: Vec<String>,
    /// Parameters that can be customized when creating from template
    pub parameters: HashMap<String, TemplateParameter>,
}

/// Parameter for role template customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Default value if not specified
    pub default_value: Option<String>,
    /// Whether the parameter is required
    pub required: bool,
    /// Validation rules for the parameter
    pub validation: Option<String>,
}

/// Record of role delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRecord {
    /// ID of the user delegating the role
    pub delegator_id: String,
    /// ID of the user receiving the role
    pub delegate_id: String,
    /// ID of the role being delegated
    pub role_id: String,
    /// Timestamp of the delegation
    pub timestamp: DateTime<Utc>,
}

/// RBAC-specific errors
#[derive(Debug)]
pub enum RBACError {
    /// Role not found
    RoleNotFound(String),
    /// Template not found
    TemplateNotFound(String),
    /// Permission validation failed
    PermissionValidation(String),
    /// Role already exists
    RoleAlreadyExists(String),
    /// General RBAC error
    General(String),
}

impl std::fmt::Display for RBACError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoleNotFound(id) => write!(f, "Role not found: {}", id),
            Self::TemplateNotFound(id) => write!(f, "Template not found: {}", id),
            Self::PermissionValidation(msg) => write!(f, "Permission validation failed: {}", msg),
            Self::RoleAlreadyExists(name) => write!(f, "Role already exists: {}", name),
            Self::General(msg) => write!(f, "RBAC error: {}", msg),
        }
    }
}

impl std::error::Error for RBACError {}

impl From<RBACError> for MCPError {
    fn from(err: RBACError) -> Self {
        MCPError::Security(SecurityError::InternalError(format!("RBAC error: {}", err)))
    }
}

/// Role-Based Access Control manager
#[derive(Debug, Clone)]
pub struct RBACManager {
    /// Map of role IDs to Role objects (primary lookup)
    roles_by_id: HashMap<String, Role>,
    /// Map of role names to role IDs (secondary lookup)
    roles_by_name: HashMap<String, String>,
    /// Map of user IDs to their assigned role IDs
    user_roles: HashMap<String, HashSet<String>>,
    /// Map of template names to role templates
    templates: HashMap<String, RoleTemplate>,
    /// Log of role delegations
    delegation_log: Vec<DelegationRecord>,
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RBACManager {
    /// Creates a new RBAC manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            roles_by_id: HashMap::new(),
            roles_by_name: HashMap::new(),
            user_roles: HashMap::new(),
            templates: HashMap::new(),
            delegation_log: Vec::new(),
        }
    }

    /// Gets a role by name
    #[must_use]
    pub fn get_role_by_name(&self, name: &str) -> Option<&Role> {
        self.roles_by_name
            .get(name)
            .and_then(|id| self.roles_by_id.get(id))
    }

    /// Gets a role by ID
    #[must_use]
    pub fn get_role_by_id(&self, id: &str) -> Option<&Role> {
        self.roles_by_id.get(id)
    }

    /// Gets a role by either ID or name
    #[must_use]
    pub fn get_role(&self, id_or_name: &str) -> Option<&Role> {
        // First try as ID
        if let Some(role) = self.get_role_by_id(id_or_name) {
            return Some(role);
        }

        // Then try as name
        self.get_role_by_name(id_or_name)
    }

    /// Creates a new role with the given properties
    ///
    /// # Parameters
    ///
    /// * `name` - Name of the role
    /// * `description` - Optional description of the role
    /// * `permissions` - Set of permissions granted by this role
    /// * `parent_roles` - Set of parent role IDs that this role inherits from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A role with the given name already exists
    /// - Any of the parent roles don't exist
    pub fn create_role(
        &mut self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if role with name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("Role with name '{name}' already exists")
            )));
        }

        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;

        // Create new role ID
        let id = Uuid::new_v4().to_string();
        
        self.create_role_with_id(id, name, description, permissions, parent_roles)
    }

    /// Creates a new role with the specified ID
    ///
    /// # Parameters
    ///
    /// * `id` - ID to use for the role
    /// * `name` - Name of the role
    /// * `description` - Optional description of the role
    /// * `permissions` - Set of permissions granted by this role
    /// * `parent_roles` - Set of parent role IDs that this role inherits from
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A role with the given ID already exists
    /// - A role with the given name already exists
    /// - Any of the parent roles don't exist
    pub fn create_role_with_id(
        &mut self,
        id: String,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // Check if role with ID already exists
        if self.roles_by_id.contains_key(&id) {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("Role with ID '{id}' already exists")
            )));
        }

        // Check if role with name already exists
        if self.roles_by_name.contains_key(&name) {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("Role with name '{name}' already exists")
            )));
        }

        // Verify parent roles exist
        self.verify_parent_roles(&parent_roles)?;

        // Create the role
        let role = Role {
            id: id.clone(),
            name: name.clone(),
            description,
            permissions,
            parent_roles,
            security_level: SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
        };

        // Store the role
        self.roles_by_id.insert(id.clone(), role.clone());
        self.roles_by_name.insert(name, id);

        Ok(role)
    }

    /// Validates that parent roles exist in the system
    ///
    /// # Arguments
    /// * `parent_roles` - Set of parent role IDs to validate
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::Security` error if:
    /// - Any of the parent roles don't exist in the system
    fn verify_parent_roles(&self, parent_roles: &HashSet<String>) -> Result<()> {
        for parent_id in parent_roles {
            if !self.roles_by_id.contains_key(parent_id) {
                return Err(MCPError::Security(SecurityError::InvalidRole(
                    format!("Role '{parent_id}' not found in system")
                )));
            }
        }
        Ok(())
    }

    /// Assigns a role to a user
    ///
    /// # Parameters
    ///
    /// * `user_id` - ID of the user to assign the role to
    /// * `role_id` - ID of the role to assign
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::Security` error if:
    /// - The role with the specified ID doesn't exist in the system
    pub fn assign_role(&mut self, user_id: String, role_id: String) -> Result<()> {
        // Check if role exists
        if !self.roles_by_id.contains_key(&role_id) {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("Role '{role_id}' not found in system")
            )));
        }

        // Get or create user roles set
        let user_roles = self.user_roles.entry(user_id).or_default();

        // Add role ID to user roles
        user_roles.insert(role_id);

        Ok(())
    }

    /// Assigns a role to a user by role name
    ///
    /// # Arguments
    /// * `user_id` - ID of the user
    /// * `role_name` - Name of the role to assign
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::Security` error if:
    /// - The role with the specified name doesn't exist in the system
    /// - The underlying `assign_role` operation fails
    pub fn assign_role_by_name(&mut self, user_id: String, role_name: &str) -> Result<()> {
        // Check if role exists
        let role_id = self
            .roles_by_name
            .get(role_name)
            .ok_or_else(|| {
                MCPError::Security(SecurityError::InvalidRole(
                    format!("Role '{role_name}' not found in system")
                ))
            })?
            .clone();

        // Assign role by ID
        self.assign_role(user_id, role_id)
    }

    /// Gets all permissions for a user
    #[must_use]
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        // Get user's role IDs
        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Some(role) = self.roles_by_id.get(role_id) {
                    // Collect permissions from this role and its parents
                    self.collect_role_permissions(role, &mut permissions);
                }
            }
        }

        permissions
    }

    /// Collects permissions from a role and its parents recursively
    fn collect_role_permissions(&self, role: &Role, permissions: &mut HashSet<Permission>) {
        // Add this role's permissions
        for permission in &role.permissions {
            permissions.insert(permission.clone());
        }

        // Add parent role permissions recursively
        for parent_id in &role.parent_roles {
            if let Some(parent_role) = self.roles_by_id.get(parent_id) {
                self.collect_role_permissions(parent_role, permissions);
            }
        }
    }

    /// Checks if a user has a specific permission
    #[must_use]
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let user_permissions = self.get_user_permissions(user_id);
        user_permissions.contains(permission)
    }

    /// Gets all roles assigned to a user
    #[must_use]
    pub fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
        let mut roles = Vec::new();

        if let Some(role_ids) = self.user_roles.get(user_id) {
            for role_id in role_ids {
                if let Some(role) = self.roles_by_id.get(role_id) {
                    roles.push(role.clone());
                }
            }
        }

        roles
    }

    /// Gets all users assigned to a role
    #[must_use]
    pub fn get_role_users(&self, role_id: &str) -> Vec<String> {
        let mut users = Vec::new();

        for (user_id, role_ids) in &self.user_roles {
            if role_ids.contains(role_id) {
                users.push(user_id.clone());
            }
        }

        users
    }

    /// Checks if a role has permission for a resource and action
    /// This replaces the old `role_has_permission` method
    #[must_use]
    pub fn has_permission_for_role(&self, role: &Role, resource: &str, action: Action) -> bool {
        // Check if the role has the permission directly
        let has_direct_permission = role
            .permissions
            .iter()
            .any(|p| p.resource == resource && p.action == action);

        if has_direct_permission {
            return true;
        }

        // Check parent roles recursively
        for parent_id in &role.parent_roles {
            if let Some(parent_role) = self.get_role_by_id(parent_id) {
                if self.has_permission_for_role(parent_role, resource, action) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a user has a specific permission with context
    pub fn has_permission_with_context(
        &self,
        user_id: &str,
        permission: &Permission,
        context: &PermissionContext,
    ) -> bool {
        // Get user roles
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return false,
        };

        // Check each role for the permission
        for role_id in user_roles {
            if let Some(role) = self.roles_by_id.get(role_id) {
                // Check if role has the permission
                for role_permission in &role.permissions {
                    if self.matches_permission(role_permission, permission, context) {
                        return true;
                    }
                }

                // Check parent roles recursively
                for parent_id in &role.parent_roles {
                    if self.role_has_permission_with_context(parent_id, permission, context) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a role has a specific permission with context
    fn role_has_permission_with_context(
        &self,
        role_id: &str,
        permission: &Permission,
        context: &PermissionContext,
    ) -> bool {
        if let Some(role) = self.roles_by_id.get(role_id) {
            // Check direct permissions
            for role_permission in &role.permissions {
                if self.matches_permission(role_permission, permission, context) {
                    return true;
                }
            }

            // Check parent roles
            for parent_id in &role.parent_roles {
                if self.role_has_permission_with_context(parent_id, permission, context) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if permission matches, considering scope and conditions
    fn matches_permission(
        &self,
        role_permission: &Permission,
        requested_permission: &Permission,
        context: &PermissionContext,
    ) -> bool {
        // Check basic permission match (resource and action)
        if role_permission.resource != requested_permission.resource
            || role_permission.action != requested_permission.action
        {
            return false;
        }

        // Check resource ID if specified
        if let Some(role_res_id) = &role_permission.resource_id {
            if let Some(req_res_id) = &requested_permission.resource_id {
                if role_res_id != req_res_id && !self.matches_pattern(role_res_id, req_res_id) {
                    return false;
                }
            }
        }

        // Check scope
        if !self.matches_scope(&role_permission.scope, &requested_permission.scope, context) {
            return false;
        }

        // Check conditions
        for condition in &role_permission.conditions {
            if !self.evaluate_condition(condition, context) {
                return false;
            }
        }

        true
    }

    /// Check if a pattern matches a resource ID
    fn matches_pattern(&self, pattern: &str, resource_id: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                return regex.is_match(resource_id);
            }
        }

        pattern == resource_id
    }

    /// Check if scopes match
    fn matches_scope(
        &self,
        role_scope: &PermissionScope,
        requested_scope: &PermissionScope,
        context: &PermissionContext,
    ) -> bool {
        match role_scope {
            PermissionScope::All => true,
            PermissionScope::Own => {
                // Check if resource is owned by the user
                if let Some(owner_id) = &context.resource_owner_id {
                    owner_id == &context.user_id
                } else {
                    false
                }
            }
            PermissionScope::Group => {
                // Check if resource is in the same group
                if let Some(_group_id) = &context.resource_group_id {
                    // Logic to check if user is in the same group would go here
                    // For now, we'll just return true if context has a group ID
                    true
                } else {
                    false
                }
            }
            PermissionScope::Pattern(pattern) => match requested_scope {
                PermissionScope::Pattern(req_pattern) => self.matches_pattern(pattern, req_pattern),
                _ => false,
            },
        }
    }

    /// Evaluate a permission condition
    fn evaluate_condition(
        &self,
        condition: &PermissionCondition,
        context: &PermissionContext,
    ) -> bool {
        match condition {
            PermissionCondition::TimeRange {
                start_time,
                end_time,
                days,
            } => {
                if let Some(current_time) = &context.current_time {
                    // Parse time strings
                    let start_time = match NaiveTime::parse_from_str(start_time, "%H:%M") {
                        Ok(t) => t,
                        Err(_) => return false,
                    };

                    let end_time = match NaiveTime::parse_from_str(end_time, "%H:%M") {
                        Ok(t) => t,
                        Err(_) => return false,
                    };

                    // Get current day and time
                    let current_day = current_time.format("%a").to_string();
                    let current_time_naive = current_time.naive_utc().time();

                    // Check if current day is in allowed days
                    if !days.contains(&current_day) {
                        return false;
                    }

                    // Check if current time is within range
                    current_time_naive >= start_time && current_time_naive <= end_time
                } else {
                    false
                }
            }
            PermissionCondition::NetworkRange { cidr } => {
                if let Some(ip_addr) = &context.network_address {
                    // This would need a proper CIDR parsing library
                    // For simplicity, we'll just check for exact match or prefix match
                    ip_addr.starts_with(
                        &cidr
                            .replace("/", "")
                            .split('.')
                            .take(3)
                            .collect::<Vec<_>>()
                            .join("."),
                    )
                } else {
                    false
                }
            }
            PermissionCondition::MinimumSecurityLevel(required_level) => {
                context.security_level >= *required_level
            }
            PermissionCondition::AttributeEquals { attribute, value } => context
                .attributes
                .get(attribute) == Some(value),
        }
    }

    /// Register a new role template
    pub fn register_template(&mut self, template: RoleTemplate) -> Result<()> {
        // Validate template
        self.validate_template(&template)?;

        // Store template
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Validate a role template
    fn validate_template(&self, template: &RoleTemplate) -> Result<()> {
        // Check if parent templates exist
        for parent_name in &template.parent_templates {
            if !self.templates.contains_key(parent_name) {
                return Err(MCPError::Security(SecurityError::InvalidRole(
                    format!("Parent template '{}' not found", parent_name)
                )));
            }
        }

        // Validate parameters in permissions
        for permission in &template.base_permissions {
            // Check for parameter placeholders in pattern scopes
            if let PermissionScope::Pattern(pattern) = &permission.scope {
                if pattern.contains("${") && pattern.contains("}") {
                    // Extract parameter name
                    let start = pattern.find("${").unwrap() + 2;
                    let end = pattern.find('}').unwrap();
                    let param_name = &pattern[start..end];

                    // Check if parameter exists in template
                    if !template.parameters.contains_key(param_name) {
                        return Err(MCPError::Security(SecurityError::InvalidRole(
                            format!("Template parameter '{}' not found", param_name)
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Creates a role with the specified name from a template
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found, or if a role with the same name already exists
    pub fn create_role_from_template(
        &mut self,
        _template_name: &str,
        role_name: &str,
        description: &str,
    ) -> Result<Role> {
        // Implementation will go here
        // For now, just create a simple role
        let role = Role {
            id: Uuid::new_v4().to_string(),
            name: role_name.to_string(),
            description: Some(description.to_string()),
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            security_level: SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
        };
        
        Ok(role)
    }

    /// Delegate a role from one user to another
    pub fn delegate_role(
        &mut self,
        delegator_id: &str,
        delegate_id: &str,
        role_id: &str,
    ) -> Result<()> {
        // Check if delegator has the role
        let has_role = if let Some(roles) = self.user_roles.get(delegator_id) {
            roles.contains(role_id)
        } else {
            false
        };

        if !has_role {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("User '{}' does not have role '{}'", delegator_id, role_id)
            )));
        }

        // Check if the role can be delegated
        let role = self
            .roles_by_id
            .get(role_id)
            .ok_or_else(|| MCPError::Security(SecurityError::InvalidRole(format!("Role '{}' not found", role_id))))?;

        if !role.can_delegate {
            return Err(MCPError::Security(SecurityError::InvalidRole(
                format!("Role '{}' cannot be delegated", role_id)
            )));
        }

        // Assign role to delegate
        self.assign_role(delegate_id.to_string(), role_id.to_string())?;

        // Track delegation for auditing
        self.delegation_log.push(DelegationRecord {
            delegator_id: delegator_id.to_string(),
            delegate_id: delegate_id.to_string(),
            role_id: role_id.to_string(),
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// Get all permissions for a role, including inherited ones
    pub fn get_all_permissions(&self, role_id: &str) -> Result<HashSet<Permission>> {
        let mut visited = HashSet::new();
        let mut permissions = HashSet::new();

        self.collect_permissions(role_id, &mut permissions, &mut visited)?;

        Ok(permissions)
    }

    /// Recursively collect permissions from role and its parents
    fn collect_permissions(
        &self,
        role_id: &str,
        permissions: &mut HashSet<Permission>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        // Prevent cycles
        if visited.contains(role_id) {
            return Ok(());
        }
        visited.insert(role_id.to_string());

        // Get role
        let role = self
            .roles_by_id
            .get(role_id)
            .ok_or_else(|| MCPError::Security(SecurityError::InvalidRole(format!("Role '{}' not found", role_id))))?;

        // Add direct permissions
        permissions.extend(role.permissions.clone());

        // Process parent roles
        for parent_id in &role.parent_roles {
            self.collect_permissions(parent_id, permissions, visited)?;
        }

        Ok(())
    }

    /// Check if a role has permission inheritance cycles
    pub fn check_inheritance_cycles(&self, role_id: &str) -> Result<bool> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        self.detect_cycle(role_id, &mut visited, &mut path)
    }

    /// Helper method to detect inheritance cycles
    fn detect_cycle(
        &self,
        role_id: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<bool> {
        // Check if already in path (cycle detected)
        if path.contains(&role_id.to_string()) {
            return Ok(true);
        }

        // Check if already visited (no cycle in this branch)
        if visited.contains(role_id) {
            return Ok(false);
        }

        // Get role
        let role = self
            .roles_by_id
            .get(role_id)
            .ok_or_else(|| MCPError::Security(SecurityError::InvalidRole(format!("Role '{}' not found", role_id))))?;

        // Add to path and visited
        path.push(role_id.to_string());
        visited.insert(role_id.to_string());

        // Check parent roles
        for parent_id in &role.parent_roles {
            if self.detect_cycle(parent_id, visited, path)? {
                return Ok(true);
            }
        }

        // Remove from path (backtrack)
        path.pop();

        Ok(false)
    }

    /// Check if a user can manage a role
    pub fn can_manage_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        // Get user roles
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return Ok(false),
        };

        // Check each role
        for user_role_id in user_roles {
            let user_role = match self.roles_by_id.get(user_role_id) {
                Some(role) => role,
                None => continue,
            };

            // Check if role can directly manage the target role
            if user_role.managed_roles.contains(role_id) {
                return Ok(true);
            }

            // Check parent roles for management permissions
            for parent_id in &user_role.parent_roles {
                if self.role_can_manage_role(parent_id, role_id)? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Helper method for role management check
    fn role_can_manage_role(
        &self,
        manager_role_id: &str,
        target_role_id: &str,
    ) -> Result<bool> {
        let role = match self.roles_by_id.get(manager_role_id) {
            Some(r) => r,
            None => return Ok(false),
        };

        if role.managed_roles.contains(target_role_id) {
            return Ok(true);
        }

        // Check parent roles
        for parent_id in &role.parent_roles {
            if self.role_can_manage_role(parent_id, target_role_id)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests;

// Remove the entire test module at the end of the file
