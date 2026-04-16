// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_secure_plugin_loader_new() {
    temp_env::with_var_unset("SQUIRREL_PLUGIN_DIRS", || {
        let loader = SecurePluginLoader::new();
        assert!(loader.verify_signatures);
        assert_eq!(loader.max_plugin_size, 50 * 1024 * 1024);
        assert_eq!(loader.allowed_directories.len(), 2);
    });
}

#[test]
fn test_secure_plugin_loader_default() {
    let loader = SecurePluginLoader::default();
    assert!(loader.verify_signatures);
}

#[test]
fn test_plugin_security_error_display() {
    let cases = vec![
        (
            PluginSecurityError::ValidationFailed("bad".to_string()),
            "Plugin validation failed: bad",
        ),
        (
            PluginSecurityError::SignatureVerificationFailed("sig".to_string()),
            "Plugin signature verification failed: sig",
        ),
        (
            PluginSecurityError::SandboxingFailed("sandbox".to_string()),
            "Plugin sandboxing failed: sandbox",
        ),
        (
            PluginSecurityError::LoadingDenied("denied".to_string()),
            "Plugin loading denied: denied",
        ),
        (
            PluginSecurityError::ExecutionTimeout,
            "Plugin execution timeout",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn test_validation_result_fields() {
    let result = ValidationResult {
        is_valid: true,
        checksum: "abc123".to_string(),
        signature_valid: true,
        warnings: vec![],
    };
    assert!(result.is_valid);
    assert!(result.signature_valid);
    assert!(result.warnings.is_empty());
    assert_eq!(result.checksum, "abc123");
}

#[test]
fn test_validation_result_with_warnings() {
    let result = ValidationResult {
        is_valid: false,
        checksum: "def456".to_string(),
        signature_valid: false,
        warnings: vec!["world-writable".to_string()],
    };
    assert!(!result.is_valid);
    assert!(!result.signature_valid);
    assert_eq!(result.warnings.len(), 1);
}

#[test]
fn test_is_path_allowed_nonexistent() {
    let loader = SecurePluginLoader::new();
    // Non-existent path should fail to canonicalize
    let result = loader.is_path_allowed(Path::new("/nonexistent/path/plugin.so"));
    assert!(result.is_err());
}

#[test]
fn test_secure_plugin_stub_name() {
    let metadata = PluginMetadata {
        name: "test-stub".to_string(),
        version: "1.0.0".to_string(),
        description: Some("A stub".to_string()),
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    assert_eq!(stub.name(), "test-stub");
    assert_eq!(stub.version(), "1.0.0");
    assert_eq!(stub.description(), Some("A stub"));
}

#[test]
fn test_secure_plugin_stub_no_description() {
    let metadata = PluginMetadata {
        name: "no-desc".to_string(),
        version: "0.1.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    assert!(stub.description().is_none());
}

#[tokio::test]
async fn test_secure_plugin_stub_initialize() {
    let metadata = PluginMetadata {
        name: "init-test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    let result = stub.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_secure_plugin_stub_execute_returns_security_error() {
    let metadata = PluginMetadata {
        name: "exec-test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    let result = stub.execute(&["arg1".to_string()]).await;
    assert!(
        result.is_err(),
        "Sandbox plugins must reject direct execution"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("security sandbox"),
        "Error should mention sandbox: {err}"
    );
}

#[tokio::test]
async fn test_secure_plugin_stub_cleanup() {
    let metadata = PluginMetadata {
        name: "cleanup-test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    let result = stub.cleanup().await;
    assert!(result.is_ok());
}

#[test]
fn test_secure_plugin_stub_commands_empty() {
    let metadata = PluginMetadata {
        name: "cmd-test".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    assert!(stub.commands().is_empty());
}

#[test]
fn validate_plugin_rejects_path_outside_allowed_dirs() {
    let dir = tempdir().expect("tempdir");
    let outside =
        std::env::temp_dir().join(format!("squirrel-plugin-outside-{}.so", std::process::id()));
    std::fs::write(&outside, b"x").expect("write outside plugin");
    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "x".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let err = loader
                .validate_plugin(&outside, &meta)
                .expect_err("outside allowed dirs");
            assert!(matches!(err, PluginSecurityError::LoadingDenied(_)));
        },
    );
    let _ = std::fs::remove_file(&outside);
}

#[test]
fn validate_plugin_checksum_and_signature_happy_path() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("p.so");
    std::fs::write(&plugin_path, b"plugin-bytes").expect("write plugin");
    let hash = blake3::hash(b"plugin-bytes").to_hex().to_string();
    let mut sig = std::fs::File::create(plugin_path.with_extension("sig")).expect("sig");
    writeln!(sig, "{}", hash.to_ascii_uppercase()).expect("write sig");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "plug".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let v = loader.validate_plugin(&plugin_path, &meta).expect("valid");
            assert!(v.signature_valid);
            assert!(v.is_valid);
            assert_eq!(v.checksum, hash);
        },
    );
}

#[test]
fn validate_plugin_missing_signature_marks_invalid() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("n.so");
    std::fs::write(&plugin_path, b"x").expect("write");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "n".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let v = loader
                .validate_plugin(&plugin_path, &meta)
                .expect("validate");
            assert!(!v.signature_valid);
            assert!(!v.is_valid);
        },
    );
}

#[test]
fn validate_plugin_signature_mismatch() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("m.so");
    std::fs::write(&plugin_path, b"data").expect("write");
    std::fs::write(
        plugin_path.with_extension("sig"),
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
    )
    .expect("sig");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "m".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let v = loader
                .validate_plugin(&plugin_path, &meta)
                .expect("validate");
            assert!(!v.signature_valid);
        },
    );
}

