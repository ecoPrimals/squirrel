// Enhanced permission validation for RBAC system

//
// This module provides advanced permission validation capabilities for the RBAC system,
// including fine-grained permission control, contextual validation, and permission patterns.

use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use regex::Regex;
use chrono::{DateTime, NaiveTime, Utc, Timelike, Datelike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SecurityError, Result};
use crate::security::types::Action;
use crate::error::MCPError;

/// Represents a role in the RBAC system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Role {
    /// Role ID
    pub id: String,
    
    /// Role name
    pub name: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Permissions granted to this role
    pub permissions: Vec<Permission>,
}

/// Represents the context in which a permission is being evaluated
#[derive(Debug, Clone, Default)]
pub struct PermissionContext {
    /// Context attributes
    pub attributes: std::collections::HashMap<String, String>,
    
    /// Time of evaluation
    pub time: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Time alias used in some code paths
    pub current_time: Option<chrono::DateTime<chrono::Utc>>,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// IP address
    pub ip_address: Option<std::net::IpAddr>,
    
    /// Network address (may include additional information beyond IP)
    pub network_address: Option<String>,
    
    /// Resource owner ID
    pub resource_owner_id: Option<String>,
    
    /// Resource group ID
    pub resource_group_id: Option<String>,
    
    /// Security level
    pub security_level: Option<u8>,
}

/// Represents a scope for a permission
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    /// Global scope (applies everywhere)
    Global,
    
    /// User scope (applies to specific user)
    User(String),
    
    /// Group scope (applies to specific group)
    Group(String),
    
    /// Organization scope (applies to specific organization)
    Organization(String),
    
    /// Resource scope (applies to specific resource)
    Resource(String),
    
    /// Instance scope (applies to specific instance)
    Instance(String),
    
    /// Own scope (applies to objects owned by the current user)
    Own,
    
    /// All scope (applies to everything in the specified context)
    All,
    
    /// Pattern scope (applies to resources matching a pattern)
    Pattern(String),
}

/// Represents a condition that must be satisfied for a permission to be granted
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionCondition {
    /// Time range condition
    TimeRange {
        /// Start time (24-hour format, HH:MM)
        start_time: String,
        
        /// End time (24-hour format, HH:MM)
        end_time: String,
        
        /// Days of week (0 = Sunday, 1 = Monday, etc.)
        days: Vec<u8>,
    },
    
    /// Network range condition (CIDR notation)
    NetworkRange {
        /// CIDR notation (e.g., "192.168.0.0/24")
        cidr: String,
    },
    
    /// Minimum security level required (u8 instead of SecurityLevel for Hash)
    MinimumSecurityLevel(u8),
    
    /// Attribute equals specific value
    AttributeEquals {
        /// Attribute name
        attribute: String,
        
        /// Expected value
        value: String,
    },
}

/// Represents a permission in the RBAC system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: String,
    
    /// Resource the permission applies to
    pub resource: String,
    
    /// Action that is permitted
    pub action: Action,
    
    /// Optional scope of the permission
    pub scope: Option<PermissionScope>,
    
    /// Conditions that must be satisfied for the permission to be granted
    pub conditions: Vec<PermissionCondition>,
    
    /// Alternative property used in some code paths
    pub resource_id: String,
}

/// Represents the validation result for a permission check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Permission is granted
    Granted,
    /// Permission is denied
    Denied,
    /// Permission check needs additional verification
    NeedsVerification,
}

/// Types of additional verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationType {
    /// Simple verification
    Simple,
    
    /// Required verification
    Required,
    
    /// Optional verification
    Optional,
}

/// Types of validation for permission validation rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ValidationType {
    /// Allow access if the rule matches
    Allow,
    
    /// Deny access if the rule matches
    Deny,
    
    /// Require verification if the rule matches
    Verify,
}

