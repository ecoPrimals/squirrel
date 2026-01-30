// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core input validation types
//!
//! Defines types for validation configuration, results, violations,
//! and risk levels used throughout the input validation system.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Input validation configuration
///
/// Comprehensive configuration for input validation behavior, including
/// length limits, attack detection toggles, and sanitization settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidationConfig {
    /// Maximum string length for general inputs
    pub max_string_length: usize,

    /// Maximum string length for large text fields
    pub max_text_length: usize,

    /// Maximum array/collection size
    pub max_array_length: usize,

    /// Maximum JSON object depth
    pub max_json_depth: usize,

    /// Enable HTML sanitization
    pub enable_html_sanitization: bool,

    /// Allowed HTML tags (when HTML is permitted)
    pub allowed_html_tags: HashSet<String>,

    /// Enable SQL injection detection
    pub enable_sql_injection_detection: bool,

    /// Enable XSS detection
    pub enable_xss_detection: bool,

    /// Enable path traversal detection
    pub enable_path_traversal_detection: bool,

    /// Enable command injection detection
    pub enable_command_injection_detection: bool,

    /// Enable NoSQL injection detection
    pub enable_nosql_injection_detection: bool,

    /// Strict mode (reject rather than sanitize suspicious input)
    pub strict_mode: bool,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        let mut allowed_html_tags = HashSet::new();
        allowed_html_tags.insert("b".to_string());
        allowed_html_tags.insert("i".to_string());
        allowed_html_tags.insert("em".to_string());
        allowed_html_tags.insert("strong".to_string());
        allowed_html_tags.insert("p".to_string());
        allowed_html_tags.insert("br".to_string());

        Self {
            max_string_length: 1000,
            max_text_length: 10000,
            max_array_length: 1000,
            max_json_depth: 10,
            enable_html_sanitization: true,
            allowed_html_tags,
            enable_sql_injection_detection: true,
            enable_xss_detection: true,
            enable_path_traversal_detection: true,
            enable_command_injection_detection: true,
            enable_nosql_injection_detection: true,
            strict_mode: true,
        }
    }
}

impl InputValidationConfig {
    /// Create a relaxed configuration for development
    pub fn relaxed() -> Self {
        Self {
            max_string_length: 5000,
            max_text_length: 50000,
            strict_mode: false,
            ..Default::default()
        }
    }

    /// Create a strict configuration for high-security environments
    pub fn strict() -> Self {
        Self {
            max_string_length: 500,
            max_text_length: 5000,
            max_array_length: 100,
            max_json_depth: 5,
            strict_mode: true,
            allowed_html_tags: HashSet::new(), // No HTML allowed
            ..Default::default()
        }
    }
}

/// Input validation result
///
/// Contains validation status, sanitized input, detected violations,
/// and overall risk assessment.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the input passed validation
    pub is_valid: bool,

    /// Sanitized version of the input (if applicable)
    pub sanitized_input: Option<String>,

    /// List of security violations detected
    pub violations: Vec<SecurityViolation>,

    /// Overall risk level
    pub risk_level: RiskLevel,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn valid(sanitized_input: String) -> Self {
        Self {
            is_valid: true,
            sanitized_input: Some(sanitized_input),
            violations: Vec::new(),
            risk_level: RiskLevel::Low,
        }
    }

    /// Create an invalid validation result
    pub fn invalid(violations: Vec<SecurityViolation>, risk_level: RiskLevel) -> Self {
        Self {
            is_valid: false,
            sanitized_input: None,
            violations,
            risk_level,
        }
    }

    /// Check if result has critical violations
    pub fn has_critical_violations(&self) -> bool {
        self.risk_level == RiskLevel::Critical
    }

    /// Get the highest risk level from violations
    pub fn calculate_risk_level(&self) -> RiskLevel {
        self.violations
            .iter()
            .map(|v| v.risk_level.clone())
            .max()
            .unwrap_or(RiskLevel::Low)
    }
}

/// Security violation detected in input
///
/// Describes a specific security violation with context about the
/// attack type, risk level, and recommended remediation.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityViolation {
    /// Type of violation detected
    pub violation_type: ViolationType,

    /// Human-readable description
    pub description: String,

    /// The original problematic input
    pub original_input: String,

    /// Suggested action to remediate
    pub suggested_action: String,

    /// Risk level of this violation
    pub risk_level: RiskLevel,
}

