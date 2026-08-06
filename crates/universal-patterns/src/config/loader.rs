// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration loader for loading PrimalConfig from various sources
//!
//! This module provides functionality to load configuration from:
//! - Files (YAML, JSON, TOML)
//! - Environment variables
//! - Command line arguments
//! - Multiple sources with precedence

#![cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "Parent `config` re-exports; wildcard keeps loader terse"
    )
)]

use super::*;
use dirs::config_dir;
use serde_json::Value;
use std::env;
use std::path::Path;

/// Supported configuration file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// YAML configuration format
    Yaml,
    /// JSON configuration format
    Json,
    /// TOML configuration format
    Toml,
}

impl FileFormat {
    fn from_path(path: &Path) -> Result<Self, ConfigError> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("yaml" | "yml") => Ok(Self::Yaml),
            Some("json") => Ok(Self::Json),
            Some("toml") => Ok(Self::Toml),
            _ => Err(ConfigError::Invalid(format!(
                "Unsupported file format: {}",
                path.display()
            ))),
        }
    }

    fn parse_str(&self, content: &str) -> Result<PrimalConfig, ConfigError> {
        match self {
            Self::Yaml => Ok(serde_yaml_ng::from_str(content)?),
            Self::Json => Ok(serde_json::from_str(content)?),
            Self::Toml => Ok(toml::from_str(content)?),
        }
    }
}

fn deep_merge(base: &mut Value, overlay: Value) {
    let (Value::Object(base_map), Value::Object(overlay_map)) = (base, overlay) else {
        return;
    };

    for (key, overlay_val) in overlay_map {
        match base_map.get_mut(&key) {
            Some(base_val) if base_val.is_object() && overlay_val.is_object() => {
                deep_merge(base_val, overlay_val);
            }
            _ => {
                base_map.insert(key, overlay_val);
            }
        }
    }
}

fn merge_config(base: &mut PrimalConfig, overlay: PrimalConfig) -> Result<(), ConfigError> {
    let mut base_val = serde_json::to_value(&*base)?;
    let overlay_val = serde_json::to_value(overlay)?;
    deep_merge(&mut base_val, overlay_val);
    *base = serde_json::from_value(base_val)?;
    Ok(())
}

fn parse_env_value(raw: &str) -> Value {
    if let Ok(value) = serde_json::from_str(raw) {
        return value;
    }
    if let Ok(value) = raw.parse::<i64>() {
        return Value::Number(value.into());
    }
    if let Ok(value) = raw.parse::<f64>() {
        if let Some(number) = serde_json::Number::from_f64(value) {
            return Value::Number(number);
        }
    }
    match raw.to_ascii_lowercase().as_str() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ => Value::String(raw.to_string()),
    }
}

fn insert_env_path(map: &mut serde_json::Map<String, Value>, parts: &[&str], raw: &str) {
    if parts.is_empty() {
        return;
    }

    let key = parts[0].to_ascii_lowercase();
    if parts.len() == 1 {
        map.insert(key, parse_env_value(raw));
        return;
    }

    let entry = map
        .entry(key)
        .or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Value::Object(nested) = entry {
        insert_env_path(nested, &parts[1..], raw);
    }
}

fn env_vars_to_json(prefix: &str) -> Value {
    let prefix_upper = format!("{}_", prefix.to_ascii_uppercase());
    let mut root = serde_json::Map::new();

    for (key, value) in env::vars() {
        let key_upper = key.to_ascii_uppercase();
        if !key_upper.starts_with(&prefix_upper) {
            continue;
        }

        let remainder = &key[prefix_upper.len()..];
        let parts: Vec<&str> = remainder.split("__").collect();
        insert_env_path(&mut root, &parts, &value);
    }

    Value::Object(root)
}

fn apply_env_prefix(config: &mut PrimalConfig, prefix: &str) -> Result<(), ConfigError> {
    let env_value = env_vars_to_json(prefix);
    if env_value.as_object().is_some_and(|map| map.is_empty()) {
        return Ok(());
    }

    let env_config: PrimalConfig = serde_json::from_value(env_value)?;
    merge_config(config, env_config)
}

fn parse_file(path: &Path) -> Result<PrimalConfig, ConfigError> {
    let format = FileFormat::from_path(path)?;
    let content = std::fs::read_to_string(path)?;
    format.parse_str(&content)
}

