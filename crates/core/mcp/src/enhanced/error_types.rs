//! Enhanced Error Types for MCP Platform
//!
//! This module provides comprehensive error handling to replace unwrap/expect
//! patterns and provide better error context for debugging and monitoring.

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Enhanced MCP Platform Error Types
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum EnhancedMCPError {
    /// Provider initialization failed
    #[error("Provider initialization failed: {provider} - {reason}")]
    ProviderInitialization { 
        provider: String, 
        reason: String,
    },
    
    /// Configuration validation failed
    #[error("Configuration validation failed: {field} - {reason}")]
    ConfigurationValidation { 
        field: String, 
        reason: String,
        provided_value: Option<String>,
    },
    
    /// Platform startup failed
    #[error("Platform startup failed: {component} - {reason}")]
    PlatformStartup { 
        component: String, 
        reason: String,
        retry_possible: bool,
    },
    
    /// Request processing failed
    #[error("Request processing failed: {request_id} - {reason}")]
    RequestProcessing { 
        request_id: String, 
        reason: String,
        provider: Option<String>,
        model: Option<String>,
    },
    
    /// Network configuration error
    #[error("Network configuration error: {setting} - {reason}")]
    NetworkConfiguration {
        setting: String,
        reason: String,
        suggested_value: Option<String>,
    },
    
    /// Authentication/Authorization error
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed {
        reason: String,
        retry_allowed: bool,
        required_permissions: Vec<String>,
    },
    
    /// Resource exhaustion error
    #[error("Resource exhausted: {resource} - {reason}")]
    ResourceExhausted {
        resource: String,
        reason: String,
        current_usage: Option<u64>,
        limit: Option<u64>,
    },
    
    /// Plugin operation error
    #[error("Plugin operation failed: {plugin_id} - {operation} - {reason}")]
    PluginOperation {
        plugin_id: String,
        operation: String,
        reason: String,
        recovery_possible: bool,
    },
    
    /// Streaming error
    #[error("Streaming error: {stream_id} - {reason}")]
    StreamingError {
        stream_id: String,
        reason: String,
        stream_type: String,
        recoverable: bool,
    },
    
    /// Session management error
    #[error("Session error: {session_id} - {reason}")]
    SessionError {
        session_id: String,
        reason: String,
        user_id: Option<String>,
    },
    
    /// Input validation error
    #[error("Input validation failed: {field} - {reason}")]
    InputValidation {
        field: String,
        reason: String,
        provided_value: Option<String>,
        expected_format: Option<String>,
    },
    
    /// Protocol communication error
    #[error("Protocol error: {protocol} - {reason}")]
    ProtocolError {
        protocol: String,
        reason: String,
        message_type: Option<String>,
        recovery_action: Option<String>,
    },
    
    /// Tool execution error
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolExecution {
        tool_name: String,
        reason: String,
        parameters: Option<String>,
        execution_time: Option<std::time::Duration>,
    },
    
    /// Model-related error
    #[error("Model error: {model} - {reason}")]
    ModelError {
        model: String,
        reason: String,
        provider: Option<String>,
        available_models: Vec<String>,
    },
    
    /// Performance threshold exceeded
    #[error("Performance threshold exceeded: {metric} - current: {current_value}, limit: {threshold}")]
    PerformanceThreshold {
        metric: String,
        current_value: f64,
        threshold: f64,
        suggested_action: Option<String>,
    },
    
    /// Security violation
    #[error("Security violation: {violation_type} - {reason}")]
    SecurityViolation {
        violation_type: String,
        reason: String,
        user_id: Option<String>,
        action_taken: Option<String>,
    },
}

impl EnhancedMCPError {
    /// Create a provider initialization error
    pub fn provider_init(provider: &str, reason: impl fmt::Display) -> Self {
        Self::ProviderInitialization {
            provider: provider.to_string(),
            reason: reason.to_string(),
        }
    }
    
    /// Create a configuration validation error
    pub fn config_validation(field: &str, reason: impl fmt::Display, provided_value: Option<&str>) -> Self {
        Self::ConfigurationValidation {
            field: field.to_string(),
            reason: reason.to_string(),
            provided_value: provided_value.map(|s| s.to_string()),
        }
    }
    
    /// Create a platform startup error
    pub fn platform_startup(component: &str, reason: impl fmt::Display, retry_possible: bool) -> Self {
        Self::PlatformStartup {
            component: component.to_string(),
            reason: reason.to_string(),
            retry_possible,
        }
    }
    
    /// Create a request processing error
    pub fn request_processing(
        request_id: &str, 
        reason: impl fmt::Display,
        provider: Option<&str>,
        model: Option<&str>
    ) -> Self {
        Self::RequestProcessing {
            request_id: request_id.to_string(),
            reason: reason.to_string(),
            provider: provider.map(|s| s.to_string()),
            model: model.map(|s| s.to_string()),
        }
    }
    
    /// Create a network configuration error
    pub fn network_config(setting: &str, reason: impl fmt::Display, suggested_value: Option<&str>) -> Self {
        Self::NetworkConfiguration {
            setting: setting.to_string(),
            reason: reason.to_string(),
            suggested_value: suggested_value.map(|s| s.to_string()),
        }
    }
    