/// Permission validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Optional description of the rule
    pub description: Option<String>,
    
    /// Resource pattern this rule applies to
    pub resource_pattern: String,
    
    /// Action this rule applies to
    pub action: Action,
    
    /// Validation expression to evaluate
    pub validation_expr: String,
    
    /// Whether verification is required
    pub verification: Option<VerificationType>,
    
    /// Priority (higher numbers take precedence)
    pub priority: i32,
    
    /// Whether this rule is an allow rule (true) or deny rule (false)
    pub is_allow: bool,
    
    /// Whether the rule is enabled
    pub enabled: bool,
}

/// Audit record for permission validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationAuditRecord {
    /// Record ID
    pub id: String,
    
    /// User ID
    pub user_id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Resource being accessed
    pub resource: String,
    
    /// Action being performed
    pub action: Action,
    
    /// Result of the validation
    pub result: ValidationResult,
    
    /// Rule ID that produced the result
    pub rule_id: String,
    
    /// Rule name that produced the result
    pub rule_name: String,
    
    /// Whether the rule is an allow rule
    pub is_allow: bool,
    
    /// Context information for audit
    pub context: HashMap<String, String>,
    
    /// Matched permissions during validation
    pub matched_permissions: Vec<String>,
}

/// Type of expression in a validation rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ValidationExpression {
    /// Single validation check
    Single(String),
    /// All conditions must be true
    And(Vec<String>),
    /// Any condition can be true
    Or(Vec<String>),
}

/// Permission validation engine
#[derive(Debug)]
pub(super) struct PermissionValidator {
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    
    /// Compiled resource patterns
    resource_patterns: HashMap<String, Regex>,
    
    /// Validation audit log
    audit_log: Vec<ValidationAuditRecord>,
    
    /// Maximum audit log size
    max_audit_size: usize,
    
    /// Whether audit logging is enabled
    pub audit_enabled: bool,
}

impl PermissionValidator {
    /// Create a new permission validator
    pub(super) fn new() -> Self {
        Self {
            rules: Vec::new(),
            resource_patterns: HashMap::new(),
            audit_log: Vec::new(),
            max_audit_size: 1000,
            audit_enabled: false,
        }
    }
    
