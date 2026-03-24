// SPDX-License-Identifier: AGPL-3.0-only
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

// Feature-gated reqwest::Error conversions (only available with http-client feature)
#[cfg(feature = "http-client")]
impl From<reqwest::Error> for UniversalError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for EcosystemError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for ServiceMeshError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

#[cfg(feature = "http-client")]
impl From<reqwest::Error> for HealthError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
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
mod tests {
    use super::*;

    // ========== UniversalError Display Tests ==========

    #[test]
    fn test_universal_error_display() {
        let err = UniversalError::Configuration("bad config".to_string());
        assert_eq!(err.to_string(), "Configuration error: bad config");

        let err = UniversalError::Network("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");

        let err = UniversalError::Authentication("invalid token".to_string());
        assert_eq!(err.to_string(), "Authentication error: invalid token");

        let err = UniversalError::Authorization("forbidden".to_string());
        assert_eq!(err.to_string(), "Authorization error: forbidden");

        let err = UniversalError::ServiceUnavailable("down".to_string());
        assert_eq!(err.to_string(), "Service unavailable: down");

        let err = UniversalError::InvalidRequest("bad param".to_string());
        assert_eq!(err.to_string(), "Invalid request: bad param");

        let err = UniversalError::Internal("panic".to_string());
        assert_eq!(err.to_string(), "Internal error: panic");

        let err = UniversalError::Timeout("5s exceeded".to_string());
        assert_eq!(err.to_string(), "Timeout error: 5s exceeded");

        let err = UniversalError::Serialization("invalid json".to_string());
        assert_eq!(err.to_string(), "Serialization error: invalid json");

        let err = UniversalError::Io("file not found".to_string());
        assert_eq!(err.to_string(), "IO error: file not found");

        let err = UniversalError::ServiceMesh("mesh down".to_string());
        assert_eq!(err.to_string(), "Service mesh error: mesh down");

        let err = UniversalError::HealthCheck("unhealthy".to_string());
        assert_eq!(err.to_string(), "Health check error: unhealthy");

        let err = UniversalError::Capability("not supported".to_string());
        assert_eq!(err.to_string(), "Capability error: not supported");

        let err = UniversalError::Context("expired".to_string());
        assert_eq!(err.to_string(), "Context error: expired");

        let err = UniversalError::Resource("exhausted".to_string());
        assert_eq!(err.to_string(), "Resource error: exhausted");
    }

    // ========== EcosystemError Display Tests ==========

    #[test]
    fn test_ecosystem_error_display() {
        let err = EcosystemError::ServiceRegistration("timeout".to_string());
        assert_eq!(err.to_string(), "Service registration failed: timeout");

        let err = EcosystemError::ServiceDiscovery("no peers".to_string());
        assert_eq!(err.to_string(), "Service discovery failed: no peers");

        let err = EcosystemError::UnsupportedOperation;
        assert_eq!(err.to_string(), "Unsupported operation");

        let err = EcosystemError::ServiceNotFound("squirrel".to_string());
        assert_eq!(err.to_string(), "Service not found: squirrel");
    }

    // ========== ConfigError Display Tests ==========

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingEnvVar("MY_VAR".to_string());
        assert_eq!(err.to_string(), "Missing environment variable: MY_VAR");

        let err = ConfigError::InvalidValue {
            key: "PORT".to_string(),
            value: "abc".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid configuration value for PORT: abc");

        let err = ConfigError::ValidationFailed("empty name".to_string());
        assert_eq!(
            err.to_string(),
            "Configuration validation failed: empty name"
        );

        let err = ConfigError::FileNotFound("/etc/config.toml".to_string());
        assert_eq!(
            err.to_string(),
            "Configuration file not found: /etc/config.toml"
        );

        let err = ConfigError::ParseError("unexpected token".to_string());
        assert_eq!(
            err.to_string(),
            "Configuration parsing error: unexpected token"
        );
    }

    // ========== ServiceMeshError Display Tests ==========

