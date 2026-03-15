// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Production Input Validation & Sanitization
//!
//! This module provides comprehensive input validation and sanitization to prevent:
//! - SQL injection attacks
//! - XSS (Cross-Site Scripting) attacks
//! - Command injection attacks
//! - Path traversal attacks
//! - JSON/XML injection attacks
//! - NoSQL injection attacks
//!
//! ## Architecture
//!
//! The module is organized into domain-driven sub-modules:
//! - `types`: Core validation types (config, results, violations)
//! - `patterns`: Attack detection pattern compilation
//! - `detection`: Attack detection methods
//! - `sanitization`: Input sanitization methods
//! - `mod`: Main orchestration (`ProductionInputValidator`)
//!
//! ## Usage
//!
//! ```ignore
//! use squirrel::security::input_validator::{
//!     ProductionInputValidator,
//!     InputValidationConfig,
//!     InputType,
//! };
//!
//! // Create validator with default config
//! let config = InputValidationConfig::default();
//! let validator = ProductionInputValidator::new(config)?;
//!
//! // Validate user input
//! let result = validator.validate_input(
//!     user_input,
//!     InputType::String,
//!     None,
//! );
//!
//! if result.is_valid {
//!     // Use sanitized input
//!     process(result.sanitized_input.unwrap());
//! } else {
//!     // Handle violations
//!     log_violations(result.violations);
//! }
//! ```

mod detection;
mod patterns;
mod sanitization;
mod types;

// Re-export public types
pub use types::{
    InputType, InputValidationConfig, RiskLevel, SecurityViolation, ValidationResult, ViolationType,
};

use regex::Regex;
use tracing::{debug, warn};

use crate::error::PrimalError;
use crate::observability::CorrelationId;

/// Production input validator with comprehensive security checks
///
/// Main orchestration struct that coordinates attack detection and input
/// sanitization using compiled regex patterns for high performance.
///
/// ## Performance
///
/// Regex patterns are compiled once at initialization and reused for all
/// validation operations, providing fast validation with minimal overhead.
///
/// ## Thread Safety
///
/// `ProductionInputValidator` is `Send + Sync` and can be shared across
/// threads (e.g., wrapped in `Arc`).
pub struct ProductionInputValidator {
    /// Configuration for validation behavior
    config: InputValidationConfig,

    /// Compiled attack detection patterns
    sql_injection_patterns: Vec<Regex>,
    xss_patterns: Vec<Regex>,
    command_injection_patterns: Vec<Regex>,
    path_traversal_patterns: Vec<Regex>,
    nosql_injection_patterns: Vec<Regex>,
    suspicious_patterns: Vec<Regex>,

    /// Compiled sanitization patterns
    sanitization_patterns: sanitization::SanitizationPatterns,
}

impl ProductionInputValidator {
    /// Create a new input validator
    ///
    /// Compiles all regex patterns at initialization for high-performance
    /// validation operations.
    ///
    /// ## Arguments
    /// - `config`: Configuration for validation behavior
    ///
    /// ## Returns
    /// - `Ok(ProductionInputValidator)` if all patterns compile successfully
    /// - `Err(PrimalError)` if pattern compilation fails
    ///
    /// ## Errors
    /// Returns error if any regex pattern fails to compile. This should never
    /// happen with our hardcoded patterns, but is handled for safety.
    ///
    /// ## Example
    /// ```ignore
    /// let config = InputValidationConfig::default();
    /// let validator = ProductionInputValidator::new(config)?;
    /// ```
    pub fn new(config: InputValidationConfig) -> Result<Self, PrimalError> {
        let validator = Self {
            config,
            sql_injection_patterns: patterns::compile_sql_injection_patterns()?,
            xss_patterns: patterns::compile_xss_patterns()?,
            command_injection_patterns: patterns::compile_command_injection_patterns()?,
            path_traversal_patterns: patterns::compile_path_traversal_patterns()?,
            nosql_injection_patterns: patterns::compile_nosql_injection_patterns()?,
            suspicious_patterns: patterns::compile_suspicious_patterns()?,
            sanitization_patterns: sanitization::SanitizationPatterns::compile()
                .map_err(PrimalError::Internal)?,
        };

        Ok(validator)
    }

