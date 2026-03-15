// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # biome.yaml Manifest Support for biomeOS Integration
//!
//! This module provides comprehensive support for parsing and processing biome.yaml
//! manifest files, enabling agent deployment, service discovery, and resource
//! management within the biomeOS ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

use crate::error::PrimalError;
// Removed: use squirrel_mcp_config::get_service_endpoints;

// ============================================================================
// CORE MANIFEST TYPES
// ============================================================================

/// biome.yaml manifest structure for biomeOS integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    /// Manifest metadata.
    pub metadata: BiomeMetadata,
    /// Agent specifications.
    pub agents: Vec<AgentSpec>,
    /// Service configurations by name.
    pub services: HashMap<String, ServiceConfig>,
    /// Resource limits and policies.
    pub resources: BiomeResources,
    /// Security configuration.
    pub security: BiomeSecurity,
    /// Storage configuration.
    pub storage: BiomeStorage,
    /// Networking configuration.
    pub networking: BiomeNetworking,
    /// Primal configurations by name.
    pub primals: HashMap<String, PrimalConfig>,
}

/// Metadata for the biome manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    /// Biome name.
    pub name: String,
    /// Biome description.
    pub description: String,
    /// Manifest version.
    pub version: String,
    /// biomeOS version compatibility.
    pub biomeos_version: String,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
    /// Author identifier.
    pub author: String,
    /// Arbitrary labels.
    pub labels: HashMap<String, String>,
}

// ============================================================================
// AGENT SPECIFICATIONS
// ============================================================================

/// Agent manifest data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    /// Agent version.
    pub version: String,
    /// Agent description.
    pub description: String,
    /// Agent author.
    pub author: String,
    /// Agent capabilities.
    pub capabilities: Vec<String>,
    /// Agent dependencies.
    pub dependencies: Vec<String>,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Agent name.
    pub name: String,
    /// Required capabilities.
    pub capabilities: Vec<String>,
    /// AI provider identifier.
    pub ai_provider: String,
    /// Model identifier.
    pub model: String,
    /// Execution environment (native, wasm, container, vm).
    pub execution_environment: ExecutionEnvironment,
    /// Resource limits.
    pub resource_limits: AgentResourceLimits,
    /// Resource limits (alias for compatibility).
    pub resources: AgentResourceLimits,
    /// Optional agent manifest data.
    pub manifest: Option<AgentManifest>,
    /// Security configuration.
    pub security: AgentSecurity,
    /// Storage configuration.
    pub storage: AgentStorage,
    /// Environment variables.
    pub environment: HashMap<String, String>,
    /// Additional configuration.
    pub config: HashMap<String, serde_json::Value>,
}

/// Execution environment for agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionEnvironment {
    /// Native process execution.
    #[serde(rename = "native")]
    #[default]
    Native,
    /// WebAssembly sandbox.
    #[serde(rename = "wasm")]
    Wasm,
    /// Container (e.g., Docker).
    #[serde(rename = "container")]
    Container,
    /// Virtual machine.
    #[serde(rename = "vm")]
    VirtualMachine,
}

/// Resource limits for agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceLimits {
    /// Memory limit in MB.
    pub memory_mb: u64,
    /// Memory limit in MB (alias for compatibility).
    #[serde(alias = "memory_limit")]
    pub memory_limit_mb: Option<u64>,
    /// CPU limit as percentage.
    pub cpu_percent: f64,
    /// Timeout in seconds.
    pub timeout_seconds: u64,
    /// Maximum concurrent requests.
    pub max_concurrent_requests: u32,
    /// Storage limit in MB.
    pub storage_mb: u64,
}

/// Security configuration for agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSecurity {
    /// Authentication method.
    pub auth_method: String,
    /// Permission list.
    pub permissions: Vec<String>,
    /// Security context.
    pub security_context: String,
    /// Encryption configuration.
    pub encryption: EncryptionConfig,
}

/// Storage configuration for agents.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentStorage {
    /// Persistent storage volumes.
    pub persistent: Vec<StorageVolume>,
    /// Temporary storage volumes.
    pub temporary: Vec<StorageVolume>,
    /// Cache storage volumes.
    pub cache: Vec<StorageVolume>,
}

/// Storage volume configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVolume {
    /// Volume name.
    pub name: String,
    /// Size specification.
    pub size: String,
    /// Storage tier.
    pub tier: String,
    /// Provisioner type.
    pub provisioner: String,
    /// Mount path.
    pub mount_path: String,
    /// Access mode (e.g., ReadWriteOnce).
    pub access_mode: String,
}

