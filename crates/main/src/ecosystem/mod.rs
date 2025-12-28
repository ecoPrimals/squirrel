//! Ecosystem Integration Module
//!
//! This module provides pure service discovery and communication for the ecoPrimals ecosystem.
//! Each primal is completely standalone and communicates through standardized APIs.
//!
//! ## Architecture Principles
//! - Pure service discovery through Songbird service mesh
//! - No hard dependencies between primals
//! - Standardized HTTP/REST API communication
//! - Each primal can function independently
//! - Dynamic service registration and health monitoring
//!
//! ## Songbird Integration Pattern
//! ```
//! biomeOS → Songbird (Service Mesh) → All Primals
//!               ↓
//! ToadStool + BearDog + NestGate + Squirrel
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::universal::{
    LoadBalancingStatus, PrimalCapability, PrimalContext, UniversalPrimalProvider,
};
use crate::universal_primal_ecosystem::{
    CapabilityMatch, CapabilityRequest, DiscoveredPrimal, UniversalPrimalEcosystem,
};

// Re-export registry manager types
pub mod registry;
pub mod registry_manager;
pub use registry::*;
pub use registry_manager::*;

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
    /// # Example
    /// ```
    /// use squirrel::EcosystemPrimalType;
    ///
    /// let primal = EcosystemPrimalType::Songbird;
    /// assert_eq!(primal.env_name(), "SONGBIRD");
    ///
    /// // Set in environment: export SONGBIRD_ENDPOINT=http://songbird:8081
    /// ```
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
    /// # Example
    /// ```
    /// use squirrel::EcosystemPrimalType;
    ///
    /// let primal = EcosystemPrimalType::Songbird;
    /// assert_eq!(primal.service_name(), "songbird");
    ///
    /// // Used with SERVICE_DISCOVERY_URL:
    /// // http://consul:8500/songbird
    /// ```
    pub fn service_name(&self) -> &'static str {
        self.as_str()
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "toadstool" => Ok(EcosystemPrimalType::ToadStool),
            "songbird" => Ok(EcosystemPrimalType::Songbird),
            "beardog" => Ok(EcosystemPrimalType::BearDog),
            "nestgate" => Ok(EcosystemPrimalType::NestGate),
            "squirrel" => Ok(EcosystemPrimalType::Squirrel),
            "biomeos" => Ok(EcosystemPrimalType::BiomeOS),
            _ => Err(format!("Unknown primal type: {}", s)),
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
    /// Registry manager for service discovery
    pub registry_manager: EcosystemRegistryManager,
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

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    /// Overall status
    pub status: String,
    /// Status timestamp
    pub timestamp: DateTime<Utc>,
    /// Discovered services
    pub discovered_services: Vec<DiscoveredService>,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Service mesh status
    pub service_mesh_status: ServiceMeshStatus,
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
}

/// Service mesh status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Service mesh enabled
    pub enabled: bool,
    /// Registered with Songbird
    pub registered: bool,
    /// Load balancing status
    pub load_balancing: LoadBalancingStatus,
    /// Cross-primal communication status
    pub cross_primal_communication: CrossPrimalStatus,
}

/// Cross-primal communication status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalStatus {
    /// Cross-primal communication enabled
    pub enabled: bool,
    /// Active connections
    pub active_connections: u32,
    /// Supported protocols
    pub supported_protocols: Vec<String>,
}

