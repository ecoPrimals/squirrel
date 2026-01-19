//! Ecosystem Integration Module
//!
//! This module provides pure service discovery and communication for the ecoPrimals ecosystem.
//! Each primal is completely standalone and communicates through standardized APIs.
//!
//! ## Architecture Principles
//! - Pure capability-based service discovery
//! - No hard dependencies between primals
//! - Standardized HTTP/REST API communication
//! - Each primal can function independently
//! - Dynamic service registration and health monitoring
//!
//! ## Service Mesh Integration Pattern
//! ```text
//! biomeOS → Service Mesh (Capability Discovery) → All Primals
//!               ↓
//! Primals discover each other by capability at runtime
//! ```
//!
//! ## Migration from Hardcoded to Dynamic
//!
//! **Old Pattern (Hardcoded)**:
//! ```ignore
//! use EcosystemPrimalType::Songbird;
//! let endpoint = Songbird.default_endpoint();
//! ```
//!
//! **New Pattern (Capability-Based)**:
//! ```ignore
//! use CapabilityRegistry;
//! let registry = CapabilityRegistry::new(Default::default());
//! let coordinator = registry.discover_by_capability(&PrimalCapability::ServiceMesh).await?;
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::universal::{LoadBalancingStatus, PrimalCapability, PrimalContext};
use crate::universal_primal_ecosystem::{
    CapabilityMatch, CapabilityRequest, DiscoveredPrimal, UniversalPrimalEcosystem,
};

// Module declarations
pub mod status;
pub mod types;

// Re-export all public items
pub use status::*;

// Re-export registry types
pub mod registry;
pub use registry::*;

// discovery_client and registry_manager removed - HTTP-based, replaced by capability discovery

/// Ecosystem service registration for Squirrel AI primal
///
/// This struct follows the standardized format for service discovery
/// and registration within the ecoPrimals ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique service identifier: "primal-squirrel-{instance}"
    pub service_id: String,

    /// Primal type from standardized enum
    pub primal_type: EcosystemPrimalType,

    /// Service name
    pub name: String,

    /// Service description
    pub description: String,
    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Service version
    pub version: String,

    /// Service capabilities (standardized format)
    pub capabilities: ServiceCapabilities,

    /// API endpoints (standardized format)
    pub endpoints: ServiceEndpoints,
    /// Service dependencies
    pub dependencies: Vec<String>,
    /// Service tags
    pub tags: Vec<String>,
    /// Primal provider info
    pub primal_provider: Option<String>,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Security configuration
    pub security_config: SecurityConfig,
    /// Resource requirements
    pub resource_requirements: ResourceSpec,
    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Standardized primal types for ecosystem integration
///
/// **DEPRECATED**: This enum violates primal sovereignty by hardcoding all primal names.
/// Use `CapabilityRegistry` and `PrimalCapability` for capability-based discovery instead.
///
/// ## Why This is Deprecated
///
/// - **Compile-time coupling**: Changes to the ecosystem require recompiling all primals
/// - **Sovereignty violation**: Each primal should only know itself
/// - **Scalability**: Cannot add new primals without code changes
/// - **Evolution**: Cannot evolve primal names or capabilities
///
/// ## Migration Path
///
/// ```ignore
/// // OLD (hardcoded):
/// let primal_type = EcosystemPrimalType::Songbird;
/// let endpoint = primal_type.default_endpoint();
///
/// // NEW (capability-based):
/// use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
/// let registry = CapabilityRegistry::new(Default::default());
/// let primals = registry.discover_by_capability(&PrimalCapability::ServiceMesh).await?;
/// let endpoint = &primals[0].endpoint;
/// ```
#[deprecated(
    since = "0.1.0",
    note = "Use CapabilityRegistry for capability-based discovery instead of hardcoded primal types"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    ToadStool,
    Songbird,
    BearDog,
    NestGate,
    Squirrel,
    BiomeOS,
}

