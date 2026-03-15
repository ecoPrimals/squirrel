// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for error handling and conversions

use super::*;
use std::io;

#[test]
fn test_primal_error_display() {
    let err = PrimalError::Network("connection failed".to_string());
    assert_eq!(err.to_string(), "Network error: connection failed");

    let err = PrimalError::Configuration("invalid config".to_string());
    assert_eq!(err.to_string(), "Configuration error: invalid config");

    let err = PrimalError::ParsingError("bad json".to_string());
    assert_eq!(err.to_string(), "Parsing error: bad json");
}

#[test]
fn test_primal_error_debug() {
    let err = PrimalError::InvalidOperation("test".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("InvalidOperation"));
}

#[test]
fn test_io_error_conversion() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let primal_err: PrimalError = io_err.into();

    match primal_err {
        PrimalError::Io(_) => { /* Expected */ }
        _ => panic!("Expected Io variant"),
    }
}

#[test]
fn test_serde_json_error_conversion() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let primal_err: PrimalError = json_err.into();

    match primal_err {
        PrimalError::Serialization(_) => { /* Expected */ }
        _ => panic!("Expected Serialization variant"),
    }
}

#[test]
fn test_url_parse_error_conversion() {
    let url_err = url::Url::parse("not a url").unwrap_err();
    let primal_err: PrimalError = url_err.into();

    match primal_err {
        PrimalError::UrlParse(_) => { /* Expected */ }
        _ => panic!("Expected UrlParse variant"),
    }
}

#[test]
fn test_boxed_error_conversion() {
    let boxed_err: Box<dyn std::error::Error + Send + Sync> =
        Box::new(io::Error::new(io::ErrorKind::Other, "test error"));
    let primal_err: PrimalError = boxed_err.into();

    match primal_err {
        PrimalError::Generic(msg) => {
            assert!(msg.contains("Boxed error"));
        }
        _ => panic!("Expected Generic variant"),
    }
}

#[test]
fn test_discovery_error_conversion() {
    use crate::capabilities::discovery::DiscoveryError;

    let disc_err = DiscoveryError::CapabilityNotFound("test capability".to_string());
    let primal_err: PrimalError = disc_err.into();

    match primal_err {
        PrimalError::NetworkError(msg) => {
            assert!(msg.contains("Discovery error"));
        }
        _ => panic!("Expected NetworkError variant"),
    }
}

#[test]
fn test_all_error_variants() {
    let variants = vec![
        PrimalError::Network("test".to_string()),
        PrimalError::NetworkError("test".to_string()),
        PrimalError::Authentication("test".to_string()),
        PrimalError::Configuration("test".to_string()),
        PrimalError::ConfigurationError("test".to_string()),
        PrimalError::ConfigError("test".to_string()),
        PrimalError::ParsingError("test".to_string()),
        PrimalError::InvalidOperation("test".to_string()),
        PrimalError::ServiceDiscoveryFailed("test".to_string()),
        PrimalError::ServiceDiscoveryError("test".to_string()),
        PrimalError::Registry("test".to_string()),
        PrimalError::Internal("test".to_string()),
        PrimalError::OperationFailed("test".to_string()),
        PrimalError::OperationNotSupported("test".to_string()),
        PrimalError::ResourceNotFound("test".to_string()),
        PrimalError::NotFoundError("test".to_string()),
        PrimalError::ResourceError("test".to_string()),
        PrimalError::General("test".to_string()),
        PrimalError::ValidationError("test".to_string()),
        PrimalError::SerializationError("test".to_string()),
        PrimalError::SecurityError("test".to_string()),
        PrimalError::ComputeError("test".to_string()),
        PrimalError::StorageError("test".to_string()),
        PrimalError::Generic("test".to_string()),
        PrimalError::InvalidInput("test".to_string()),
        PrimalError::NotImplemented("test".to_string()),
        PrimalError::NotSupported("test".to_string()),
        PrimalError::InvalidEndpoint("test".to_string()),
        PrimalError::InvalidResponse("test".to_string()),
        PrimalError::RemoteError("test".to_string()),
    ];

    // Ensure all variants can be created and displayed
    for err in variants {
        let _msg = err.to_string();
        let _debug = format!("{:?}", err);
    }
}

#[test]
fn test_error_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<PrimalError>();
    assert_sync::<PrimalError>();
}

#[test]
fn test_error_source_chain() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "original error");
    let primal_err: PrimalError = io_err.into();

    // PrimalError should implement std::error::Error
    let error_trait: &dyn std::error::Error = &primal_err;
    assert!(error_trait.source().is_some());
}

#[test]
fn test_duplicate_variant_messages() {
    // Test that duplicate variant names have correct messages
    let network1 = PrimalError::Network("msg1".to_string());
    let network2 = PrimalError::NetworkError("msg2".to_string());
    assert_ne!(network1.to_string(), network2.to_string());

    let config1 = PrimalError::Configuration("c1".to_string());
    let config2 = PrimalError::ConfigurationError("c2".to_string());
    let config3 = PrimalError::ConfigError("c3".to_string());
    assert_ne!(config1.to_string(), config2.to_string());
    assert_ne!(config1.to_string(), config3.to_string());
}
