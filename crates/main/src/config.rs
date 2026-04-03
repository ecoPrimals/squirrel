// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Squirrel Configuration Module
//!
//! Provides configuration loading with hierarchical precedence:
//! 1. Environment variables (highest priority)
//! 2. Configuration file
//! 3. CLI arguments
//! 4. Defaults (lowest priority)
//!
//! ## Configuration Files
//!
//! Supports TOML, YAML, and JSON formats. Default search paths:
//! 1. `./squirrel.toml` (current directory)
//! 2. `~/.config/squirrel/squirrel.toml` (user config)
//! 3. `/etc/squirrel/squirrel.toml` (system config)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use universal_constants::deployment::ports;
use universal_constants::network::BIND_ALL_INTERFACES;

fn default_bind() -> String {
    BIND_ALL_INTERFACES.to_string()
}

/// Main Squirrel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct SquirrelConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// AI router configuration
    pub ai: AiConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Discovery configuration
    pub discovery: DiscoveryConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Socket path override. When `None`, use `squirrel::rpc::unix_socket::get_socket_path(node_id)`
    /// for runtime discovery (XDG, env vars, capability-based).
    pub socket: Option<String>,

    /// Bind address — retained for config-file backward compatibility.
    /// Squirrel uses Unix sockets + localhost TCP JSON-RPC; this field is not
    /// consumed by the server runtime.
    #[serde(default = "default_bind")]
    pub bind: String,

    /// TCP port for localhost JSON-RPC (env: `SQUIRREL_PORT`, `SQUIRREL_SERVER_PORT`)
    pub port: u16,

    /// Run as daemon
    pub daemon: bool,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

/// AI router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Enable AI router
    pub enabled: bool,

    /// AI provider socket paths (comma-separated)
    pub provider_sockets: Option<String>,

    /// Enable retry with fallback providers
    pub enable_retry: bool,

    /// Maximum retry attempts
    pub max_retries: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable JSON output
    pub json: bool,

    /// Enable file logging
    pub file: Option<PathBuf>,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    /// Enable capability announcement
    pub announce_capabilities: bool,

    /// Capabilities to announce
    pub capabilities: Vec<String>,

    /// Registry socket path
    pub registry_socket: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket: None,
            bind: BIND_ALL_INTERFACES.to_string(),
            port: ports::squirrel_server(),
            daemon: false,
            max_connections: 100,
            request_timeout_secs: 30,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider_sockets: None,
            enable_retry: true,
            max_retries: 2,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json: false,
            file: None,
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            announce_capabilities: true,
            capabilities: vec![
                "ai.text_generation".to_string(),
                "ai.image_generation".to_string(),
                "ai.routing".to_string(),
                "tool.orchestration".to_string(),
            ],
            registry_socket: None,
        }
    }
}