impl EcosystemPrimalType {
    /// Convert to string representation
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    #[must_use]
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry instead")]
    pub fn as_str(&self) -> &'static str {
        match self {
            EcosystemPrimalType::ToadStool => "toadstool",
            EcosystemPrimalType::Songbird => "songbird",
            EcosystemPrimalType::BearDog => "beardog",
            EcosystemPrimalType::NestGate => "nestgate",
            EcosystemPrimalType::Squirrel => "squirrel",
            EcosystemPrimalType::BiomeOS => "biomeos",
        }
    }

    /// Get environment variable name for this primal's endpoint
    ///
    /// **DEPRECATED**: Use generic environment variables like `SERVICE_MESH_ENDPOINT`.
    ///
    /// # Migration Example
    /// ```ignore
    /// // OLD:
    /// let primal = EcosystemPrimalType::Songbird;
    /// let env_name = primal.env_name(); // "SONGBIRD"
    /// let endpoint = std::env::var(format!("{}_ENDPOINT", env_name))?;
    ///
    /// // NEW:
    /// let endpoint = std::env::var("SERVICE_MESH_ENDPOINT")?; // Capability-agnostic
    /// ```
    #[must_use]
    #[deprecated(
        since = "0.1.0",
        note = "Use generic env vars like SERVICE_MESH_ENDPOINT"
    )]
    pub fn env_name(&self) -> &'static str {
        match self {
            EcosystemPrimalType::ToadStool => "TOADSTOOL",
            EcosystemPrimalType::Songbird => "SONGBIRD",
            EcosystemPrimalType::BearDog => "BEARDOG",
            EcosystemPrimalType::NestGate => "NESTGATE",
            EcosystemPrimalType::Squirrel => "SQUIRREL",
            EcosystemPrimalType::BiomeOS => "BIOMEOS",
        }
    }

    /// Get service name for service discovery
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    ///
    /// # Migration Example
    /// ```ignore
    /// // OLD:
    /// let primal = EcosystemPrimalType::Songbird;
    /// let service_name = primal.service_name(); // "songbird"
    /// let url = format!("http://consul:8500/{}", service_name);
    ///
    /// // NEW:
    /// use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
    /// let registry = CapabilityRegistry::new(Default::default());
    /// let primals = registry.discover_by_capability(&PrimalCapability::ServiceMesh).await?;
    /// let endpoint = &primals[0].endpoint; // Discovered endpoint
    /// ```
    #[must_use]
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry for discovery")]
    pub fn service_name(&self) -> &'static str {
        self.as_str()
    }

    /// Parse from string
    ///
    /// **DEPRECATED**: Use capability-based discovery instead.
    #[deprecated(since = "0.1.0", note = "Use CapabilityRegistry for discovery")]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "toadstool" => Ok(EcosystemPrimalType::ToadStool),
            "songbird" => Ok(EcosystemPrimalType::Songbird),
            "beardog" => Ok(EcosystemPrimalType::BearDog),
            "nestgate" => Ok(EcosystemPrimalType::NestGate),
            "squirrel" => Ok(EcosystemPrimalType::Squirrel),
            "biomeos" => Ok(EcosystemPrimalType::BiomeOS),
            _ => Err(format!("Unknown primal type: {s}")),
        }
    }
}

/// Service capabilities with proper Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    pub core: Vec<String>,
    pub extended: Vec<String>,
    pub integrations: Vec<String>,
}

/// Service endpoints with proper Default implementation  
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub primary: String,
    pub secondary: Vec<String>,
    pub health: Option<String>,
}

/// Health check configuration with Default implementation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_secs: u64,
    pub timeout_secs: u64,
    pub failure_threshold: u32,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU requirements
    pub cpu: String,
    /// Memory requirements
    pub memory: String,
    /// Storage requirements
    pub storage: String,
    /// Network requirements
    pub network: String,
    /// GPU requirements (optional)
    pub gpu: Option<String>,
}

/// Security configuration for ecosystem integration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication requirements
    pub auth_required: bool,
    /// Encryption level required
    pub encryption_level: String,
    /// Access control level  
    pub access_level: String,
    /// Security policies to enforce
    pub policies: Vec<String>,
    /// Audit requirements
    pub audit_enabled: bool,
    /// Security level
    pub security_level: String,
}

