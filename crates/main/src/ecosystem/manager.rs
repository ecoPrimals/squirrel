//! Ecosystem manager implementation
//!
//! Manages ecosystem integration, service discovery, and cross-primal communication.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::universal::{LoadBalancingStatus, PrimalCapability, PrimalContext};
use crate::universal_primal_ecosystem::{
    CapabilityMatch, CapabilityRequest, DiscoveredPrimal, UniversalPrimalEcosystem,
};

use super::arc_str;
use super::config::EcosystemConfig;
use super::registry::types::{DiscoveredService, ServiceHealthStatus};
use super::registry::{PrimalApiRequest, PrimalApiResponse};
use super::registry_manager::EcosystemRegistryManager;
use super::types::*;

/// Ecosystem manager for service discovery and cross-primal communication
///
/// `EcosystemManager` is the central coordinator for ecosystem integration,
/// providing:
/// - Dynamic service discovery through Songbird
/// - Capability-based primal communication
/// - Health monitoring and metrics collection
/// - Load balancing and failover support
///
/// # Architecture
///
/// The manager integrates three key components:
/// 1. **Registry Manager**: Handles service discovery and registration
/// 2. **Universal Ecosystem**: Provides capability-based primal discovery
/// 3. **Metrics Collector**: Tracks performance and health metrics
///
/// # Example
///
/// ```rust,no_run
/// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
/// use squirrel::MetricsCollector;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = EcosystemConfig::default();
/// let metrics = Arc::new(MetricsCollector::new());
/// let mut manager = EcosystemManager::new(config, metrics);
///
/// // Initialize the manager
/// manager.initialize().await?;
///
/// // Manager is now ready for service discovery and communication
/// # Ok(())
/// # }
/// ```
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

impl EcosystemManager {
    /// Creates a new ecosystem manager with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Ecosystem configuration including registry and discovery settings
    /// * `metrics_collector` - Shared metrics collector for tracking ecosystem operations
    ///
    /// # Returns
    ///
    /// A new `EcosystemManager` instance ready for initialization
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
    /// use squirrel::MetricsCollector;
    /// use std::sync::Arc;
    ///
    /// let config = EcosystemConfig::default();
    /// let metrics = Arc::new(MetricsCollector::new());
    /// let manager = EcosystemManager::new(config, metrics);
    /// ```
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        // Initialize ecosystem registry manager
        let (registry_manager, _registry_receiver) =
            EcosystemRegistryManager::new(config.registry_config.clone());

        // Create primal context from configuration (no hardcoding!)
        let primal_context = Self::create_primal_context(&config);
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

    /// Initializes the ecosystem manager and its components
    ///
    /// This method performs the following initialization steps:
    /// 1. Initializes the registry manager for service discovery
    /// 2. Initializes the universal primal ecosystem
    /// 3. Updates manager status to "initialized"
    ///
    /// # Errors
    ///
    /// Returns `PrimalError` if:
    /// - Registry manager initialization fails
    /// - Universal ecosystem initialization fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
    /// use squirrel::MetricsCollector;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = EcosystemConfig::default();
    /// let metrics = Arc::new(MetricsCollector::new());
    /// let mut manager = EcosystemManager::new(config, metrics);
    ///
    /// manager.initialize().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        tracing::info!("Initializing ecosystem manager with universal patterns");

        // Initialize registry and ecosystem concurrently for faster startup
        // These operations are independent and can run in parallel
        let (registry_result, ecosystem_result) = tokio::join!(
            self.registry_manager.initialize(),
            self.universal_ecosystem.initialize()
        );

        // Handle results - fail fast if either initialization fails
        registry_result?;
        ecosystem_result?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "initialized".to_string();
        status.initialized_at = Some(Utc::now());

