// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core manifest data types for biome.yaml.
//!
//! This module defines the structure of biome.yaml manifest files used for
//! agent deployment, service discovery, and resource management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// CORE MANIFEST TYPES
// ============================================================================

/// biome.yaml manifest structure for biomeOS integration.
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

/// Metadata for the biome manifest.
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

/// Agent manifest data.
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

/// Agent deployment specification.
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
// DEFAULT IMPLEMENTATIONS
// ============================================================================

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
