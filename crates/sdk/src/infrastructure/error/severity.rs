// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error severity and categorization for the Squirrel Plugin SDK

use super::core::PluginError;
use serde::{Deserialize, Serialize};

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - warning level
    Low = 0,
    /// Medium severity - error level
    Medium = 1,
    /// High severity - critical error
    High = 2,
    /// Critical severity - system failure
    Critical = 3,
}

impl ErrorSeverity {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "LOW",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Critical => "CRITICAL",
        }
    }
}

/// Error categories for better organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// User input or parameter errors
    User,
    /// Network and communication errors
    Network,
    /// File system and storage errors
    Storage,
    /// Configuration and setup errors
    Configuration,
    /// Security and authentication errors
    Security,
    /// Plugin lifecycle and management errors
    Plugin,
    /// System and resource errors
    System,
    /// External service errors
    External,
    /// Development and integration errors
    Development,
    /// Unknown or uncategorized errors
    Unknown,
}

impl ErrorCategory {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::User => "USER",
            ErrorCategory::Network => "NETWORK",
            ErrorCategory::Storage => "STORAGE",
            ErrorCategory::Configuration => "CONFIGURATION",
            ErrorCategory::Security => "SECURITY",
            ErrorCategory::Plugin => "PLUGIN",
            ErrorCategory::System => "SYSTEM",
            ErrorCategory::External => "EXTERNAL",
            ErrorCategory::Development => "DEVELOPMENT",
            ErrorCategory::Unknown => "UNKNOWN",
        }
    }
}

/// Extension trait for PluginError to provide severity and category information
pub trait PluginErrorClassification {
    /// Get the error severity
    fn severity(&self) -> ErrorSeverity;

    /// Get the error category
    fn category(&self) -> ErrorCategory;

    /// Check if this error is recoverable
    fn is_recoverable(&self) -> bool;

    /// Get suggested recovery actions
    fn recovery_suggestions(&self) -> Vec<String>;

    /// Get error type as string
    fn error_type(&self) -> &'static str;
}

// Backward compatibility: PluginError deprecated in favor of universal_error::sdk::SDKError
#[expect(
    deprecated,
    reason = "backward compat: PluginError during migration to universal_error::sdk::SDKError"
)]
impl PluginErrorClassification for PluginError {
    fn severity(&self) -> ErrorSeverity {
        match self {
            PluginError::Deprecated { .. } => ErrorSeverity::Low,
            PluginError::UnknownCommand { .. }
            | PluginError::MissingParameter { .. }
            | PluginError::InvalidParameter { .. }
            | PluginError::ValidationError { .. } => ErrorSeverity::Medium,
            PluginError::TimeoutError { .. }
            | PluginError::NetworkError { .. }
            | PluginError::FileSystemError { .. }
            | PluginError::ConfigurationError { .. }
            | PluginError::InvalidConfiguration { .. }
            | PluginError::ResourceLimitExceeded { .. }
            | PluginError::QuotaExceeded { .. } => ErrorSeverity::High,
            PluginError::SecurityViolation { .. }
            | PluginError::PermissionDenied { .. }
            | PluginError::InternalError { .. }
            | PluginError::InitializationError { .. }
            | PluginError::PermanentFailure { .. } => ErrorSeverity::Critical,
            _ => ErrorSeverity::Medium,
        }
    }

