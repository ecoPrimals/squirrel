// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::path::PathBuf;

use super::*;

/// Socket-related env vars — used for temp_env isolation.
/// `temp_env::with_vars` acquires an internal process-wide mutex so all
/// env-modifying tests are safe to run concurrently without `#[serial]`.
const SOCKET_ENV_VARS: &[&str] = &[
    "SQUIRREL_SOCKET",
    "BIOMEOS_SOCKET_PATH",
    "PRIMAL_SOCKET",
    "SQUIRREL_FAMILY_ID",
    "BIOMEOS_FAMILY_ID",
    "FAMILY_ID",
    "SQUIRREL_NODE_ID",
    "XDG_RUNTIME_DIR",
];

#[test]
fn test_resolve_socket_path_for_ipc_absolute_unchanged() {
    assert_eq!(
        resolve_socket_path_for_ipc("/var/foo/squirrel.sock"),
        PathBuf::from("/var/foo/squirrel.sock")
    );
}

#[test]
fn test_resolve_socket_path_for_ipc_relative_under_tmp_biomeos() {
    temp_env::with_vars([("XDG_RUNTIME_DIR", None::<&str>)], || {
        assert_eq!(
            resolve_socket_path_for_ipc("squirrel.sock"),
            PathBuf::from("/tmp/biomeos/squirrel.sock")
        );
    });
}

#[test]
fn test_resolve_socket_path_for_ipc_relative_with_xdg_runtime() {
    temp_env::with_var("XDG_RUNTIME_DIR", Some("/run/user/4242"), || {
        assert_eq!(
            resolve_socket_path_for_ipc("squirrel.sock"),
            PathBuf::from("/run/user/4242/biomeos/squirrel.sock")
        );
    });
}

#[test]
fn test_socket_path_tier1_squirrel_socket() {
    temp_env::with_var("SQUIRREL_SOCKET", Some("/custom/path/socket.sock"), || {
        let path = get_socket_path("test-node");
        assert_eq!(path, "/custom/path/socket.sock");
    });
}

#[test]

fn test_socket_path_tier2_biomeos_socket_path() {
    temp_env::with_var(
        "BIOMEOS_SOCKET_PATH",
        Some("/tmp/squirrel-nat0.sock"),
        || {
            let path = get_socket_path("test-node");
            assert_eq!(path, "/tmp/squirrel-nat0.sock");
        },
    );
}

#[test]

fn test_squirrel_socket_overrides_biomeos_socket_path() {
    temp_env::with_vars(
        [
            ("SQUIRREL_SOCKET", Some("/custom/squirrel.sock")),
            ("BIOMEOS_SOCKET_PATH", Some("/tmp/squirrel-nat0.sock")),
        ],
        || {
            let path = get_socket_path("test-node");
            assert_eq!(path, "/custom/squirrel.sock");
        },
    );
}

#[test]

fn test_socket_path_tier3_primal_socket() {
    temp_env::with_vars(
        [
            ("PRIMAL_SOCKET", Some("/custom/primal")),
            ("SQUIRREL_FAMILY_ID", Some("nat0")),
        ],
        || {
            let path = get_socket_path("test-node");
            assert_eq!(path, "/custom/primal-nat0");
        },
    );
}

#[test]

fn test_socket_path_tier4_and_tier5_fallback() {
    temp_env::with_var("SQUIRREL_FAMILY_ID", Some("test0"), || {
        let path = get_socket_path("test-node");
        assert!(
            path.contains("/biomeos/squirrel-test0.sock") || path.contains("/tmp/squirrel-test0")
        );
    });
}

#[test]

fn test_get_family_id_default() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        let family_id = get_family_id();
        assert_eq!(family_id, "default");
    });
}

#[test]

fn test_get_family_id_from_env() {
    temp_env::with_var("SQUIRREL_FAMILY_ID", Some("nat0"), || {
        let family_id = get_family_id();
        assert_eq!(family_id, "nat0");
    });
}

#[test]

fn test_get_node_id_default() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        let node_id = get_node_id();
        assert!(!node_id.is_empty());
    });
}

#[test]

fn test_get_node_id_from_env() {
    temp_env::with_var("SQUIRREL_NODE_ID", Some("custom-node"), || {
        let node_id = get_node_id();
        assert_eq!(node_id, "custom-node");
    });
}

#[test]
fn test_prepare_socket_path_creates_directory() {
    use tempfile::tempdir;

    let dir = tempdir().expect("should succeed");
    let socket_path = dir.path().join("subdir/test.sock");
    let socket_str = socket_path.to_str().expect("should succeed");

    // Should create parent directory and return path
    let result = prepare_socket_path(socket_str);
    assert!(result.is_ok());
    assert!(socket_path.parent().expect("should succeed").exists());
}

#[test]

fn test_verify_socket_config() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        let result = verify_socket_config();
        assert!(result.is_ok());

        let message = result.expect("should succeed");
        assert!(message.contains("Socket:"));
        assert!(message.contains("Family:"));
        assert!(message.contains("Node:"));
    });
}

#[test]

fn test_xdg_socket_path_format() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        if let Some(xdg_path) = get_xdg_socket_path("default") {
            assert!(xdg_path.starts_with("/run/user/"));
            assert!(xdg_path.contains("/biomeos/"));
            assert!(xdg_path.ends_with("/squirrel.sock"));
        }
    });
}