impl SecurityViolation {
    /// Create a new security violation
    pub fn new(
        violation_type: ViolationType,
        description: impl Into<String>,
        original_input: impl Into<String>,
        risk_level: RiskLevel,
    ) -> Self {
        let suggested_action = match &violation_type {
            ViolationType::SqlInjection => "Use parameterized queries",
            ViolationType::XssAttack => "Sanitize or reject HTML content",
            ViolationType::CommandInjection => "Reject shell metacharacters",
            ViolationType::PathTraversal => "Validate against allowed paths",
            ViolationType::NoSqlInjection => "Use typed database queries",
            ViolationType::MaliciousScript => "Remove script tags",
            ViolationType::ExcessiveLength => "Truncate to acceptable length",
            ViolationType::InvalidCharacters => "Remove invalid characters",
            ViolationType::SuspiciousPattern => "Review input manually",
            ViolationType::JsonDepthExceeded => "Reduce JSON nesting",
        };

        Self {
            violation_type,
            description: description.into(),
            original_input: original_input.into(),
            suggested_action: suggested_action.to_string(),
            risk_level,
        }
    }
}

/// Types of security violations
///
/// Categorizes different types of input attacks and validation failures.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ViolationType {
    /// SQL injection attack pattern
    SqlInjection,
    /// Cross-site scripting (XSS) attack
    XssAttack,
    /// Command injection attempt
    CommandInjection,
    /// Path traversal attempt
    PathTraversal,
    /// NoSQL injection pattern
    NoSqlInjection,
    /// Malicious script detected
    MaliciousScript,
    /// Input exceeds length limits
    ExcessiveLength,
    /// Invalid characters detected
    InvalidCharacters,
    /// Suspicious pattern detected
    SuspiciousPattern,
    /// JSON nesting exceeds limits
    JsonDepthExceeded,
}

/// Risk level for input validation violations
///
/// Ordered from least to most severe for comparison and sorting.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Input type classification for appropriate validation
///
/// Different input types require different validation strategies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputType {
    /// General text input
    Text,
    /// HTML content (requires sanitization)
    Html,
    /// File path
    FilePath,
    /// Database query parameter
    DatabaseParam,
    /// URL/URI
    Url,
    /// Email address
    Email,
    /// JSON data
    Json,
    /// Command line parameter
    CommandParam,
    /// Username/identifier
    Username,
    /// Password (special handling)
    Password,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_validation_config_default() {
        let config = InputValidationConfig::default();

        assert_eq!(config.max_string_length, 1000);
        assert_eq!(config.max_text_length, 10000);
        assert_eq!(config.max_array_length, 1000);
        assert!(config.enable_sql_injection_detection);
        assert!(config.enable_xss_detection);
        assert!(config.strict_mode);
        assert_eq!(config.allowed_html_tags.len(), 6);
    }

    #[test]
    fn test_config_relaxed() {
        let config = InputValidationConfig::relaxed();
        assert_eq!(config.max_string_length, 5000);
        assert!(!config.strict_mode);
    }

    #[test]
    fn test_config_strict() {
        let config = InputValidationConfig::strict();
        assert_eq!(config.max_string_length, 500);
        assert!(config.strict_mode);
        assert_eq!(config.allowed_html_tags.len(), 0);
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid("clean input".to_string());
        assert!(result.is_valid);
        assert_eq!(result.sanitized_input, Some("clean input".to_string()));
        assert_eq!(result.violations.len(), 0);
        assert!(!result.has_critical_violations());
    }

    #[test]
    fn test_validation_result_invalid() {
        let violation = SecurityViolation::new(
            ViolationType::SqlInjection,
            "SQL injection detected",
            "'; DROP TABLE users--",
            RiskLevel::Critical,
        );
        let result = ValidationResult::invalid(vec![violation], RiskLevel::Critical);

        assert!(!result.is_valid);
        assert!(result.sanitized_input.is_none());
        assert_eq!(result.violations.len(), 1);
        assert!(result.has_critical_violations());
    }

    #[test]
    fn test_violation_type_equality() {
        assert_eq!(ViolationType::SqlInjection, ViolationType::SqlInjection);
        assert_ne!(ViolationType::SqlInjection, ViolationType::XssAttack);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_security_violation_creation() {
        let violation = SecurityViolation::new(
            ViolationType::XssAttack,
            "XSS pattern detected",
            "<script>alert('xss')</script>",
            RiskLevel::High,
        );

        assert_eq!(violation.violation_type, ViolationType::XssAttack);
        assert_eq!(violation.risk_level, RiskLevel::High);
        assert!(!violation.suggested_action.is_empty());
    }

    #[test]
    fn test_calculate_risk_level() {
        let violations = vec![
            SecurityViolation::new(
                ViolationType::ExcessiveLength,
                "Too long",
                "long input",
                RiskLevel::Low,
            ),
            SecurityViolation::new(
                ViolationType::SqlInjection,
                "SQL detected",
                "malicious",
                RiskLevel::Critical,
            ),
        ];

        let result = ValidationResult::invalid(violations, RiskLevel::Critical);
        assert_eq!(result.calculate_risk_level(), RiskLevel::Critical);
    }
}