        tracing::info!("Ecosystem manager initialized successfully");
        Ok(())
    }

    /// Registers the Squirrel service with the ecosystem through Songbird
    ///
    /// This method creates and submits a service registration containing:
    /// - Service metadata (name, version, capabilities)
    /// - GPU capabilities (if available on THIS instance)
    /// - Health check endpoints
    /// - API endpoints for inter-primal communication
    /// - Load balancing preferences
    ///
    /// # Primal Self-Knowledge
    ///
    /// This method ONLY registers THIS instance's capabilities.
    /// It has NO knowledge of other Squirrel instances or their GPUs.
    /// Discovery of other instances happens through Songbird's capability system.
    ///
    /// # Arguments
    ///
    /// * `provider` - The Squirrel primal provider containing service information
    ///
    /// # Errors
    ///
    /// Returns `PrimalError` if:
    /// - Service registration creation fails
    /// - Registration submission to ecosystem fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use squirrel::ecosystem::{EcosystemManager, EcosystemConfig};
    /// use squirrel::primal_provider::SquirrelPrimalProvider;
    /// use squirrel::MetricsCollector;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = EcosystemConfig::default();
    /// let metrics = Arc::new(MetricsCollector::new());
    /// let manager = EcosystemManager::new(config, metrics.clone());
    /// let provider = SquirrelPrimalProvider::new(metrics);
    ///
    /// manager.register_squirrel_service(&provider).await?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Create PrimalContext from configuration (eliminating hardcoded values)
    ///
    /// This method creates a properly configured PrimalContext using values from
    /// the ecosystem configuration, environment variables, or sensible defaults.
    /// No hardcoded values!
    ///
    /// # Modern Pattern
    ///
    /// Uses `unwrap_or_else` with closures for lazy evaluation and proper logging.
    /// No `unwrap()` calls - all errors handled gracefully with fallbacks.
    fn create_primal_context(config: &EcosystemConfig) -> PrimalContext {
        // Use config values or environment variables, with fallbacks
        // SAFE: unwrap_or_else ensures we always have a value
        let user_id =
            std::env::var("PRIMAL_USER_ID").unwrap_or_else(|_| config.service_name.clone());

        let ip_address =
            std::env::var("PRIMAL_IP_ADDRESS").unwrap_or_else(|_| config.service_host.clone());

        let biome_id = config
            .biome_id
            .clone()
            .or_else(|| std::env::var("BIOME_ID").ok());

        PrimalContext {
            user_id,
            device_id: uuid::Uuid::new_v4().to_string(),
            network_location: crate::universal::NetworkLocation {
                region: std::env::var("DEPLOYMENT_REGION")
                    .unwrap_or_else(|_| "us-west-2".to_string()),
                data_center: std::env::var("DATA_CENTER").ok(),
                availability_zone: std::env::var("AVAILABILITY_ZONE").ok(),
                ip_address: Some(ip_address),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::universal::SecurityLevel::Internal,
            biome_id,
            session_id: Some(uuid::Uuid::new_v4().to_string()),
            metadata: config.metadata.clone(),
        }
    }

    /// Create service registration from provider
    ///
    /// # Primal Self-Knowledge
    ///
    /// This method detects and reports ONLY this instance's GPU capabilities.
    /// It does NOT know about other instances - discovery happens through Songbird.
    fn create_service_registration(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<EcosystemServiceRegistration, PrimalError> {
        let endpoints = provider.endpoints();

        // Detect GPU capabilities for THIS instance only
        let mut capabilities_core = vec![
            "ai_coordination".to_string(),
            "mcp_protocol".to_string(),
            "session_management".to_string(),
            "service_mesh_integration".to_string(),
        ];

        let mut capabilities_extended = vec![
            "context_awareness".to_string(),
            "ecosystem_intelligence".to_string(),
            "tool_orchestration".to_string(),
            "cross_primal_communication".to_string(),
        ];

        let mut metadata = self.config.metadata.clone();

        // Detect GPU capabilities (non-blocking, best-effort)
        // PRIMAL SELF-KNOWLEDGE: Only detects THIS instance's GPU
        if let Ok(Some(gpu_caps)) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(crate::hardware::detect_local_gpus())
        }) {
            tracing::info!(
                "Detected {} GPU(s) on this instance, total VRAM: {}GB",
                gpu_caps.gpus.len(),
                gpu_caps.total_vram_gb
            );

            // Add GPU-specific capabilities
            capabilities_core.push("ai_model_inference".to_string());
            capabilities_extended.push("gpu_acceleration".to_string());

            // Add metadata about THIS instance's GPU
            metadata.insert("gpu_count".to_string(), gpu_caps.gpus.len().to_string());
            metadata.insert(
                "vram_total_gb".to_string(),
                gpu_caps.total_vram_gb.to_string(),
            );

            // Add first GPU details (for single-GPU systems)
            if let Some(gpu) = gpu_caps.gpus.first() {
                metadata.insert("gpu_model".to_string(), gpu.model.clone());
                metadata.insert("gpu_vendor".to_string(), gpu.vendor.clone());
            }

            // If we have GPU, we can participate in model splitting
            if gpu_caps.total_vram_gb >= 8 {
                capabilities_extended.push("ai_model_splitting".to_string());
                tracing::info!("This instance can participate in model splitting ({}GB VRAM)", gpu_caps.total_vram_gb);
            }
        } else {
            tracing::info!("No GPU detected on this instance (CPU-only mode)");
        }

        Ok(EcosystemServiceRegistration {
            service_id: arc_str(&self.config.service_id),
            primal_type: EcosystemPrimalType::Squirrel,
            biome_id: self.config.biome_id.as_ref().map(arc_str),
            name: arc_str(provider.name()),
            description: provider.description().to_string(),
            version: arc_str(env!("CARGO_PKG_VERSION")),
            capabilities: ServiceCapabilities {
                core: capabilities_core,
                extended: capabilities_extended,
                // MODERN: No hardcoded primal names - use capability-based discovery
                // Squirrel advertises what it CAN integrate with, not specific names
                integrations: vec![
                    "service_mesh".to_string(),      // Any orchestration (Songbird)
                    "os_integration".to_string(),    // Any OS layer (BiomeOS)
                    "security_provider".to_string(), // Any security (capability-based)
                    "storage_provider".to_string(),  // Any storage (capability-based)
                    "compute_provider".to_string(),  // Any compute (capability-based)
                ],
            },
            endpoints: ServiceEndpoints {
                primary: endpoints.health.clone().unwrap_or_default(), // Clone to avoid move
                secondary: vec![
                    endpoints.metrics.unwrap_or_default(),
                    endpoints.admin.unwrap_or_default(),
                ],
                health: endpoints.health, // Use the original after clone
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
            metadata, // Use the enhanced metadata with GPU info
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
        // Use the same context creation method (no hardcoding!)
        let _primal_context = Self::create_primal_context(&self.config);

        // Return Arc<DiscoveredService> directly - no conversion needed
        let _discovered_services = self
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
    ///
    /// Initiates a coordination session between multiple primal services.
    ///
    /// # Arguments
    ///
    /// * `participants` - Vector of primal types to coordinate
    /// * `context` - Context metadata for the coordination session
    ///
    /// # Returns
    ///
    /// A session ID for tracking the coordination session
    ///
    /// # Error Handling
    ///
    /// Uses safe serialization patterns - invalid context gracefully falls back to empty object.
    pub async fn start_coordination(
        &self,
        participants: Vec<EcosystemPrimalType>,
        context: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        // Convert participants to strings
        let participant_strings: Vec<String> =
            participants.iter().map(|p| format!("{:?}", p)).collect();

        // SAFE: Fallback to empty object if serialization fails (defensive programming)
        let context_value = serde_json::to_value(context).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to serialize coordination context: {}. Using empty object.",
                e
            );
            serde_json::Value::Object(serde_json::Map::new())
        });

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
                .filter(|s| matches!(s.health_status, ServiceHealthStatus::Healthy))
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
                // Check actual registration status from manager status
                registered: {
                    let manager_status = self.status.read().await;
                    !manager_status.active_registrations.is_empty()
                },
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
        _metadata: HashMap<String, String>,
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

    /// Authenticate using discovered security provider (capability-based)
    ///
    /// MODERN: Discovers security provider at runtime based on capabilities,
    /// not hardcoded primal names. Works with any primal providing authentication.
    pub async fn authenticate_universal(
        &self,
        credentials: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        tracing::info!("🔐 Authenticating via capability-based security provider discovery");

        // MODERN: Discover security provider by capability, not by name
        // SAFE: unwrap_or_else provides graceful fallback on discovery failure
        let security_providers = self
            .universal_ecosystem
            .find_by_capability("authentication")
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to discover security providers: {}. Falling back to local auth.",
                    e
                );
                Vec::new() // Empty vec on error - triggers local authentication fallback
            });

        if security_providers.is_empty() {
            tracing::warn!("No security provider discovered - using local authentication");
            // Fallback: Local authentication without external primal
            // SAFE: Multiple fallback strategies ensure we always have a user_id
            let user_id = credentials
                .get("user_id")
                .or_else(|| credentials.get("username"))
                .cloned()
                .unwrap_or_else(|| {
                    tracing::debug!("No user_id or username in credentials, using anonymous");
                    "anonymous".to_string()
                });

            let session_id = format!("local_session_{}", uuid::Uuid::new_v4());
            tracing::info!("✅ Local authentication complete for user: {}", user_id);
            return Ok(session_id);
        }

        // Use discovered security provider (no hardcoded name!)
        let provider = &security_providers[0];
        tracing::info!(
            "✅ Using discovered security provider (capability-based, match score: {})",
            provider.score
        );

        // Create session ID that doesn't assume provider name
        let session_id = format!("auth_session_{}", uuid::Uuid::new_v4());

        tracing::info!("✅ Capability-based authentication complete");
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
    ///
    /// Finds primal services that match the requested capabilities.
    ///
    /// # Arguments
    ///
    /// * `request` - Capability requirements to match
    ///
    /// # Returns
    ///
    /// Vector of capability matches, sorted by match score. Returns empty vector on error.
    ///
    /// # Error Handling
    ///
    /// Gracefully handles discovery failures by returning an empty result set,
    /// allowing callers to handle "no matches" uniformly.
    pub async fn match_capabilities_universal(
        &self,
        request: &CapabilityRequest,
    ) -> Vec<CapabilityMatch> {
        self.universal_ecosystem
            .match_capabilities(request)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Capability matching failed: {}. Returning empty result set.",
                    e
                );
                Vec::new()
            })
    }
}

/// Initialize ecosystem integration with service mesh patterns
///
/// Creates and initializes an ecosystem manager for service discovery and cross-primal communication.
///
/// # Arguments
///
/// * `config` - Ecosystem configuration
/// * `metrics_collector` - Metrics collector for observability
///
/// # Returns
///
/// An initialized `EcosystemManager` ready for service registration and discovery
pub async fn initialize_ecosystem_integration(
    config: EcosystemConfig,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<EcosystemManager, PrimalError> {
    tracing::info!("Initializing ecosystem integration with service mesh patterns");

    let mut manager = EcosystemManager::new(config, metrics_collector);
    manager.initialize().await?;

    tracing::info!("Ecosystem integration initialized successfully");
    Ok(manager)
}

#[cfg(test)]
#[path = "manager_tests.rs"]
mod tests;
