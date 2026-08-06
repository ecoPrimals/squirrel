// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use thiserror::Error;

/// Error types for the squirrel-core crate
/// Core error types
#[derive(Debug, Error)]
pub enum CoreError {
    /// General error
    #[error("General error: {0}")]
    General(String),
    /// Service discovery error
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(String),
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    /// Invalid service configuration
    #[error("Invalid service config: {0}")]
    InvalidServiceConfig(String),
    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
}

/// Core result type
pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_error_display_covers_all_variants() {
        let cases = vec![
            (CoreError::General("g".into()), "General error: g"),
            (
                CoreError::ServiceDiscovery("d".into()),
                "Service discovery error: d",
            ),
            (
                CoreError::Configuration("c".into()),
                "Configuration error: c",
            ),
            (CoreError::Network("n".into()), "Network error: n"),
            (
                CoreError::Serialization("s".into()),
                "Serialization error: s",
            ),
            (CoreError::Timeout("t".into()), "Timeout error: t"),
            (CoreError::NotFound("nf".into()), "Not found: nf"),
            (CoreError::AlreadyExists("ae".into()), "Already exists: ae"),
            (
                CoreError::InvalidServiceConfig("isc".into()),
                "Invalid service config: isc",
            ),
            (
                CoreError::ServiceNotFound("snf".into()),
                "Service not found: snf",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn core_error_implements_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(CoreError::General("e".into()));
        assert_eq!(err.to_string(), "General error: e");
    }
}
