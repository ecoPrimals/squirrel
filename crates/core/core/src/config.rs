use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the ecosystem service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    pub enabled: bool,
    pub mode: EcosystemMode,
    pub discovery: DiscoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemMode {
    Standalone,
    Sovereign,
    Coordinated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub auto_discovery: bool,
    pub songbird_endpoint: Option<String>,
    pub direct_endpoints: HashMap<String, String>,
    pub probe_interval: Duration,
    pub health_check_timeout: Duration,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: EcosystemMode::Sovereign,
            discovery: DiscoveryConfig::default(),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: true,
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT").ok(),
            direct_endpoints: HashMap::new(),
            probe_interval: Duration::seconds(60),
            health_check_timeout: Duration::seconds(5),
        }
    }
}

/// Configuration error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration parse error: {0}")]
    Parse(String),
    #[error("Missing required configuration: {0}")]
    Missing(String),
    #[error("Invalid configuration value: {0}")]
    Invalid(String),
}
