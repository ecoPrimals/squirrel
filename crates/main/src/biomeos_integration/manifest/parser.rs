// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Manifest parsing and validation for biome.yaml files.
//!
//! This module provides the parser implementation for loading, validating,
//! and merging biome manifest configurations.

use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

use crate::error::PrimalError;

use super::types::{
    AgentResourceLimits, AgentSecurity, AgentSpec, AgentStorage, AuthenticationConfig,
    AuthorizationConfig, BiomeManifest, BiomeMetadata, BiomeNetworking, BiomeResources,
    BiomeSecurity, BiomeStorage, DnsConfig, EncryptionConfig, ExecutionEnvironment, IngressConfig,
    ResourceLimits, ResourcePolicies, ResourceReservations, SecurityPolicies, TokenConfig,
};

// ============================================================================
// PARSER TYPES
// ============================================================================

/// Parser for biome.yaml manifest files.
#[derive(Debug)]
pub struct BiomeManifestParser {
    /// Parser configuration options
    pub config: ManifestParserConfig,
}

/// Configuration for manifest parsing behavior.
#[derive(Debug, Clone)]
pub struct ManifestParserConfig {
    /// Whether to perform strict validation
    pub strict_validation: bool,
    /// Whether to validate against schema
    pub schema_validation: bool,
    /// Whether to allow unknown fields
    pub allow_unknown_fields: bool,
    /// Default values for missing fields
    pub default_values: HashMap<String, serde_json::Value>,
}

// ============================================================================
// PARSER IMPLEMENTATION
// ============================================================================