    /// Validate and sanitize input based on type
    ///
    /// Performs comprehensive validation and sanitization of user input,
    /// detecting various types of attacks and providing sanitized output.
    ///
    /// ## Arguments
    /// - `input`: The user input to validate
    /// - `input_type`: Type of input (determines validation strategy)
    /// - `correlation_id`: Optional correlation ID for logging
    ///
    /// ## Returns
    /// `ValidationResult` containing:
    /// - `is_valid`: Whether input passed validation
    /// - `sanitized_input`: Sanitized version of input (if applicable)
    /// - `violations`: List of security violations detected
    /// - `risk_level`: Highest risk level among violations
    ///
    /// ## Behavior
    ///
    /// In **strict mode** (default):
    /// - Any non-low-risk violation → `is_valid = false`
    /// - No sanitization attempted
    /// - Input should be rejected
    ///
    /// In **non-strict mode**:
    /// - Only critical violations → `is_valid = false`
    /// - Sanitization attempted for medium/high violations
    /// - Sanitized input can be used
    ///
    /// ## Example
    /// ```ignore
    /// let result = validator.validate_input(
    ///     "user input",
    ///     InputType::String,
    ///     Some(correlation_id),
    /// );
    ///
    /// if result.is_valid {
    ///     let safe_input = result.sanitized_input.unwrap();
    ///     // Use safe_input
    /// } else {
    ///     // Reject or log violations
    ///     for violation in result.violations {
    ///         tracing::error!("Violation: {:?}", violation);
    ///     }
    /// }
    /// ```
    pub fn validate_input(
        &self,
        input: &str,
        input_type: InputType,
        correlation_id: Option<CorrelationId>,
    ) -> ValidationResult {
        let correlation_id = correlation_id.unwrap_or_default();
        let mut violations = Vec::new();
        let mut risk_level = RiskLevel::Low;

        debug!(
            correlation_id = %correlation_id,
            input_type = ?input_type,
            input_length = input.len(),
            operation = "input_validation_start",
            "Starting input validation"
        );

        // Check input length
        let max_length = match input_type {
            InputType::Text | InputType::Html => self.config.max_text_length,
            _ => self.config.max_string_length,
        };

        if input.len() > max_length {
            violations.push(SecurityViolation {
                violation_type: ViolationType::ExcessiveLength,
                description: format!(
                    "Input length {} exceeds maximum {}",
                    input.len(),
                    max_length
                ),
                original_input: input[..100.min(input.len())].to_string() + "...",
                suggested_action: "Truncate input or increase limits".to_string(),
                risk_level: RiskLevel::Medium,
            });
            risk_level = risk_level.max(RiskLevel::Medium);
        }

        // SQL Injection detection
        if self.config.enable_sql_injection_detection {
            if let Some(violation) = detection::detect_sql_injection(
                input,
                &self.sql_injection_patterns,
                &correlation_id,
            ) {
                risk_level = risk_level.max(violation.risk_level.clone());
                violations.push(violation);
            }
        }

        // XSS detection
        if self.config.enable_xss_detection {
            if let Some(violation) =
                detection::detect_xss(input, &self.xss_patterns, &correlation_id)
            {
                risk_level = risk_level.max(violation.risk_level.clone());
                violations.push(violation);
            }
        }

        // Command injection detection
        if self.config.enable_command_injection_detection {
            if let Some(violation) = detection::detect_command_injection(
                input,
                &self.command_injection_patterns,
                &correlation_id,
            ) {
                risk_level = risk_level.max(violation.risk_level.clone());
                violations.push(violation);
            }
        }

        // Path traversal detection (only for file paths)
        if self.config.enable_path_traversal_detection && input_type == InputType::FilePath {
            if let Some(violation) = detection::detect_path_traversal(
                input,
                &self.path_traversal_patterns,
                &correlation_id,
            ) {
                risk_level = risk_level.max(violation.risk_level.clone());
                violations.push(violation);
            }
        }

        // NoSQL injection detection
        if self.config.enable_nosql_injection_detection {
            if let Some(violation) = detection::detect_nosql_injection(
                input,
                &self.nosql_injection_patterns,
                &correlation_id,
            ) {
                risk_level = risk_level.max(violation.risk_level.clone());
                violations.push(violation);
            }
        }

        // Suspicious pattern detection
        if let Some(violation) =
            detection::detect_suspicious_patterns(input, &self.suspicious_patterns, &correlation_id)
        {
            risk_level = risk_level.max(violation.risk_level.clone());
            violations.push(violation);
        }

        // Determine if input is valid
        let is_valid = if self.config.strict_mode {
            violations.is_empty() || violations.iter().all(|v| v.risk_level == RiskLevel::Low)
        } else {
            violations
                .iter()
                .all(|v| v.risk_level != RiskLevel::Critical)
        };

        // Sanitize input if needed
        let sanitized_input = if !is_valid && !self.config.strict_mode {
            Some(sanitization::sanitize_input(
                input,
                &input_type,
                &self.config,
                &self.sanitization_patterns,
            ))
        } else if is_valid {
            Some(input.to_string())
        } else {
            None
        };

        if !violations.is_empty() {
            warn!(
                correlation_id = %correlation_id,
                violation_count = violations.len(),
                risk_level = ?risk_level,
                input_valid = is_valid,
                operation = "input_validation_violations",
                "Input validation violations detected"
            );
        }

        ValidationResult {
            is_valid,
            sanitized_input,
            violations,
            risk_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_validator() -> ProductionInputValidator {
        let config = InputValidationConfig::default();
        ProductionInputValidator::new(config).expect("Failed to create validator")
    }

    #[test]
    fn test_validator_creation() {
        let config = InputValidationConfig::default();
        let result = ProductionInputValidator::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_safe_input() {
        let validator = create_test_validator();
        let result = validator.validate_input("hello world", InputType::Text, None);

        assert!(result.is_valid);
        assert_eq!(result.violations.len(), 0);
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_validate_sql_injection() {
        let validator = create_test_validator();
        let result = validator.validate_input("' OR 1=1 --", InputType::DatabaseParam, None);

        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
        assert_eq!(result.risk_level, RiskLevel::Critical);
    }

    #[test]
    fn test_validate_xss_attack() {
        let validator = create_test_validator();
        let result =
            validator.validate_input("<script>alert('xss')</script>", InputType::Html, None);

        assert!(!result.is_valid);
        assert!(!result.violations.is_empty());
    }

    #[test]
    fn test_validate_excessive_length() {
        let validator = create_test_validator();
        // Text type uses max_text_length (10000), so use more than that
        let long_input = "a".repeat(15000);
        let result = validator.validate_input(&long_input, InputType::Text, None);

        assert!(!result.is_valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::ExcessiveLength));
    }

    #[test]
    fn test_validate_command_injection() {
        let validator = create_test_validator();
        let result = validator.validate_input("ls; rm -rf /", InputType::CommandParam, None);

        assert!(!result.is_valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::CommandInjection));
    }

    #[test]
    fn test_validate_path_traversal() {
        let validator = create_test_validator();
        let result = validator.validate_input("../../etc/passwd", InputType::FilePath, None);

        assert!(!result.is_valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.violation_type == ViolationType::PathTraversal));
    }

    #[test]
    fn test_sanitization_in_non_strict_mode() {
        let mut config = InputValidationConfig::default();
        config.strict_mode = false;

        let validator = ProductionInputValidator::new(config).unwrap();
        let result =
            validator.validate_input("hello<script>alert('xss')</script>", InputType::Html, None);

        // In non-strict mode, should attempt sanitization
        assert!(result.sanitized_input.is_some());
        let sanitized = result.sanitized_input.unwrap();

        // Script tags should be removed
        assert!(!sanitized.contains("<script>"));
    }
}
