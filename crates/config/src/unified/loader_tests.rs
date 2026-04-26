// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_config_loader_default() {
    let loader = ConfigLoader::new();
    let config = loader.build().expect("should succeed");
    assert!(!config.system.instance_id.is_empty());
}

#[test]
fn test_config_validation() {
    let mut loader = ConfigLoader::new();
    loader.config.security.enabled = false;
    let result = loader.validate();
    assert!(result.is_ok());
}

#[test]
fn test_load_nonexistent_file() {
    let loader = ConfigLoader::new();
    let result = loader.with_file_if_exists("nonexistent.toml");
    assert!(result.is_ok());
}

#[test]
fn test_sources_tracking() {
    let mut cfg_loader = ConfigLoader::new();
    cfg_loader.config.security.enabled = false;
    let resolved = cfg_loader
        .validate()
        .expect("should succeed")
        .build_with_sources()
        .expect("should succeed");
    assert!(!resolved.sources().is_empty());
    assert!(resolved.has_source("secure_defaults"));
}

#[test]
fn test_unsupported_format_error() {
    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("config.xyz");
    fs::write(&config_path, "invalid").expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    let err = result.expect_err("expected UnsupportedFormat error");
    assert!(matches!(err, ConfigError::UnsupportedFormat { .. }));
    assert!(err.to_string().contains("xyz"));
}

#[test]
fn test_invalid_toml_parse_error() {
    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("config.toml");
    fs::write(&config_path, "invalid toml [[[[").expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    let err = result.expect_err("expected ParseError");
    assert!(matches!(err, ConfigError::ParseError { .. }));
}

#[test]
fn test_invalid_json_parse_error() {
    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("config.json");
    fs::write(&config_path, "{ invalid json }").expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    let err = result.expect_err("expected ParseError");
    assert!(matches!(err, ConfigError::ParseError { .. }));
}

#[test]
fn test_loaded_config_into_config() {
    let mut cfg_loader = ConfigLoader::new();
    cfg_loader.config.security.enabled = false;
    let resolved = cfg_loader
        .validate()
        .expect("should succeed")
        .build_with_sources()
        .expect("should succeed");
    let config = resolved.into_config();
    assert!(!config.system.instance_id.is_empty());
}

#[test]
fn test_config_loader_default_impl() {
    let loader = ConfigLoader::default();
    let config = loader.build().expect("should succeed");
    assert!(!config.system.instance_id.is_empty());
}

#[test]
fn test_with_platform_detection() {
    let result = ConfigLoader::new().with_platform_detection();
    assert!(result.is_ok());
    let loader = result.expect("should succeed");
    let config = loader.build().expect("should succeed");
    assert!(
        !config
            .system
            .data_dir
            .as_os_str()
            .to_string_lossy()
            .is_empty()
    );
    assert!(
        !config
            .system
            .plugin_dir
            .as_os_str()
            .to_string_lossy()
            .is_empty()
    );
}

#[test]
fn test_with_env_prefix() {
    let mut cfg_loader = ConfigLoader::new();
    cfg_loader.config.security.enabled = false;
    let result = cfg_loader.with_env_prefix("SQUIRREL_");
    assert!(result.is_ok());
    let resolved = result
        .expect("should succeed")
        .build_with_sources()
        .expect("should succeed");
    assert!(resolved.has_source("env:"));
}

#[test]
fn test_valid_toml_file_loading() {
    let default_config = SquirrelUnifiedConfig::default();
    let toml_content = toml::to_string(&default_config).expect("should succeed");

    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("squirrel.toml");
    fs::write(&config_path, &toml_content).expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
    let loader = result.expect("should succeed");
    let config = loader.build().expect("should succeed");
    assert_eq!(config.system.environment, default_config.system.environment);
    assert_eq!(config.network.http_port, default_config.network.http_port);
}

#[test]
fn test_valid_json_file_loading() {
    let default_config = SquirrelUnifiedConfig::default();
    let json_content = serde_json::to_string(&default_config).expect("should succeed");

    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("config.json");
    fs::write(&config_path, &json_content).expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
    let config = result
        .expect("should succeed")
        .build()
        .expect("should succeed");
    assert_eq!(config.system.environment, default_config.system.environment);
    assert_eq!(config.network.http_port, default_config.network.http_port);
}

#[test]
fn test_valid_yaml_file_loading() {
    let default_config = SquirrelUnifiedConfig::default();
    let yaml_content = serde_yaml_ng::to_string(&default_config).expect("should succeed");

    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(&config_path, &yaml_content).expect("should succeed");

    let result = ConfigLoader::new().with_file_if_exists(&config_path);
    assert!(result.is_ok(), "Expected Ok but got Err: {:?}", result);
    let config = result
        .expect("should succeed")
        .build()
        .expect("should succeed");
    assert_eq!(config.system.environment, default_config.system.environment);
    assert_eq!(config.network.http_port, default_config.network.http_port);
}

#[test]
fn test_config_error_display() {
    let err = ConfigError::UnsupportedFormat {
        format: "xml".to_string(),
    };
    assert!(err.to_string().contains("xml"));
    assert!(err.to_string().contains("Unsupported"));

    let err = ConfigError::FileRead {
        path: PathBuf::from("/nonexistent"),
        error: "Permission denied".to_string(),
    };
    assert!(err.to_string().contains("Permission denied"));
}

