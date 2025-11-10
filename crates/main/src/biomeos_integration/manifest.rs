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
    pub metadata: BiomeMetadata,
    pub agents: Vec<AgentSpec>,
    pub services: HashMap<String, ServiceConfig>,
    pub resources: BiomeResources,
    pub security: BiomeSecurity,
    pub storage: BiomeStorage,
    pub networking: BiomeNetworking,
    pub primals: HashMap<String, PrimalConfig>,
}

/// Metadata for the biome manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub biomeos_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: String,
    pub labels: HashMap<String, String>,
}

// ============================================================================
// AGENT SPECIFICATIONS
// ============================================================================

/// Agent manifest data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>, // Agent capabilities from manifest
    pub dependencies: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Agent deployment specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub capabilities: Vec<String>,
    pub ai_provider: String,
    pub model: String,
    pub execution_environment: ExecutionEnvironment,
    pub resource_limits: AgentResourceLimits,
    pub resources: AgentResourceLimits,  // Alias for compatibility
    pub manifest: Option<AgentManifest>, // Agent manifest data
    pub security: AgentSecurity,
    pub storage: AgentStorage,
    pub environment: HashMap<String, String>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionEnvironment {
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "wasm")]
    Wasm,
    #[serde(rename = "container")]
    Container,
    #[serde(rename = "vm")]
    VirtualMachine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceLimits {
    pub memory_mb: u64,
    #[serde(alias = "memory_limit")]
    pub memory_limit_mb: Option<u64>, // Alias for compatibility, optional
    pub cpu_percent: f64,
    pub timeout_seconds: u64,
    pub max_concurrent_requests: u32,
    pub storage_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSecurity {
    pub auth_method: String,
    pub permissions: Vec<String>,
    pub security_context: String,
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStorage {
    pub persistent: Vec<StorageVolume>,
    pub temporary: Vec<StorageVolume>,
    pub cache: Vec<StorageVolume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVolume {
    pub name: String,
    pub size: String,
    pub tier: String,
    pub provisioner: String,
    pub mount_path: String,
    pub access_mode: String,
}

// ============================================================================
// SERVICE CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub service_type: String,
    pub endpoints: Vec<ServiceEndpoint>,
    pub dependencies: Vec<String>,
    pub load_balancer: LoadBalancerConfig,
    pub health_check: HealthCheckConfig,
    pub scaling: ScalingConfig,
    pub service_mesh: ServiceMeshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub port: u16,
    pub protocol: String,
    pub path: String,
    pub health_check_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub algorithm: String,
    pub health_check: HealthCheckConfig,
    pub session_affinity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub path: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub enabled: bool,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_percent: f64,
    pub target_memory_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshConfig {
    pub enabled: bool,
    pub provider: String,
    pub mtls: MtlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MtlsConfig {
    pub enabled: bool,
    pub ca: String,
    pub rotation_period: String,
}

// ============================================================================
// RESOURCE MANAGEMENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeResources {
    pub limits: ResourceLimits,
    pub reservations: ResourceReservations,
    pub policies: ResourcePolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_gb: f64,
    pub cpu_cores: f64,
    pub storage_gb: f64,
    pub network_bandwidth_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReservations {
    pub memory_gb: f64,
    pub cpu_cores: f64,
    pub storage_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicies {
    pub memory_over_commit: bool,
    pub cpu_over_commit: bool,
    pub storage_over_commit: bool,
    pub resource_quotas: HashMap<String, String>,
}

// ============================================================================
// SECURITY CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSecurity {
    pub authentication: AuthenticationConfig,
    pub authorization: AuthorizationConfig,
    pub encryption: EncryptionConfig,
    pub tokens: TokenConfig,
    pub policies: SecurityPolicies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    pub enabled: bool,
    pub method: String,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    pub enabled: bool,
    pub method: String,
    pub policies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_size: u32,
    pub at_rest: bool,
    pub in_transit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub enabled: bool,
    pub expiration_seconds: u64,
    pub refresh_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicies {
    pub network_policies: Vec<String>,
    pub pod_security_policies: Vec<String>,
    pub rbac_policies: Vec<String>,
}

// ============================================================================
// STORAGE AND NETWORKING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeStorage {
    pub volumes: Vec<StorageVolume>,
    pub volume_claim_templates: Vec<VolumeClaimTemplate>,
    pub storage_classes: Vec<StorageClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeClaimTemplate {
    pub name: String,
    pub size: String,
    pub storage_class: String,
    pub access_modes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClass {
    pub name: String,
    pub provisioner: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeNetworking {
    pub ingress: IngressConfig,
    pub network_policies: Vec<NetworkPolicy>,
    pub dns: DnsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    pub enabled: bool,
    pub host: String,
    pub tls_enabled: bool,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub name: String,
    pub selectors: HashMap<String, String>,
    pub ingress_rules: Vec<NetworkRule>,
    pub egress_rules: Vec<NetworkRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    pub from_selectors: Vec<HashMap<String, String>>,
    pub ports: Vec<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub enabled: bool,
    pub servers: Vec<String>,
    pub search_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    pub primal_type: String,
    pub endpoint: String,
    pub enabled: bool,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

// ============================================================================
// MANIFEST PARSER IMPLEMENTATION
// ============================================================================

#[derive(Debug)]
pub struct BiomeManifestParser {
    pub config: ManifestParserConfig,
}

#[derive(Debug, Clone)]
pub struct ManifestParserConfig {
    pub strict_validation: bool,
    pub schema_validation: bool,
    pub allow_unknown_fields: bool,
    pub default_values: HashMap<String, serde_json::Value>,
}

impl BiomeManifestParser {
    pub fn new() -> Self {
        Self {
            config: ManifestParserConfig::default(),
        }
    }

    pub fn with_config(config: ManifestParserConfig) -> Self {
        Self { config }
    }

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
                    host: std::env::var("UI_HOST")
                        .unwrap_or_else(|_| "localhost".to_string()),
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

    pub async fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<BiomeManifest, PrimalError> {
        let path = path.as_ref();
        info!("Parsing biome.yaml manifest from: {}", path.display());

        let content = fs::read_to_string(path).await.map_err(|e| {
            PrimalError::ConfigError(format!("Failed to read manifest file: {}", e))
        })?;

        self.parse_content(&content).await
    }

    pub async fn parse_content(&self, content: &str) -> Result<BiomeManifest, PrimalError> {
        debug!("Parsing biome.yaml manifest content");

        let mut manifest: BiomeManifest = serde_yaml::from_str(content)
            .map_err(|e| PrimalError::ConfigError(format!("Failed to parse YAML: {}", e)))?;

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
                    "Service '{}' must have at least one endpoint",
                    name
                )));
            }
        }

        Ok(())
    }

    pub fn validate_yaml_schema(&self, content: &str) -> Result<(), PrimalError> {
        // Basic YAML syntax validation
        let _: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| PrimalError::ConfigError(format!("Invalid YAML syntax: {}", e)))?;

        Ok(())
    }

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

impl Default for ExecutionEnvironment {
    fn default() -> Self {
        Self::Native
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

impl Default for AgentStorage {
    fn default() -> Self {
        Self {
            persistent: Vec::new(),
            temporary: Vec::new(),
            cache: Vec::new(),
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
