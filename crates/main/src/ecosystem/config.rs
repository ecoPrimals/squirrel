// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem configuration
//!
//! Configuration types and defaults for ecosystem integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::registry::EcosystemRegistryConfig;
use super::types::{HealthCheckConfig, ResourceSpec, SecurityConfig};

/// Serde helpers for `Arc<str>` (avoids O(n) String clones when sharing IDs)
fn serialize_arc_str<S>(arc_str: &Arc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(arc_str)
}

fn deserialize_arc_str<'de, D>(deserializer: D) -> Result<Arc<str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Arc::from(s))
}

/// Ecosystem configuration for Squirrel primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    /// Service identifier (`Arc<str>` for O(1) clone when sharing across registrations)
    #[serde(
        serialize_with = "serialize_arc_str",
        deserialize_with = "deserialize_arc_str"
    )]
    pub service_id: Arc<str>,

    /// Service name
    pub service_name: String,

    /// Service host
    pub service_host: String,

    /// Service port
    pub service_port: u16,

    /// Service mesh endpoint URL (capability-based discovery)
    /// This is the endpoint for discovering other services via capabilities,
    /// not a hardcoded reference to any specific primal.
    pub service_mesh_endpoint: String,

    /// Biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Registry configuration
    pub registry_config: EcosystemRegistryConfig,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        use uuid::Uuid;

        Self {
            service_id: Arc::from(format!("primal-squirrel-{}", Uuid::new_v4())),
            service_name: "Squirrel AI Primal".to_string(),
            service_host: std::env::var("SQUIRREL_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            service_port: std::env::var("SQUIRREL_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8002),
            service_mesh_endpoint: std::env::var("SERVICE_MESH_ENDPOINT").unwrap_or_else(|_| {
                use universal_constants::network::get_service_port;
                let port = std::env::var("SERVICE_MESH_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| get_service_port("service_mesh"));
                format!("http://localhost:{port}")
            }),
            biome_id: std::env::var("BIOME_ID").ok(),
            registry_config: EcosystemRegistryConfig::default(),
            resource_requirements: ResourceSpec::default(),
            security_config: SecurityConfig::default(),
            health_check: HealthCheckConfig::default(),
            metadata: HashMap::new(),
        }
    }
}

impl EcosystemConfig {
    /// Create a new ecosystem configuration with specified values
    #[must_use]
    pub fn new(service_name: String, service_host: String, service_port: u16) -> Self {
        use uuid::Uuid;

        Self {
            service_id: Arc::from(format!("primal-squirrel-{}", Uuid::new_v4())),
            service_name,
            service_host,
            service_port,
            ..Default::default()
        }
    }

    /// Create configuration from environment variables
    ///
    /// Uses environment variables with sensible defaults:
    /// - `SQUIRREL_HOST` (default: localhost)
    /// - `SQUIRREL_PORT` (default: 8002)
    /// - `SERVICE_MESH_ENDPOINT` (default: http://localhost:8001)
    /// - `BIOME_ID` (optional)
    #[must_use]
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Validate the configuration
    ///
    /// Returns `Ok(())` if configuration is valid, or an error message.
    pub fn validate(&self) -> Result<(), String> {
        if self.service_name.is_empty() {
            return Err("Service name cannot be empty".to_string());
        }

        if self.service_host.is_empty() {
            return Err("Service host cannot be empty".to_string());
        }

        if self.service_port == 0 {
            return Err("Service port must be greater than 0".to_string());
        }

        if self.service_mesh_endpoint.is_empty() {
            return Err("Service mesh endpoint cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get the full service URL
    #[must_use]
    pub fn service_url(&self) -> String {
        format!("http://{}:{}", self.service_host, self.service_port)
    }

    /// Add metadata entry
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EcosystemConfig::default();
        assert!(!config.service_name.is_empty());
        assert!(!config.service_host.is_empty());
        assert!(config.service_port > 0);
    }

    #[test]
    fn test_config_validation() {
        let config = EcosystemConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = config;
        invalid_config.service_name = String::new();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_service_url() {
        let config =
            EcosystemConfig::new("Test Service".to_string(), "example.com".to_string(), 8080);
        assert_eq!(config.service_url(), "http://example.com:8080");
    }

    #[test]
    fn test_with_metadata() {
        let config = EcosystemConfig::default()
            .with_metadata("key1".to_string(), "value1".to_string())
            .with_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(config.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.metadata.get("key2"), Some(&"value2".to_string()));
    }
}
