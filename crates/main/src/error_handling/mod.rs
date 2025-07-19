pub mod safe_operations;

// Re-export commonly used error handling utilities
pub use safe_operations::*;

/// Module for improved error handling patterns
///
/// This module provides utilities and patterns for safe error handling:
/// - Safe alternatives to unwrap() and expect()
/// - Comprehensive error types with context
/// - Recovery strategies for failed operations
/// - Timeout handling for async operations
/// - Safe lock acquisition patterns
/// - Production-ready error handling macros
///
/// These utilities are designed to prevent panics in production code
/// and provide graceful error handling with proper logging and recovery.
pub mod error_handling {
    pub use super::safe_operations::*;
}

// Version information for error handling features
pub const ERROR_HANDLING_VERSION: &str = "1.0.0";

/// Check if enhanced error handling is enabled
pub fn enhanced_error_handling_enabled() -> bool {
    cfg!(feature = "enhanced_error_handling") || true // Default to enabled
}

/// Get error handling system information
pub fn get_error_handling_info() -> ErrorHandlingInfo {
    ErrorHandlingInfo {
        version: ERROR_HANDLING_VERSION.to_string(),
        enabled: enhanced_error_handling_enabled(),
        features: vec![
            "safe_operations".to_string(),
            "safe_locks".to_string(),
            "safe_channels".to_string(),
            "safe_serialization".to_string(),
            "safe_networking".to_string(),
            "safe_configuration".to_string(),
            "recovery_strategies".to_string(),
            "timeout_handling".to_string(),
        ],
    }
}

/// Error handling system information
#[derive(Debug, Clone)]
pub struct ErrorHandlingInfo {
    pub version: String,
    pub enabled: bool,
    pub features: Vec<String>,
}

/// Convenience re-exports for commonly used safe operations
pub mod prelude {
    pub use super::safe_operations::{
        RecoveryStrategy, SafeConfig, SafeError, SafeOps, SafeResult, SafeService, SafeSession,
    };

    pub use crate::{safe_expect, safe_option, safe_option_with_default, safe_unwrap};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling_info() {
        let info = get_error_handling_info();
        assert_eq!(info.version, ERROR_HANDLING_VERSION);
        assert!(info.enabled);
        assert!(!info.features.is_empty());
    }

    #[test]
    fn test_enhanced_error_handling_enabled() {
        assert!(enhanced_error_handling_enabled());
    }
}
