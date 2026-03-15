// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the ecosystem service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    /// Whether ecosystem is enabled.
    pub enabled: bool,
    /// Ecosystem operation mode.
    pub mode: EcosystemMode,
    /// Discovery configuration.
    pub discovery: DiscoveryConfig,
}

/// Ecosystem operation mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemMode {
    /// Single-node standalone.
    Standalone,
    /// Sovereign node (can join federation).
    Sovereign,
    /// Coordinated with service mesh.
    Coordinated,
}

/// Discovery configuration for finding primals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Whether to auto-discover primals.
    pub auto_discovery: bool,
    /// Optional Songbird endpoint for discovery.
    pub songbird_endpoint: Option<String>,
    /// Direct endpoint overrides.
    pub direct_endpoints: HashMap<String, String>,
    /// Interval between discovery probes.
    pub probe_interval: Duration,
    /// Timeout for health checks.
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

/// Configuration error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration parsing failed.
    #[error("Configuration parse error: {0}")]
    Parse(String),
    /// Required configuration is missing.
    #[error("Missing required configuration: {0}")]
    Missing(String),
    /// Configuration value is invalid.
    #[error("Invalid configuration value: {0}")]
    Invalid(String),
}
