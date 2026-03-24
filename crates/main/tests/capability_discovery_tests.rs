// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)] // Test code: explicit unwrap/expect and local lint noise
//! Tests for capability discovery - env var, error paths, `DiscoveryError`

use squirrel::capabilities::discovery::{CapabilityProvider, DiscoveryError, discover_capability};
use tempfile::tempdir;

#[test]
fn test_discover_capability_via_env_var() {
    let dir = tempdir().expect("should succeed");
    let socket_path = dir.path().join("test_cap.sock");
    std::fs::write(&socket_path, "").expect("should succeed");

    let env_var = "TEST_CAPABILITY_PROVIDER_SOCKET";
    let path_str = socket_path.to_str().expect("should succeed").to_string();
    temp_env::with_var(env_var, Some(path_str.as_str()), || {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        let result = rt.block_on(discover_capability("test.capability"));

        assert!(result.is_ok());
        let provider = result.expect("should succeed");
        assert_eq!(provider.id, "test.capability-provider");
        assert_eq!(provider.capabilities, vec!["test.capability"]);
        assert_eq!(provider.socket, socket_path);
        assert!(provider.discovered_via.starts_with("env:"));
    });
}

#[test]
fn test_discover_capability_env_var_nonexistent_path() {
    temp_env::with_var(
        "FAKE_CAP_PROVIDER_SOCKET",
        Some("/nonexistent/path/socket.sock"),
        || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            let result = rt.block_on(discover_capability("fake.cap"));

            assert!(result.is_err());
            if let Err(DiscoveryError::CapabilityNotFound(cap)) = result {
                assert_eq!(cap, "fake.cap");
            } else {
                assert!(result.is_err());
            }
        },
    );
}

#[test]
fn test_discover_capability_not_found() {
    temp_env::with_var_unset("OBSCURE_CAPABILITY_XYZ_PROVIDER_SOCKET", || {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        let result = rt.block_on(discover_capability("obscure.capability.xyz"));

        assert!(result.is_err());
        if let Err(DiscoveryError::CapabilityNotFound(cap)) = result {
            assert_eq!(cap, "obscure.capability.xyz");
        }
    });
}

#[test]
fn test_capability_provider_debug() {
    let provider = CapabilityProvider {
        id: "test-id".to_string(),
        capabilities: vec!["cap1".to_string()],
        socket: std::path::PathBuf::from("/tmp/test.sock"),
        metadata: std::collections::HashMap::new(),
        discovered_via: "env:TEST".to_string(),
    };
    let debug_str = format!("{provider:?}");
    assert!(debug_str.contains("test-id"));
    assert!(debug_str.contains("cap1"));
}

#[test]
fn test_discovery_error_display() {
    let err = DiscoveryError::CapabilityNotFound("test.cap".to_string());
    assert!(err.to_string().contains("test.cap"));

    let err = DiscoveryError::ProbeFailed("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));
}
