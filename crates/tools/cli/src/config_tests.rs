// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use anyhow::Result;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let config = CliConfig::default();
    assert_eq!(config.log_level, "info");
    assert_eq!(config.output_format, "text");
    assert_eq!(config.mcp_host, "127.0.0.1"); // Updated to match DEFAULT_HOST constant
    assert_eq!(config.mcp_port, 9000);
    assert!(!config.verbose);
    assert!(!config.quiet);
    assert!(config.custom.is_empty());
}

#[test]
fn test_config_get_set() -> Result<()> {
    let mut config = CliConfig::default();

    // Test standard fields
    config.set("log_level", "debug".to_string())?;
    assert_eq!(config.get("log_level")?, "debug");

    config.set("output_format", "json".to_string())?;
    assert_eq!(config.get("output_format")?, "json");

    config.set("mcp_port", "9001".to_string())?;
    assert_eq!(config.get("mcp_port")?, "9001");

    // Test custom fields
    config.set("custom_key", "custom_value".to_string())?;
    assert_eq!(config.get("custom_key")?, "custom_value");

    Ok(())
}

#[test]
fn test_config_save_load() -> Result<()> {
    let dir = tempdir()?;
    let config_path = dir.path().join("test_config.toml");

    // Create and save config
    let mut config = CliConfig {
        log_level: "debug".to_string(),
        output_format: "json".to_string(),
        ..Default::default()
    };
    config.set("custom_key", "custom_value".to_string())?;

    config.save_to_file(&config_path)?;

    // Load config
    let loaded_config = CliConfig::load_from_file(&config_path)?;

    // Verify loaded config
    assert_eq!(loaded_config.log_level, "debug");
    assert_eq!(loaded_config.output_format, "json");
    assert_eq!(loaded_config.get("custom_key")?, "custom_value");

    Ok(())
}

#[test]
fn test_config_merge() {
    let mut config1 = CliConfig::default();
    let mut config2 = CliConfig::default();

    // Set different values in config1
    config1.log_level = "info".to_string();

    // Set different values in config2
    config2.output_format = "json".to_string();
    config2.mcp_port = 9001;

    // Add custom values
    config1
        .custom
        .insert("key1".to_string(), "value1".to_string());
    config2
        .custom
        .insert("key2".to_string(), "value2".to_string());

    // Merge configs
    config1.merge(config2);

    // Verify merged config
    assert_eq!(config1.log_level, "info");
    assert_eq!(config1.output_format, "json");
    assert_eq!(config1.mcp_port, 9001);
    assert_eq!(
        config1.custom.get("key1").expect("should succeed"),
        "value1"
    );
    assert_eq!(
        config1.custom.get("key2").expect("should succeed"),
        "value2"
    );
}

#[test]
fn test_load_from_file_invalid_toml() {
    let dir = tempdir().expect("should succeed");
    let path = dir.path().join("bad.toml");
    std::fs::write(&path, "not [[ valid").expect("should succeed");
    let err = CliConfig::load_from_file(&path).unwrap_err();
    assert!(matches!(err, ConfigError::ParseError(_)));
}

#[test]
fn test_get_key_not_found() {
    let config = CliConfig::default();
    let err = config.get("no_such_standard_key").unwrap_err();
    assert!(matches!(err, ConfigError::KeyNotFound(_)));
}

#[test]
fn test_set_invalid_mcp_port() {
    let mut config = CliConfig::default();
    let err = config.set("mcp_port", "xyz".to_string()).unwrap_err();
    assert!(matches!(err, ConfigError::PathError(_)));
}

#[test]
fn test_set_invalid_bool() {
    let mut config = CliConfig::default();
    let err = config.set("verbose", "maybe".to_string()).unwrap_err();
    assert!(matches!(err, ConfigError::PathError(_)));
}

#[test]
fn test_cli_config_serde_roundtrip() {
    let mut c = CliConfig::default();
    c.set("custom_key", "v".to_string())
        .expect("should succeed");
    let toml = toml::to_string(&c).expect("should succeed");
    let back: CliConfig = toml::from_str(&toml).expect("should succeed");
    assert_eq!(back.get("custom_key").expect("should succeed"), "v");
}

#[test]
fn test_config_manager_save_errors_without_path() {
    let mgr = ConfigManager::new();
    let err = mgr.save(None).unwrap_err();
    assert!(matches!(err, ConfigError::PathError(_)));
}

#[test]
fn test_config_manager_list() {
    let mut mgr = ConfigManager::with_config(CliConfig::default());
    mgr.config_mut()
        .set("custom_x", "y".to_string())
        .expect("should succeed");
    let m = mgr.list();
    assert_eq!(m.get("mcp_port").expect("should succeed"), "9000");
    assert_eq!(m.get("custom_x").expect("should succeed"), "y");
}

