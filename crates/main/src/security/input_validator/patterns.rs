// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Regex pattern compilation for attack detection
#![allow(dead_code)] // Security infrastructure awaiting orchestrator wiring
//!
//! Compiles and stores regex patterns used to detect various types
//! of injection attacks and malicious input patterns.

use crate::error::PrimalError;
use regex::Regex;

/// Compile SQL injection detection patterns
///
/// Patterns detect common SQL injection attempts including:
/// - SQL keywords (SELECT, INSERT, DELETE, etc.)
/// - SQL comments (--,#, /* */)
/// - Boolean logic attacks (OR 1=1)
/// - UNION-based attacks
/// - Stored procedure calls
pub fn compile_sql_injection_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![
        r"(\b(select|insert|update|delete|drop|create|alter|exec|execute|union|script)\b)",
        r"(--|#|/\*|\*/)",
        r"(\bor\b.*\b1\s*=\s*1\b)",
        r"(\bunion\b.*\bselect\b)",
        r"(';.*--)",
        r"(\bxp_\w+\b)",
        r"(\bsp_\w+\b)",
    ];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p)
                .map_err(|e| PrimalError::Internal(format!("Failed to compile SQL regex: {e}")))
        })
        .collect()
}

/// Compile XSS (Cross-Site Scripting) detection patterns
///
/// Patterns detect XSS attack vectors including:
/// - Script tags
/// - Event handlers (onclick, onload, etc.)
/// - Dangerous URI schemes (javascript:, data:, vbscript:)
/// - Embedded objects and iframes
pub fn compile_xss_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![
        r"<script[^>]*>",
        r"javascript:",
        r"on\w+\s*=",
        r"<iframe[^>]*>",
        r"<object[^>]*>",
        r"<embed[^>]*>",
        r"vbscript:",
        r"data:.*base64",
    ];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p)
                .map_err(|e| PrimalError::Internal(format!("Failed to compile XSS regex: {e}")))
        })
        .collect()
}

/// Compile command injection detection patterns
///
/// Patterns detect command injection attempts including:
/// - Shell metacharacters (;, |, &, `)
/// - Command substitution ($(), ${})
/// - Piping and redirection
/// - Logical operators (&&, ||)
pub fn compile_command_injection_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![
        r"[;&|`]",
        r"\$\(",
        r"\$\{",
        r"<\(",
        r">\(",
        r"\|\s*\w+",
        r"&&\s*\w+",
        r"\|\|\s*\w+",
    ];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p).map_err(|e| {
                PrimalError::Internal(format!("Failed to compile command injection regex: {e}"))
            })
        })
        .collect()
}

/// Compile path traversal detection patterns
///
/// Patterns detect directory traversal attempts including:
/// - Relative path navigation (../)
/// - Windows-style traversal (..\)
/// - Absolute paths with traversal
pub fn compile_path_traversal_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![r"\.\./", r"\.\.\\", r"/\.\./", r"\\\.\.\\"];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p).map_err(|e| {
                PrimalError::Internal(format!("Failed to compile path traversal regex: {e}"))
            })
        })
        .collect()
}

/// Compile NoSQL injection detection patterns
///
/// Patterns detect NoSQL injection attempts including:
/// - MongoDB operators ($where, $ne, $gt, etc.)
/// - Query manipulation
/// - Object ID injection
pub fn compile_nosql_injection_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![
        r"\$where",
        r"\$ne",
        r"\$gt",
        r"\$lt",
        r"\$regex",
        r"\$or",
        r"\$and",
        r"ObjectId\(",
    ];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p)
                .map_err(|e| PrimalError::Internal(format!("Failed to compile NoSQL regex: {e}")))
        })
        .collect()
}

/// Compile suspicious pattern detection
///
/// Patterns detect general suspicious inputs including:
/// - Control characters
/// - Excessively long strings
pub fn compile_suspicious_patterns() -> Result<Vec<Regex>, PrimalError> {
    let patterns = vec![
        r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]", // Control characters
        r".{1000,}",                         // Very long strings
    ];

    patterns
        .into_iter()
        .map(|p| {
            Regex::new(p).map_err(|e| {
                PrimalError::Internal(format!("Failed to compile suspicious pattern regex: {e}"))
            })
        })
        .collect()
}