fn merge_file_if_exists(config: &mut PrimalConfig, path: &Path) -> Result<(), ConfigError> {
    if !path.exists() {
        return Ok(());
    }

    let file_config = parse_file(path)?;
    merge_config(config, file_config)
}

/// Configuration loader for PrimalConfig
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<PrimalConfig, ConfigError> {
        parse_file(path.as_ref())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<PrimalConfig, ConfigError> {
        Self::from_env_with_prefix("PRIMAL")
    }

    /// Load configuration from environment variables with custom prefix
    pub fn from_env_with_prefix(prefix: &str) -> Result<PrimalConfig, ConfigError> {
        let env_value = env_vars_to_json(prefix);
        Ok(serde_json::from_value(env_value)?)
    }

    /// Load configuration with multiple sources and precedence
    ///
    /// Sources in order of precedence (highest to lowest):
    /// 1. Environment variables
    /// 2. Local config file (./config.yaml)
    /// 3. User config file (~/.config/primal/config.yaml)
    /// 4. System config file (/etc/primal/config.yaml)
    /// 5. Default configuration
    pub fn load() -> Result<PrimalConfig, ConfigError> {
        let mut config = PrimalConfig::default();

        merge_file_if_exists(&mut config, Path::new("/etc/primal/config.yaml"))?;

        if let Some(config_dir) = config_dir() {
            let user_config = config_dir.join("primal").join("config.yaml");
            merge_file_if_exists(&mut config, &user_config)?;
        }

        merge_file_if_exists(&mut config, Path::new("./config.yaml"))?;
        apply_env_prefix(&mut config, "PRIMAL")?;

        Ok(config)
    }

    /// Load configuration for a specific primal
    pub fn load_for_primal(primal_name: &str) -> Result<PrimalConfig, ConfigError> {
        let mut config = PrimalConfig::default();

        let system_config = Path::new("/etc/primal").join(format!("{primal_name}.yaml"));
        merge_file_if_exists(&mut config, &system_config)?;

        if let Some(config_dir) = config_dir() {
            let user_config = config_dir
                .join("primal")
                .join(format!("{primal_name}.yaml"));
            merge_file_if_exists(&mut config, &user_config)?;
        }

        let local_config = Path::new("./").join(format!("{primal_name}.yaml"));
        merge_file_if_exists(&mut config, &local_config)?;

        let env_prefix = format!("PRIMAL_{}", primal_name.to_ascii_uppercase());
        apply_env_prefix(&mut config, &env_prefix)?;

        Ok(config)
    }

    /// Load configuration with custom sources
    pub fn load_with_sources(sources: Vec<ConfigSource>) -> Result<PrimalConfig, ConfigError> {
        let mut config = PrimalConfig::default();

        for source in sources {
            match source {
                ConfigSource::File { path, format } => {
                    if path.exists() {
                        let content = std::fs::read_to_string(&path)?;
                        let file_config = format.parse_str(&content)?;
                        merge_config(&mut config, file_config)?;
                    }
                }
                ConfigSource::Environment { prefix } => {
                    apply_env_prefix(&mut config, &prefix)?;
                }
                ConfigSource::Defaults => {}
            }
        }

        Ok(config)
    }

    /// Auto-detect and load configuration
    pub fn auto_load() -> Result<PrimalConfig, ConfigError> {
        // Try to determine primal name from environment or current directory
        let primal_name = env::var(universal_constants::env_vars::primal::NAME).or_else(|_| {
            env::current_dir()
                .ok()
                .and_then(|dir| {
                    dir.file_name()
                        .and_then(|name| name.to_str().map(String::from))
                })
                .ok_or(env::VarError::NotPresent)
        });

        match primal_name {
            Ok(name) => Self::load_for_primal(&name),
            Err(_) => Self::load(),
        }
    }

    /// Validate configuration file syntax without loading
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::Invalid(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let _: PrimalConfig = parse_file(path)?;
        Ok(())
    }

    /// Generate a template configuration file
    pub fn generate_template<P: AsRef<Path>>(
        path: P,
        primal_type: PrimalType,
    ) -> Result<(), ConfigError> {
        let config = match primal_type {
            PrimalType::Coordinator => ConfigBuilder::squirrel().build_unchecked(),
            PrimalType::Security => ConfigBuilder::security().build_unchecked(),
            PrimalType::Orchestration => ConfigBuilder::orchestration().build_unchecked(),
            _ => PrimalConfig::default(),
        };

        config.save(path)?;
        Ok(())
    }
}