    /// Create an input validation error
    pub fn input_validation(
        field: &str, 
        reason: impl fmt::Display, 
        provided_value: Option<&str>,
        expected_format: Option<&str>
    ) -> Self {
        Self::InputValidation {
            field: field.to_string(),
            reason: reason.to_string(),
            provided_value: provided_value.map(|s| s.to_string()),
            expected_format: expected_format.map(|s| s.to_string()),
        }
    }
    
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::PlatformStartup { retry_possible, .. } => *retry_possible,
            Self::AuthenticationFailed { retry_allowed, .. } => *retry_allowed,
            Self::PluginOperation { recovery_possible, .. } => *recovery_possible,
            Self::StreamingError { recoverable, .. } => *recoverable,
            Self::SecurityViolation { .. } => false,
            _ => true, // Most errors are recoverable by default
        }
    }
    
    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::SecurityViolation { .. } => ErrorSeverity::Critical,
            Self::ResourceExhausted { .. } => ErrorSeverity::High,
            Self::PlatformStartup { .. } => ErrorSeverity::High,
            Self::ProviderInitialization { .. } => ErrorSeverity::Medium,
            Self::ConfigurationValidation { .. } => ErrorSeverity::Medium,
            Self::PerformanceThreshold { .. } => ErrorSeverity::Medium,
            Self::RequestProcessing { .. } => ErrorSeverity::Low,
            Self::InputValidation { .. } => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }
    
    /// Get suggested recovery actions
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::ConfigurationValidation { field, .. } => {
                vec![format!("Check {} configuration", field)]
            }
            Self::NetworkConfiguration { suggested_value: Some(value), .. } => {
                vec![format!("Try using: {}", value)]
            }
            Self::ProviderInitialization { provider, .. } => {
                vec![
                    format!("Check {} provider configuration", provider),
                    "Verify API keys and endpoints".to_string(),
                    "Check network connectivity".to_string(),
                ]
            }
            Self::AuthenticationFailed { required_permissions, .. } => {
                let mut suggestions = vec!["Check credentials".to_string()];
                if !required_permissions.is_empty() {
                    suggestions.push(format!("Required permissions: {}", required_permissions.join(", ")));
                }
                suggestions
            }
            Self::ModelError { available_models, .. } => {
                if !available_models.is_empty() {
                    vec![format!("Available models: {}", available_models.join(", "))]
                } else {
                    vec!["Check model availability".to_string()]
                }
            }
            Self::PerformanceThreshold { suggested_action: Some(action), .. } => {
                vec![action.clone()]
            }
            _ => vec!["Retry the operation".to_string()],
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Error context for logging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error: EnhancedMCPError,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub operation: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    pub fn new(
        error: EnhancedMCPError,
        component: &str,
        operation: &str,
    ) -> Self {
        Self {
            error,
            timestamp: chrono::Utc::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            user_id: None,
            session_id: None,
            request_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
    
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

/// Result type alias for Enhanced MCP operations
pub type EnhancedResult<T> = std::result::Result<T, EnhancedMCPError>;

/// Conversion from standard errors to Enhanced MCP errors
impl From<std::io::Error> for EnhancedMCPError {
    fn from(err: std::io::Error) -> Self {
        Self::ProtocolError {
            protocol: "IO".to_string(),
            reason: err.to_string(),
            message_type: None,
            recovery_action: Some("Check file permissions and disk space".to_string()),
        }
    }
}

impl From<serde_json::Error> for EnhancedMCPError {
    fn from(err: serde_json::Error) -> Self {
        Self::InputValidation {
            field: "JSON".to_string(),
            reason: err.to_string(),
            provided_value: None,
            expected_format: Some("Valid JSON format".to_string()),
        }
    }
}

impl From<reqwest::Error> for EnhancedMCPError {
    fn from(err: reqwest::Error) -> Self {
        Self::ProviderInitialization {
            provider: "HTTP Client".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<tokio::time::error::Elapsed> for EnhancedMCPError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        Self::PerformanceThreshold {
            metric: "Request Timeout".to_string(),
            current_value: 0.0,
            threshold: 0.0,
            suggested_action: Some("Increase timeout or optimize request".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let error = EnhancedMCPError::provider_init("openai", "Invalid API key");
        assert!(matches!(error, EnhancedMCPError::ProviderInitialization { .. }));
    }
    
    #[test]
    fn test_error_severity() {
        let error = EnhancedMCPError::SecurityViolation {
            violation_type: "Unauthorized access".to_string(),
            reason: "Invalid token".to_string(),
            user_id: None,
            action_taken: None,
        };
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }
    
    #[test]
    fn test_error_recoverability() {
        let recoverable_error = EnhancedMCPError::RequestProcessing {
            request_id: "test".to_string(),
            reason: "Temporary failure".to_string(),
            provider: None,
            model: None,
        };
        assert!(recoverable_error.is_recoverable());
        
        let non_recoverable_error = EnhancedMCPError::SecurityViolation {
            violation_type: "Malicious request".to_string(),
            reason: "Security policy violated".to_string(),
            user_id: None,
            action_taken: None,
        };
        assert!(!non_recoverable_error.is_recoverable());
    }
    
    #[test]
    fn test_error_context() {
        let error = EnhancedMCPError::config_validation("port", "Invalid port number", Some("abc"));
        let context = ErrorContext::new(error, "server", "initialization")
            .with_user_id("test-user")
            .with_session_id("test-session");
        
        assert_eq!(context.component, "server");
        assert_eq!(context.operation, "initialization");
        assert_eq!(context.user_id, Some("test-user".to_string()));
        assert_eq!(context.session_id, Some("test-session".to_string()));
    }
    
    #[test]
    fn test_recovery_suggestions() {
        let error = EnhancedMCPError::ProviderInitialization {
            provider: "openai".to_string(),
            reason: "API key invalid".to_string(),
        };
        
        let suggestions = error.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("openai")));
    }
} 