/// Compile sanitization regex patterns
///
/// Pre-compiled patterns for efficient input sanitization.
pub struct SanitizationPatterns {
    /// Script tag pattern
    pub script_regex: Regex,
    /// Dangerous HTML attributes pattern
    pub dangerous_attrs: Regex,
    /// HTML tag pattern
    pub tag_regex: Regex,
    /// Dangerous path characters
    pub path_dangerous_chars: Regex,
    /// Dangerous URL schemes
    pub url_dangerous_schemes: Regex,
    /// Dangerous email characters
    pub email_dangerous_chars: Regex,
    /// Control characters
    pub control_chars: Regex,
}

impl SanitizationPatterns {
    /// Compile all sanitization patterns
    pub fn compile() -> Result<Self, PrimalError> {
        Ok(Self {
            script_regex: Regex::new(r"(?i)<script[^>]*>.*?</script>").map_err(|e| {
                PrimalError::Internal(format!("Failed to compile script regex: {e}"))
            })?,
            dangerous_attrs: Regex::new(r"(?i)\s(on\w+|javascript:|data:|vbscript:)[^>]*")
                .map_err(|e| {
                    PrimalError::Internal(format!("Failed to compile dangerous attrs regex: {e}"))
                })?,
            tag_regex: Regex::new(r"</?([a-zA-Z0-9]+)[^>]*>")
                .map_err(|e| PrimalError::Internal(format!("Failed to compile tag regex: {e}")))?,
            path_dangerous_chars: Regex::new(r#"[<>:"|?*]"#).map_err(|e| {
                PrimalError::Internal(format!("Failed to compile path dangerous chars regex: {e}"))
            })?,
            url_dangerous_schemes: Regex::new(r"(?i)(javascript|data|vbscript):").map_err(|e| {
                PrimalError::Internal(format!("Failed to compile URL schemes regex: {e}"))
            })?,
            email_dangerous_chars: Regex::new(r#"[<>"']"#).map_err(|e| {
                PrimalError::Internal(format!("Failed to compile email chars regex: {e}"))
            })?,
            control_chars: Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]").map_err(|e| {
                PrimalError::Internal(format!("Failed to compile control chars regex: {e}"))
            })?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sql_injection_patterns() {
        let patterns = compile_sql_injection_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 7);
    }

    #[test]
    fn test_compile_xss_patterns() {
        let patterns = compile_xss_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 8);
    }

    #[test]
    fn test_compile_command_injection_patterns() {
        let patterns = compile_command_injection_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 8);
    }

    #[test]
    fn test_compile_path_traversal_patterns() {
        let patterns = compile_path_traversal_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 4);
    }

    #[test]
    fn test_compile_nosql_injection_patterns() {
        let patterns = compile_nosql_injection_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 8);
    }

    #[test]
    fn test_compile_suspicious_patterns() {
        let patterns = compile_suspicious_patterns().unwrap();
        assert!(!patterns.is_empty());
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_sanitization_patterns_compile() {
        let patterns = SanitizationPatterns::compile().unwrap();
        assert!(
            patterns
                .script_regex
                .is_match("<script>alert('xss')</script>")
        );
        assert!(patterns.dangerous_attrs.is_match(" onclick='bad()'"));
        assert!(patterns.tag_regex.is_match("<div>content</div>"));
    }

    #[test]
    fn test_sql_pattern_detection() {
        let patterns = compile_sql_injection_patterns().unwrap();
        let malicious = "'; DROP TABLE users--";

        let detected = patterns.iter().any(|p| p.is_match(malicious));
        assert!(detected);
    }

    #[test]
    fn test_xss_pattern_detection() {
        let patterns = compile_xss_patterns().unwrap();
        let malicious = "<script>alert('xss')</script>";

        let detected = patterns.iter().any(|p| p.is_match(malicious));
        assert!(detected);
    }
}
