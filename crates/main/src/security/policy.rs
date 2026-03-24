// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Security Policy Management
//!
//! This module contains types and functionality for managing security policies
//! and policy enforcement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy identifier
    pub policy_id: String,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Policy type
    pub policy_type: PolicyType,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
    /// Policy enforcement level
    pub enforcement_level: PolicyEnforcementLevel,
    /// Policy metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Policy version
    pub version: String,
    /// Policy status
    pub status: PolicyStatus,
}

/// Policy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyType {
    /// Authentication policy
    Authentication,
    /// Authorization policy
    Authorization,
    /// Access control policy
    AccessControl,
    /// Data protection policy
    DataProtection,
    /// Compliance policy
    Compliance,
    /// Security monitoring policy
    SecurityMonitoring,
    /// Threat detection policy
    ThreatDetection,
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name
    pub name: String,
    /// Rule condition
    pub condition: PolicyCondition,
    /// Rule action
    pub action: PolicyAction,
    /// Rule priority
    pub priority: u32,
    /// Rule enabled status
    pub enabled: bool,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    /// Condition type
    pub condition_type: String,
    /// Condition parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Condition operator
    pub operator: PolicyOperator,
}

/// Policy operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyOperator {
    /// Equals
    Equals,
    /// Not equals
    NotEquals,
    /// Contains
    Contains,
    /// Does not contain
    NotContains,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// In list
    In,
    /// Not in list
    NotIn,
    /// Regex match
    RegexMatch,
    /// And condition
    And,
    /// Or condition
    Or,
}

/// Policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAction {
    /// Action type
    pub action_type: PolicyActionType,
    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Policy action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyActionType {
    /// Allow the operation
    Allow,
    /// Deny the operation
    Deny,
    /// Log the operation
    Log,
    /// Alert on the operation
    Alert,
    /// Require additional authentication
    RequireAuth,
    /// Throttle the operation
    Throttle,
    /// Quarantine the resource
    Quarantine,
}

/// Policy enforcement levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyEnforcementLevel {
    /// Advisory only - log violations but don't block
    Advisory,
    /// Warning - log violations and warn but don't block
    Warning,
    /// Enforced - block operations that violate policy
    Enforced,
    /// Strict - block operations and alert on violations
    Strict,
}

/// Policy status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyStatus {
    /// Policy is active
    Active,
    /// Policy is inactive
    Inactive,
    /// Policy is being tested
    Testing,
    /// Policy is deprecated
    Deprecated,
}

impl SecurityPolicy {
    /// Create a new security policy
    #[must_use]
    pub fn new(policy_id: String, name: String, description: String) -> Self {
        let now = Utc::now();
        Self {
            policy_id,
            name,
            description,
            policy_type: PolicyType::AccessControl,
            rules: Vec::new(),
            enforcement_level: PolicyEnforcementLevel::Enforced,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            version: "1.0.0".to_string(),
            status: PolicyStatus::Active,
        }
    }

    /// Add a rule to the policy
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        self.updated_at = Utc::now();
    }

    /// Remove a rule from the policy
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|rule| rule.rule_id != rule_id);

        if self.rules.len() == initial_len {
            false
        } else {
            self.updated_at = Utc::now();
            true
        }
    }

    /// Update policy status
    pub fn update_status(&mut self, status: PolicyStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Check if policy is active
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == PolicyStatus::Active
    }

    /// Get enabled rules
    #[must_use]
    pub fn get_enabled_rules(&self) -> Vec<&PolicyRule> {
        self.rules.iter().filter(|rule| rule.enabled).collect()
    }

    /// Update policy version
    pub fn update_version(&mut self, version: String) {
        self.version = version;
        self.updated_at = Utc::now();
    }
}

impl PolicyRule {
    /// Create a new policy rule
    #[must_use]
    pub const fn new(
        rule_id: String,
        name: String,
        condition: PolicyCondition,
        action: PolicyAction,
    ) -> Self {
        Self {
            rule_id,
            name,
            condition,
            action,
            priority: 100,
            enabled: true,
        }
    }

    /// Set rule priority
    #[must_use]
    pub const fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable or disable rule
    pub const fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PolicyCondition {
    /// Create a new policy condition
    #[must_use]
    pub fn new(condition_type: String, operator: PolicyOperator) -> Self {
        Self {
            condition_type,
            parameters: HashMap::new(),
            operator,
        }
    }

    /// Add parameter to condition
    #[must_use]
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

impl PolicyAction {
    /// Create a new policy action
    #[must_use]
    pub fn new(action_type: PolicyActionType) -> Self {
        Self {
            action_type,
            parameters: HashMap::new(),
        }
    }

    /// Add parameter to action
    #[must_use]
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = SecurityPolicy::new(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            "A test security policy".to_string(),
        );

        assert_eq!(policy.policy_id, "test-policy");
        assert_eq!(policy.name, "Test Policy");
        assert_eq!(policy.description, "A test security policy");
        assert_eq!(policy.policy_type, PolicyType::AccessControl);
        assert_eq!(policy.status, PolicyStatus::Active);
        assert!(policy.is_active());
        assert_eq!(policy.rules.len(), 0);
    }

    #[test]
    fn test_policy_rules() {
        let mut policy = SecurityPolicy::new(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            "A test security policy".to_string(),
        );

        let condition = PolicyCondition::new("user_role".to_string(), PolicyOperator::Equals)
            .with_parameter("role".to_string(), serde_json::json!("admin"));

        let action = PolicyAction::new(PolicyActionType::Allow);

        let rule = PolicyRule::new(
            "rule-1".to_string(),
            "Admin Access Rule".to_string(),
            condition,
            action,
        )
        .with_priority(1);

        policy.add_rule(rule);

        assert_eq!(policy.rules.len(), 1);
        assert_eq!(policy.rules[0].rule_id, "rule-1");
        assert_eq!(policy.rules[0].priority, 1);
        assert!(policy.rules[0].enabled);

        let enabled_rules = policy.get_enabled_rules();
        assert_eq!(enabled_rules.len(), 1);

        assert!(policy.remove_rule("rule-1"));
        assert_eq!(policy.rules.len(), 0);
        assert!(!policy.remove_rule("nonexistent"));
    }

    #[test]
    fn test_policy_status() {
        let mut policy = SecurityPolicy::new(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            "A test security policy".to_string(),
        );

        assert!(policy.is_active());

        policy.update_status(PolicyStatus::Inactive);
        assert!(!policy.is_active());
        assert_eq!(policy.status, PolicyStatus::Inactive);

        policy.update_status(PolicyStatus::Testing);
        assert!(!policy.is_active());
        assert_eq!(policy.status, PolicyStatus::Testing);
    }
}
