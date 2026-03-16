// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

// error_handling module
//
// safe_operations removed - HTTP test utilities not needed in Pure Rust build

/// Module for improved error handling patterns
///
/// This module provides utilities and patterns for safe error handling.
///
/// **v1.4.3**: HTTP-based safe_operations utilities have been removed.
/// Production code uses standard Rust error handling with Result<T, E>.
pub mod core {}

/// Version information for error handling features.
pub const ERROR_HANDLING_VERSION: &str = "1.4.3";

/// Check if enhanced error handling is enabled
#[must_use]
pub const fn enhanced_error_handling_enabled() -> bool {
    true // Standard Rust error handling always enabled
}

/// Get error handling system information
#[must_use]
pub fn get_error_handling_info() -> ErrorHandlingInfo {
    ErrorHandlingInfo {
        version: ERROR_HANDLING_VERSION.to_string(),
        enabled: enhanced_error_handling_enabled(),
        features: vec![
            "result_types".to_string(),
            "error_context".to_string(),
            "anyhow_integration".to_string(),
            "thiserror_integration".to_string(),
        ],
    }
}

/// Error handling system information
#[derive(Debug, Clone)]
pub struct ErrorHandlingInfo {
    /// Version of the error handling system.
    pub version: String,
    /// Whether enhanced error handling is enabled.
    pub enabled: bool,
    /// List of enabled error handling features.
    pub features: Vec<String>,
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
