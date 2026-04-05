// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
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
