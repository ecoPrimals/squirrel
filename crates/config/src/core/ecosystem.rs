//! Ecosystem configuration types for Squirrel MCP
//!
//! This module defines ecosystem-related configuration including
//! discovery settings, coordination with other primals, and biome manifest.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// Ecosystem configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EcosystemConfig {
    pub enabled: bool,
    pub mode: EcosystemMode,
    pub discovery: DiscoveryConfig,
    pub coordination: CoordinationConfig,
    pub biome_manifest: BiomeManifestConfig,
}

/// Ecosystem operation modes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EcosystemMode {
    #[serde(rename = "sovereign")]
    Sovereign, // Operate independently, coordinate when available
    #[serde(rename = "coordinated")]
    Coordinated, // Require coordination with other primals
    #[serde(rename = "standalone")]
    Standalone, // Operate without any coordination
}

/// Discovery configuration for finding other primals
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    pub songbird_endpoint: Option<String>,
    pub auto_discovery: bool,
    pub probe_interval: Duration,
    pub direct_endpoints: HashMap<String, String>,
    pub health_check_timeout: Duration,
}

/// Coordination configuration for working with other primals
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoordinationConfig {
    pub nestgate: Option<NestGateCoordination>,
    pub beardog: Option<BearDogCoordination>,
    pub toadstool: Option<ToadStoolCoordination>,
    pub fallback_strategies: HashMap<String, String>,
}

/// NestGate coordination settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NestGateCoordination {
    pub endpoint: Option<String>,
    pub auto_provision: bool,
    pub storage_class: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

/// BearDog coordination settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BearDogCoordination {
    pub endpoint: Option<String>,
    pub auto_auth: bool,
    pub security_level: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

/// ToadStool coordination settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToadStoolCoordination {
    pub endpoint: Option<String>,
    pub auto_delegate: bool,
    pub compute_class: String,
    pub fallback_to_local: bool,
    pub capabilities: Vec<String>,
}

/// Biome manifest configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BiomeManifestConfig {
    pub auto_generate: bool,
    pub output_path: String,
    pub metadata: BiomeMetadata,
}

/// Biome metadata for manifest generation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: EcosystemMode::Sovereign,
            discovery: DiscoveryConfig {
                songbird_endpoint: env::var("SONGBIRD_DISCOVERY_ENDPOINT").ok(),
                auto_discovery: true,
                probe_interval: Duration::from_secs(30),
                direct_endpoints: HashMap::from([
                    ("nestgate".to_string(), "http://localhost:8444".to_string()),
                    ("beardog".to_string(), "http://localhost:8443".to_string()),
                    ("toadstool".to_string(), "http://localhost:8445".to_string()),
                ]),
                health_check_timeout: Duration::from_secs(5),
            },
            coordination: CoordinationConfig {
                nestgate: Some(NestGateCoordination {
                    endpoint: env::var("NESTGATE_STORAGE_ENDPOINT").ok(),
                    auto_provision: true,
                    storage_class: "standard".to_string(),
                    fallback_to_local: true,
                    capabilities: vec![
                        "storage".to_string(),
                        "persistence".to_string(),
                        "backup".to_string(),
                    ],
                }),
                beardog: Some(BearDogCoordination {
                    endpoint: env::var("BEARDOG_AUTH_ENDPOINT").ok(),
                    auto_auth: true,
                    security_level: "enterprise".to_string(),
                    fallback_to_local: true,
                    capabilities: vec![
                        "authentication".to_string(),
                        "authorization".to_string(),
                        "encryption".to_string(),
                    ],
                }),
                toadstool: Some(ToadStoolCoordination {
                    endpoint: env::var("TOADSTOOL_COMPUTE_ENDPOINT").ok(),
                    auto_delegate: true,
                    compute_class: "standard".to_string(),
                    fallback_to_local: true,
                    capabilities: vec![
                        "compute".to_string(),
                        "containers".to_string(),
                        "wasm".to_string(),
                    ],
                }),
                fallback_strategies: HashMap::from([
                    ("storage".to_string(), "local-storage".to_string()),
                    ("auth".to_string(), "local-auth".to_string()),
                    ("compute".to_string(), "local-execution".to_string()),
                ]),
            },
            biome_manifest: BiomeManifestConfig {
                auto_generate: true,
                output_path: "biome.yaml".to_string(),
                metadata: BiomeMetadata {
                    name: "squirrel-mcp".to_string(),
                    description: "AI Agent Platform with MCP Protocol".to_string(),
                    version: "2.0.0".to_string(),
                    primal_type: "squirrel".to_string(),
                    capabilities: vec![
                        "mcp".to_string(),
                        "ai-agents".to_string(),
                        "context-management".to_string(),
                        "plugin-execution".to_string(),
                    ],
                },
            },
        }
    }
}