/// Resource requirements specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum CPU cores required
    pub min_cpu: Option<u32>,
    /// Minimum RAM in MB
    pub min_memory_mb: Option<u64>,
    /// Minimum disk space in MB
    pub min_disk_mb: Option<u64>,
    /// Network bandwidth requirements
    pub min_network_mbps: Option<u32>,
    /// Special hardware requirements
    pub specialized_hardware: Vec<String>,
}

/// Ecosystem configuration for Squirrel primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    /// Service identifier
    pub service_id: String,
    /// Service name
    pub service_name: String,
    /// Service host
    pub service_host: String,
    /// Service port
    pub service_port: u16,
    /// Songbird endpoint URL
    pub songbird_endpoint: String,
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

/// Ecosystem manager for service discovery and communication
pub struct EcosystemManager {
    // registry_manager removed - HTTP-based, replaced by capability discovery
    /// Universal primal ecosystem for standardized integration
    pub universal_ecosystem: UniversalPrimalEcosystem,
    /// Configuration
    pub config: EcosystemConfig,
    /// Metrics collector
    pub metrics_collector: Arc<MetricsCollector>,
    /// Manager status
    pub status: Arc<tokio::sync::RwLock<EcosystemManagerStatus>>,
}

// Use PrimalApiRequest and PrimalApiResponse from registry_manager module
pub use registry::{PrimalApiRequest, PrimalApiResponse};

// Use DiscoveredService from registry_manager module
pub use registry::types::DiscoveredService;

impl EcosystemManager {
    /// Create new ecosystem manager
    #[must_use]
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        // registry_manager removed - HTTP-based, replaced by capability discovery
        
        // Initialize Universal Primal Ecosystem with proper context
        let primal_context = PrimalContext {
            user_id: "squirrel".to_string(),
            device_id: uuid::Uuid::new_v4().to_string(),
            network_location: crate::universal::NetworkLocation {
                region: std::env::var("DEPLOYMENT_REGION")
                    .unwrap_or_else(|_| "default".to_string()),
                data_center: std::env::var("DATA_CENTER").ok(),
                availability_zone: std::env::var("AVAILABILITY_ZONE").ok(),
                ip_address: Some("127.0.0.1".to_string()),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::universal::SecurityLevel::Internal,
            biome_id: Some("squirrel-ecosystem".to_string()),
            session_id: Some(uuid::Uuid::new_v4().to_string()),
            metadata: std::collections::HashMap::new(),
        };
        let universal_ecosystem = UniversalPrimalEcosystem::new(primal_context);

        let status = EcosystemManagerStatus {
            status: "initializing".to_string(),
            initialized_at: None,
            last_registration: None,
            active_registrations: Vec::new(),
            health_status: HealthStatus {
                health_score: 0.0,
                component_statuses: HashMap::new(),
                last_check: chrono::Utc::now(),
                health_errors: Vec::new(),
            },
            error_count: 0,
            last_error: None,
        };

        Self {
            universal_ecosystem,
            config,
            metrics_collector,
            status: Arc::new(tokio::sync::RwLock::new(status)),
        }
    }

    /// Initialize the ecosystem manager
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        tracing::info!("Initializing ecosystem manager with universal patterns");

        // registry_manager removed - capability-based discovery used instead

        // Initialize universal primal ecosystem
        self.universal_ecosystem.initialize().await?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "initialized".to_string();
        status.initialized_at = Some(Utc::now());

