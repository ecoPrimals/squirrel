// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for the context module

use thiserror::Error;

/// Error types for context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Plugins are disabled
    #[error("Plugins are disabled")]
    PluginsDisabled,

    /// Plugin not found
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    /// Transformation not found
    #[error("Transformation not found: {0}")]
    TransformationNotFound(String),

    /// Transformation failed
    #[error("Transformation failed for {0}: {1}")]
    TransformationFailed(String, String),

    /// Adapter not found
    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    /// Manager not initialized
    #[error("Context manager not initialized")]
    NotInitialized,

    /// Initialization failed
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Generic error
    #[error("Context error: {0}")]
    Other(String),

    /// I/O error (filesystem operations)
    #[error("I/O error: {0}")]
    Io(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid format error
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// Result type for context operations
pub type Result<T> = std::result::Result<T, ContextError>;

impl From<Box<dyn std::error::Error + Send + Sync>> for ContextError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ContextError::Other(err.to_string())
    }
}

impl From<anyhow::Error> for ContextError {
    fn from(err: anyhow::Error) -> Self {
        ContextError::Other(err.to_string())
    }
}

#[cfg(test)]
#[expect(
    clippy::unnecessary_literal_unwrap,
    reason = "Tests intentionally create Result types"
)]
mod tests {
    use super::*;

    #[test]
    fn test_plugins_disabled_error() {
        let err = ContextError::PluginsDisabled;
        assert_eq!(err.to_string(), "Plugins are disabled");
    }

    #[test]
    fn test_plugin_not_found_error() {
        let err = ContextError::PluginNotFound("test-plugin".to_string());
        assert_eq!(err.to_string(), "Plugin not found: test-plugin");
    }

    #[test]
    fn test_transformation_not_found_error() {
        let err = ContextError::TransformationNotFound("test-transform".to_string());
        assert_eq!(err.to_string(), "Transformation not found: test-transform");
    }

    #[test]
    fn test_transformation_failed_error() {
        let err =
            ContextError::TransformationFailed("test-transform".to_string(), "reason".to_string());
        assert_eq!(
            err.to_string(),
            "Transformation failed for test-transform: reason"
        );
    }

    #[test]
    fn test_adapter_not_found_error() {
        let err = ContextError::AdapterNotFound("test-adapter".to_string());
        assert_eq!(err.to_string(), "Adapter not found: test-adapter");
    }

    #[test]
    fn test_not_initialized_error() {
        let err = ContextError::NotInitialized;
        assert_eq!(err.to_string(), "Context manager not initialized");
    }

    #[test]
    fn test_initialization_failed_error() {
        let err = ContextError::InitializationFailed("config error".to_string());
        assert_eq!(err.to_string(), "Initialization failed: config error");
    }

    #[test]
    fn test_not_found_error() {
        let err = ContextError::NotFound("context-id".to_string());
        assert_eq!(err.to_string(), "Context not found: context-id");
    }

    #[test]
    fn test_invalid_state_error() {
        let err = ContextError::InvalidState("state description".to_string());
        assert_eq!(err.to_string(), "Invalid state: state description");
    }

    #[test]
    fn test_other_error() {
        let err = ContextError::Other("generic error".to_string());
        assert_eq!(err.to_string(), "Context error: generic error");
    }

    #[test]
    fn test_io_error() {
        let err = ContextError::Io("file not found".to_string());
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn test_serialization_error() {
        let err = ContextError::Serialization("invalid json".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid json");
    }

    #[test]
    fn test_invalid_format_error() {
        let err = ContextError::InvalidFormat("bad format".to_string());
        assert_eq!(err.to_string(), "Invalid format: bad format");
    }

    #[test]
    fn test_error_debug() {
        let err = ContextError::PluginsDisabled;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("PluginsDisabled"));
    }

    #[test]
    fn test_from_boxed_error() {
        let boxed_err: Box<dyn std::error::Error + Send + Sync> =
            Box::new(std::io::Error::other("test error"));
        let context_err: ContextError = boxed_err.into();
        assert!(matches!(context_err, ContextError::Other(_)));
        assert!(context_err.to_string().contains("test error"));
    }

    #[test]
    fn test_result_type_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.expect("ok"), 42);
    }

    #[test]
    fn test_result_type_err() {
        let result: Result<i32> = Err(ContextError::PluginsDisabled);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextError::PluginsDisabled));
    }
}