#[test]
fn test_xdg_socket_path_family_scoped() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        if let Some(xdg_path) = get_xdg_socket_path("nat0") {
            assert!(xdg_path.starts_with("/run/user/"));
            assert!(xdg_path.contains("/biomeos/"));
            assert!(xdg_path.ends_with("/squirrel-nat0.sock"));
        }
    });
}

#[test]
fn test_ensure_biomeos_directory() {
    // Test directory creation (idempotent - works even if already exists)
    let result = ensure_biomeos_directory();
    assert!(result.is_ok());

    let path = result.expect("should succeed");
    assert!(path.exists());
    assert!(path.to_str().expect("should succeed").ends_with("biomeos"));

    // Verify directory is accessible (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path).expect("should succeed");
        let mode = metadata.permissions().mode();
        // Check that user has rwx permissions (at minimum)
        assert_eq!(mode & 0o700, 0o700, "User should have rwx permissions");
    }
}

#[test]

fn test_socket_path_uses_biomeos_directory() {
    temp_env::with_vars_unset(SOCKET_ENV_VARS, || {
        let path = get_socket_path("test-node");
        assert!(path.contains("/biomeos/squirrel.sock") || path.starts_with("/tmp/"));
    });
}

// ── Injectable SocketConfig tests (no temp_env, no serial) ──────────

#[test]
fn di_tier1_squirrel_socket() {
    let config = SocketConfig {
        squirrel_socket: Some("/custom/squirrel.sock".into()),
        ..Default::default()
    };
    assert_eq!(
        get_socket_path_with(&config, "node1"),
        "/custom/squirrel.sock"
    );
}

#[test]
fn di_tier2_biomeos_socket_path() {
    let config = SocketConfig {
        biomeos_socket_path: Some("/tmp/squirrel-nat0.sock".into()),
        ..Default::default()
    };
    assert_eq!(
        get_socket_path_with(&config, "node1"),
        "/tmp/squirrel-nat0.sock"
    );
}

#[test]
fn di_tier1_overrides_tier2() {
    let config = SocketConfig {
        squirrel_socket: Some("/custom/squirrel.sock".into()),
        biomeos_socket_path: Some("/tmp/should-not-use.sock".into()),
        ..Default::default()
    };
    assert_eq!(
        get_socket_path_with(&config, "node1"),
        "/custom/squirrel.sock"
    );
}

#[test]
fn di_tier3_primal_socket_with_family() {
    let config = SocketConfig {
        primal_socket: Some("/custom/primal".into()),
        family_id: Some("nat0".into()),
        ..Default::default()
    };
    assert_eq!(
        get_socket_path_with(&config, "node1"),
        "/custom/primal-nat0"
    );
}

#[test]
fn di_family_id_from_config() {
    let config = SocketConfig {
        family_id: Some("test-family-42".into()),
        ..Default::default()
    };
    assert_eq!(get_family_id_with(&config), "test-family-42");
}

#[test]
fn di_family_id_default() {
    let config = SocketConfig::default();
    assert_eq!(get_family_id_with(&config), "default");
}

#[test]
fn di_node_id_from_config() {
    let config = SocketConfig {
        node_id: Some("custom-node".into()),
        ..Default::default()
    };
    assert_eq!(get_node_id_with(&config), "custom-node");
}

#[test]
fn di_node_id_default_hostname() {
    let config = SocketConfig::default();
    let node_id = get_node_id_with(&config);
    assert!(!node_id.is_empty());
}

// ── BIOMEOS_INSECURE guard (BTSP §Security Model, GAP-MATRIX-12) ───

#[test]
fn insecure_guard_ok_neither_set() {
    assert!(validate_insecure_guard_with(false, false).is_ok());
}

#[test]
fn insecure_guard_ok_family_only() {
    assert!(validate_insecure_guard_with(true, false).is_ok());
}

#[test]
fn insecure_guard_ok_insecure_only() {
    assert!(validate_insecure_guard_with(false, true).is_ok());
}

#[test]
fn insecure_guard_rejects_both() {
    let result = validate_insecure_guard_with(true, true);
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(msg.contains("FAMILY_ID"));
    assert!(msg.contains("BIOMEOS_INSECURE"));
    assert!(msg.contains("BTSP"));
}

#[test]
fn insecure_guard_env_no_conflict() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn insecure_guard_env_family_only() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("my-family")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}

#[test]
fn insecure_guard_env_rejects_family_plus_insecure() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("prod-family")),
            ("BIOMEOS_INSECURE", Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_err());
        },
    );
}

#[test]
fn insecure_guard_env_rejects_primal_family_plus_insecure() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", Some("squirrel-family")),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", Some("true")),
        ],
        || {
            assert!(validate_insecure_guard().is_err());
        },
    );
}

#[test]
fn insecure_guard_env_default_family_is_not_production() {
    temp_env::with_vars(
        [
            ("SQUIRREL_FAMILY_ID", None::<&str>),
            ("FAMILY_ID", Some("default")),
            ("BIOMEOS_INSECURE", Some("1")),
        ],
        || {
            assert!(validate_insecure_guard().is_ok());
        },
    );
}