// ============================================================================
// SERVICE CONFIGURATION
// ============================================================================

/// Service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Type of service.
    pub service_type: String,
    /// Service endpoints.
    pub endpoints: Vec<ServiceEndpoint>,
    /// Service dependencies.
    pub dependencies: Vec<String>,
    /// Load balancer configuration.
    pub load_balancer: LoadBalancerConfig,
    /// Health check configuration.
    pub health_check: HealthCheckConfig,
    /// Scaling configuration.
    pub scaling: ScalingConfig,
    /// Service mesh configuration.
    pub service_mesh: ServiceMeshConfig,
}

/// Service endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint name.
    pub name: String,
    /// Port number.
    pub port: u16,
    /// Protocol (e.g., http, grpc).
    pub protocol: String,
    /// URL path.
    pub path: String,
    /// Health check path.
    pub health_check_path: String,
}

/// Load balancer configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// Load balancing algorithm.
    pub algorithm: String,
    /// Health check configuration.
    pub health_check: HealthCheckConfig,
    /// Session affinity setting.
    pub session_affinity: String,
}

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled.
    pub enabled: bool,
    /// Health check path.
    pub path: String,
    /// Check interval in seconds.
    pub interval_seconds: u64,
    /// Check timeout in seconds.
    pub timeout_seconds: u64,
    /// Number of retries before unhealthy.
    pub retries: u32,
}

/// Scaling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Whether scaling is enabled.
    pub enabled: bool,
    /// Minimum replica count.
    pub min_replicas: u32,
    /// Maximum replica count.
    pub max_replicas: u32,
    /// Target CPU percentage for scaling.
    pub target_cpu_percent: f64,
    /// Target memory percentage for scaling.
    pub target_memory_percent: f64,
}

/// Service mesh configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    /// Whether service mesh is enabled.
    pub enabled: bool,
    /// Mesh provider.
    pub provider: String,
    /// mTLS configuration.
    pub mtls: MtlsConfig,
}

/// mTLS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    /// Whether mTLS is enabled.
    pub enabled: bool,
    /// CA certificate path or content.
    pub ca: String,
    /// Certificate rotation period.
    pub rotation_period: String,
}

// ============================================================================
// RESOURCE MANAGEMENT
// ============================================================================

/// Biome resource configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeResources {
    /// Resource limits.
    pub limits: ResourceLimits,
    /// Resource reservations.
    pub reservations: ResourceReservations,
    /// Resource policies.
    pub policies: ResourcePolicies,
}

/// Resource limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Memory limit in GB.
    pub memory_gb: f64,
    /// CPU limit in cores.
    pub cpu_cores: f64,
    /// Storage limit in GB.
    pub storage_gb: f64,
    /// Network bandwidth limit in Mbps.
    pub network_bandwidth_mbps: f64,
}

/// Resource reservations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservations {
    /// Reserved memory in GB.
    pub memory_gb: f64,
    /// Reserved CPU cores.
    pub cpu_cores: f64,
    /// Reserved storage in GB.
    pub storage_gb: f64,
}

/// Resource policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicies {
    /// Whether memory over-commit is allowed.
    pub memory_over_commit: bool,
    /// Whether CPU over-commit is allowed.
    pub cpu_over_commit: bool,
    /// Whether storage over-commit is allowed.
    pub storage_over_commit: bool,
    /// Resource quotas by resource name.
    pub resource_quotas: HashMap<String, String>,
}

// ============================================================================
// SECURITY CONFIGURATION
// ============================================================================

/// Biome security configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSecurity {
    /// Authentication configuration.
    pub authentication: AuthenticationConfig,
    /// Authorization configuration.
    pub authorization: AuthorizationConfig,
    /// Encryption configuration.
    pub encryption: EncryptionConfig,
    /// Token configuration.
    pub tokens: TokenConfig,
    /// Security policies.
    pub policies: SecurityPolicies,
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Whether authentication is enabled.
    pub enabled: bool,
    /// Authentication method.
    pub method: String,
    /// Auth provider identifiers.
    pub providers: Vec<String>,
}

/// Authorization configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Whether authorization is enabled.
    pub enabled: bool,
    /// Authorization method (e.g., rbac, abac)
    pub method: String,
    /// Policy identifiers to apply
    pub policies: Vec<String>,
}