fn write_valid_config(path: &std::path::Path, overrides: &[(&str, &str)]) {
    let mut config = SquirrelUnifiedConfig::default();
    config.security.enabled = false;
    for (key, value) in overrides {
        match *key {
            "system.instance_id" => config.system.instance_id = value.to_string(),
            "system.environment" => config.system.environment = value.to_string(),
            "system.log_level" => config.system.log_level = value.to_string(),
            "network.http_port" => {
                if let Ok(port) = value.parse() {
                    config.network.http_port = port;
                } else {
                    tracing::warn!("Skipping invalid network.http_port override: {value:?}");
                }
            }
            "network.websocket_port" => {
                if let Ok(port) = value.parse() {
                    config.network.websocket_port = port;
                } else {
                    tracing::warn!("Skipping invalid network.websocket_port override: {value:?}");
                }
            }
            _ => {}
        }
    }
    let toml = toml::to_string(&config).expect("should succeed");
    fs::write(path, toml).expect("should succeed");
}

#[test]
fn test_merge_config_non_overlapping_fields() {
    let temp_dir = TempDir::new().expect("should succeed");
    let file1 = temp_dir.path().join("a.toml");
    write_valid_config(
        &file1,
        &[
            ("system.instance_id", "instance-from-a"),
            ("system.environment", "staging"),
        ],
    );
    let file2 = temp_dir.path().join("b.toml");
    write_valid_config(
        &file2,
        &[
            ("system.instance_id", ""),
            ("system.environment", ""),
            ("system.log_level", ""),
            ("network.http_port", "9090"),
            ("network.websocket_port", "9091"),
        ],
    );

    let mut loader = ConfigLoader::new();
    loader.config.security.enabled = false;
    let loader = loader
        .with_file_if_exists(&file1)
        .expect("should succeed")
        .with_file_if_exists(&file2)
        .expect("should succeed");
    let config = loader.build().expect("should succeed");

    assert_eq!(config.system.instance_id, "instance-from-a");
    assert_eq!(config.system.environment, "staging");
    assert_eq!(config.network.http_port, 9090);
    assert_eq!(config.network.websocket_port, 9091);
}

#[test]
fn test_merge_config_precedence_later_wins() {
    let temp_dir = TempDir::new().expect("should succeed");
    let file1 = temp_dir.path().join("first.toml");
    write_valid_config(
        &file1,
        &[
            ("system.instance_id", "first-instance"),
            ("system.environment", "development"),
            ("network.http_port", "8080"),
        ],
    );
    let file2 = temp_dir.path().join("second.toml");
    write_valid_config(
        &file2,
        &[
            ("system.instance_id", "second-instance"),
            ("system.environment", "production"),
            ("network.http_port", "9999"),
        ],
    );

    let mut loader = ConfigLoader::new();
    loader.config.security.enabled = false;
    let loader = loader
        .with_file_if_exists(&file1)
        .expect("should succeed")
        .with_file_if_exists(&file2)
        .expect("should succeed");
    let config = loader.build().expect("should succeed");

    assert_eq!(config.system.instance_id, "second-instance");
    assert_eq!(config.system.environment, "production");
    assert_eq!(config.network.http_port, 9999);
}

#[test]
fn test_merge_config_partial_overrides() {
    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("partial.toml");
    write_valid_config(&config_path, &[("network.http_port", "3000")]);

    let mut loader = ConfigLoader::new();
    loader.config.security.enabled = false;
    let loader = loader
        .with_file_if_exists(&config_path)
        .expect("should succeed");
    let config = loader.build().expect("should succeed");

    assert_eq!(config.network.http_port, 3000);
    assert!(!config.system.instance_id.is_empty());
    assert!(config.network.websocket_port > 0);
}

#[test]
fn test_merge_config_with_default() {
    let temp_dir = TempDir::new().expect("should succeed");
    let config_path = temp_dir.path().join("override.toml");
    write_valid_config(&config_path, &[("system.log_level", "debug")]);

    let mut loader = ConfigLoader::new();
    loader.config.security.enabled = false;
    let loader = loader
        .with_file_if_exists(&config_path)
        .expect("should succeed");
    let config = loader.build().expect("should succeed");

    assert_eq!(config.system.log_level, "debug");
    assert!(!config.security.enabled);
}

#[test]
fn test_config_loader_load_integration() {
    let temp_dir = TempDir::new().expect("should succeed");
    write_valid_config(
        temp_dir.path().join("squirrel.toml").as_path(),
        &[
            ("system.instance_id", "load-test-instance"),
            ("system.environment", "staging"),
            ("network.http_port", "7777"),
        ],
    );

    let original_cwd = std::env::current_dir().expect("should succeed");
    std::env::set_current_dir(temp_dir.path()).expect("should succeed");

    let result = temp_env::with_vars(
        [
            ("SQUIRREL_HTTP_PORT", Some("8888")),
            (
                "JWT_SECRET",
                Some("test-jwt-secret-at-least-32-characters-long"),
            ),
        ],
        ConfigLoader::load,
    );

    std::env::set_current_dir(&original_cwd).expect("should succeed");

    let loaded = result.expect("load should succeed");
    assert!(loaded.has_source("file:"));
    assert!(loaded.has_source("secure_defaults"));

    let config = loaded.config();
    assert_eq!(config.system.instance_id, "load-test-instance");
    assert_eq!(config.system.environment, "staging");
    assert_eq!(config.network.http_port, 7777);
}
