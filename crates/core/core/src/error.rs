// SPDX-License-Identifier: AGPL-3.0-only
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
