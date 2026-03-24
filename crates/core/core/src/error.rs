// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

/// Error types for the squirrel-core crate
/// Core error types
#[derive(Debug)]
pub enum CoreError {
    /// General error
    General(String),
    /// Service discovery error
    ServiceDiscovery(String),
    /// Configuration error
    Configuration(String),
    /// Network error
    Network(String),
    /// Serialization error
    Serialization(String),
    /// Timeout error
    Timeout(String),
    /// Not found error
    NotFound(String),
    /// Already exists error
    AlreadyExists(String),
    /// Invalid service configuration
    InvalidServiceConfig(String),
    /// Service not found
    ServiceNotFound(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(msg) => write!(f, "General error: {msg}"),
            Self::ServiceDiscovery(msg) => write!(f, "Service discovery error: {msg}"),
            Self::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout error: {msg}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::AlreadyExists(msg) => write!(f, "Already exists: {msg}"),
            Self::InvalidServiceConfig(msg) => write!(f, "Invalid service config: {msg}"),
            Self::ServiceNotFound(msg) => write!(f, "Service not found: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

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
