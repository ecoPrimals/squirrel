// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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

use super::{ErrorContextTrait, ErrorSeverity};
use thiserror::Error;

/// Top-level Tools error type
///
/// This encompasses all tools-related errors with automatic conversions
/// from sub-domain errors via `#[from]` attribute.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
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
        matches!(
            self,
            ToolsError::AITools(AIToolsError::RateLimitExceeded(_))
                | ToolsError::CLI(CLIError::MissingArgument(_))
                | ToolsError::CLI(CLIError::InvalidArgument(_, _))
        )
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
            CLIError::MissingArgument(_)
                | CLIError::InvalidArgument(_, _)
                | CLIError::UnknownCommand(_)
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

    // Additional comprehensive tests
    #[test]
    fn test_ai_tools_error_display() {
        assert_eq!(
            AIToolsError::Provider("OpenAI".to_string()).to_string(),
            "AI provider error: OpenAI"
        );
        assert_eq!(
            AIToolsError::ModelNotFound("gpt-5".to_string()).to_string(),
            "Model not found: gpt-5"
        );
        assert_eq!(
            AIToolsError::RateLimitExceeded("Claude".to_string()).to_string(),
            "Rate limit exceeded for Claude"
        );
    }

    #[test]
    fn test_ai_tools_error_severity() {
        // Provider not specifically matched, defaults to Medium
        assert_eq!(
            AIToolsError::Provider("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
        // ModelNotFound is High (see impl)
        assert_eq!(
            AIToolsError::ModelNotFound("test".to_string()).severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            AIToolsError::RateLimitExceeded("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            AIToolsError::Router("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
    }

    #[test]
    fn test_ai_tools_error_recoverability() {
        assert!(AIToolsError::RateLimitExceeded("test".to_string()).is_recoverable());
        // ModelNotFound is not recoverable (not in the match)
        assert!(!AIToolsError::ModelNotFound("test".to_string()).is_recoverable());
        assert!(!AIToolsError::Provider("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_cli_error_display() {
        assert_eq!(
            CLIError::MissingArgument("--port".to_string()).to_string(),
            "Missing required argument: --port"
        );
        assert_eq!(
            CLIError::UnknownCommand("invalid".to_string()).to_string(),
            "Unknown command: invalid"
        );
    }

    #[test]
    fn test_cli_error_severity() {
        assert_eq!(
            CLIError::MissingArgument("test".to_string()).severity(),
            ErrorSeverity::Low
        );
        assert_eq!(
            CLIError::UnknownCommand("test".to_string()).severity(),
            ErrorSeverity::Low
        );
        // Configuration is High (see impl)
        assert_eq!(
            CLIError::Configuration("test".to_string()).severity(),
            ErrorSeverity::High
        );
        // Plugin defaults to Low
        assert_eq!(
            CLIError::Plugin("test".to_string()).severity(),
            ErrorSeverity::Low
        );
    }

    #[test]
    fn test_rule_system_error_display() {
        assert_eq!(
            RuleSystemError::Execution("failed".to_string()).to_string(),
            "Rule execution error: failed"
        );
        assert_eq!(
            RuleSystemError::Conflict("duplicate".to_string()).to_string(),
            "Rule conflict: duplicate"
        );
    }

    #[test]
    fn test_rule_system_error_severity() {
        // Execution is High (see impl)
        assert_eq!(
            RuleSystemError::Execution("test".to_string()).severity(),
            ErrorSeverity::High
        );
        // Validation defaults to Medium
        assert_eq!(
            RuleSystemError::Validation("test".to_string()).severity(),
            ErrorSeverity::Medium
        );
        assert_eq!(
            RuleSystemError::Conflict("test".to_string()).severity(),
            ErrorSeverity::High
        );
    }

    #[test]
    fn test_tools_error_general() {
        let err = ToolsError::General("general error".to_string());
        assert_eq!(err.to_string(), "Tools error: general error");
        assert_eq!(err.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_cli_error_conversion() {
        let cli_err = CLIError::Configuration("invalid config".to_string()); // Fixed: was ConfigError
        let tools_err: ToolsError = cli_err.into();
        assert!(matches!(tools_err, ToolsError::CLI(_)));
    }

    #[test]
    fn test_rule_system_error_conversion() {
        let rule_err = RuleSystemError::Validation("invalid rule".to_string());
        let tools_err: ToolsError = rule_err.into();
        assert!(matches!(tools_err, ToolsError::RuleSystem(_)));
    }

    #[test]
    fn test_all_ai_tools_error_variants() {
        let variants = vec![
            AIToolsError::Provider("test".to_string()),
            AIToolsError::Router("test".to_string()),
            AIToolsError::Local("test".to_string()), // Fixed: was LocalAI
            AIToolsError::ModelNotFound("test".to_string()),
            AIToolsError::RateLimitExceeded("test".to_string()),
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity();
            let _ = variant.is_recoverable();
        }
    }

    #[test]
    fn test_all_cli_error_variants() {
        let variants = vec![
            CLIError::MissingArgument("test".to_string()),
            CLIError::UnknownCommand("test".to_string()),
            CLIError::Configuration("test".to_string()), // Fixed: was ConfigError
            CLIError::Plugin("test".to_string()),        // Fixed: was PluginError
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity();
            let _ = variant.is_recoverable();
        }
    }

    #[test]
    fn test_all_rule_system_error_variants() {
        let variants = vec![
            RuleSystemError::Execution("test".to_string()),
            RuleSystemError::Validation("test".to_string()),
            RuleSystemError::Conflict("test".to_string()),
        ];

        for variant in variants {
            assert!(variant.component().is_some());
            let _ = variant.severity();
        }
    }
}
