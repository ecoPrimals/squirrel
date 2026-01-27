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
    /// Socket path (overrides default)
    pub socket: Option<String>,

    /// Bind address (for future HTTP server)
    pub bind: String,

    /// Port (for future HTTP server)
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
            bind: "0.0.0.0".to_string(),
            port: 9010,
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
            Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)
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
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[server]"));
        assert!(toml.contains("[ai]"));
        assert!(toml.contains("[logging]"));
        assert!(toml.contains("[discovery]"));
    }
}
