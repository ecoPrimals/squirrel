// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Attack detection methods for input validation
//!
//! This module contains methods for detecting various types of security attacks
//! in user input, including SQL injection, XSS, command injection, path traversal,
//! and NoSQL injection.

use regex::Regex;
use tracing::error;

use crate::observability::CorrelationId;

use super::types::{RiskLevel, SecurityViolation, ViolationType};

/// Detect SQL injection patterns in input
///
/// Scans input for common SQL injection attack patterns using compiled regex patterns.
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled SQL injection detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if SQL injection detected
/// - `None` if input is safe
pub fn detect_sql_injection(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    let input_lower = input.to_lowercase();

    for pattern in patterns {
        if pattern.is_match(&input_lower) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "sql_injection_detected",
                "SQL injection pattern detected in input"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::SqlInjection,
                description: "SQL injection pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "Use parameterized queries and input sanitization".to_string(),
                risk_level: RiskLevel::Critical,
            });
        }
    }

    None
}

/// Detect XSS (Cross-Site Scripting) attack patterns
///
/// Scans input for XSS attack vectors that could be used to inject malicious
/// JavaScript into rendered HTML.
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled XSS detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if XSS attack detected
/// - `None` if input is safe
pub fn detect_xss(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    let input_lower = input.to_lowercase();

    for pattern in patterns {
        if pattern.is_match(&input_lower) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "xss_attack_detected",
                "XSS attack pattern detected in input"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::XssAttack,
                description: "XSS attack pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "HTML encode output and validate input".to_string(),
                risk_level: RiskLevel::High,
            });
        }
    }

    None
}

/// Detect command injection patterns
///
/// Scans input for patterns that could be used to inject operating system
/// commands or shell metacharacters.
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled command injection detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if command injection detected
/// - `None` if input is safe
pub fn detect_command_injection(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    for pattern in patterns {
        if pattern.is_match(input) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "command_injection_detected",
                "Command injection pattern detected in input"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::CommandInjection,
                description: "Command injection pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "Avoid executing user input as commands".to_string(),
                risk_level: RiskLevel::Critical,
            });
        }
    }

    None
}

/// Detect path traversal patterns
///
/// Scans input for patterns that could be used to escape from intended
/// directory structures and access unauthorized files.
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled path traversal detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if path traversal detected
/// - `None` if input is safe
pub fn detect_path_traversal(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    for pattern in patterns {
        if pattern.is_match(input) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "path_traversal_detected",
                "Path traversal pattern detected in input"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::PathTraversal,
                description: "Path traversal pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "Validate and sanitize file paths".to_string(),
                risk_level: RiskLevel::High,
            });
        }
    }

    None
}

/// Detect NoSQL injection patterns
///
/// Scans input for patterns that could be used to inject malicious queries
/// into NoSQL databases (MongoDB, CouchDB, etc.).
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled NoSQL injection detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if NoSQL injection detected
/// - `None` if input is safe
pub fn detect_nosql_injection(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    for pattern in patterns {
        if pattern.is_match(input) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "nosql_injection_detected",
                "NoSQL injection pattern detected in input"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::NoSqlInjection,
                description: "NoSQL injection pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "Use proper NoSQL query builders and validation".to_string(),
                risk_level: RiskLevel::High,
            });
        }
    }

    None
}

/// Detect suspicious patterns that don't fit specific attack categories
///
/// Scans for generic suspicious patterns like excessive special characters,
/// unusual character sequences, or other anomalies.
///
/// ## Arguments
/// - `input`: The user input to check
/// - `patterns`: Compiled suspicious pattern detection patterns
/// - `correlation_id`: Request correlation ID for logging
///
/// ## Returns
/// - `Some(SecurityViolation)` if suspicious patterns detected
/// - `None` if input is safe
pub fn detect_suspicious_patterns(
    input: &str,
    patterns: &[Regex],
    correlation_id: &CorrelationId,
) -> Option<SecurityViolation> {
    for pattern in patterns {
        if pattern.is_match(input) {
            error!(
                correlation_id = %correlation_id,
                pattern = pattern.as_str(),
                operation = "suspicious_pattern_detected",
                "Suspicious input pattern detected"
            );

            return Some(SecurityViolation {
                violation_type: ViolationType::SuspiciousPattern,
                description: "Suspicious input pattern detected".to_string(),
                original_input: input.to_string(),
                suggested_action: "Review input for potential malicious content".to_string(),
                risk_level: RiskLevel::Medium,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn create_test_correlation_id() -> CorrelationId {
        CorrelationId::new()
    }

    #[test]
    fn test_detect_sql_injection() {
        let patterns = vec![Regex::new(r"(?i)(\bunion\b.*\bselect\b|\bor\b.*1\s*=\s*1)").unwrap()];
        let correlation_id = create_test_correlation_id();

        // Should detect SQL injection
        let result = detect_sql_injection("' OR 1=1 --", &patterns, &correlation_id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().violation_type, ViolationType::SqlInjection);

        // Should not detect in safe input
        let result = detect_sql_injection("normal user input", &patterns, &correlation_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_xss() {
        let patterns = vec![Regex::new(r"(?i)<script|javascript:|onerror=").unwrap()];
        let correlation_id = create_test_correlation_id();

        // Should detect XSS
        let result = detect_xss("<script>alert('xss')</script>", &patterns, &correlation_id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().violation_type, ViolationType::XssAttack);

        // Should not detect in safe input
        let result = detect_xss("Hello world", &patterns, &correlation_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_command_injection() {
        let patterns = vec![Regex::new(r"[;&|`$]|(\.\./)|(/bin/)").unwrap()];
        let correlation_id = create_test_correlation_id();

        // Should detect command injection
        let result = detect_command_injection("ls; rm -rf /", &patterns, &correlation_id);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().violation_type,
            ViolationType::CommandInjection
        );

        // Should not detect in safe input
        let result = detect_command_injection("filename.txt", &patterns, &correlation_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_path_traversal() {
        let patterns = vec![Regex::new(r"\.\./|\.\.\\").unwrap()];
        let correlation_id = create_test_correlation_id();

        // Should detect path traversal
        let result = detect_path_traversal("../../etc/passwd", &patterns, &correlation_id);
        assert!(result.is_some());
        assert_eq!(result.unwrap().violation_type, ViolationType::PathTraversal);

        // Should not detect in safe input
        let result = detect_path_traversal("safe/path/file.txt", &patterns, &correlation_id);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_nosql_injection() {
        let patterns = vec![Regex::new(r"(?i)\$where|\$ne|\$gt").unwrap()];
        let correlation_id = create_test_correlation_id();

        // Should detect NoSQL injection
        let result = detect_nosql_injection(r#"{"$ne": null}"#, &patterns, &correlation_id);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().violation_type,
            ViolationType::NoSqlInjection
        );

        // Should not detect in safe input
        let result = detect_nosql_injection(r#"{"name": "John"}"#, &patterns, &correlation_id);
        assert!(result.is_none());
    }
}