impl BiomeManifestParser {
    /// Creates a new parser with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ManifestParserConfig::default(),
        }
    }

    /// Creates a parser with the given configuration.
    #[must_use]
    pub const fn with_config(config: ManifestParserConfig) -> Self {
        Self { config }
    }

    /// Generates a template biome manifest for reference or scaffolding.
    #[must_use]
    pub fn generate_template() -> BiomeManifest {
        let ui_host = std::env::var("UI_HOST").unwrap_or_else(|_| "localhost".to_string());

        BiomeManifest {
            metadata: BiomeMetadata {
                name: "example-biome".to_string(),
                version: "1.0.0".to_string(),
                description: "Example biome manifest".to_string(),
                author: "Squirrel AI".to_string(),
                biomeos_version: "2.0.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                labels: HashMap::new(),
            },
            agents: vec![AgentSpec {
                name: "example-agent".to_string(),
                capabilities: vec!["chat".to_string(), "search".to_string()],
                ai_provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                execution_environment: ExecutionEnvironment::Container,
                resource_limits: AgentResourceLimits {
                    memory_mb: 512,
                    memory_limit_mb: Some(512),
                    cpu_percent: 50.0,
                    timeout_seconds: 300,
                    max_concurrent_requests: 10,
                    storage_mb: 1024,
                },
                resources: AgentResourceLimits {
                    memory_mb: 512,
                    memory_limit_mb: Some(512),
                    cpu_percent: 50.0,
                    timeout_seconds: 300,
                    max_concurrent_requests: 10,
                    storage_mb: 1024,
                },
                manifest: None,
                security: AgentSecurity {
                    auth_method: "bearer".to_string(),
                    permissions: vec!["read".to_string()],
                    security_context: "default".to_string(),
                    encryption: EncryptionConfig {
                        enabled: true,
                        algorithm: "AES256".to_string(),
                        key_size: 256,
                        at_rest: true,
                        in_transit: true,
                    },
                },
                storage: AgentStorage {
                    persistent: vec![],
                    temporary: vec![],
                    cache: vec![],
                },
                environment: HashMap::new(),
                config: HashMap::new(),
            }],
            services: HashMap::new(),
            resources: BiomeResources {
                limits: ResourceLimits {
                    memory_gb: 8.0,
                    cpu_cores: 4.0,
                    storage_gb: 100.0,
                    network_bandwidth_mbps: 1000.0,
                },
                reservations: ResourceReservations {
                    memory_gb: 2.0,
                    cpu_cores: 1.0,
                    storage_gb: 10.0,
                },
                policies: ResourcePolicies {
                    memory_over_commit: false,
                    cpu_over_commit: true,
                    storage_over_commit: false,
                    resource_quotas: HashMap::new(),
                },
            },
            security: BiomeSecurity {
                authentication: AuthenticationConfig {
                    enabled: true,
                    method: "oauth2".to_string(),
                    providers: vec!["github".to_string()],
                },
                authorization: AuthorizationConfig {
                    enabled: true,
                    method: "rbac".to_string(),
                    policies: vec!["default".to_string()],
                },
                encryption: EncryptionConfig::default(),
                tokens: TokenConfig {
                    enabled: true,
                    expiration_seconds: 3600,
                    refresh_enabled: true,
                },
                policies: SecurityPolicies {
                    network_policies: vec![],
                    pod_security_policies: vec![],
                    rbac_policies: vec![],
                },
            },
            storage: BiomeStorage {
                volumes: vec![],
                volume_claim_templates: vec![],
                storage_classes: vec![],
            },
            networking: BiomeNetworking {
                ingress: IngressConfig {
                    enabled: false,
                    host: ui_host,
                    tls_enabled: false,
                    annotations: HashMap::new(),
                },
                network_policies: vec![],
                dns: DnsConfig {
                    enabled: true,
                    servers: vec!["8.8.8.8".to_string()],
                    search_domains: vec![],
                },
            },
            primals: HashMap::new(),
        }
    }

    /// Parses a manifest from a file path.
    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<BiomeManifest, PrimalError> {
        let path = path.as_ref();
        info!("Parsing biome.yaml manifest from: {}", path.display());

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| PrimalError::ConfigError(format!("Failed to read manifest file: {e}")))?;

        self.parse_content(&content).await
    }

    /// Parses manifest content from a string.
    pub async fn parse_content(&self, content: &str) -> Result<BiomeManifest, PrimalError> {
        debug!("Parsing biome.yaml manifest content");

        let mut manifest: BiomeManifest = serde_yml::from_str(content)
            .map_err(|e| PrimalError::ConfigError(format!("Failed to parse YAML: {e}")))?;

        if self.config.strict_validation {
            self.validate_manifest(&mut manifest)?;
        }

        Ok(manifest)
    }

    fn validate_manifest(&self, manifest: &mut BiomeManifest) -> Result<(), PrimalError> {
        // Validate metadata
        if manifest.metadata.name.is_empty() {
            return Err(PrimalError::ConfigError(
                "Biome name cannot be empty".to_string(),
            ));
        }

        // Validate agents
        for agent in &manifest.agents {
            if agent.name.is_empty() {
                return Err(PrimalError::ConfigError(
                    "Agent name cannot be empty".to_string(),
                ));
            }
            if agent.capabilities.is_empty() {
                return Err(PrimalError::ConfigError(format!(
                    "Agent '{}' must have at least one capability",
                    agent.name
                )));
            }
        }

        // Validate services
        for (name, service) in &manifest.services {
            if service.endpoints.is_empty() {
                return Err(PrimalError::ConfigError(format!(
                    "Service '{name}' must have at least one endpoint"
                )));
            }
        }

        Ok(())
    }

    /// Validates YAML syntax without full manifest parsing.
    pub fn validate_yaml_schema(&self, content: &str) -> Result<(), PrimalError> {
        let _: serde_yml::Value = serde_yml::from_str(content)
            .map_err(|e| PrimalError::ConfigError(format!("Invalid YAML syntax: {e}")))?;

        Ok(())
    }

    /// Merges a base manifest with an overlay, with overlay values taking precedence.
    pub fn merge_manifests(
        &self,
        base: BiomeManifest,
        overlay: BiomeManifest,
    ) -> Result<BiomeManifest, PrimalError> {
        let mut merged = base;

        // Merge metadata
        if !overlay.metadata.name.is_empty() {
            merged.metadata.name = overlay.metadata.name;
        }
        if !overlay.metadata.description.is_empty() {
            merged.metadata.description = overlay.metadata.description;
        }

        // Merge agents
        merged.agents.extend(overlay.agents);

        // Merge services
        merged.services.extend(overlay.services);

        // Merge primals
        merged.primals.extend(overlay.primals);

        Ok(merged)
    }
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for ManifestParserConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            schema_validation: false,
            allow_unknown_fields: false,
            default_values: HashMap::new(),
        }
    }
}

impl Default for BiomeManifestParser {
    fn default() -> Self {
        Self::new()
    }
}
