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
        match self {
            PluginError::SecurityViolation { .. }
            | PluginError::PermissionDenied { .. }
            | PluginError::PermanentFailure { .. }
            | PluginError::NotImplemented { .. }
            | PluginError::NotSupported { .. }
            | PluginError::VersionIncompatible { .. } => false,
            PluginError::InternalError { .. } | PluginError::InitializationError { .. } => false,
            _ => true,
        }
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
