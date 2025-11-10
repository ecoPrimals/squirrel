//! Tools Error Types
//!
//! This module provides error types for Squirrel tools including AI tools,
//! CLI commands, and the rule system, following the MCP error architecture pattern.
//!
//! # Architecture
//!
//! ```text
//! ToolsError
//!     ├── AITools (providers, routers, local AI)
//!     ├── CLI (commands, plugins, configuration)
//!     ├── RuleSystem (execution, validation)
//!     └── General (catch-all for tool operations)
//! ```
//!
//! # Examples
//!
//! ```
//! use universal_error::tools::{ToolsError, AIToolsError};
//!
//! fn call_ai_provider(prompt: &str) -> Result<String, ToolsError> {
//!     if prompt.is_empty() {
//!         return Err(AIToolsError::Provider(
//!             "Prompt cannot be empty".to_string()
//!         ).into());
//!     }
//!     Ok("response".to_string())
//! }
//! ```

use thiserror::Error;
use super::{ErrorContextTrait, ErrorSeverity};

/// Top-level Tools error type
///
/// This encompasses all tools-related errors with automatic conversions
/// from sub-domain errors via `#[from]` attribute.
#[derive(Error, Debug, Clone)]
pub enum ToolsError {
    /// Error originating from AI tools
    #[error(transparent)]
    AITools(#[from] AIToolsError),
    
    /// Error originating from CLI
    #[error(transparent)]
    CLI(#[from] CLIError),
    
    /// Error originating from rule system
    #[error(transparent)]
    RuleSystem(#[from] RuleSystemError),
    
    /// General tools error
    #[error("Tools error: {0}")]
    General(String),
}

/// AI Tools-related errors
///
/// Covers AI providers, routers, and local AI operations.
#[derive(Error, Debug, Clone)]
pub enum AIToolsError {
    /// AI provider error (OpenAI, Anthropic, etc.)
    #[error("AI provider error: {0}")]
    Provider(String),
    
    /// Router error (request routing, load balancing)
    #[error("Router error: {0}")]
    Router(String),
    
    /// Local AI error (Ollama, native models)
    #[error("Local AI error: {0}")]
    Local(String),
    
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded for {0}")]
    RateLimitExceeded(String),
    
    /// Invalid response from AI
    #[error("Invalid AI response: {0}")]
    InvalidResponse(String),
    
    /// Network/HTTP error
    #[error("Network error: {0}")]
    Network(String),
    
    /// API error from AI service
    #[error("API error: {0}")]
    Api(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Parse/serialization error
    #[error("Parse error: {0}")]
    Parse(String),
    
    /// Unsupported provider
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
    
    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
}

/// CLI-related errors
///
/// Covers command execution, plugin management, and CLI configuration.
#[derive(Error, Debug, Clone)]
pub enum CLIError {
    /// Command execution error
    #[error("Command error: {0}")]
    Command(String),
    
    /// Plugin error
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    /// Configuration error
    #[error("CLI configuration error: {0}")]
    Configuration(String),
    
    /// Unknown command
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    
    /// Missing required argument
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    /// Invalid argument
    #[error("Invalid argument '{0}': {1}")]
    InvalidArgument(String, String),
}

/// Rule System-related errors
///
/// Covers rule execution, validation, and rule management.
#[derive(Error, Debug, Clone)]
pub enum RuleSystemError {
    /// Rule execution error
    #[error("Rule execution error: {0}")]
    Execution(String),
    
    /// Rule validation error
    #[error("Rule validation error: {0}")]
    Validation(String),
    
    /// Rule not found
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    
    /// Rule conflict
    #[error("Rule conflict: {0}")]
    Conflict(String),
    
    /// Rule parsing error
    #[error("Rule parsing error: {0}")]
    Parsing(String),
}

// Implement ErrorContextTrait for Tools errors following MCP pattern
impl ErrorContextTrait for ToolsError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            ToolsError::AITools(_) => ErrorSeverity::Medium,
            ToolsError::CLI(_) => ErrorSeverity::Low,
            ToolsError::RuleSystem(_) => ErrorSeverity::Medium,
            ToolsError::General(_) => ErrorSeverity::Low,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Tools")
    }
    
    fn is_recoverable(&self) -> bool {
        match self {
            ToolsError::AITools(AIToolsError::RateLimitExceeded(_)) => true,
            ToolsError::CLI(CLIError::MissingArgument(_)) => true,
            ToolsError::CLI(CLIError::InvalidArgument(_, _)) => true,
            _ => false,
        }
    }
}

impl ErrorContextTrait for AIToolsError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            AIToolsError::RateLimitExceeded(_) => ErrorSeverity::Medium,
            AIToolsError::ModelNotFound(_) => ErrorSeverity::High,
            AIToolsError::InvalidResponse(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Tools.AITools")
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(self, AIToolsError::RateLimitExceeded(_))
    }
}

impl ErrorContextTrait for CLIError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            CLIError::Configuration(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Low,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Tools.CLI")
    }
    
    fn is_recoverable(&self) -> bool {
        matches!(
            self,
            CLIError::MissingArgument(_) | CLIError::InvalidArgument(_, _) | CLIError::UnknownCommand(_)
        )
    }
}

impl ErrorContextTrait for RuleSystemError {
    
    fn severity(&self) -> ErrorSeverity {
        match self {
            RuleSystemError::Conflict(_) => ErrorSeverity::High,
            RuleSystemError::Execution(_) => ErrorSeverity::High,
            _ => ErrorSeverity::Medium,
        }
    }
    
    fn component(&self) -> Option<&str> {
        Some("Tools.RuleSystem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ai_tools_error() {
        let err = AIToolsError::RateLimitExceeded("OpenAI".to_string());
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_cli_error() {
        let err = CLIError::MissingArgument("--config".to_string());
        assert!(err.is_recoverable());
    }
    
    #[test]
    fn test_rule_system_error() {
        let err = RuleSystemError::Conflict("duplicate rule".to_string());
        assert_eq!(err.severity(), ErrorSeverity::High);
    }
    
    #[test]
    fn test_tools_error_conversion() {
        let ai_err = AIToolsError::Provider("test".to_string());
        let tools_err: ToolsError = ai_err.into();
        assert!(matches!(tools_err, ToolsError::AITools(_)));
        assert_eq!(tools_err.component(), Some("Tools"));
    }
}