/// Encryption configuration for data protection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Whether encryption is enabled
    pub enabled: bool,
    /// Encryption algorithm (e.g., AES256)
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Whether data at rest is encrypted
    pub at_rest: bool,
    /// Whether data in transit is encrypted
    pub in_transit: bool,
}

/// Token configuration for authentication tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Whether token-based auth is enabled
    pub enabled: bool,
    /// Token expiration time in seconds
    pub expiration_seconds: u64,
    /// Whether token refresh is enabled
    pub refresh_enabled: bool,
}

/// Security policies for the biome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicies {
    /// Network policy identifiers
    pub network_policies: Vec<String>,
    /// Pod security policy identifiers
    pub pod_security_policies: Vec<String>,
    /// RBAC policy identifiers
    pub rbac_policies: Vec<String>,
}

// ============================================================================
// STORAGE AND NETWORKING
// ============================================================================

/// Storage configuration for the biome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeStorage {
    /// Storage volumes to provision
    pub volumes: Vec<StorageVolume>,
    /// Volume claim templates for dynamic provisioning
    pub volume_claim_templates: Vec<VolumeClaimTemplate>,
    /// Storage class definitions
    pub storage_classes: Vec<StorageClass>,
}

/// Template for creating persistent volume claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeClaimTemplate {
    /// Volume claim name
    pub name: String,
    /// Size specification (e.g., 10Gi)
    pub size: String,
    /// Storage class to use
    pub storage_class: String,
    /// Access modes (e.g., ReadWriteOnce)
    pub access_modes: Vec<String>,
}

/// Storage class definition for dynamic provisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClass {
    /// Storage class name
    pub name: String,
    /// Provisioner type
    pub provisioner: String,
    /// Provisioner-specific parameters
    pub parameters: HashMap<String, String>,
}

/// Networking configuration for the biome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeNetworking {
    /// Ingress configuration for external access
    pub ingress: IngressConfig,
    /// Network policies for traffic control
    pub network_policies: Vec<NetworkPolicy>,
    /// DNS configuration
    pub dns: DnsConfig,
}

/// Ingress configuration for external traffic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    /// Whether ingress is enabled
    pub enabled: bool,
    /// Hostname for ingress
    pub host: String,
    /// Whether TLS is enabled
    pub tls_enabled: bool,
    /// Ingress annotations
    pub annotations: HashMap<String, String>,
}

/// Network policy for traffic control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,
    /// Selectors for matching resources
    pub selectors: HashMap<String, String>,
    /// Ingress rules
    pub ingress_rules: Vec<NetworkRule>,
    /// Egress rules
    pub egress_rules: Vec<NetworkRule>,
}

/// Network rule for ingress or egress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// Selectors for traffic sources/destinations
    pub from_selectors: Vec<HashMap<String, String>>,
    /// Allowed port numbers
    pub ports: Vec<u16>,
}

/// DNS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    /// Whether custom DNS is enabled
    pub enabled: bool,
    /// DNS server addresses
    pub servers: Vec<String>,
    /// Search domains for name resolution
    pub search_domains: Vec<String>,
}

/// Configuration for a primal in the biome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Type of primal (e.g., squirrel, toadstool)
    pub primal_type: String,
    /// Service endpoint URL
    pub endpoint: String,
    /// Whether the primal is enabled
    pub enabled: bool,
    /// Capability identifiers
    pub capabilities: Vec<String>,
    /// Primal-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

