//! Tests for error types

use super::*;

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let primal_err: PrimalError = io_err.into();

    assert!(matches!(primal_err, PrimalError::Io(_)));
}

#[test]
fn test_error_from_serde() {
    let json_err = serde_json::from_str::<serde_json::Value>("{invalid").unwrap_err();
    let primal_err: PrimalError = json_err.into();

    assert!(matches!(primal_err, PrimalError::Serialization(_)));
}

#[test]
fn test_error_from_url_parse() {
    let url_err = url::Url::parse("not a url").unwrap_err();
    let primal_err: PrimalError = url_err.into();

    assert!(matches!(primal_err, PrimalError::UrlParse(_)));
}

#[test]
fn test_network_error_variants() {
    let err1 = PrimalError::Network("connection refused".to_string());
    let err2 = PrimalError::NetworkError("timeout".to_string());

    assert!(err1.to_string().contains("connection refused"));
    assert!(err2.to_string().contains("timeout"));
}

#[test]
fn test_configuration_error_variants() {
    let err1 = PrimalError::Configuration("invalid config".to_string());
    let err2 = PrimalError::ConfigurationError("bad yaml".to_string());
    let err3 = PrimalError::ConfigError("missing field".to_string());

    assert!(err1.to_string().contains("invalid config"));
    assert!(err2.to_string().contains("bad yaml"));
    assert!(err3.to_string().contains("missing field"));
}

#[test]
fn test_authentication_error() {
    let err = PrimalError::Authentication("invalid token".to_string());

    assert!(err.to_string().contains("Authentication failed"));
    assert!(err.to_string().contains("invalid token"));
}

#[test]
fn test_service_discovery_errors() {
    let err1 = PrimalError::ServiceDiscoveryFailed("no services".to_string());
    let err2 = PrimalError::ServiceDiscoveryError("lookup failed".to_string());

    assert!(err1.to_string().contains("Service discovery failed"));
    assert!(err2.to_string().contains("Service discovery error"));
}

#[test]
fn test_operation_errors() {
    let err1 = PrimalError::InvalidOperation("bad op".to_string());
    let err2 = PrimalError::OperationFailed("timeout".to_string());
    let err3 = PrimalError::OperationNotSupported("unsupported".to_string());

    assert!(err1.to_string().contains("Invalid operation"));
    assert!(err2.to_string().contains("Operation failed"));
    assert!(err3.to_string().contains("Operation not supported"));
}

#[test]
fn test_resource_errors() {
    let err1 = PrimalError::ResourceNotFound("user/123".to_string());
    let err2 = PrimalError::NotFoundError("page not found".to_string());
    let err3 = PrimalError::ResourceError("locked".to_string());

    assert!(err1.to_string().contains("Resource not found"));
    assert!(err2.to_string().contains("Not found"));
    assert!(err3.to_string().contains("Resource error"));
}

#[test]
fn test_general_errors() {
    let err1 = PrimalError::Internal("internal error".to_string());
    let err2 = PrimalError::General("something went wrong".to_string());
    let err3 = PrimalError::Generic("generic error".to_string());

    assert!(err1.to_string().contains("Internal error"));
    assert!(err2.to_string().contains("General error"));
    assert!(err3.to_string().contains("Generic error"));
}

#[test]
fn test_specialized_errors() {
    let err1 = PrimalError::ParsingError("syntax error".to_string());
    let err2 = PrimalError::ValidationError("invalid input".to_string());
    let err3 = PrimalError::SerializationError("failed to serialize".to_string());
    let err4 = PrimalError::SecurityError("access denied".to_string());
    let err5 = PrimalError::ComputeError("computation failed".to_string());
    let err6 = PrimalError::StorageError("disk full".to_string());
    let err7 = PrimalError::Registry("registry unavailable".to_string());

    assert!(err1.to_string().contains("Parsing error"));
    assert!(err2.to_string().contains("Validation error"));
    assert!(err3.to_string().contains("Serialization error"));
    assert!(err4.to_string().contains("Security error"));
    assert!(err5.to_string().contains("Compute error"));
    assert!(err6.to_string().contains("Storage error"));
    assert!(err7.to_string().contains("Registry error"));
}

#[test]
fn test_boxed_error_conversion() {
    let boxed: Box<dyn std::error::Error + Send + Sync> =
        Box::new(std::io::Error::other("test error"));

    let primal_err: PrimalError = boxed.into();

    assert!(matches!(primal_err, PrimalError::Generic(_)));
    assert!(primal_err.to_string().contains("Boxed error"));
}

#[test]
fn test_error_display() {
    let errors = vec![
        PrimalError::Network("test".to_string()),
        PrimalError::Authentication("test".to_string()),
        PrimalError::Configuration("test".to_string()),
        PrimalError::InvalidOperation("test".to_string()),
    ];

    for err in errors {
        let display = format!("{}", err);
        assert!(!display.is_empty());
    }
}

#[test]
fn test_error_debug() {
    let err = PrimalError::Internal("debug test".to_string());
    let debug_str = format!("{:?}", err);

    assert!(debug_str.contains("Internal"));
    assert!(debug_str.contains("debug test"));
}