/// Configuration loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from file with environment variable overrides
    pub fn load(config_path: Option<&Path>) -> Result<SquirrelConfig> {
        let mut config = if let Some(path) = config_path {
            // Load from specified path
            Self::load_from_file(path)?
        } else {
            // Try default paths
            Self::load_from_default_paths()?
        };

        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config)?;

        Ok(config)
    }

    /// Load from default search paths
    fn load_from_default_paths() -> Result<SquirrelConfig> {
        // Search paths in order
        let search_paths = vec![
            PathBuf::from("squirrel.toml"),
            PathBuf::from("config/squirrel.toml"),
            dirs::config_dir()
                .map(|p| p.join("squirrel").join("squirrel.toml"))
                .unwrap_or_default(),
            PathBuf::from("/etc/squirrel/squirrel.toml"),
        ];

        for path in search_paths {
            if path.exists() {
                tracing::info!("📄 Loading configuration from: {}", path.display());
                return Self::load_from_file(&path);
            }
        }

        tracing::info!("📄 No configuration file found, using defaults");
        Ok(SquirrelConfig::default())
    }

    /// Load configuration from a specific file
    fn load_from_file(path: &Path) -> Result<SquirrelConfig> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        // Determine format from extension
        let config = match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&contents)
                .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?,
            Some("yaml" | "yml") => serde_yaml_ng::from_str(&contents)
                .with_context(|| format!("Failed to parse YAML config: {}", path.display()))?,
            Some("json") => serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse JSON config: {}", path.display()))?,
            _ => {
                anyhow::bail!("Unsupported config file format: {}", path.display());
            }
        };

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(config: &mut SquirrelConfig) -> Result<()> {
        // Server overrides
        if let Ok(socket) = std::env::var("SQUIRREL_SOCKET") {
            config.server.socket = Some(socket);
        }
        if let Ok(bind) = std::env::var("SQUIRREL_BIND") {
            config.server.bind = bind;
        }
        if let Ok(port) = std::env::var("SQUIRREL_PORT") {
            config.server.port = port.parse().context("Invalid SQUIRREL_PORT value")?;
        }
        if let Ok(daemon) = std::env::var("SQUIRREL_DAEMON") {
            config.server.daemon = daemon.parse().context("Invalid SQUIRREL_DAEMON value")?;
        }

        // AI overrides
        if let Ok(sockets) = std::env::var("AI_PROVIDER_SOCKETS") {
            config.ai.provider_sockets = Some(sockets);
        }
        if let Ok(enabled) = std::env::var("SQUIRREL_AI_ENABLED") {
            config.ai.enabled = enabled
                .parse()
                .context("Invalid SQUIRREL_AI_ENABLED value")?;
        }

        // Logging overrides
        if let Ok(level) = std::env::var("SQUIRREL_LOG_LEVEL") {
            config.logging.level = level;
        }
        if let Ok(json) = std::env::var("SQUIRREL_LOG_JSON") {
            config.logging.json = json.parse().context("Invalid SQUIRREL_LOG_JSON value")?;
        }

        // Discovery overrides
        if let Ok(registry) = std::env::var("SQUIRREL_REGISTRY_SOCKET") {
            config.discovery.registry_socket = Some(registry);
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Config tests use expect for serde/TOML round-trips"
)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SquirrelConfig::default();
        assert_eq!(config.server.port, 9010);
        assert_eq!(config.server.bind, "0.0.0.0");
        assert!(config.ai.enabled);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_serialization() {
        let config = SquirrelConfig::default();
        let toml = toml::to_string(&config).expect("default config serializes to TOML");
        assert!(toml.contains("[server]"));
        assert!(toml.contains("[ai]"));
        assert!(toml.contains("[logging]"));
        assert!(toml.contains("[discovery]"));
    }

    #[test]
    fn test_default_server_config() {
        let server = ServerConfig::default();
        assert_eq!(server.bind, "0.0.0.0");
        assert_eq!(server.port, 9010);
        assert!(!server.daemon);
        assert_eq!(server.max_connections, 100);
        assert_eq!(server.request_timeout_secs, 30);
        assert!(server.socket.is_none());
    }

    #[test]
    fn test_default_ai_config() {
        let ai = AiConfig::default();
        assert!(ai.enabled);
        assert!(ai.provider_sockets.is_none());
        assert!(ai.enable_retry);
        assert_eq!(ai.max_retries, 2);
    }

    #[test]
    fn test_default_logging_config() {
        let logging = LoggingConfig::default();
        assert_eq!(logging.level, "info");
        assert!(!logging.json);
        assert!(logging.file.is_none());
    }

    #[test]
    fn test_default_discovery_config() {
        let discovery = DiscoveryConfig::default();
        assert!(discovery.announce_capabilities);
        assert!(
            discovery
                .capabilities
                .contains(&"ai.text_generation".to_string())
        );
        assert!(
            discovery
                .capabilities
                .contains(&"ai.image_generation".to_string())
        );
        assert!(discovery.capabilities.contains(&"ai.routing".to_string()));
        assert!(
            discovery
                .capabilities
                .contains(&"tool.orchestration".to_string())
        );
        assert!(discovery.registry_socket.is_none());
    }

    #[test]
    fn test_config_json_roundtrip() {
        let config = SquirrelConfig::default();
        let json = serde_json::to_string(&config).expect("config json roundtrip");
        let deserialized: SquirrelConfig =
            serde_json::from_str(&json).expect("config deserializes from json");
        assert_eq!(deserialized.server.port, config.server.port);
        assert_eq!(deserialized.server.bind, config.server.bind);
        assert_eq!(deserialized.ai.enabled, config.ai.enabled);
        assert_eq!(deserialized.logging.level, config.logging.level);
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let config = SquirrelConfig::default();
        let toml_str = toml::to_string(&config).expect("config toml roundtrip serialize");
        let deserialized: SquirrelConfig =
            toml::from_str(&toml_str).expect("config deserializes from toml");
        assert_eq!(deserialized.server.port, config.server.port);
        assert_eq!(deserialized.ai.max_retries, config.ai.max_retries);
    }

    const SQUIRREL_ENV_VARS: &[&str] = &[
        "SQUIRREL_SOCKET",
        "SQUIRREL_BIND",
        "SQUIRREL_PORT",
        "SQUIRREL_DAEMON",
        "AI_PROVIDER_SOCKETS",
        "SQUIRREL_AI_ENABLED",
        "SQUIRREL_LOG_LEVEL",
        "SQUIRREL_LOG_JSON",
        "SQUIRREL_REGISTRY_SOCKET",
    ];

    #[test]
    fn test_env_overrides_all() {
        temp_env::with_vars(
            [
                ("SQUIRREL_SOCKET", Some("/tmp/test-squirrel.sock")),
                ("SQUIRREL_BIND", Some("127.0.0.1")),
                ("SQUIRREL_PORT", Some("9999")),
                ("SQUIRREL_DAEMON", Some("true")),
            ],
            || {
                let mut config = SquirrelConfig::default();
                ConfigLoader::apply_env_overrides(&mut config)
                    .expect("apply_env_overrides with valid test env");
                assert_eq!(
                    config.server.socket.as_deref(),
                    Some("/tmp/test-squirrel.sock")
                );
                assert_eq!(config.server.bind, "127.0.0.1");
                assert_eq!(config.server.port, 9999);
                assert!(config.server.daemon);
            },
        );

        temp_env::with_vars(
            [
                ("AI_PROVIDER_SOCKETS", Some("/tmp/ai1.sock,/tmp/ai2.sock")),
                ("SQUIRREL_AI_ENABLED", Some("false")),
            ],
            || {
                let mut config = SquirrelConfig::default();
                ConfigLoader::apply_env_overrides(&mut config)
                    .expect("apply_env_overrides with valid test env");
                assert_eq!(
                    config.ai.provider_sockets.as_deref(),
                    Some("/tmp/ai1.sock,/tmp/ai2.sock")
                );
                assert!(!config.ai.enabled);
            },
        );

        temp_env::with_vars(
            [
                ("SQUIRREL_LOG_LEVEL", Some("debug")),
                ("SQUIRREL_LOG_JSON", Some("true")),
            ],
            || {
                let mut config = SquirrelConfig::default();
                ConfigLoader::apply_env_overrides(&mut config)
                    .expect("apply_env_overrides with valid test env");
                assert_eq!(config.logging.level, "debug");
                assert!(config.logging.json);
            },
        );

        temp_env::with_var(
            "SQUIRREL_REGISTRY_SOCKET",
            Some("/tmp/registry.sock"),
            || {
                let mut config = SquirrelConfig::default();
                ConfigLoader::apply_env_overrides(&mut config)
                    .expect("apply_env_overrides with valid test env");
                assert_eq!(
                    config.discovery.registry_socket.as_deref(),
                    Some("/tmp/registry.sock")
                );
            },
        );

        temp_env::with_var("SQUIRREL_PORT", Some("not-a-number"), || {
            let mut config = SquirrelConfig::default();
            let result = ConfigLoader::apply_env_overrides(&mut config);
            assert!(result.is_err());
        });

        temp_env::with_var("SQUIRREL_DAEMON", Some("not-a-bool"), || {
            let mut config = SquirrelConfig::default();
            let result = ConfigLoader::apply_env_overrides(&mut config);
            assert!(result.is_err());
        });

        temp_env::with_vars_unset(SQUIRREL_ENV_VARS, || {
            let config = ConfigLoader::load(None).expect("load default config with env unset");
            assert_eq!(config.server.port, 9010);
        });
    }

    #[test]
    fn test_load_from_toml_file() {
        temp_env::with_vars_unset(SQUIRREL_ENV_VARS, || {
            let dir = tempfile::tempdir().expect("tempdir for config file test");
            let config_path = dir.path().join("squirrel.toml");
            std::fs::write(
                &config_path,
                r#"
[server]
port = 8080
bind = "127.0.0.1"
max_connections = 50

[ai]
enabled = false
max_retries = 5

[logging]
level = "debug"
json = true

[discovery]
announce_capabilities = false
"#,
            )
            .expect("write test config file");

            let config = ConfigLoader::load(Some(config_path.as_path()))
                .expect("load config from test file");
            assert_eq!(config.server.port, 8080);
            assert_eq!(config.server.bind, "127.0.0.1");
            assert_eq!(config.server.max_connections, 50);
            assert!(!config.ai.enabled);
            assert_eq!(config.ai.max_retries, 5);
            assert_eq!(config.logging.level, "debug");
            assert!(config.logging.json);
            assert!(!config.discovery.announce_capabilities);
        });
    }

    #[test]
    fn test_load_from_yaml_file() {
        temp_env::with_vars_unset(SQUIRREL_ENV_VARS, || {
            let dir = tempfile::tempdir().expect("tempdir for config file test");
            let config_path = dir.path().join("squirrel.yaml");
            std::fs::write(
                &config_path,
                r#"
server:
  port: 7070
  bind: "0.0.0.0"
ai:
  enabled: true
  max_retries: 3
logging:
  level: "warn"
  json: false
discovery:
  announce_capabilities: true
"#,
            )
            .expect("write test config file");

            let config = ConfigLoader::load(Some(config_path.as_path()))
                .expect("load config from test file");
            assert_eq!(config.server.port, 7070);
            assert!(config.ai.enabled);
            assert_eq!(config.ai.max_retries, 3);
            assert_eq!(config.logging.level, "warn");
        });
    }

    #[test]
    fn test_load_from_json_file() {
        temp_env::with_vars_unset(SQUIRREL_ENV_VARS, || {
            let dir = tempfile::tempdir().expect("tempdir for config file test");
            let config_path = dir.path().join("squirrel.json");
            std::fs::write(
                &config_path,
                r#"{"server":{"port":6060,"bind":"0.0.0.0"},"ai":{"enabled":true},"logging":{"level":"info"},"discovery":{"announce_capabilities":true}}"#,
            )
            .expect("write test config file");

            let config = ConfigLoader::load(Some(config_path.as_path()))
                .expect("load config from test file");
            assert_eq!(config.server.port, 6060);
        });
    }

    #[test]
    fn test_load_from_nonexistent_file() {
        let result = ConfigLoader::load(Some(std::path::Path::new("/nonexistent/squirrel.toml")));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_unsupported_format() {
        let dir = tempfile::tempdir().expect("tempdir for unsupported format test");
        let config_path = dir.path().join("squirrel.txt");
        std::fs::write(&config_path, "invalid format").expect("write invalid config file");

        let result = ConfigLoader::load(Some(config_path.as_path()));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported config file format")
        );
    }

    #[test]
    fn test_load_from_invalid_toml() {
        let dir = tempfile::tempdir().expect("tempdir for invalid toml test");
        let config_path = dir.path().join("squirrel.toml");
        std::fs::write(&config_path, "invalid toml [[[[").expect("write invalid toml");

        let result = ConfigLoader::load(Some(config_path.as_path()));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_invalid_json() {
        let dir = tempfile::tempdir().expect("tempdir for invalid json test");
        let config_path = dir.path().join("squirrel.json");
        std::fs::write(&config_path, "{ invalid json }").expect("write invalid json");

        let result = ConfigLoader::load(Some(config_path.as_path()));
        assert!(result.is_err());
    }

    #[test]
    fn test_env_override_invalid_squirrel_ai_enabled() {
        temp_env::with_var("SQUIRREL_AI_ENABLED", Some("not-a-bool"), || {
            let mut config = SquirrelConfig::default();
            let result = ConfigLoader::apply_env_overrides(&mut config);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_env_override_invalid_squirrel_log_json() {
        temp_env::with_var("SQUIRREL_LOG_JSON", Some("invalid"), || {
            let mut config = SquirrelConfig::default();
            let result = ConfigLoader::apply_env_overrides(&mut config);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_server_config_socket_override() {
        let server = ServerConfig::default();
        assert!(server.socket.is_none());
        let with_socket = ServerConfig {
            socket: Some("/tmp/custom.sock".to_string()),
            ..server
        };
        assert_eq!(with_socket.socket.as_deref(), Some("/tmp/custom.sock"));
    }

    #[test]
    fn test_logging_config_file_path() {
        let mut logging = LoggingConfig::default();
        assert!(logging.file.is_none());
        logging.file = Some(std::path::PathBuf::from("/var/log/squirrel.log"));
        assert_eq!(
            logging.file.as_ref().expect("logging.file set above"),
            &std::path::PathBuf::from("/var/log/squirrel.log")
        );
    }
}