// ============================================================================
// MANIFEST PARSER IMPLEMENTATION
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
    pub fn with_config(config: ManifestParserConfig) -> Self {
        Self { config }
    }

    /// Generates a template biome manifest for reference or scaffolding.
    #[must_use]
    pub fn generate_template() -> BiomeManifest {
        BiomeManifest {
            metadata: BiomeMetadata {
                name: "example-biome".to_string(),
                version: "1.0.0".to_string(),
                description: "Example biome manifest".to_string(),
                author: "Squirrel AI".to_string(),
                biomeos_version: "2.0.0".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
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
                    host: std::env::var("UI_HOST").unwrap_or_else(|_| "localhost".to_string()),
                    tls_enabled: false,
                    annotations: std::collections::HashMap::new(),
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
        // Basic YAML syntax validation
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

impl Default for AgentResourceLimits {
    fn default() -> Self {
        Self {
            memory_mb: 512,
            memory_limit_mb: Some(512),
            cpu_percent: 50.0,
            timeout_seconds: 300,
            max_concurrent_requests: 10,
            storage_mb: 1024,
        }
    }
}

impl Default for AgentSecurity {
    fn default() -> Self {
        Self {
            auth_method: "bearer".to_string(),
            permissions: vec!["read".to_string()],
            security_context: "default".to_string(),
            encryption: EncryptionConfig::default(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES256".to_string(),
            key_size: 256,
            at_rest: true,
            in_transit: true,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/health".to_string(),
            interval_seconds: 30,
            timeout_seconds: 5,
            retries: 3,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let metadata = BiomeMetadata {
            name: "test-biome".to_string(),
            description: "Test biome manifest".to_string(),
            version: "1.0.0".to_string(),
            biomeos_version: "0.1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            author: "test-author".to_string(),
            labels: HashMap::new(),
        };

        let manifest = BiomeManifest {
            metadata,
            agents: Vec::new(),
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
                    network_policies: Vec::new(),
                    pod_security_policies: Vec::new(),
                    rbac_policies: Vec::new(),
                },
            },
            storage: BiomeStorage {
                volumes: Vec::new(),
                volume_claim_templates: Vec::new(),
                storage_classes: Vec::new(),
            },
            networking: BiomeNetworking {
                ingress: IngressConfig {
                    enabled: true,
                    host: "test.example.com".to_string(),
                    tls_enabled: true,
                    annotations: HashMap::new(),
                },
                network_policies: Vec::new(),
                dns: DnsConfig {
                    enabled: true,
                    servers: vec!["8.8.8.8".to_string()],
                    search_domains: vec![],
                },
            },
            primals: HashMap::new(),
        };

        assert_eq!(manifest.metadata.name, "test-biome");
        assert_eq!(manifest.metadata.version, "1.0.0");
        assert!(manifest.security.authentication.enabled);
        assert!(manifest.networking.ingress.enabled);
    }

    #[test]
    fn test_agent_spec_creation() {
        let agent = AgentSpec {
            name: "test-agent".to_string(),
            capabilities: vec!["chat".to_string(), "search".to_string()],
            ai_provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            execution_environment: ExecutionEnvironment::Container,
            resource_limits: AgentResourceLimits::default(),
            resources: AgentResourceLimits::default(),
            manifest: None,
            security: AgentSecurity::default(),
            storage: AgentStorage::default(),
            environment: HashMap::new(),
            config: HashMap::new(),
        };

        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.capabilities.len(), 2);
        assert_eq!(agent.ai_provider, "openai");
        assert_eq!(agent.execution_environment, ExecutionEnvironment::Container);
    }

    #[tokio::test]
    async fn test_manifest_parser() {
        let parser = BiomeManifestParser::new();
        assert!(parser.config.strict_validation);

        let yaml_content = r#"
metadata:
  name: test-biome
  description: Test biome
  version: 1.0.0
  biomeos_version: 0.1.0
  created_at: 2024-01-01T00:00:00Z
  updated_at: 2024-01-01T00:00:00Z
  author: test-author
  labels: {}
agents: []
services: {}
resources:
  limits:
    memory_gb: 8.0
    cpu_cores: 4.0
    storage_gb: 100.0
    network_bandwidth_mbps: 1000.0
  reservations:
    memory_gb: 2.0
    cpu_cores: 1.0
    storage_gb: 10.0
  policies:
    memory_over_commit: false
    cpu_over_commit: true
    storage_over_commit: false
    resource_quotas: {}
security:
  authentication:
    enabled: true
    method: oauth2
    providers: [github]
  authorization:
    enabled: true
    method: rbac
    policies: [default]
  encryption:
    enabled: true
    algorithm: AES256
    key_size: 256
    at_rest: true
    in_transit: true
  tokens:
    enabled: true
    expiration_seconds: 3600
    refresh_enabled: true
  policies:
    network_policies: []
    pod_security_policies: []
    rbac_policies: []
storage:
  volumes: []
  volume_claim_templates: []
  storage_classes: []
networking:
  ingress:
    enabled: true
    host: test.example.com
    tls_enabled: true
    annotations: {}
  network_policies: []
  dns:
    enabled: true
    servers: ["8.8.8.8"]
    search_domains: ["test.local"]
primals: {}
"#;

        let result = parser.parse_content(yaml_content).await;
        if let Err(ref e) = result {
            eprintln!("Manifest parsing error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Failed to parse manifest: {:?}",
            result.as_ref().unwrap_err()
        );

        let manifest = result.unwrap();
        assert_eq!(manifest.metadata.name, "test-biome");
        assert_eq!(manifest.metadata.version, "1.0.0");
    }
}