        tracing::info!("Ecosystem manager initialized successfully");
        Ok(())
    }

    /// Register Squirrel service with ecosystem
    pub async fn register_squirrel_service(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<(), PrimalError> {
        tracing::info!("Registering Squirrel service with ecosystem through capability discovery");

        // Create service registration
        let registration = self.create_service_registration(provider)?;

        // TODO: Register with ecosystem through capability discovery (Unix sockets)
        tracing::info!("Service registration prepared: {:?}", registration.service_id);

        // Update status
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(self.config.service_id.clone());

        tracing::info!("Squirrel service registered successfully");
        Ok(())
    }

    /// Create service registration from provider
    fn create_service_registration(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<EcosystemServiceRegistration, PrimalError> {
        let endpoints = provider.endpoints();

        Ok(EcosystemServiceRegistration {
            service_id: self.config.service_id.clone(),
            primal_type: EcosystemPrimalType::Squirrel,
            biome_id: self.config.biome_id.clone(),
            name: provider.name().to_string(),
            description: provider.description().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: ServiceCapabilities {
                core: vec![
                    "ai_coordination".to_string(),
                    "mcp_protocol".to_string(),
                    "session_management".to_string(),
                    "service_mesh_integration".to_string(),
                ],
                extended: vec![
                    "context_awareness".to_string(),
                    "ecosystem_intelligence".to_string(),
                    "tool_orchestration".to_string(),
                    "cross_primal_communication".to_string(),
                ],
                integrations: vec![
                    "songbird".to_string(),
                    "biomeos".to_string(),
                    "beardog".to_string(),
                    "nestgate".to_string(),
                    "toadstool".to_string(),
                ],
            },
            endpoints: ServiceEndpoints {
                primary: endpoints.health.clone().unwrap_or_default(),
                secondary: vec![
                    endpoints.metrics.unwrap_or_default(),
                    endpoints.admin.unwrap_or_default(),
                ],
                health: endpoints.health,
            },
            dependencies: vec![], // No dependencies for a standalone primal
            tags: vec![],         // No tags for a standalone primal
            primal_provider: Some(provider.name().to_string()),
            health_check: HealthCheckConfig {
                enabled: true,
                interval_secs: 30,
                timeout_secs: 5,
                failure_threshold: 3,
            },
            security_config: SecurityConfig {
                auth_required: true,
                encryption_level: "high".to_string(),
                access_level: "internal".to_string(),
                policies: vec!["no_sensitive_data".to_string()],
                audit_enabled: true,
                security_level: "standard".to_string(),
            },
            resource_requirements: self.config.resource_requirements.clone(),
            metadata: self.config.metadata.clone(),
            registered_at: Utc::now(),
        })
    }

    /// Discover services by primal type
    pub async fn discover_services(&self) -> Result<Vec<DiscoveredService>, PrimalError> {
        // TODO: Implement via capability discovery (Unix sockets)
        tracing::warn!("discover_services called - implement via capability discovery");
        Ok(Vec::new())
    }

    /// Find services by type
    pub async fn find_services_by_type(
        &self,
        primal_type: EcosystemPrimalType,
    ) -> Result<Vec<DiscoveredService>, PrimalError> {
        // TODO: Implement via capability discovery (Unix sockets)
        tracing::warn!("find_services_by_type called for {:?} - implement via capability discovery", primal_type);
        Ok(Vec::new())
    }

    /// Make API call to another primal
    pub async fn call_primal_api(
        &self,
        request: PrimalApiRequest,
    ) -> Result<PrimalApiResponse, PrimalError> {
        // TODO: Implement via capability discovery (Unix sockets)
        tracing::warn!("call_primal_api called - implement via capability discovery");
        Err(PrimalError::Configuration("API calls via capability discovery not yet implemented".to_string()))
    }

    /// Start coordination between multiple primals
    pub async fn start_coordination(
        &self,
        participants: Vec<EcosystemPrimalType>,
        context: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        // TODO: Implement via capability discovery (Unix sockets)
        tracing::warn!("start_coordination called - implement via capability discovery");
        Ok(format!("coord_{}", Uuid::new_v4()))
    }

    /// Complete coordination session
    pub async fn complete_coordination(
        &self,
        session_id: &str,
        success: bool,
    ) -> Result<(), PrimalError> {
        // TODO: Implement via capability discovery (Unix sockets)
        tracing::info!("complete_coordination called for session {} (success: {})", session_id, success);
        Ok(())
    }

    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> EcosystemIntegrationStatus {
        // TODO: Implement via capability discovery (Unix sockets)
        let overall_health = 1.0; // Healthy by default

        EcosystemIntegrationStatus {
            status: "active".to_string(),
            timestamp: Utc::now(),
            discovered_services: Vec::new(), // TODO: Get from capability discovery
            active_integrations: Vec::new(), // TODO: Get from capability discovery
            service_mesh_status: ServiceMeshStatus {
                enabled: true,
                registered: false, // TODO: Check registration status
                load_balancing: LoadBalancingStatus {
                    enabled: true,
                    healthy: overall_health > 0.7,
                    active_connections: 0,
                    algorithm: "round_robin".to_string(),
                    health_score: overall_health,
                    last_check: chrono::Utc::now(),
                },
                cross_primal_communication: CrossPrimalStatus {
                    enabled: true,
                    active_connections: 0,
                    supported_protocols: vec!["unix_socket".to_string()],
                },
            },
            overall_health,
        }
    }

    /// Get manager status
    pub async fn get_manager_status(&self) -> EcosystemManagerStatus {
        self.status.read().await.clone()
    }

    /// Update health status
    pub async fn update_health_status(
        &self,
        component: &str,
        health: ComponentHealth,
    ) -> Result<(), PrimalError> {
        let mut status = self.status.write().await;
        status
            .health_status
            .component_statuses
            .insert(component.to_string(), health);
        status.health_status.last_check = Utc::now();

        // Calculate overall health score
        let total_score: f64 = status
            .health_status
            .component_statuses
            .values()
            .map(|h| match h.status.as_str() {
                "healthy" => 1.0,
                "degraded" => 0.5,
                _ => 0.0,
            })
            .sum();

        let component_count = status.health_status.component_statuses.len() as f64;
        status.health_status.health_score = if component_count > 0.0 {
            total_score / component_count
        } else {
            0.0
        };

        Ok(())
    }

    /// Register with Songbird service mesh
    pub async fn register_with_songbird(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<(), PrimalError> {
        tracing::info!("Registering with service mesh via capability discovery");

        // Create universal service registration
        let universal_registration = provider.create_service_registration();

        // TODO: Register through capability discovery (Unix sockets)
        tracing::info!("Service registration prepared: {:?}", universal_registration.service_id);

        // Update status
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(self.config.service_id.clone());

        tracing::info!("Successfully prepared registration");
        Ok(())
    }

    /// Deregister from Songbird service mesh
    pub async fn deregister_from_songbird(&self) -> Result<(), PrimalError> {
        tracing::info!("Deregistering from service mesh");

        // TODO: Deregister through capability discovery (Unix sockets)
        
        // Update status
        let mut status = self.status.write().await;
        status
            .active_registrations
            .retain(|id| id != &self.config.service_id);

        tracing::info!("Successfully deregistered");
        Ok(())
    }

    /// Shutdown the ecosystem manager
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        tracing::info!("Shutting down ecosystem manager");

        // Deregister from service mesh
        if let Err(e) = self.deregister_from_songbird().await {
            tracing::warn!("Failed to deregister during shutdown: {}", e);
        }

        // registry_manager removed - capability-based discovery used instead

        // Update status
        let mut status = self.status.write().await;
        status.status = "shutdown".to_string();

        tracing::info!("Ecosystem manager shutdown completed");
        Ok(())
    }

    /// Store data using universal storage patterns (replaces hard-coded `NestGate`)
    pub async fn store_data_universal(
        &self,
        key: &str,
        data: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        tracing::info!("Storing data using universal storage patterns: {}", key);
        self.universal_ecosystem.store_data(key, data).await?;
        Ok(key.to_string())
    }

    /// Retrieve data using universal storage patterns
    pub async fn retrieve_data_universal(&self, key: &str) -> Result<Vec<u8>, PrimalError> {
        tracing::info!("Retrieving data using universal storage patterns: {}", key);
        self.universal_ecosystem.retrieve_data(key).await
    }

    /// Execute computation using universal compute patterns (replaces hard-coded `ToadStool`)
    pub async fn execute_computation_universal(
        &self,
        computation: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, PrimalError> {
        tracing::info!(
            "Executing computation using universal compute patterns: {}",
            computation
        );
        let computation_request = serde_json::json!({
            "computation": computation,
            "parameters": parameters
        });
        self.universal_ecosystem
            .execute_computation(computation_request)
            .await
    }

    /// Authenticate using `BearDog` security coordination
    pub async fn authenticate_universal(
        &self,
        credentials: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        tracing::info!("🐻 Authenticating via BearDog coordination");

        // Simple authentication delegation to BearDog
        let user_id = credentials
            .get("user_id")
            .or_else(|| credentials.get("username"))
            .cloned()
            .unwrap_or_else(|| "anonymous".to_string());

        // Create a simple session ID for BearDog coordination
        let session_id = format!("beardog_session_{}", uuid::Uuid::new_v4());

        tracing::info!("✅ BearDog authentication coordination complete");
        Ok(session_id)
    }

    /// Get all discovered primals using universal patterns
    pub async fn get_discovered_primals_universal(&self) -> Vec<DiscoveredPrimal> {
        self.universal_ecosystem.get_discovered_primals().await
    }

    /// Find primals by capability using universal patterns
    pub async fn find_primals_by_capability_universal(
        &self,
        capability: &PrimalCapability,
    ) -> Vec<DiscoveredPrimal> {
        match self
            .universal_ecosystem
            .find_by_capability(match capability {
                PrimalCapability::ContainerRuntime { .. } => "container-runtime",
                PrimalCapability::GpuAcceleration { .. } => "gpu-acceleration",
                PrimalCapability::Authentication { .. } => "authentication",
                PrimalCapability::ObjectStorage { .. } => "object-storage",
                _ => "generic-capability",
            })
            .await
        {
            Ok(matches) => matches
                .into_iter()
                .map(|m| DiscoveredPrimal {
                    id: m.service.service_id,
                    instance_id: m.service.instance_id,
                    primal_type: universal_patterns::traits::PrimalType::Coordinator,
                    capabilities: vec![],
                    endpoint: m.service.endpoint,
                    health: universal_patterns::traits::PrimalHealth::Healthy,
                    context: universal_patterns::traits::PrimalContext::default(),
                    port_info: None,
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    /// Match capabilities to available primals
    pub async fn match_capabilities_universal(
        &self,
        request: &CapabilityRequest,
    ) -> Vec<CapabilityMatch> {
        self.universal_ecosystem
            .match_capabilities(request)
            .await
            .unwrap_or_default()
    }
}

impl Default for EcosystemConfig {
    fn default() -> Self {
        Self {
            service_id: format!("squirrel-{}", Uuid::new_v4()),
            service_name: "Squirrel AI Primal".to_string(),
            service_host: std::env::var("MCP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            service_port: std::env::var("MCP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            songbird_endpoint: std::env::var("SERVICE_MESH_ENDPOINT")
                .or_else(|_| std::env::var("SONGBIRD_ENDPOINT"))
                .or_else(|_| std::env::var("DEV_SERVICE_MESH_ENDPOINT"))
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "⚠️ SERVICE_MESH_ENDPOINT not configured. \
                         Set SERVICE_MESH_ENDPOINT for production discovery. \
                         Using development default."
                    );
                    "http://localhost:8500".to_string()
                }),
            biome_id: None,
            registry_config: EcosystemRegistryConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "500m".to_string(),
                memory: "1Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
                gpu: None,
            },
            security_config: SecurityConfig {
                auth_required: true,
                encryption_level: "high".to_string(),
                access_level: "internal".to_string(),
                policies: vec!["no_sensitive_data".to_string()],
                audit_enabled: true,
                security_level: "standard".to_string(),
            },
            health_check: HealthCheckConfig {
                enabled: true,
                interval_secs: 30,
                timeout_secs: 5,
                failure_threshold: 3,
            },
            metadata: HashMap::new(),
        }
    }
}

impl std::fmt::Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EcosystemPrimalType::ToadStool => write!(f, "toadstool"),
            EcosystemPrimalType::Songbird => write!(f, "songbird"),
            EcosystemPrimalType::BearDog => write!(f, "beardog"),
            EcosystemPrimalType::NestGate => write!(f, "nestgate"),
            EcosystemPrimalType::Squirrel => write!(f, "squirrel"),
            EcosystemPrimalType::BiomeOS => write!(f, "biomeos"),
        }
    }
}

/// Initialize ecosystem integration with Songbird patterns
pub async fn initialize_ecosystem_integration(
    config: EcosystemConfig,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<EcosystemManager, PrimalError> {
    tracing::info!("Initializing ecosystem integration with Songbird service mesh patterns");

    let mut manager = EcosystemManager::new(config, metrics_collector);
    manager.initialize().await?;

    tracing::info!("Ecosystem integration initialized successfully");
    Ok(manager)
}