    /// Add a validation rule
    pub(super) fn add_rule(&mut self, rule: ValidationRule) -> Result<()> {
        // Compile resource pattern regex
        let regex = Regex::new(&rule.resource_pattern)
            .map_err(|e| MCPError::Security(
                format!("Invalid resource pattern regex: {e}").into()
            ))?;
        
        self.resource_patterns.insert(rule.id.clone(), regex);
        
        // Add rule to collection, sorted by priority
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    /// Remove a validation rule
    pub(super) fn remove_rule(&mut self, rule_id: &str) {
        self.rules.retain(|r| r.id != rule_id);
        self.resource_patterns.remove(rule_id);
    }
    
    /// Validate a permission request
    pub(super) fn validate(
        &mut self,
        user_id: &str,
        resource: &str,
        action: Action,
        user_roles: &[Role],
        user_permissions: &HashSet<Permission>,
        context: &PermissionContext,
    ) -> ValidationResult {
        let action_clone = action.clone();
        
        // Create audit record base
        let audit_record_id = Uuid::new_v4().to_string();
        let mut audit_record = ValidationAuditRecord {
            id: audit_record_id,
            user_id: user_id.to_string(),
            timestamp: Utc::now(),
            resource: resource.to_string(),
            action: action_clone,
            result: ValidationResult::Denied,
            rule_id: String::new(),
            rule_name: String::new(),
            is_allow: false,
            context: HashMap::new(),
            matched_permissions: Vec::new(),
        };
        
        // Add context to audit record
        if let Some(addr) = &context.network_address {
            audit_record.context.insert("network_address".to_string(), addr.clone());
        }
        
        if let Some(time) = context.current_time {
            audit_record.context.insert(
                "timestamp".to_string(),
                time.to_rfc3339(),
            );
        }
        
        for (key, value) in &context.attributes {
            audit_record.context.insert(
                format!("attr_{key}"),
                value.clone(),
            );
        }
        
        // Check if any permission directly allows access
        for permission in user_permissions {
            if Self::matches_permission(permission, resource, &action, context) {
                // Permission matches direct permission, grant access
                // Create a basic audit record
                if self.audit_enabled {
                    let record_copy = ValidationAuditRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        user_id: user_id.to_string(),
                        timestamp: Utc::now(),
                        resource: resource.to_string(),
                        action,
                        result: ValidationResult::Granted,
                        rule_id: format!("permission:{}", permission.id),
                        rule_name: format!("Permission: {}", permission.resource),
                        is_allow: true,
                        context: context.attributes.clone(),
                        matched_permissions: vec![permission.id.clone()],
                    };
                    
                    self.record_audit(record_copy);
                }
                
                return ValidationResult::Granted;
            }
        }
        
        // Apply validation rules in priority order
        for i in 0..self.rules.len() {
            // Get rule reference
            let rule = &self.rules[i];

            // Skip rules for other actions
            if rule.action != action && rule.action != Action::admin() {
                continue;
            }

            // Check if resource matches pattern
            let pattern_matches = self.resource_patterns.get(&rule.id)
                .map_or(false, |pattern| pattern.is_match(resource));
            
            if !pattern_matches {
                continue;
            }

            // Clone rule data we need to avoid borrow issues
            let rule_id = rule.id.clone();
            let rule_name = rule.name.clone();
            let is_allow = rule.is_allow;
            let validation_expr = rule.validation_expr.clone();
            let verification = rule.verification.clone();
            
            // Rule applies
            audit_record.rule_id = rule_id;
            audit_record.rule_name = rule_name;
            audit_record.is_allow = is_allow;
            
            // Evaluate expression without holding a borrow to self.rules
            let result = Self::evaluate_expression(&ValidationExpression::Single(validation_expr), context, user_roles);
            if (is_allow && result) || (!is_allow && !result) {
                // Allow rule matched, check for verification requirements
                if let Some(_verification_type) = verification {
                    audit_record.result = ValidationResult::NeedsVerification;
                    
                    let record_copy = audit_record.clone();
                    self.record_audit(record_copy);
                    
                    return ValidationResult::NeedsVerification;
                }
                
                // Immediately grant without verification
                audit_record.result = ValidationResult::Granted;
                
                let record_copy = audit_record.clone();
                self.record_audit(record_copy);
                
                return ValidationResult::Granted;
            }
            
            // Deny rule matched
            audit_record.result = ValidationResult::Denied;
            
            let record_copy = audit_record.clone();
            self.record_audit(record_copy);
            
            return ValidationResult::Denied;
        }
        
        // If no explicit rules matched, use the permission matching result
        if audit_record.matched_permissions.is_empty() {
            audit_record.result = ValidationResult::Denied;
            self.record_audit(audit_record);
            ValidationResult::Denied
        } else {
            audit_record.result = ValidationResult::Granted;
            self.record_audit(audit_record);
            ValidationResult::Granted
        }
    }
    
    /// Check if a rule applies to a resource and action
    fn rule_applies(&self, rule: &ValidationRule, resource: &str, action: Action) -> bool {
        // Check if rule applies to this action
        if rule.action != action && rule.action != Action::admin() {
            return false;
        }
        
        // Check if resource matches pattern
        if let Some(pattern) = self.resource_patterns.get(&rule.id) {
            return pattern.is_match(resource);
        }
        
        false
    }
    
    /// Check if a permission matches the given resource and action in the context
    fn matches_permission(
        permission: &Permission,
        resource: &str,
        action: &Action,
        context: &PermissionContext,
    ) -> bool {
        // Check action
        if permission.action != *action && permission.action != Action::admin() {
            return false;
        }
        
        // Check resource
        let resource_match = if permission.resource_id == resource {
            true
        } else {
            permission.resource == resource
        };
        
        if !resource_match {
            return false;
        }
        
        // Check if the permission scope matches
        let scope_match = match &permission.scope {
            None => true, // No scope constraint
            Some(scope) => match scope {
                PermissionScope::Own => {
                    context.resource_owner_id.as_ref().map_or(false, |owner_id| 
                        context.user_id.as_ref().map_or(false, |user_id| owner_id == user_id)
                    )
                },
                
                PermissionScope::Group(group_id) => {
                    context.resource_group_id.as_ref().map_or(false, |_resource_group| 
                        // In a real implementation, check if user is in the same group
                        !group_id.is_empty()
                    )
                },
                
                PermissionScope::All => true,
                
                PermissionScope::Pattern(pattern) => {
                    // Try to match the pattern against the resource
                    Regex::new(pattern).map_or(false, |regex| regex.is_match(resource))
                },

                // Default case for other scope types
                _ => false,
            }
        };
        
        if !scope_match {
            return false;
        }
        
        // Check conditions
        for condition in &permission.conditions {
            if !Self::evaluate_condition(condition, context) {
                return false;
            }
        }
        
        true
    }
    
