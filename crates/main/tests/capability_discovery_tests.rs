// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for capability discovery - env var, error paths, DiscoveryError

use squirrel::capabilities::discovery::{discover_capability, CapabilityProvider, DiscoveryError};
use std::sync::Mutex;
use tempfile::tempdir;

// Serialize capability discovery tests - they modify env vars
static CAPABILITY_TEST_LOCK: Mutex<()> = Mutex::new(());

#[tokio::test]
async fn test_discover_capability_via_env_var() {
    let _guard = CAPABILITY_TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_cap.sock");
    std::fs::write(&socket_path, "").unwrap(); // Create file (socket path)

    let env_var = "TEST_CAPABILITY_PROVIDER_SOCKET";
    std::env::set_var(env_var, socket_path.to_str().unwrap());

    let result = discover_capability("test.capability").await;

    std::env::remove_var(env_var);

    assert!(result.is_ok());
    let provider = result.unwrap();
    assert_eq!(provider.id, "test.capability-provider");
    assert_eq!(provider.capabilities, vec!["test.capability"]);
    assert_eq!(provider.socket, socket_path);
    assert!(provider.discovered_via.starts_with("env:"));
}

#[tokio::test]
async fn test_discover_capability_env_var_nonexistent_path() {
    let _guard = CAPABILITY_TEST_LOCK.lock().unwrap();
    std::env::set_var("FAKE_CAP_PROVIDER_SOCKET", "/nonexistent/path/socket.sock");

    let result = discover_capability("fake.cap").await;

    std::env::remove_var("FAKE_CAP_PROVIDER_SOCKET");

    // Should not find via env (path doesn't exist), falls through to registry/scan
    // which also fail, so we get CapabilityNotFound
    assert!(result.is_err());
    if let Err(DiscoveryError::CapabilityNotFound(cap)) = result {
        assert_eq!(cap, "fake.cap");
    } else {
        // May also fail with other errors from scan
        assert!(result.is_err());
    }
}

#[tokio::test]
async fn test_discover_capability_not_found() {
    let _guard = CAPABILITY_TEST_LOCK.lock().unwrap();
    // Ensure no env var is set for this capability
    std::env::remove_var("OBSCURE_CAPABILITY_XYZ_PROVIDER_SOCKET");

    let result = discover_capability("obscure.capability.xyz").await;

    assert!(result.is_err());
    match result {
        Err(DiscoveryError::CapabilityNotFound(cap)) => assert_eq!(cap, "obscure.capability.xyz"),
        _ => {}
    }
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
    let debug_str = format!("{:?}", provider);
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