#[test]
fn cli_config_new_matches_default() {
    assert_eq!(CliConfig::new().log_level, CliConfig::default().log_level);
}

#[test]
fn from_env_with_no_matching_vars_returns_ok() {
    let cfg = CliConfig::from_env("___CLI_TEST_NO_MATCH___").expect("from_env");
    assert_eq!(cfg.log_level, "info");
}

#[test]
fn merge_skips_empty_log_level_but_merges_port() {
    let mut base = CliConfig {
        log_level: "warn".to_string(),
        ..CliConfig::default()
    };
    let other = CliConfig {
        log_level: String::new(),
        mcp_port: 9002,
        ..CliConfig::default()
    };
    base.merge(other);
    assert_eq!(base.log_level, "warn");
    assert_eq!(base.mcp_port, 9002);
}

#[test]
fn config_manager_import_export_file() -> Result<()> {
    let dir = tempdir()?;
    let src = dir.path().join("source.toml");
    let dst = dir.path().join("export.toml");
    let mut file_cfg = CliConfig::default();
    file_cfg.set("region", "us-west".to_string())?;
    file_cfg.save_to_file(&src)?;
    let mut mgr = ConfigManager::with_config(CliConfig::default());
    mgr.import(src)?;
    assert_eq!(mgr.get("region")?, "us-west");
    mgr.export(dst.clone())?;
    let round = CliConfig::load_from_file(&dst)?;
    assert_eq!(round.get("region")?, "us-west");
    Ok(())
}

#[test]
fn config_manager_save_with_explicit_path() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("out.toml");
    let mut mgr = ConfigManager::with_config(CliConfig::default());
    mgr.config_mut().set("log_level", "trace".to_string())?;
    mgr.save(Some(path.clone()))?;
    let loaded = CliConfig::load_from_file(&path)?;
    assert_eq!(loaded.log_level, "trace");
    Ok(())
}

#[test]
fn load_from_file_missing_returns_read_error() {
    let err = CliConfig::load_from_file("/nonexistent/path/that/does/not/exist.toml").unwrap_err();
    assert!(matches!(err, ConfigError::ReadError(_)));
}

#[test]
fn config_error_display_includes_context() {
    let read = ConfigError::ReadError(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    assert!(read.to_string().contains("read"));
    let parse = ConfigError::ParseError(toml::from_str::<CliConfig>("[").unwrap_err());
    assert!(parse.to_string().contains("parse"));
    let path = ConfigError::PathError("bad port".to_string());
    assert!(path.to_string().contains("bad port"));
    let key = ConfigError::KeyNotFound("k".to_string());
    assert!(key.to_string().contains('k'));
}

#[test]
fn merge_skips_zero_mcp_port_and_empty_strings() {
    let mut base = CliConfig {
        mcp_port: 9001,
        mcp_host: "10.0.0.1".to_string(),
        output_format: "yaml".to_string(),
        ..Default::default()
    };
    let other = CliConfig {
        mcp_port: 0,
        mcp_host: String::new(),
        output_format: String::new(),
        log_level: String::new(),
        ..Default::default()
    };
    base.merge(other);
    assert_eq!(base.mcp_port, 9001);
    assert_eq!(base.mcp_host, "10.0.0.1");
    assert_eq!(base.output_format, "yaml");
}

#[test]
fn get_and_set_quiet_roundtrip() -> Result<()> {
    let mut c = CliConfig::default();
    c.set("quiet", "true".to_string())?;
    assert_eq!(c.get("quiet")?, "true");
    c.set("quiet", "false".to_string())?;
    assert_eq!(c.get("quiet")?, "false");
    Ok(())
}

#[test]
fn save_to_file_creates_parent_directories() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path().join("nested").join("cfg.toml");
    let cfg = CliConfig::default();
    cfg.save_to_file(&path)?;
    assert!(path.exists());
    Ok(())
}

#[test]
fn config_manager_load_with_missing_explicit_path_merges_env() {
    temp_env::with_vars(
        [
            ("SQUIRREL_LOG_LEVEL", Some("trace")),
            ("SQUIRREL_MCP_PORT", Some("9003")),
        ],
        || {
            let mgr =
                ConfigManager::load(Some(PathBuf::from("/no/such/squirrel.toml"))).expect("load");
            assert_eq!(mgr.config().log_level, "trace");
            assert_eq!(mgr.config().mcp_port, 9003);
        },
    );
}

#[test]
fn config_manager_config_path_and_mut() {
    let mut mgr = ConfigManager::new();
    assert!(mgr.config_path().is_none());
    mgr.config_mut().verbose = true;
    assert!(mgr.config().verbose);
}