    /// Evaluate a permission condition
    fn evaluate_condition(condition: &PermissionCondition, context: &PermissionContext) -> bool {
        match condition {
            PermissionCondition::TimeRange { start_time, end_time, days } => {
                if let Some(current_time) = context.current_time {
                    // Parse start and end times
                    if let (Ok(start), Ok(end)) = (
                        NaiveTime::parse_from_str(start_time, "%H:%M"),
                        NaiveTime::parse_from_str(end_time, "%H:%M"),
                    ) {
                        let current_time_naive = current_time.time();
                        
                        // Check if current day is in allowed days
                        let current_weekday = current_time.naive_utc().weekday().num_days_from_sunday() as u8;
                        if !days.contains(&current_weekday) {
                            return false;
                        }
                        
                        // Check if current time is within range
                        if start <= end {
                            // Normal time range (e.g., 9:00-17:00)
                            return current_time_naive >= start && current_time_naive <= end;
                        }
                        
                        // Overnight time range (e.g., 22:00-6:00)
                        return current_time_naive >= start || current_time_naive <= end;
                    }
                }
                false
            },
            
            PermissionCondition::NetworkRange { cidr } => {
                context.network_address.as_ref().map_or(false, |addr| {
                    // In a real implementation, use a proper CIDR matching library
                    // Extract network part of CIDR without unwrap
                    let network_part = cidr.split('/').next().unwrap_or_default();
                    addr.starts_with(network_part)
                })
            },
            
            PermissionCondition::MinimumSecurityLevel(level) => {
                context.security_level.map_or(false, |sl| sl >= *level)
            },
            
            PermissionCondition::AttributeEquals { attribute, value } => {
                context.attributes.get(attribute).map_or(false, |attr_value| attr_value == value)
            },
        }
    }
    
    /// Evaluate a validation expression
    fn evaluate_expression(
        expression: &ValidationExpression,
        context: &PermissionContext,
        roles: &[Role],
    ) -> bool {
        match expression {
            ValidationExpression::Single(condition) => {
                Self::evaluate_string_condition(condition, context, roles)
            },
            ValidationExpression::And(conditions) => {
                for condition in conditions {
                    if !Self::evaluate_string_condition(condition, context, roles) {
                        return false;
                    }
                }
                true
            },
            ValidationExpression::Or(conditions) => {
                for condition in conditions {
                    if Self::evaluate_string_condition(condition, context, roles) {
                        return true;
                    }
                }
                false
            },
        }
    }
    
    /// Evaluate a string condition expression
    fn evaluate_string_condition(
        condition: &str, 
        context: &PermissionContext,
        _roles: &[Role],
    ) -> bool {
        // Simple implementation for evaluating string expressions
        Self::evaluate_basic_condition(condition, context)
    }
    