    #[test]
    fn test_service_mesh_error_display() {
        let err = ServiceMeshError::ConnectionFailed("refused".to_string());
        assert_eq!(err.to_string(), "Connection failed: refused");

        let err = ServiceMeshError::RegistrationFailed("dup".to_string());
        assert_eq!(err.to_string(), "Registration failed: dup");

        let err = ServiceMeshError::ServiceNotFound("svc".to_string());
        assert_eq!(err.to_string(), "Service not found: svc");

        let err = ServiceMeshError::RateLimitExceeded("100/s".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: 100/s");
    }

    // ========== HealthError Display Tests ==========

    #[test]
    fn test_health_error_display() {
        let err = HealthError::ServiceUnhealthy("cpu 100%".to_string());
        assert_eq!(err.to_string(), "Service unhealthy: cpu 100%");

        let err = HealthError::ResourceExhausted("memory".to_string());
        assert_eq!(err.to_string(), "Resource exhausted: memory");

        let err = HealthError::DependencyUnhealthy("db".to_string());
        assert_eq!(err.to_string(), "Dependency unhealthy: db");
    }

    // ========== CapabilityError Display Tests ==========

    #[test]
    fn test_capability_error_display() {
        let err = CapabilityError::NotSupported("gpu".to_string());
        assert_eq!(err.to_string(), "Capability not supported: gpu");

        let err = CapabilityError::DependencyNotMet("auth".to_string());
        assert_eq!(err.to_string(), "Dependency not met: auth");
    }

    // ========== ContextError Display Tests ==========

    #[test]
    fn test_context_error_display() {
        let err = ContextError::Invalid("null".to_string());
        assert_eq!(err.to_string(), "Invalid context: null");

        let err = ContextError::Expired("session".to_string());
        assert_eq!(err.to_string(), "Context expired: session");

        let err = ContextError::PermissionDenied("admin".to_string());
        assert_eq!(err.to_string(), "Context permission denied: admin");
    }

    // ========== ResourceError Display Tests ==========

    #[test]
    fn test_resource_error_display() {
        let err = ResourceError::NotAvailable("gpu".to_string());
        assert_eq!(err.to_string(), "Resource not available: gpu");

        let err = ResourceError::LimitExceeded("100MB".to_string());
        assert_eq!(err.to_string(), "Resource limit exceeded: 100MB");
    }

    // ========== From Conversion Tests ==========

    #[test]
    fn test_config_error_to_universal() {
        let config_err = ConfigError::MissingEnvVar("TEST".to_string());
        let universal: UniversalError = config_err.into();
        assert!(matches!(universal, UniversalError::Configuration(_)));
        assert!(
            universal
                .to_string()
                .contains("Missing environment variable")
        );
    }

    #[test]
    fn test_service_mesh_error_to_universal() {
        let mesh_err = ServiceMeshError::ConnectionFailed("timeout".to_string());
        let universal: UniversalError = mesh_err.into();
        assert!(matches!(universal, UniversalError::ServiceMesh(_)));
    }

    #[test]
    fn test_health_error_to_universal() {
        let health_err = HealthError::ServiceUnhealthy("down".to_string());
        let universal: UniversalError = health_err.into();
        assert!(matches!(universal, UniversalError::HealthCheck(_)));
    }

    #[test]
    fn test_capability_error_to_universal() {
        let cap_err = CapabilityError::NotSupported("wasm".to_string());
        let universal: UniversalError = cap_err.into();
        assert!(matches!(universal, UniversalError::Capability(_)));
    }

    #[test]
    fn test_context_error_to_universal() {
        let ctx_err = ContextError::Expired("session".to_string());
        let universal: UniversalError = ctx_err.into();
        assert!(matches!(universal, UniversalError::Context(_)));
    }

    #[test]
    fn test_resource_error_to_universal() {
        let res_err = ResourceError::Exhausted("memory".to_string());
        let universal: UniversalError = res_err.into();
        assert!(matches!(universal, UniversalError::Resource(_)));
    }

    #[test]
    fn test_io_error_to_universal() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let universal: UniversalError = io_err.into();
        assert!(matches!(universal, UniversalError::Io(_)));
    }

    #[test]
    fn test_serde_error_to_universal() {
        let json_str = "not valid json";
        let serde_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let universal: UniversalError = serde_err.into();
        assert!(matches!(universal, UniversalError::Serialization(_)));
    }

    #[test]
    fn test_url_error_to_universal() {
        let url_err = url::Url::parse("not a url").unwrap_err();
        let universal: UniversalError = url_err.into();
        assert!(matches!(universal, UniversalError::Network(_)));
    }