#[test]
fn validate_plugin_short_signature_file_rejected() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("short.so");
    std::fs::write(&plugin_path, b"z").expect("write");
    let real = blake3::hash(b"z").to_hex().to_string();
    std::fs::write(
        plugin_path.with_extension("sig"),
        &real[..real.len().saturating_sub(4)],
    )
    .expect("sig");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "s".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let v = loader
                .validate_plugin(&plugin_path, &meta)
                .expect("validate");
            assert!(!v.signature_valid);
        },
    );
}

#[test]
fn load_plugin_secure_fails_when_validation_not_valid() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("bad.so");
    std::fs::write(&plugin_path, b"q").expect("write");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "bad".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let Err(err) = loader.load_plugin_secure(&plugin_path, &meta) else {
                panic!("expected validation failure without signature");
            };
            assert!(matches!(err, PluginSecurityError::ValidationFailed(_)));
        },
    );
}

#[test]
fn load_plugin_secure_returns_stub_when_valid() {
    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("ok.so");
    std::fs::write(&plugin_path, b"ok-bytes").expect("write");
    let h = blake3::hash(b"ok-bytes").to_hex().to_string();
    std::fs::write(plugin_path.with_extension("sig"), format!("{h}\n")).expect("sig");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "okp".to_string(),
                version: "2".to_string(),
                description: Some("d".to_string()),
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let plug = loader
                .load_plugin_secure(&plugin_path, &meta)
                .expect("stub");
            assert_eq!(plug.name(), "okp");
        },
    );
}

#[tokio::test]
async fn secure_plugin_stub_register_commands_ok() {
    let metadata = PluginMetadata {
        name: "reg".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        author: None,
        homepage: None,
        capabilities: vec![],
    };
    let stub = SecurePluginStub::new(metadata);
    let reg = crate::commands::registry::CommandRegistry::new();
    assert!(stub.register_commands(&reg).is_ok());
}

#[cfg(unix)]
#[test]
fn validate_plugin_world_writable_adds_warning() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    let plugin_path = dir.path().join("ww.so");
    std::fs::write(&plugin_path, b"w").expect("write");
    let h = blake3::hash(b"w").to_hex().to_string();
    std::fs::write(plugin_path.with_extension("sig"), &h).expect("sig");
    let mut perms = std::fs::metadata(&plugin_path).expect("meta").permissions();
    perms.set_mode(0o666);
    std::fs::set_permissions(&plugin_path, perms).expect("chmod");

    temp_env::with_var(
        "SQUIRREL_PLUGIN_DIRS",
        Some(dir.path().to_str().expect("utf8")),
        || {
            let loader = SecurePluginLoader::new();
            let meta = PluginMetadata {
                name: "ww".to_string(),
                version: "1".to_string(),
                description: None,
                author: None,
                homepage: None,
                capabilities: vec![],
            };
            let v = loader
                .validate_plugin(&plugin_path, &meta)
                .expect("validate");
            assert!(v.signature_valid);
            assert!(!v.warnings.is_empty());
            assert!(!v.is_valid);
        },
    );
}