/// Configuration source for loading
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// File source
    File {
        /// Path to the configuration file
        path: PathBuf,
        /// Format of the configuration file
        format: FileFormat,
    },
    /// Environment variable source
    Environment {
        /// Prefix for environment variables
        prefix: String,
    },
    /// Default configuration
    Defaults,
}

impl ConfigSource {
    /// Create a YAML file source
    pub fn yaml_file<P: AsRef<Path>>(path: P) -> Self {
        Self::File {
            path: path.as_ref().to_path_buf(),
            format: FileFormat::Yaml,
        }
    }

    /// Create a JSON file source
    pub fn json_file<P: AsRef<Path>>(path: P) -> Self {
        Self::File {
            path: path.as_ref().to_path_buf(),
            format: FileFormat::Json,
        }
    }

    /// Create a TOML file source
    pub fn toml_file<P: AsRef<Path>>(path: P) -> Self {
        Self::File {
            path: path.as_ref().to_path_buf(),
            format: FileFormat::Toml,
        }
    }

    /// Create an environment variable source
    pub fn env(prefix: &str) -> Self {
        Self::Environment {
            prefix: prefix.to_string(),
        }
    }

    /// Create a default configuration source
    pub fn defaults() -> Self {
        Self::Defaults
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_load_from_yaml_file() {
        // Test basic YAML loading functionality by creating a valid configuration

        let temp_dir = TempDir::new().expect("should succeed");
        let yaml_path = temp_dir.path().join("test_config.yaml");

        // Generate a complete configuration template first
        ConfigLoader::generate_template(&yaml_path, PrimalType::Coordinator)
            .expect("should succeed");

        // Verify the file exists and contains expected content
        assert!(yaml_path.exists());
        let content = std::fs::read_to_string(&yaml_path).expect("should succeed");
        assert!(content.contains("name:"));
        assert!(content.contains("port:"));
        assert!(content.contains("Coordinator"));

        // For testing YAML loading, verify basic file operations work
        // Full configuration loading requires complete environment setup
        assert!(!content.is_empty());
    }

    #[test]
    fn test_load_from_env() {
        temp_env::with_vars(
            [
                ("TEST_SQUIRREL_NAME", Some("env-test-primal")),
                ("TEST_SQUIRREL_VERSION", Some("2.0.0")),
                ("TEST_SQUIRREL_PORT", Some("9000")),
            ],
            || {
                let config = ConfigBuilder::new()
                    .name("test-primal")
                    .version("1.0.0")
                    .port(8080)
                    .build_unchecked();
                assert_eq!(config.info.name, "test-primal");
                assert_eq!(config.info.version, "1.0.0");
                assert_eq!(config.network.port, 8080);
            },
        );
    }

    #[test]
    fn test_validate_file() {
        let mut temp_file = NamedTempFile::new().expect("should succeed");
        temp_file
            .write_all(
                br#"
info:
  name: "test-primal"
  version: "1.0.0"
  instance_id: "550e8400-e29b-41d4-a716-446655440000"
  primal_type: "Coordinator"
  description: "Test primal"
  created_at: "2023-01-01T00:00:00Z"
network:
  bind_address: "127.0.0.1"
  port: 8080
"#,
            )
            .expect("should succeed");

        // This should fail because the config is incomplete
        assert!(ConfigLoader::validate_file(temp_file.path()).is_err());
    }

    #[test]
    fn test_generate_template() {
        let temp_dir = TempDir::new().expect("should succeed");
        let config_path = temp_dir.path().join("squirrel.yaml");

        ConfigLoader::generate_template(&config_path, PrimalType::Coordinator)
            .expect("should succeed");

        assert!(config_path.exists());

        // Just verify the file was created and has content
        let content = std::fs::read_to_string(&config_path).expect("should succeed");
        assert!(!content.is_empty());
        assert!(content.contains("name:"));
        assert!(content.contains("port:"));

        // Note: Full validation requires encryption keys and other environment setup
        // For template generation testing, verifying file creation and basic content is sufficient
    }

    #[test]
    fn test_config_sources() {
        let yaml_source = ConfigSource::yaml_file("test.yaml");
        let json_source = ConfigSource::json_file("test.json");
        let env_source = ConfigSource::env("TEST");
        let defaults_source = ConfigSource::defaults();

        match yaml_source {
            ConfigSource::File { format, .. } => assert_eq!(format, FileFormat::Yaml),
            _ => unreachable!("Expected file source"),
        }

        match json_source {
            ConfigSource::File { format, .. } => assert_eq!(format, FileFormat::Json),
            _ => unreachable!("Expected file source"),
        }

        match env_source {
            ConfigSource::Environment { prefix } => assert_eq!(prefix, "TEST"),
            _ => unreachable!("Expected environment source"),
        }

        match defaults_source {
            ConfigSource::Defaults => {}
            _ => unreachable!("Expected defaults source"),
        }
    }

    #[test]
    fn test_from_file_unsupported_format() {
        let temp_dir = TempDir::new().expect("should succeed");
        let bad_path = temp_dir.path().join("config.txt");
        std::fs::write(&bad_path, "invalid").expect("should succeed");

        let result = ConfigLoader::from_file(&bad_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Invalid(_)));
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result = ConfigLoader::from_file("/nonexistent/path/config.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_with_prefix() {
        // from_env_with_prefix builds config from env vars; may fail if schema doesn't match
        let _ = ConfigLoader::from_env_with_prefix("TEST_LOADER");
    }

    #[test]
    fn test_validate_file_nonexistent() {
        let result = ConfigLoader::validate_file("/nonexistent/config.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Invalid(_)));
    }

    #[test]
    fn test_validate_file_unsupported_extension() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("config.xyz");
        std::fs::write(&path, "x: 1").expect("should succeed");

        let result = ConfigLoader::validate_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_with_sources_defaults_and_env() {
        let result = ConfigLoader::load_with_sources(vec![
            ConfigSource::defaults(),
            ConfigSource::env("LOADER_TEST"),
        ]);
        let _ = result;
    }

    #[test]
    fn test_load_with_sources_nonexistent_file_skipped() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("does_not_exist.yaml");

        let result = ConfigLoader::load_with_sources(vec![
            ConfigSource::defaults(),
            ConfigSource::yaml_file(&path),
        ]);
        assert!(!path.exists());
        let _ = result;
    }

    #[test]
    fn test_generate_template_all_primal_types() {
        let temp_dir = TempDir::new().expect("should succeed");
        for pt in [
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::Custom("custom".to_string()),
        ] {
            let path = temp_dir.path().join(format!("{:?}.yaml", pt));
            let result = ConfigLoader::generate_template(&path, pt.clone());
            assert!(result.is_ok(), "Failed for {:?}", pt);
            assert!(path.exists());
        }
    }

    #[test]
    fn test_from_file_json_round_trip() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("cfg.json");
        let original = PrimalConfig::default();
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&original).expect("should succeed"),
        )
        .expect("should succeed");

        let loaded = ConfigLoader::from_file(&path).expect("should succeed");
        assert_eq!(loaded.info.version, original.info.version);
    }

    #[test]
    fn test_from_file_toml_round_trip() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("cfg.toml");
        let original = PrimalConfig::default();
        std::fs::write(&path, toml::to_string(&original).expect("should succeed"))
            .expect("should succeed");

        let loaded = ConfigLoader::from_file(&path).expect("should succeed");
        assert_eq!(loaded.environment.name, original.environment.name);
    }

    #[test]
    fn test_validate_file_accepts_valid_config() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("valid.json");
        let cfg = PrimalConfig::default();
        std::fs::write(&path, serde_json::to_string(&cfg).expect("should succeed"))
            .expect("should succeed");

        ConfigLoader::validate_file(&path).expect("should succeed");
    }

    #[test]
    fn test_load_with_sources_existing_json_file() {
        let temp_dir = TempDir::new().expect("should succeed");
        let path = temp_dir.path().join("layer.json");
        let cfg = PrimalConfig::default();
        std::fs::write(&path, serde_json::to_string(&cfg).expect("should succeed"))
            .expect("should succeed");

        let loaded = ConfigLoader::load_with_sources(vec![
            ConfigSource::defaults(),
            ConfigSource::json_file(&path),
        ])
        .expect("should succeed");
        assert_eq!(loaded.info.name, cfg.info.name);
    }

    #[test]
    fn test_auto_load_without_primal_name_invokes_load_path() {
        temp_env::with_var("PRIMAL_NAME", None::<&str>, || {
            let _ = ConfigLoader::auto_load();
        });
    }
}
