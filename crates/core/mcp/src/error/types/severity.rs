// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error severity levels for categorizing and prioritizing errors.

use serde::{Deserialize, Serialize};

/// Error severity levels for categorizing and prioritizing errors.
///
/// Severity levels help determine error handling strategy, logging priority,
/// and whether immediate attention or alerts are required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - minimal impact, typically handled automatically
    Low,

    /// Medium severity - moderate impact, may require attention
    Medium,

    /// High severity - significant impact, requires attention
    High,

    /// Critical severity - severe impact, requires immediate attention
    Critical,
}

impl ErrorSeverity {
    /// Check if severity requires immediate attention
    pub const fn requires_immediate_attention(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Check if severity should trigger alerts
    pub const fn should_alert(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_requires_immediate_attention_low() {
        let severity = ErrorSeverity::Low;
        assert!(!severity.requires_immediate_attention());
    }

    #[test]
    fn test_severity_requires_immediate_attention_medium() {
        let severity = ErrorSeverity::Medium;
        assert!(!severity.requires_immediate_attention());
    }

    #[test]
    fn test_severity_requires_immediate_attention_high() {
        let severity = ErrorSeverity::High;
        assert!(severity.requires_immediate_attention());
    }

    #[test]
    fn test_severity_requires_immediate_attention_critical() {
        let severity = ErrorSeverity::Critical;
        assert!(severity.requires_immediate_attention());
    }

    #[test]
    fn test_severity_should_alert_low() {
        let severity = ErrorSeverity::Low;
        assert!(!severity.should_alert());
    }

    #[test]
    fn test_severity_should_alert_medium() {
        let severity = ErrorSeverity::Medium;
        assert!(!severity.should_alert());
    }

    #[test]
    fn test_severity_should_alert_high() {
        let severity = ErrorSeverity::High;
        assert!(severity.should_alert());
    }

    #[test]
    fn test_severity_should_alert_critical() {
        let severity = ErrorSeverity::Critical;
        assert!(severity.should_alert());
    }

    #[test]
    fn test_severity_equality() {
        assert_eq!(ErrorSeverity::Low, ErrorSeverity::Low);
        assert_eq!(ErrorSeverity::Medium, ErrorSeverity::Medium);
        assert_eq!(ErrorSeverity::High, ErrorSeverity::High);
        assert_eq!(ErrorSeverity::Critical, ErrorSeverity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(ErrorSeverity::Low, ErrorSeverity::Medium);
        assert_ne!(ErrorSeverity::Medium, ErrorSeverity::High);
        assert_ne!(ErrorSeverity::High, ErrorSeverity::Critical);
        assert_ne!(ErrorSeverity::Low, ErrorSeverity::Critical);
    }

    #[test]
    fn test_severity_clone() {
        let severity = ErrorSeverity::High;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_severity_copy() {
        let severity = ErrorSeverity::Medium;
        let copied = severity;
        assert_eq!(severity, copied);
    }

    #[test]
    fn test_severity_debug_formatting() {
        let low = ErrorSeverity::Low;
        let debug_str = format!("{low:?}");
        assert!(debug_str.contains("Low"));

        let critical = ErrorSeverity::Critical;
        let debug_str = format!("{critical:?}");
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_severity_serialization() {
        use serde_json;

        let severity = ErrorSeverity::High;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"High\"");

        let deserialized: ErrorSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, severity);
    }

    #[test]
    fn test_all_severities_serialization() {
        use serde_json;

        let severities = vec![
            ErrorSeverity::Low,
            ErrorSeverity::Medium,
            ErrorSeverity::High,
            ErrorSeverity::Critical,
        ];

        for severity in severities {
            let json = serde_json::to_string(&severity).unwrap();
            let deserialized: ErrorSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, severity);
        }
    }

    #[test]
    fn test_severity_pattern_matching() {
        fn classify_severity(severity: ErrorSeverity) -> &'static str {
            match severity {
                ErrorSeverity::Low => "can_ignore",
                ErrorSeverity::Medium => "should_log",
                ErrorSeverity::High => "must_handle",
                ErrorSeverity::Critical => "emergency",
            }
        }

        assert_eq!(classify_severity(ErrorSeverity::Low), "can_ignore");
        assert_eq!(classify_severity(ErrorSeverity::Medium), "should_log");
        assert_eq!(classify_severity(ErrorSeverity::High), "must_handle");
        assert_eq!(classify_severity(ErrorSeverity::Critical), "emergency");
    }

    #[test]
    fn test_severity_all_variants() {
        // Ensure all variants are tested
        let all_severities = [
            ErrorSeverity::Low,
            ErrorSeverity::Medium,
            ErrorSeverity::High,
            ErrorSeverity::Critical,
        ];

        assert_eq!(all_severities.len(), 4);

        // Verify each has unique behavior
        assert_eq!(
            all_severities
                .iter()
                .filter(|s| s.requires_immediate_attention())
                .count(),
            2
        );
        assert_eq!(
            all_severities.iter().filter(|s| s.should_alert()).count(),
            2
        );
    }
}