    /// Evaluate a basic string condition
    fn evaluate_basic_condition(condition: &str, context: &PermissionContext) -> bool {
        // Simple condition evaluation based on context attributes
        // In a real implementation, this would use a proper expression evaluator
        
        // Handle time-based conditions like "time_between(9:00,17:00)"
        if condition.starts_with("time_between(") {
            if let Some(time) = context.current_time {
                let parts: Vec<&str> = condition
                    .trim_start_matches("time_between(")
                    .trim_end_matches(')')
                    .split(',')
                    .collect();
                
                if parts.len() == 2 {
                    // Basic parsing of time ranges
                    let start_parts: Vec<&str> = parts[0].split(':').collect();
                    let end_parts: Vec<&str> = parts[1].split(':').collect();
                    
                    if start_parts.len() == 2 && end_parts.len() == 2 {
                        let current_hour = time.hour();
                        let current_minute = time.minute();
                        
                        let start_hour: u32 = start_parts[0].parse().unwrap_or(0);
                        let start_minute: u32 = start_parts[1].parse().unwrap_or(0);
                        let end_hour: u32 = end_parts[0].parse().unwrap_or(0);
                        let end_minute: u32 = end_parts[1].parse().unwrap_or(0);
                        
                        let current_mins = current_hour * 60 + current_minute;
                        let start_mins = start_hour * 60 + start_minute;
                        let end_mins = end_hour * 60 + end_minute;
                        
                        return current_mins >= start_mins && current_mins <= end_mins;
                    }
                }
            }
            return false;
        }
        
        // Handle attribute-based conditions like "attribute(department)=Engineering"
        if condition.starts_with("attribute(") {
            let parts: Vec<&str> = condition.split('=').collect();
            if parts.len() == 2 {
                let attr_name = parts[0]
                    .trim_start_matches("attribute(")
                    .trim_end_matches(')')
                    .trim();
                
                let attr_value = parts[1].trim();
                
                if let Some(actual_value) = context.attributes.get(attr_name) {
                    return actual_value == attr_value;
                }
            }
            return false;
        }
        
        // Handle security level conditions like "security_level>=High"
        if condition.starts_with("security_level") {
            let parts: Vec<&str> = condition.split(['>', '=', '<']).collect();
            if parts.len() == 2 {
                let operator = if condition.contains(">=") {
                    ">="
                } else if condition.contains("<=") {
                    "<="
                } else if condition.contains('>') {
                    ">"
                } else if condition.contains('<') {
                    "<"
                } else if condition.contains("==") {
                    "=="
                } else {
                    return false;
                };
                
                let level_str = parts[1].trim();
                let required_level = match level_str {
                    "Low" => 0,
                    "Standard" => 1,
                    "High" => 2,
                    "Critical" => 3,
                    _ => return false,
                };
                
                // Return false if security_level is None
                let security_level = match context.security_level {
                    Some(level) => level,
                    None => return false,
                };
                
                return match operator {
                    ">=" => security_level >= required_level,
                    "<=" => security_level <= required_level,
                    ">" => security_level > required_level,
                    "<" => security_level < required_level,
                    "==" => security_level == required_level,
                    _ => false,
                };
            }
            return false;
        }
        
        // Default to false for unknown conditions
        false
    }
    
    /// Record a validation audit record
    fn record_audit(&mut self, record: ValidationAuditRecord) {
        self.audit_log.push(record);
        
        // Trim audit log if it exceeds the maximum size
        if self.audit_log.len() > self.max_audit_size {
            self.audit_log.drain(0..self.audit_log.len() - self.max_audit_size);
        }
    }
    
    /// Get audit records for a user
    pub(super) fn get_user_audit(&self, user_id: &str) -> Vec<ValidationAuditRecord> {
        self.audit_log
            .iter()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }
    
    /// Get audit records for a resource
    pub(super) fn get_resource_audit(&self, resource: &str) -> Vec<ValidationAuditRecord> {
        self.audit_log
            .iter()
            .filter(|r| r.resource == resource)
            .cloned()
            .collect()
    }
    
    /// Get all audit records
    pub(super) fn get_all_audit(&self) -> Vec<ValidationAuditRecord> {
        self.audit_log.clone()
    }
    
    /// Clear audit records
    pub(super) fn clear_audit(&mut self) {
        self.audit_log.clear();
    }
    
    /// Set maximum audit log size
    pub(super) fn set_max_audit_size(&mut self, size: usize) {
        self.max_audit_size = size;
        
        // Trim audit log if it exceeds the new maximum size
        if self.audit_log.len() > self.max_audit_size {
            self.audit_log.drain(0..self.audit_log.len() - self.max_audit_size);
        }
    }
}