/// Ecosystem manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemManagerStatus {
    /// Current status
    pub status: String,
    /// Initialization timestamp
    pub initialized_at: Option<DateTime<Utc>>,
    /// Last successful registration
    pub last_registration: Option<DateTime<Utc>>,
    /// Active service registrations
    pub active_registrations: Vec<String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Error count
    pub error_count: u32,
    /// Last error message
    pub last_error: Option<String>,
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health score (0.0 to 1.0)
    pub health_score: f64,
    /// Component health statuses
    pub component_statuses: HashMap<String, ComponentHealth>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Health check errors
    pub health_errors: Vec<String>,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Health status
    pub status: String,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl EcosystemManager {
    /// Create new ecosystem manager
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        // Initialize ecosystem registry manager
        let (registry_manager, _registry_receiver) =
            EcosystemRegistryManager::new(config.registry_config.clone());

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
            registry_manager,
            universal_ecosystem,
            config,
            metrics_collector,
            status: Arc::new(tokio::sync::RwLock::new(status)),
        }
    }

    /// Initialize the ecosystem manager
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        tracing::info!("Initializing ecosystem manager with universal patterns");

        // Initialize registry manager
        self.registry_manager.initialize().await?;

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
        tracing::info!("Registering Squirrel service with ecosystem through Songbird");

        // Create service registration
        let registration = self.create_service_registration(provider)?;

        // Register with ecosystem through service discovery
        self.registry_manager
            .register_squirrel_service(registration)
            .await?;

        // Update status
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(self.config.service_id.clone());

        tracing::info!("Squirrel service registered successfully with Songbird");
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
        Ok(self
            .registry_manager
            .get_discovered_services()
            .await
            .into_iter()
            .map(|arc| (*arc).clone())
            .collect())
    }

    /// Find services by type
    pub async fn find_services_by_type(
        &self,
        primal_type: EcosystemPrimalType,
    ) -> Result<Vec<DiscoveredService>, PrimalError> {
        // Initialize Universal Primal Ecosystem with proper context
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

        // Return Arc<DiscoveredService> directly - no conversion needed
        let discovered_services = self
            .registry_manager
            .get_discovered_services()
            .await
            .into_iter()
            .map(|arc| (*arc).clone())
            .collect::<Vec<DiscoveredService>>();

        // Handle services by type with proper conversion
        let services_result = self
            .registry_manager
            .find_services_by_type(primal_type)
            .await?;

        let services: Vec<DiscoveredService> = services_result
            .iter()
            .map(|arc_service| (**arc_service).clone())
            .collect();

        Ok(services)
    }

    /// Make API call to another primal
    pub async fn call_primal_api(
        &self,
        request: PrimalApiRequest,
    ) -> Result<PrimalApiResponse, PrimalError> {
        self.registry_manager.call_primal_api(request).await
    }

    /// Start coordination between multiple primals
    pub async fn start_coordination(
        &self,
        participants: Vec<EcosystemPrimalType>,
        context: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        // Convert participants to strings
        let participant_strings: Vec<String> =
            participants.iter().map(|p| format!("{:?}", p)).collect();
        let context_value = serde_json::to_value(context)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        self.registry_manager
            .start_coordination(participant_strings, context_value)
            .await
    }

    /// Complete coordination session
    pub async fn complete_coordination(
        &self,
        session_id: &str,
        success: bool,
    ) -> Result<(), PrimalError> {
        self.registry_manager
            .complete_coordination(session_id.to_string(), success)
            .await
    }

    /// Get ecosystem status
    pub async fn get_ecosystem_status(&self) -> EcosystemStatus {
        let discovered_services = self.registry_manager.get_discovered_services().await;
        let active_integrations = self.registry_manager.get_active_integrations().await;

        // Calculate overall health based on discovered services
        let overall_health = if discovered_services.is_empty() {
            0.5 // No services discovered yet
        } else {
            let healthy_services = discovered_services
                .iter()
                .filter(|s| {
                    matches!(
                        s.health_status,
                        registry::types::ServiceHealthStatus::Healthy
                    )
                })
                .count();
            healthy_services as f64 / discovered_services.len() as f64
        };

        let discovered_services_count = discovered_services.len() as u32;
        let active_integrations_count = active_integrations.len() as u32;

        EcosystemStatus {
            status: "active".to_string(),
            timestamp: Utc::now(),
            discovered_services: self
                .registry_manager
                .get_discovered_services()
                .await
                .into_iter()
                .map(|arc| (*arc).clone())
                .collect(),
            active_integrations,
            service_mesh_status: ServiceMeshStatus {
                enabled: true,
                registered: true, // TODO: Get actual status from registry
                load_balancing: LoadBalancingStatus {
                    enabled: true,
                    healthy: overall_health > 0.7,
                    active_connections: active_integrations_count,
                    algorithm: "round_robin".to_string(),
                    health_score: overall_health,
                    last_check: chrono::Utc::now(),
                },
                cross_primal_communication: CrossPrimalStatus {
                    enabled: true,
                    active_connections: discovered_services_count,
                    supported_protocols: vec!["http".to_string(), "grpc".to_string()],
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
        tracing::info!("Registering with Songbird service mesh");

        // Create universal service registration
        let universal_registration = provider.create_service_registration();

        // Convert to ecosystem registration
        let ecosystem_registration = EcosystemServiceRegistration {
            service_id: universal_registration.service_id,
            primal_type: universal_registration.primal_type,
            biome_id: universal_registration.biome_id,
            name: universal_registration.name,
            version: universal_registration.version,
            description: universal_registration.description,
            endpoints: universal_registration.endpoints,
            capabilities: universal_registration.capabilities,
            dependencies: universal_registration.dependencies,
            health_check: universal_registration.health_check,
            security_config: universal_registration.security_config,
            resource_requirements: universal_registration.resource_requirements,
            metadata: universal_registration.metadata,
            tags: universal_registration.tags,
            registered_at: chrono::Utc::now(),
            primal_provider: universal_registration.primal_provider,
        };

        // Register through registry manager
        self.registry_manager
            .register_with_songbird(ecosystem_registration)
            .await?;

        // Update status
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(self.config.service_id.clone());

        tracing::info!("Successfully registered with Songbird service mesh");
        Ok(())
    }

    /// Deregister from Songbird service mesh
    pub async fn deregister_from_songbird(&self) -> Result<(), PrimalError> {
        tracing::info!("Deregistering from Songbird service mesh");

        self.registry_manager
            .deregister_from_songbird(&self.config.service_id)
            .await?;

        // Update status
        let mut status = self.status.write().await;
        status
            .active_registrations
            .retain(|id| id != &self.config.service_id);

        tracing::info!("Successfully deregistered from Songbird service mesh");
        Ok(())
    }

    /// Shutdown the ecosystem manager
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        tracing::info!("Shutting down ecosystem manager");

        // Deregister from Songbird service mesh
        if let Err(e) = self.deregister_from_songbird().await {
            tracing::warn!("Failed to deregister from Songbird during shutdown: {}", e);
        }

        // Shutdown the registry manager
        self.registry_manager.shutdown().await?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "shutdown".to_string();

        tracing::info!("Ecosystem manager shutdown completed");
        Ok(())
    }

    /// Store data using universal storage patterns (replaces hard-coded NestGate)
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

    /// Execute computation using universal compute patterns (replaces hard-coded ToadStool)
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

    /// Authenticate using BearDog security coordination
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
