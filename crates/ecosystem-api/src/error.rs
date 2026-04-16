// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Error types for ecosystem integration
//!
//! This module contains standardized error types used throughout the
//! ecoPrimals ecosystem for consistent error handling and reporting.

use thiserror::Error;

/// Result type for universal operations
pub type UniversalResult<T> = Result<T, UniversalError>;

/// Universal error type for ecosystem operations
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UniversalError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Service mesh error
    #[error("Service mesh error: {0}")]
    ServiceMesh(String),

    /// Health check error
    #[error("Health check error: {0}")]
    HealthCheck(String),

    /// Capability error
    #[error("Capability error: {0}")]
    Capability(String),

    /// Context error
    #[error("Context error: {0}")]
    Context(String),

    /// Resource error
    #[error("Resource error: {0}")]
    Resource(String),
}

/// Ecosystem-specific error types
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EcosystemError {
    /// Service registration failed
    #[error("Service registration failed: {0}")]
    ServiceRegistration(String),

    /// Service discovery failed
    #[error("Service discovery failed: {0}")]
    ServiceDiscovery(String),

    /// Health reporting failed
    #[error("Health reporting failed: {0}")]
    HealthReportFailed(String),

    /// Capability update failed
    #[error("Capability update failed: {0}")]
    CapabilityUpdate(String),

    /// Unsupported operation
    #[error("Unsupported operation")]
    UnsupportedOperation,

    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Service mesh connection failed
    #[error("Service mesh connection failed: {0}")]
    ServiceMeshConnection(String),

    /// Universal error
    #[error("Universal error: {0}")]
    Universal(#[from] UniversalError),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Missing environment variable
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    /// Invalid configuration value
    #[error("Invalid configuration value for {key}: {value}")]
    InvalidValue {
        /// Configuration key that contains the invalid value
        key: String,
        /// Invalid value that failed validation
        value: String,
    },

    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    /// Configuration parsing error
    #[error("Configuration parsing error: {0}")]
    ParseError(String),

    /// Environment variable parsing error
    #[error("Environment variable parsing error: {0}")]
    EnvVarParsing(#[from] std::env::VarError),

    /// Number parsing error
    #[error("Number parsing error: {0}")]
    NumberParsing(#[from] std::num::ParseIntError),

    /// Float parsing error
    #[error("Float parsing error: {0}")]
    FloatParsing(#[from] std::num::ParseFloatError),

    /// Boolean parsing error
    #[error("Boolean parsing error: {0}")]
    BoolParsing(#[from] std::str::ParseBoolError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Service mesh error types
#[derive(Debug, Error)]
pub enum ServiceMeshError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Service discovery failed
    #[error("Service discovery failed: {0}")]
    DiscoveryFailed(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    /// Heartbeat failed
    #[error("Heartbeat failed: {0}")]
    HeartbeatFailed(String),

    /// Service not found
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// Invalid service response
    #[error("Invalid service response: {0}")]
    InvalidResponse(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Timeout
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParsing(#[from] url::ParseError),
}

/// Health check error types
#[derive(Debug, Error)]
pub enum HealthError {
    /// Service unhealthy
    #[error("Service unhealthy: {0}")]
    ServiceUnhealthy(String),

    /// Health check timeout
    #[error("Health check timeout: {0}")]
    Timeout(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    CheckFailed(String),

    /// Invalid health status
    #[error("Invalid health status: {0}")]
    InvalidStatus(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Dependency unhealthy
    #[error("Dependency unhealthy: {0}")]
    DependencyUnhealthy(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Capability error types
#[derive(Debug, Error)]
pub enum CapabilityError {
    /// Capability not supported
    #[error("Capability not supported: {0}")]
    NotSupported(String),

    /// Capability unavailable
    #[error("Capability unavailable: {0}")]
    Unavailable(String),

    /// Invalid capability
    #[error("Invalid capability: {0}")]
    Invalid(String),

    /// Capability conflict
    #[error("Capability conflict: {0}")]
    Conflict(String),

    /// Capability registration failed
    #[error("Capability registration failed: {0}")]
    RegistrationFailed(String),

    /// Capability update failed
    #[error("Capability update failed: {0}")]
    UpdateFailed(String),

    /// Dependency not met
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),

    /// Resource requirement not met
    #[error("Resource requirement not met: {0}")]
    ResourceNotMet(String),
}

/// Context error types
#[derive(Debug, Error)]
pub enum ContextError {
    /// Invalid context
    #[error("Invalid context: {0}")]
    Invalid(String),

    /// Context not found
    #[error("Context not found: {0}")]
    NotFound(String),

    /// Context expired
    #[error("Context expired: {0}")]
    Expired(String),

    /// Context permission denied
    #[error("Context permission denied: {0}")]
    PermissionDenied(String),

    /// Context update failed
    #[error("Context update failed: {0}")]
    UpdateFailed(String),

    /// Context serialization failed
    #[error("Context serialization failed: {0}")]
    SerializationFailed(String),

    /// Context validation failed
    #[error("Context validation failed: {0}")]
    ValidationFailed(String),

    /// Context conflict
    #[error("Context conflict: {0}")]
    Conflict(String),
}

/// Resource error types
#[derive(Debug, Error)]
pub enum ResourceError {
    /// Resource not available
    #[error("Resource not available: {0}")]
    NotAvailable(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    Exhausted(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    LimitExceeded(String),

    /// Resource allocation failed
    #[error("Resource allocation failed: {0}")]
    AllocationFailed(String),

    /// Resource deallocation failed
    #[error("Resource deallocation failed: {0}")]
    DeallocationFailed(String),

    /// Resource monitoring failed
    #[error("Resource monitoring failed: {0}")]
    MonitoringFailed(String),

    /// Invalid resource specification
    #[error("Invalid resource specification: {0}")]
    InvalidSpec(String),

    /// Resource conflict
    #[error("Resource conflict: {0}")]
    Conflict(String),
}

// Implement From conversions for common error types
impl From<ConfigError> for UniversalError {
    fn from(err: ConfigError) -> Self {
        Self::Configuration(err.to_string())
    }
}

impl From<ServiceMeshError> for UniversalError {
    fn from(err: ServiceMeshError) -> Self {
        Self::ServiceMesh(err.to_string())
    }
}

impl From<HealthError> for UniversalError {
    fn from(err: HealthError) -> Self {
        Self::HealthCheck(err.to_string())
    }
}

impl From<CapabilityError> for UniversalError {
    fn from(err: CapabilityError) -> Self {
        Self::Capability(err.to_string())
    }
}

impl From<ContextError> for UniversalError {
    fn from(err: ContextError) -> Self {
        Self::Context(err.to_string())
    }
}

impl From<ResourceError> for UniversalError {
    fn from(err: ResourceError) -> Self {
        Self::Resource(err.to_string())
    }
}

impl From<serde_json::Error> for UniversalError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for UniversalError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<url::ParseError> for UniversalError {
    fn from(err: url::ParseError) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<std::env::VarError> for UniversalError {
    fn from(err: std::env::VarError) -> Self {
        Self::Configuration(err.to_string())
    }
}

impl From<anyhow::Error> for UniversalError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod tests;