/// Thread-safe permission validator
#[derive(Debug)]
pub struct AsyncPermissionValidator {
    /// Inner permission validator
    validator: RwLock<PermissionValidator>,
}

impl Default for AsyncPermissionValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncPermissionValidator {
    /// Create a new async permission validator
    #[must_use] pub fn new() -> Self {
        Self {
            validator: RwLock::new(PermissionValidator::new()),
        }
    }
    
    /// Add a validation rule to the permission validator.
    ///
    /// # Arguments
    /// * `rule` - The validation rule to add
    ///
    /// # Returns
    /// * `Ok(())` if the rule was added successfully
    ///
    /// # Errors
    /// * Returns a `SecurityError::RBACError` if the resource pattern regex is invalid
    /// * Returns an error if the rule could not be added due to internal failures
    pub async fn add_rule(&self, rule: ValidationRule) -> Result<()> {
        let mut validator = self.validator.write().await;
        validator.add_rule(rule)
    }
    
    /// Remove a validation rule
    pub async fn remove_rule(&self, rule_id: &str) {
        let mut validator = self.validator.write().await;
        validator.remove_rule(rule_id);
    }
    
    /// Validate a permission request
    pub async fn validate(
        &self,
        user_id: &str,
        resource: &str,
        action: Action,
        user_roles: &[Role],
        user_permissions: &HashSet<Permission>,
        context: &PermissionContext,
    ) -> ValidationResult {
        let mut validator = self.validator.write().await;
        validator.validate(user_id, resource, action, user_roles, user_permissions, context)
    }
    
    /// Get audit records for a user
    pub async fn get_user_audit(&self, user_id: &str) -> Vec<ValidationAuditRecord> {
        let validator = self.validator.read().await;
        validator.get_user_audit(user_id)
    }
    
    /// Get audit records for a resource
    pub async fn get_resource_audit(&self, resource: &str) -> Vec<ValidationAuditRecord> {
        let validator = self.validator.read().await;
        validator.get_resource_audit(resource)
    }
    
    /// Get all audit records
    pub async fn get_all_audit(&self) -> Vec<ValidationAuditRecord> {
        let validator = self.validator.read().await;
        validator.get_all_audit()
    }
    
    /// Clear audit records
    pub async fn clear_audit(&self) {
        let mut validator = self.validator.write().await;
        validator.clear_audit();
    }
    
    /// Set maximum audit log size
    pub async fn set_max_audit_size(&self, size: usize) {
        let mut validator = self.validator.write().await;
        validator.set_max_audit_size(size);
    }
    
    /// Get a rule by ID
    pub async fn get_rule(&self, rule_id: &str) -> Option<ValidationRule> {
        let validator = self.validator.read().await;
        validator.rules.iter().find(|r| r.id == rule_id).cloned()
    }
    
    /// Enable or disable audit logging
    pub async fn set_audit_enabled(&self, enabled: bool) {
        let mut validator = self.validator.write().await;
        validator.audit_enabled = enabled;
    }
}

// Make the module public
pub(super) fn validate_permission_format(permission: &str) -> Result<(String, String)> {
    // Format should be "action:resource"
    let parts: Vec<&str> = permission.split(':').collect();
    if parts.len() != 2 {
        return Err(SecurityError::RBACError(
            format!("Invalid permission format: {permission}, expected 'action:resource'")
        ).into());
    }
    
    let action = parts[0].trim();
    let resource = parts[1].trim();
    
    if action.is_empty() {
        return Err(SecurityError::RBACError(
            format!("Empty action in permission: {permission}")
        ).into());
    }
    
    if resource.is_empty() {
        return Err(SecurityError::RBACError(
            format!("Empty resource in permission: {permission}")
        ).into());
    }
    
    Ok((action.to_string(), resource.to_string()))
}

/// Create a new permission
pub fn new(
    id: String,
    resource: String,
    action: Action,
    scope: Option<PermissionScope>,
    conditions: Vec<PermissionCondition>,
) -> Permission {
    Permission {
        id,
        resource: resource.clone(),
        action,
        scope,
        conditions,
        resource_id: resource,
    }
} 