    fn category(&self) -> ErrorCategory {
        match self {
            PluginError::UnknownCommand { .. }
            | PluginError::MissingParameter { .. }
            | PluginError::InvalidParameter { .. }
            | PluginError::ValidationError { .. } => ErrorCategory::User,
            PluginError::NetworkError { .. }
            | PluginError::ConnectionError { .. }
            | PluginError::TimeoutError { .. }
            | PluginError::HttpError { .. }
            | PluginError::CommunicationError { .. } => ErrorCategory::Network,
            PluginError::FileSystemError { .. }
            | PluginError::StorageError { .. }
            | PluginError::CacheError { .. } => ErrorCategory::Storage,
            PluginError::ConfigurationError { .. } | PluginError::InvalidConfiguration { .. } => {
                ErrorCategory::Configuration
            }
            PluginError::SecurityViolation { .. }
            | PluginError::PermissionDenied { .. }
            | PluginError::AuthenticationError { .. }
            | PluginError::AuthorizationError { .. } => ErrorCategory::Security,
            PluginError::PluginNotFound { .. }
            | PluginError::PluginAlreadyExists { .. }
            | PluginError::InitializationError { .. }
            | PluginError::LifecycleError { .. }
            | PluginError::DependencyError { .. }
            | PluginError::VersionIncompatible { .. }
            | PluginError::InvalidVersion { .. } => ErrorCategory::Plugin,
            PluginError::ResourceLimitExceeded { .. }
            | PluginError::QuotaExceeded { .. }
            | PluginError::LockError { .. }
            | PluginError::InternalError { .. } => ErrorCategory::System,
            PluginError::ExternalServiceError { .. } | PluginError::McpError { .. } => {
                ErrorCategory::External
            }
            PluginError::NotImplemented { .. }
            | PluginError::NotSupported { .. }
            | PluginError::Deprecated { .. }
            | PluginError::JsError { .. }
            | PluginError::JsonError { .. }
            | PluginError::SerializationError { .. } => ErrorCategory::Development,
            _ => ErrorCategory::Unknown,
        }
    }

