// SPDX-License-Identifier: AGPL-3.0-or-later
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
    /// Optional discovery service endpoint.
    #[serde(alias = "songbird_endpoint")]
    pub discovery_endpoint: Option<String>,
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
            discovery_endpoint: std::env::var("SERVICE_MESH_ENDPOINT")
                .or_else(|_| std::env::var("DISCOVERY_ENDPOINT"))
                .or_else(|_| std::env::var("SONGBIRD_ENDPOINT"))
                .ok(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_config_default_matches_expected_shape() {
        let c = EcosystemConfig::default();
        assert!(c.enabled);
        assert!(matches!(c.mode, EcosystemMode::Sovereign));
        assert!(c.discovery.auto_discovery);
        assert_eq!(c.discovery.probe_interval, Duration::seconds(60));
        assert_eq!(c.discovery.health_check_timeout, Duration::seconds(5));
        assert!(c.discovery.direct_endpoints.is_empty());
    }

    #[test]
    fn discovery_config_manual_roundtrip_serde() {
        let mut direct = HashMap::new();
        direct.insert("a".to_string(), "http://localhost:1".to_string());
        let cfg = DiscoveryConfig {
            auto_discovery: false,
            discovery_endpoint: Some("sb".to_string()),
            direct_endpoints: direct,
            probe_interval: Duration::seconds(30),
            health_check_timeout: Duration::seconds(10),
        };
        let json = serde_json::to_string(&cfg).expect("serialize");
        let back: DiscoveryConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(!back.auto_discovery);
        assert_eq!(back.discovery_endpoint.as_deref(), Some("sb"));
        assert_eq!(
            back.direct_endpoints.get("a").map(String::as_str),
            Some("http://localhost:1")
        );
        assert_eq!(back.probe_interval, Duration::seconds(30));
        assert_eq!(back.health_check_timeout, Duration::seconds(10));
    }

    #[test]
    fn ecosystem_config_full_roundtrip_serde() {
        let cfg = EcosystemConfig {
            enabled: false,
            mode: EcosystemMode::Coordinated,
            discovery: DiscoveryConfig {
                auto_discovery: true,
                discovery_endpoint: None,
                direct_endpoints: HashMap::new(),
                probe_interval: Duration::zero(),
                health_check_timeout: Duration::seconds(1),
            },
        };
        let v = serde_json::to_value(&cfg).expect("to_value");
        let back: EcosystemConfig = serde_json::from_value(v).expect("from_value");
        assert!(!back.enabled);
        assert!(matches!(back.mode, EcosystemMode::Coordinated));
        assert_eq!(back.discovery.health_check_timeout, Duration::seconds(1));
    }

    #[test]
    fn config_error_display() {
        assert_eq!(
            Error::Parse("bad".to_string()).to_string(),
            "Configuration parse error: bad"
        );
        assert_eq!(
            Error::Missing("key".to_string()).to_string(),
            "Missing required configuration: key"
        );
        assert_eq!(
            Error::Invalid("x".to_string()).to_string(),
            "Invalid configuration value: x"
        );
    }

    #[test]
    fn ecosystem_mode_variants_are_distinct_in_json() {
        let standalone = serde_json::to_string(&EcosystemMode::Standalone).expect("should succeed");
        let sovereign = serde_json::to_string(&EcosystemMode::Sovereign).expect("should succeed");
        assert_ne!(standalone, sovereign);
    }
}