    #[test]
    fn test_universal_error_to_ecosystem() {
        let universal = UniversalError::Internal("test".to_string());
        let ecosystem: EcosystemError = universal.into();
        assert!(matches!(ecosystem, EcosystemError::Universal(_)));
    }

    #[test]
    fn test_universal_result_type() {
        let ok: UniversalResult<i32> = Ok(42);
        assert_eq!(ok.as_ref().expect("should succeed"), &42);

        let err: UniversalResult<i32> = Err(UniversalError::Internal("fail".to_string()));
        assert!(err.is_err());
    }

    #[test]
    fn test_anyhow_into_universal() {
        let anyhow_err = anyhow::anyhow!("wrapped failure");
        let u: UniversalError = anyhow_err.into();
        assert!(matches!(u, UniversalError::Internal(_)));
        assert!(u.to_string().contains("wrapped failure"));
    }

    #[test]
    fn test_var_error_into_universal() {
        let var_err =
            std::env::var("DEFINITELY_MISSING_VAR_FOR_ECOSYSTEM_API_TEST_XYZ").unwrap_err();
        let u: UniversalError = var_err.into();
        assert!(matches!(u, UniversalError::Configuration(_)));
    }

    #[test]
    fn test_ecosystem_error_from_serde_json() {
        let e = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let eco: EcosystemError = e.into();
        assert!(matches!(eco, EcosystemError::Serialization(_)));
    }

    #[test]
    fn test_ecosystem_error_from_url_parse() {
        let e = url::Url::parse("not a valid url").unwrap_err();
        let eco: EcosystemError = e.into();
        assert!(matches!(eco, EcosystemError::UrlParsing(_)));
    }

    #[test]
    fn test_ecosystem_error_from_io() {
        let e = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let eco: EcosystemError = e.into();
        assert!(matches!(eco, EcosystemError::Io(_)));
    }

    #[test]
    fn test_service_mesh_error_remaining_variants_display() {
        let cases = [
            ServiceMeshError::DiscoveryFailed("d".into()),
            ServiceMeshError::HealthCheckFailed("h".into()),
            ServiceMeshError::HeartbeatFailed("hb".into()),
            ServiceMeshError::InvalidResponse("bad".into()),
            ServiceMeshError::AuthenticationFailed("auth".into()),
            ServiceMeshError::Timeout("t".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_health_error_remaining_variants_display() {
        let cases = [
            HealthError::Timeout("t".into()),
            HealthError::CheckFailed("c".into()),
            HealthError::InvalidStatus("s".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_capability_error_remaining_variants_display() {
        let cases = [
            CapabilityError::Unavailable("u".into()),
            CapabilityError::Invalid("i".into()),
            CapabilityError::Conflict("c".into()),
            CapabilityError::RegistrationFailed("r".into()),
            CapabilityError::UpdateFailed("u".into()),
            CapabilityError::ResourceNotMet("res".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_context_error_remaining_variants_display() {
        let cases = [
            ContextError::NotFound("n".into()),
            ContextError::UpdateFailed("u".into()),
            ContextError::SerializationFailed("s".into()),
            ContextError::ValidationFailed("v".into()),
            ContextError::Conflict("c".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_resource_error_remaining_variants_display() {
        let cases = [
            ResourceError::Exhausted("e".into()),
            ResourceError::AllocationFailed("a".into()),
            ResourceError::DeallocationFailed("d".into()),
            ResourceError::MonitoringFailed("m".into()),
            ResourceError::InvalidSpec("i".into()),
            ResourceError::Conflict("c".into()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn test_config_error_from_parsing_errors() {
        let n = "x".parse::<i32>().unwrap_err();
        let ce: ConfigError = n.into();
        assert!(matches!(ce, ConfigError::NumberParsing(_)));

        let f = "x".parse::<f64>().unwrap_err();
        let ce: ConfigError = f.into();
        assert!(matches!(ce, ConfigError::FloatParsing(_)));

        let b = "maybe".parse::<bool>().unwrap_err();
        let ce: ConfigError = b.into();
        assert!(matches!(ce, ConfigError::BoolParsing(_)));
    }
}