    fn is_recoverable(&self) -> bool {
        !matches!(
            self,
            PluginError::SecurityViolation { .. }
                | PluginError::PermissionDenied { .. }
                | PluginError::PermanentFailure { .. }
                | PluginError::NotImplemented { .. }
                | PluginError::NotSupported { .. }
                | PluginError::VersionIncompatible { .. }
                | PluginError::InternalError { .. }
                | PluginError::InitializationError { .. }
        )
    }

    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            PluginError::NetworkError { .. } => vec![
                "Check network connectivity".to_string(),
                "Retry the operation".to_string(),
                "Verify server availability".to_string(),
            ],
            PluginError::TimeoutError { .. } => vec![
                "Increase timeout duration".to_string(),
                "Check network latency".to_string(),
                "Retry with smaller requests".to_string(),
            ],
            PluginError::MissingParameter { .. } => vec![
                "Check parameter documentation".to_string(),
                "Verify required parameters".to_string(),
            ],
            PluginError::InvalidParameter { .. } => vec![
                "Check parameter format".to_string(),
                "Verify parameter values".to_string(),
                "Consult parameter documentation".to_string(),
            ],
            PluginError::ResourceLimitExceeded { .. } => vec![
                "Reduce resource usage".to_string(),
                "Increase resource limits".to_string(),
                "Optimize resource allocation".to_string(),
            ],
            PluginError::ConfigurationError { .. } => vec![
                "Check configuration file".to_string(),
                "Verify configuration syntax".to_string(),
                "Reset to default configuration".to_string(),
            ],
            PluginError::DependencyError { .. } => vec![
                "Check dependency availability".to_string(),
                "Update dependencies".to_string(),
                "Verify dependency versions".to_string(),
            ],
            _ => vec!["Consult documentation".to_string()],
        }
    }

    fn error_type(&self) -> &'static str {
        match self {
            PluginError::UnknownCommand { .. } => "UnknownCommand",
            PluginError::MissingParameter { .. } => "MissingParameter",
            PluginError::InvalidParameter { .. } => "InvalidParameter",
            PluginError::PermissionDenied { .. } => "PermissionDenied",
            PluginError::NetworkError { .. } => "NetworkError",
            PluginError::FileSystemError { .. } => "FileSystemError",
            PluginError::McpError { .. } => "McpError",
            PluginError::InitializationError { .. } => "InitializationError",
            PluginError::ConfigurationError { .. } => "ConfigurationError",
            PluginError::SerializationError { .. } => "SerializationError",
            PluginError::TimeoutError { .. } => "TimeoutError",
            PluginError::ResourceLimitExceeded { .. } => "ResourceLimitExceeded",
            PluginError::QuotaExceeded { .. } => "QuotaExceeded",
            PluginError::PluginNotFound { .. } => "PluginNotFound",
            PluginError::PluginAlreadyExists { .. } => "PluginAlreadyExists",
            PluginError::DependencyError { .. } => "DependencyError",
            PluginError::VersionIncompatible { .. } => "VersionIncompatible",
            PluginError::InvalidVersion { .. } => "InvalidVersion",
            PluginError::SecurityViolation { .. } => "SecurityViolation",
            PluginError::InternalError { .. } => "InternalError",
            PluginError::ExecutionError { .. } => "ExecutionError",
            PluginError::InvalidConfiguration { .. } => "InvalidConfiguration",
            PluginError::JsError { .. } => "JsError",
            PluginError::Unknown { .. } => "Unknown",
            PluginError::HttpError { .. } => "HttpError",
            PluginError::JsonError { .. } => "JsonError",
            PluginError::ValidationError { .. } => "ValidationError",
            PluginError::ConnectionError { .. } => "ConnectionError",
            PluginError::AuthenticationError { .. } => "AuthenticationError",
            PluginError::AuthorizationError { .. } => "AuthorizationError",
            PluginError::RateLimitError { .. } => "RateLimitError",
            PluginError::LifecycleError { .. } => "LifecycleError",
            PluginError::CommandExecutionError { .. } => "CommandExecutionError",
            PluginError::EventHandlingError { .. } => "EventHandlingError",
            PluginError::ContextError { .. } => "ContextError",
            PluginError::StorageError { .. } => "StorageError",
            PluginError::CacheError { .. } => "CacheError",
            PluginError::LockError { .. } => "LockError",
            PluginError::CommunicationError { .. } => "CommunicationError",
            PluginError::ResourceNotFound { .. } => "ResourceNotFound",
            PluginError::ResourceAlreadyExists { .. } => "ResourceAlreadyExists",
            PluginError::TemporaryFailure { .. } => "TemporaryFailure",
            PluginError::PermanentFailure { .. } => "PermanentFailure",
            PluginError::ExternalServiceError { .. } => "ExternalServiceError",
            PluginError::NotImplemented { .. } => "NotImplemented",
            PluginError::NotSupported { .. } => "NotSupported",
            PluginError::Deprecated { .. } => "Deprecated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(deprecated)] // Tests deprecated path for backward compatibility
    fn make_error(variant: &str) -> PluginError {
        match variant {
            "UnknownCommand" => PluginError::UnknownCommand {
                command: "test".into(),
            },
            "MissingParameter" => PluginError::MissingParameter {
                parameter: "test".into(),
            },
            "InvalidParameter" => PluginError::InvalidParameter {
                name: "test".into(),
                reason: "bad".into(),
            },
            "PermissionDenied" => PluginError::PermissionDenied {
                operation: "op".into(),
                reason: "denied".into(),
            },
            "NetworkError" => PluginError::NetworkError {
                operation: "fetch".into(),
                message: "timeout".into(),
            },
            "FileSystemError" => PluginError::FileSystemError {
                operation: "read".into(),
                message: "fail".into(),
            },
            "McpError" => PluginError::McpError {
                message: "protocol error".into(),
            },
            "InitializationError" => PluginError::InitializationError {
                reason: "fail".into(),
            },
            "ConfigurationError" => PluginError::ConfigurationError {
                message: "bad config".into(),
            },
            "SerializationError" => PluginError::SerializationError {
                message: "bad json".into(),
            },
            "TimeoutError" => PluginError::TimeoutError {
                operation: "fetch".into(),
                seconds: 30,
            },
            "ResourceLimitExceeded" => PluginError::ResourceLimitExceeded {
                resource: "memory".into(),
                limit: "1GB".into(),
            },
            "QuotaExceeded" => PluginError::QuotaExceeded {
                resource: "api".into(),
                message: "over limit".into(),
            },
            "PluginNotFound" => PluginError::PluginNotFound {
                plugin_id: "test-plugin".into(),
            },
            "PluginAlreadyExists" => PluginError::PluginAlreadyExists {
                plugin_id: "test-plugin".into(),
            },
            "DependencyError" => PluginError::DependencyError {
                dependency: "dep".into(),
                message: "missing".into(),
            },
            "VersionIncompatible" => PluginError::VersionIncompatible {
                required: "2.0".into(),
                found: "1.0".into(),
            },
            "InvalidVersion" => PluginError::InvalidVersion {
                version: "abc".into(),
                reason: "not semver".into(),
            },
            "SecurityViolation" => PluginError::SecurityViolation {
                violation: "unauthorized".into(),
            },
            "InternalError" => PluginError::InternalError {
                message: "crash".into(),
            },
            "ExecutionError" => PluginError::ExecutionError {
                context: "run".into(),
                message: "fail".into(),
            },
            "InvalidConfiguration" => PluginError::InvalidConfiguration {
                message: "bad".into(),
            },
            "Deprecated" => PluginError::Deprecated {
                feature: "old_api".into(),
                alternative: "new_api".into(),
            },
            "ValidationError" => PluginError::ValidationError {
                field: "name".into(),
                message: "empty".into(),
            },
            "ConnectionError" => PluginError::ConnectionError {
                endpoint: "host".into(),
                message: "refused".into(),
            },
            "AuthenticationError" => PluginError::AuthenticationError {
                message: "invalid creds".into(),
            },
            "AuthorizationError" => PluginError::AuthorizationError {
                resource: "admin".into(),
                message: "denied".into(),
            },
            "RateLimitError" => PluginError::RateLimitError {
                resource: "api".into(),
                retry_after: 60,
            },
            "LifecycleError" => PluginError::LifecycleError {
                state: "running".into(),
                target_state: "stopped".into(),
                message: "fail".into(),
            },
            "StorageError" => PluginError::StorageError {
                operation: "write".into(),
                message: "disk full".into(),
            },
            "CacheError" => PluginError::CacheError {
                operation: "get".into(),
                message: "expired".into(),
            },
            "LockError" => PluginError::LockError {
                resource: "mutex".into(),
                message: "deadlock".into(),
            },
            "CommunicationError" => PluginError::CommunicationError {
                target: "service".into(),
                message: "fail".into(),
            },
            "TemporaryFailure" => PluginError::TemporaryFailure {
                operation: "call".into(),
                message: "retry".into(),
            },
            "PermanentFailure" => PluginError::PermanentFailure {
                operation: "init".into(),
                message: "fatal".into(),
            },
            "ExternalServiceError" => PluginError::ExternalServiceError {
                service: "api".into(),
                message: "down".into(),
            },
            "NotImplemented" => PluginError::NotImplemented {
                feature: "streaming".into(),
            },
            "NotSupported" => PluginError::NotSupported {
                feature: "gpu".into(),
            },
            "HttpError" => PluginError::HttpError {
                status: 500,
                message: "server error".into(),
            },
            "JsonError" => PluginError::JsonError {
                message: "parse fail".into(),
            },
            "JsError" => PluginError::JsError {
                message: "js crash".into(),
            },
            "ResourceNotFound" => PluginError::ResourceNotFound {
                resource: "file".into(),
            },
            "ResourceAlreadyExists" => PluginError::ResourceAlreadyExists {
                resource: "file".into(),
            },
            "CommandExecutionError" => PluginError::CommandExecutionError {
                command: "run".into(),
                message: "fail".into(),
            },
            "EventHandlingError" => PluginError::EventHandlingError {
                event_type: "click".into(),
                message: "fail".into(),
            },
            "ContextError" => PluginError::ContextError {
                context: "ctx".into(),
                message: "lost".into(),
            },
            "Unknown" => PluginError::Unknown {
                message: "???".into(),
            },
            _ => PluginError::Unknown {
                message: variant.into(),
            },
        }
    }

    #[test]
    fn test_error_severity_as_str() {
        assert_eq!(ErrorSeverity::Low.as_str(), "LOW");
        assert_eq!(ErrorSeverity::Medium.as_str(), "MEDIUM");
        assert_eq!(ErrorSeverity::High.as_str(), "HIGH");
        assert_eq!(ErrorSeverity::Critical.as_str(), "CRITICAL");
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Low < ErrorSeverity::Medium);
        assert!(ErrorSeverity::Medium < ErrorSeverity::High);
        assert!(ErrorSeverity::High < ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_severity_serde() {
        let severity = ErrorSeverity::High;
        let json = serde_json::to_string(&severity).unwrap();
        let deserialized: ErrorSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, severity);
    }

    #[test]
    fn test_error_category_as_str() {
        assert_eq!(ErrorCategory::User.as_str(), "USER");
        assert_eq!(ErrorCategory::Network.as_str(), "NETWORK");
        assert_eq!(ErrorCategory::Storage.as_str(), "STORAGE");
        assert_eq!(ErrorCategory::Configuration.as_str(), "CONFIGURATION");
        assert_eq!(ErrorCategory::Security.as_str(), "SECURITY");
        assert_eq!(ErrorCategory::Plugin.as_str(), "PLUGIN");
        assert_eq!(ErrorCategory::System.as_str(), "SYSTEM");
        assert_eq!(ErrorCategory::External.as_str(), "EXTERNAL");
        assert_eq!(ErrorCategory::Development.as_str(), "DEVELOPMENT");
        assert_eq!(ErrorCategory::Unknown.as_str(), "UNKNOWN");
    }

    #[test]
    fn test_error_category_serde() {
        let category = ErrorCategory::Network;
        let json = serde_json::to_string(&category).unwrap();
        let deserialized: ErrorCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, category);
    }

    #[test]
    fn test_severity_low_for_deprecated() {
        let err = make_error("Deprecated");
        assert_eq!(err.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_severity_medium_for_user_errors() {
        for variant in &[
            "UnknownCommand",
            "MissingParameter",
            "InvalidParameter",
            "ValidationError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.severity(),
                ErrorSeverity::Medium,
                "Expected Medium for {}",
                variant
            );
        }
    }

    #[test]
    fn test_severity_high_for_operational_errors() {
        for variant in &[
            "TimeoutError",
            "NetworkError",
            "FileSystemError",
            "ConfigurationError",
            "InvalidConfiguration",
            "ResourceLimitExceeded",
            "QuotaExceeded",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.severity(),
                ErrorSeverity::High,
                "Expected High for {}",
                variant
            );
        }
    }

    #[test]
    fn test_severity_critical_for_security_and_internal() {
        for variant in &[
            "SecurityViolation",
            "PermissionDenied",
            "InternalError",
            "InitializationError",
            "PermanentFailure",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.severity(),
                ErrorSeverity::Critical,
                "Expected Critical for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_user() {
        for variant in &[
            "UnknownCommand",
            "MissingParameter",
            "InvalidParameter",
            "ValidationError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::User,
                "Expected User for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_network() {
        for variant in &[
            "NetworkError",
            "ConnectionError",
            "TimeoutError",
            "HttpError",
            "CommunicationError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Network,
                "Expected Network for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_storage() {
        for variant in &["FileSystemError", "StorageError", "CacheError"] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Storage,
                "Expected Storage for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_configuration() {
        for variant in &["ConfigurationError", "InvalidConfiguration"] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Configuration,
                "Expected Configuration for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_security() {
        for variant in &[
            "SecurityViolation",
            "PermissionDenied",
            "AuthenticationError",
            "AuthorizationError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Security,
                "Expected Security for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_plugin() {
        for variant in &[
            "PluginNotFound",
            "PluginAlreadyExists",
            "InitializationError",
            "LifecycleError",
            "DependencyError",
            "VersionIncompatible",
            "InvalidVersion",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Plugin,
                "Expected Plugin for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_system() {
        for variant in &[
            "ResourceLimitExceeded",
            "QuotaExceeded",
            "LockError",
            "InternalError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::System,
                "Expected System for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_external() {
        for variant in &["ExternalServiceError", "McpError"] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::External,
                "Expected External for {}",
                variant
            );
        }
    }

    #[test]
    fn test_category_development() {
        for variant in &[
            "NotImplemented",
            "NotSupported",
            "Deprecated",
            "JsError",
            "JsonError",
            "SerializationError",
        ] {
            let err = make_error(variant);
            assert_eq!(
                err.category(),
                ErrorCategory::Development,
                "Expected Development for {}",
                variant
            );
        }
    }

    #[test]
    fn test_is_recoverable_true() {
        for variant in &[
            "NetworkError",
            "TimeoutError",
            "TemporaryFailure",
            "StorageError",
            "ConfigurationError",
        ] {
            let err = make_error(variant);
            assert!(err.is_recoverable(), "Expected recoverable for {}", variant);
        }
    }

    #[test]
    fn test_is_recoverable_false() {
        for variant in &[
            "SecurityViolation",
            "PermissionDenied",
            "PermanentFailure",
            "NotImplemented",
            "NotSupported",
            "VersionIncompatible",
            "InternalError",
            "InitializationError",
        ] {
            let err = make_error(variant);
            assert!(
                !err.is_recoverable(),
                "Expected NOT recoverable for {}",
                variant
            );
        }
    }

    #[test]
    fn test_recovery_suggestions_network() {
        let err = make_error("NetworkError");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("network")));
    }

    #[test]
    fn test_recovery_suggestions_timeout() {
        let err = make_error("TimeoutError");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("timeout")));
    }

    #[test]
    fn test_recovery_suggestions_missing_param() {
        let err = make_error("MissingParameter");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 1);
    }

    #[test]
    fn test_recovery_suggestions_resource_limit() {
        let err = make_error("ResourceLimitExceeded");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("resource")));
    }

    #[test]
    fn test_recovery_suggestions_config() {
        let err = make_error("ConfigurationError");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("configuration")));
    }

    #[test]
    fn test_recovery_suggestions_dependency() {
        let err = make_error("DependencyError");
        let suggestions = err.recovery_suggestions();
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("dependency")));
    }

    #[test]
    fn test_recovery_suggestions_default() {
        let err = make_error("Unknown");
        let suggestions = err.recovery_suggestions();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].contains("documentation"));
    }

    #[test]
    fn test_error_type_all_variants() {
        let variants = vec![
            ("UnknownCommand", "UnknownCommand"),
            ("MissingParameter", "MissingParameter"),
            ("InvalidParameter", "InvalidParameter"),
            ("PermissionDenied", "PermissionDenied"),
            ("NetworkError", "NetworkError"),
            ("FileSystemError", "FileSystemError"),
            ("McpError", "McpError"),
            ("InitializationError", "InitializationError"),
            ("ConfigurationError", "ConfigurationError"),
            ("SerializationError", "SerializationError"),
            ("TimeoutError", "TimeoutError"),
            ("SecurityViolation", "SecurityViolation"),
            ("InternalError", "InternalError"),
            ("Deprecated", "Deprecated"),
            ("NotImplemented", "NotImplemented"),
            ("NotSupported", "NotSupported"),
            ("ExternalServiceError", "ExternalServiceError"),
            ("PermanentFailure", "PermanentFailure"),
            ("TemporaryFailure", "TemporaryFailure"),
            ("Unknown", "Unknown"),
        ];
        for (input, expected) in variants {
            let err = make_error(input);
            assert_eq!(
                err.error_type(),
                expected,
                "error_type mismatch for {}",
                input
            );
        }
    }
}
