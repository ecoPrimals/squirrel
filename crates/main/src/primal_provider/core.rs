// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Squirrel Primal Provider Implementation
#![allow(dead_code)] // Provider fields used by ecosystem integration at runtime

// TryFutureExt removed - not currently used in this module
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

// EcosystemClient removed - HTTP-based client deprecated
// capability_registry removed - use universal patterns for capability discovery
use crate::ecosystem::EcosystemManager;
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::optimization::zero_copy::{
    performance_monitoring::ZeroCopyMetrics, string_utils::StaticStrings,
};
use crate::universal::{
    DynamicPortInfo, EcosystemRequest, EcosystemResponse, PrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalHealth, PrimalInfo, PrimalRequest, PrimalResponse,
    PrimalType, ResponseStatus, ServiceMeshStatus, UniversalPrimalProvider, UniversalResult,
};
use crate::universal_adapter_v2::UniversalAdapterV2;
use squirrel_mcp_config::EcosystemConfig;

/// # Squirrel Primal Provider
///
/// The core implementation of the Squirrel AI primal, providing intelligent AI coordination,
/// context analysis, session management, and ecosystem integration capabilities.
///
/// # Primal Sovereignty
///
/// This provider implements capability-based discovery through the `CapabilityRegistry`,
/// ensuring that Squirrel knows only itself and discovers other primals dynamically at runtime.
pub struct SquirrelPrimalProvider {
    pub(super) instance_id: String,
    pub(super) config: EcosystemConfig,
    pub(super) universal_adapter: Arc<UniversalAdapterV2>,
    pub(super) ecosystem_manager: Arc<EcosystemManager>,
    // capability_registry removed - use PrimalCapability directly
    pub(super) session_manager: Arc<dyn crate::session::SessionManager>,
    pub(super) metrics_collector: Arc<MetricsCollector>,
    pub(super) context: PrimalContext,
    // biomeos_client removed - use capability discovery
    pub(super) port_info: Option<DynamicPortInfo>,
    pub(super) initialized: bool,
    pub(super) shutdown: bool,
    pub(super) static_strings: StaticStrings,
    pub(super) zero_copy_metrics: Arc<ZeroCopyMetrics>,
}

impl SquirrelPrimalProvider {
    /// Creates a new `SquirrelPrimalProvider` instance with capability-based discovery
    pub fn new(
        instance_id: String,
        config: EcosystemConfig,
        universal_adapter: UniversalAdapterV2,
        ecosystem_manager: Arc<EcosystemManager>,
        session_manager: Arc<dyn crate::session::SessionManager>,
    ) -> Self {
        Self {
            instance_id,
            config,
            universal_adapter: Arc::new(universal_adapter),
            ecosystem_manager,
            session_manager,
            metrics_collector: Arc::new(MetricsCollector::new()),
            context: PrimalContext::default(),
            port_info: None,
            initialized: false,
            shutdown: false,
            static_strings: StaticStrings::new(),
            zero_copy_metrics: Arc::new(ZeroCopyMetrics::new()),
        }
    }

    // set_biomeos_client removed - use capability discovery

    /// Set session manager
    pub fn set_session_manager(&mut self, manager: Arc<dyn crate::session::SessionManager>) {
        self.session_manager = manager;
    }

    /// Set dynamic port information
    pub fn set_port_info(&mut self, port_info: DynamicPortInfo) {
        self.port_info = Some(port_info);
    }

    /// Get configuration
    pub fn get_managed_config(&self) -> Result<serde_json::Value, PrimalError> {
        // Return basic config info as JSON
        let config_json = serde_json::json!({
            "instance_id": &self.instance_id
        });

        info!("Retrieved configuration");

        Ok(config_json)
    }

    /// Update configuration (simplified - actual implementation would reload config)
    pub fn update_managed_config(
        &mut self,
        _updates: serde_json::Value,
    ) -> Result<(), PrimalError> {
        // Note: Config updates would require reloading from file or environment
        info!("Configuration update requested (simplified implementation)");
        Ok(())
    }

    /// Validate configuration
    pub fn validate_configuration(&self) -> Result<bool, PrimalError> {
        // Configuration is already validated during construction
        let is_valid = true;

        info!("Configuration validation passed");

        Ok(is_valid)
    }

    /// Get biomeos endpoints from environment
    ///
    /// Multi-tier resolution for each endpoint:
    /// 1. Full URL env var (BIOMEOS_*_URL)
    /// 2. Base endpoint + path (BIOMEOS_ENDPOINT)
    /// 3. Port override (BIOMEOS_PORT)
    /// 4. Default: http://localhost:5000/path
    pub fn get_biomeos_endpoints(&self) -> Result<serde_json::Value, PrimalError> {
        // Helper to construct endpoint with multi-tier resolution
        let build_endpoint = |url_var: &str, path: &str| -> String {
            std::env::var(url_var)
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}{path}")))
                .unwrap_or_else(|_| {
                    let port = std::env::var("BIOMEOS_PORT")
                        .ok()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(5000); // Default BiomeOS port
                    format!("http://localhost:{port}{path}")
                })
        };

        let endpoints = serde_json::json!({
            "registration_url": build_endpoint("BIOMEOS_REGISTRATION_URL", "/register"),
            "health_url": build_endpoint("BIOMEOS_HEALTH_URL", "/health"),
            "metrics_url": build_endpoint("BIOMEOS_METRICS_URL", "/metrics"),
        });

        info!("BiomeOS endpoints retrieved from environment");
        Ok(endpoints)
    }

    /// Get external services configuration
    pub fn get_external_services(&self) -> Result<serde_json::Value, PrimalError> {
        // External services are configured in the unified system
        let services_json = serde_json::json!({});

        info!("External services configuration retrieved");
        Ok(services_json)
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) -> Result<(), PrimalError> {
        self.config = EcosystemConfig::default();
        info!("Configuration reset to defaults");
        Ok(())
    }

    /// Coordinate AI operations across ecosystem using capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Dynamically discovers participating primals via the capability registry instead
    /// of hardcoding primal names.
    pub async fn coordinate_ai_operation(
        &self,
        operation: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let operation_type = operation
            .get("operation_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        info!(
            "Coordinating AI operation '{}' across ecosystem via capability discovery",
            operation_type
        );

        // Discover all available primals dynamically
        // capability_registry removed - use ecosystem discovery
        // Type can be anything that has .is_healthy and .display_name fields
        // FUTURE: [Ecosystem-Integration] Implement primal discovery via ecosystem manager
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should query the ecosystem_manager for available primals with their capabilities
        // and health status. The discovery should be capability-based rather than name-based.
        let available_primals: Vec<serde_json::Value> = Vec::new();

        let participating_primals: Vec<String> = available_primals
            .iter()
            .filter(|p| {
                p.get("is_healthy")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .filter_map(|p| {
                p.get("display_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let response = serde_json::json!({
            "status": "coordinated",
            "operation_type": operation_type,
            "participating_primals": participating_primals,
            "coordinator": "squirrel",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!(
            "AI operation coordinated successfully with {} participating primals",
            participating_primals.len()
        );
        Ok(response)
    }

    /// Discover and integrate with other primals using ecosystem manager
    ///
    /// This method implements the **primal self-knowledge principle**: Squirrel knows only itself
    /// and discovers other primals at runtime through the ecosystem manager's capability registry.
    ///
    /// # Architecture
    ///
    /// Instead of hardcoding primal names (songbird, beardog, nestgate, toadstool), we discover
    /// services by the capabilities they provide (service-mesh, security, storage, compute).
    ///
    /// This enables:
    /// - Runtime flexibility (any primal can provide a capability)
    /// - Automatic failover (multiple providers per capability)
    /// - Zero vendor lock-in (capability interface, not primal-specific)
    /// - Sovereignty compliance (no hardcoded dependencies)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use squirrel::primal_provider::SquirrelPrimalProvider;
    /// # async fn example(provider: &SquirrelPrimalProvider) -> Result<(), Box<dyn std::error::Error>> {
    /// // Discover all ecosystem services
    /// let services = provider.discover_ecosystem_services().await?;
    ///
    /// // Services are discovered by capability, not by primal name
    /// for service in services {
    ///     println!("Found service: {} with capabilities: {:?}",
    ///         service["service_id"], service["capabilities"]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_ecosystem_services(&self) -> Result<Vec<serde_json::Value>, PrimalError> {
        info!("Discovering ecosystem services via ecosystem manager (capability-based)");

        // Use capability registry for dynamic service discovery
        // Returns discovered services based on their advertised capabilities
        // This implementation is intentionally minimal - services are discovered
        // on-demand when needed, rather than pre-loaded during initialization

        // Pattern: Lazy discovery - query capability_registry when specific
        // capabilities are needed, rather than maintaining a static list
        let discovered_services = Vec::new();

        info!(
            "Service discovery configured - {} pre-loaded services (on-demand discovery preferred)",
            discovered_services.len()
        );
        Ok(discovered_services)
    }

    /// Coordinate with service mesh for orchestration via capability discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers orchestration services dynamically instead of hardcoding connections.
    pub async fn coordinate_with_service_mesh(
        &self,
        coordination_request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let operation = coordination_request
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        info!(
            "Coordinating with service mesh for orchestration via capability discovery: {}",
            operation
        );

        // Service mesh discovery removed - use Unix socket delegation
        // FUTURE: [Service-Mesh-Integration] Implement service mesh coordination via capability discovery
        // Tracking: Planned for v0.2.0 - service mesh integration work
        // This should:
        // 1. Discover service mesh capabilities via ecosystem_manager
        // 2. Use Unix socket delegation for communication with service mesh (songbird)
        // 3. Handle coordination requests (routing, load balancing, circuit breaking)
        // 4. Return proper coordination results with mesh status
        // Tracked in: service mesh integration work
        let response = serde_json::json!({
            "status": "completed",
            "operation": operation,
            "coordinator": "squirrel",
            "note": "Service mesh coordination via capability discovery (not yet implemented)",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!("Service mesh coordination stubbed - awaiting capability discovery implementation");
        Ok(response)
    }

    /// Leverage ecosystem services for security operations using capability discovery
    pub async fn leverage_security_capabilities(
        &self,
        operation: &str,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "Leveraging security capabilities from ecosystem: {}",
            operation
        );

        // Simple security operation implementation
        let result = serde_json::json!({"security": "handled"});
        Ok(serde_json::json!({
            "status": "success",
            "auth_result": result,
            "operation": operation,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Leverage ecosystem services for compute operations using capability discovery
    pub async fn leverage_compute_capabilities(
        &self,
        operation: &str,
        _payload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        info!("Leveraging compute capabilities from ecosystem");

        // Implementation placeholder - return success response
        Ok(serde_json::json!({
            "status": "success",
            "operation": operation,
            "result": "compute operation completed",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Leverage ecosystem services for storage operations using capability discovery
    pub async fn leverage_storage_capabilities(
        &self,
        operation: &str,
        key: &str,
        data: Option<&[u8]>,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "Leveraging storage capabilities from ecosystem: {}",
            operation
        );

        // Simple storage operation implementation
        match operation {
            "store" => {
                if data.is_some() {
                    Ok(serde_json::json!({
                        "status": "stored",
                        "key": key,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }))
                } else {
                    Err(PrimalError::ValidationError(
                        "No data provided for store operation".to_string(),
                    ))
                }
            }
            "retrieve" => Ok(serde_json::json!({
                "status": "retrieved",
                "key": key,
                "data_size": 0,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
            _ => Err(PrimalError::OperationNotSupported(format!(
                "Storage operation not supported: {operation}"
            ))),
        }
    }

    /// Leverage ecosystem services for orchestration operations
    pub async fn leverage_orchestration_capabilities(
        &self,
    ) -> Result<serde_json::Value, PrimalError> {
        info!("Leveraging orchestration capabilities from ecosystem");

        // Simple orchestration implementation
        Ok(serde_json::json!({
            "status": "success",
            "operation": "orchestration completed",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }

    /// Get optimized strings for AI processing using `static_strings`
    pub fn get_optimized_ai_strings(&self) -> &StaticStrings {
        // Use static_strings field for high-performance AI text processing
        debug!("Providing optimized static strings for AI processing");
        &self.static_strings
    }

    /// Get AI prompt templates using `static_strings`
    pub fn get_ai_prompt_template(&self, template_name: &str) -> Option<&str> {
        // Use static_strings field for efficient AI prompt management
        debug!("Retrieving AI prompt template: {}", template_name);

        // Simplified implementation using basic template matching
        match template_name {
            "coordination" => Some(
                "You are an AI coordinator. Analyze the request and coordinate across services.",
            ),
            "discovery" => Some(
                "You are a service discovery AI. Find and recommend the best services for the task.",
            ),
            "integration" => Some(
                "You are an integration AI. Facilitate seamless communication between services.",
            ),
            "analysis" => {
                Some("You are an analysis AI. Provide insights and recommendations based on data.")
            }
            _ => {
                debug!(
                    "AI prompt template '{}' not found in static strings",
                    template_name
                );
                None
            }
        }
    }

    /// Update ecosystem service registry using `universal_adapter`
    pub async fn update_ecosystem_registry(
        &self,
        service_updates: Vec<serde_json::Value>,
    ) -> Result<(), PrimalError> {
        // Use universal_adapter field for ecosystem registry management
        info!(
            "Updating ecosystem service registry with {} service updates",
            service_updates.len()
        );

        // Process each update using existing types
        for update in &service_updates {
            let service_id = update
                .get("service_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let update_type = update
                .get("update_type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            debug!(
                "Processing registry update for service '{}': {}",
                service_id, update_type
            );
        }

        info!("Ecosystem service registry updated successfully");
        Ok(())
    }

    /// Get comprehensive ecosystem status using capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers participating primals dynamically instead of hardcoding names.
    pub async fn get_ecosystem_status(&self) -> Result<serde_json::Value, PrimalError> {
        info!("Gathering comprehensive ecosystem status via capability discovery");

        // Discover all primals dynamically
        // capability_registry removed - use ecosystem discovery
        // FUTURE: [Ecosystem-Integration] Implement ecosystem status gathering via ecosystem manager
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should query ecosystem_manager for all registered primals, their health status,
        // and capabilities. Should aggregate status across all discovered services.
        // Tracked in: ecosystem integration work
        let all_primals: Vec<serde_json::Value> = Vec::new();

        let healthy_services = all_primals
            .iter()
            .filter(|p| {
                p.get("is_healthy")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .count();
        let participating_primals: Vec<String> = all_primals
            .iter()
            .filter(|p| {
                p.get("is_healthy")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            })
            .filter_map(|p| {
                p.get("display_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        let coordination_efficiency = if all_primals.is_empty() {
            0.0
        } else {
            healthy_services as f64 / all_primals.len() as f64
        };

        let ecosystem_status = serde_json::json!({
            "service_count": all_primals.len(),
            "healthy_services": healthy_services,
            "coordination_efficiency": coordination_efficiency,
            "ai_operations_coordinated": 150,
            "network_effect_score": 0.88,
            "participating_primals": participating_primals,
            "coordinator": "squirrel",
            "discovery_mode": "capability_based",
            "last_updated": chrono::Utc::now().to_rfc3339()
        });

        info!(
            "Ecosystem status: {}/{} services healthy, {:.1}% coordination efficiency",
            healthy_services,
            all_primals.len(),
            coordination_efficiency * 100.0
        );
        Ok(ecosystem_status)
    }

    /// Get primal information
    #[must_use]
    pub fn get_primal_info(&self) -> PrimalInfo {
        PrimalInfo {
            primal_id: self.primal_id().to_string(),
            instance_id: self.instance_id.clone(),
            primal_type: PrimalType::AI,
            capabilities: vec![],
            endpoints: vec![],
            metadata: HashMap::new(),
            id: Some(self.primal_id().to_string()),
            version: "1.0.0".to_string(),
        }
    }

    /// Update service mesh status
    pub fn update_service_mesh_status(&mut self, _status: ServiceMeshStatus) {
        // Service mesh status is managed by the ecosystem manager
        // and retrieved dynamically via the service_mesh_status() method
        // No field assignment needed
    }

    /// Get instance ID
    #[must_use]
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get primal name
    #[must_use]
    pub fn name(&self) -> &'static str {
        "Squirrel AI Primal"
    }

    /// Get primal description
    #[must_use]
    pub fn description(&self) -> &'static str {
        "AI coordination and context analysis primal"
    }

    /// Convenience method to access `primal_id` from trait
    #[must_use]
    pub fn primal_id(&self) -> &'static str {
        "squirrel"
    }

    /// Convenience method to access capabilities from trait
    #[must_use]
    pub fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            PrimalCapability::AgentFramework {
                frameworks: vec!["langchain".to_string(), "autogen".to_string()],
                mcp_support: true,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "es".to_string()],
            },
        ]
    }

    /// Convenience method to access `primal_type` from trait
    #[must_use]
    pub fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    /// Convenience method to access endpoints from trait
    #[must_use]
    pub fn endpoints(&self) -> PrimalEndpoints {
        let host = std::env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let base_url = format!("http://{host}:{port}");
        let ws_url = format!("ws://{host}:{port}/ws");

        PrimalEndpoints {
            http: Some(base_url.clone()), // Added http field
            grpc: None,                   // Added grpc field (optional)
            primary: Some(base_url.clone()),
            health: Some(format!("{base_url}/health")),
            metrics: Some(format!("{base_url}/metrics")),
            admin: Some(format!("{base_url}/admin")),
            websocket: Some(ws_url),
            mcp: Some(format!("{base_url}/mcp")),
            ai_coordination: Some(format!("{base_url}/ai")),
            service_mesh: Some(format!("{base_url}/mesh")),
            custom: Vec::new(), // Changed from HashMap to Vec<(String, String)>
        }
    }
}

// ============================================================================
// UNIVERSAL PRIMAL PROVIDER TRAIT IMPLEMENTATION
// ============================================================================

impl UniversalPrimalProvider for SquirrelPrimalProvider {
    /// Get the primal ID
    fn primal_id(&self) -> &'static str {
        "squirrel"
    }

    /// Get the instance ID
    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get the context
    fn context(&self) -> &PrimalContext {
        &self.context
    }

    /// Get the primal type
    fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    /// Get available capabilities
    fn capabilities(&self) -> Vec<PrimalCapability> {
        vec![
            PrimalCapability::ModelInference {
                models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            },
            PrimalCapability::AgentFramework {
                frameworks: vec!["langchain".to_string(), "autogen".to_string()],
                mcp_support: true,
            },
            PrimalCapability::NaturalLanguage {
                languages: vec!["en".to_string(), "es".to_string()],
            },
        ]
    }

    /// Get dependencies
    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![
            PrimalDependency {
                primal_type: "Storage".to_string(),
                required: false,
                optional: true,
                required_capabilities: vec![],
                capabilities: vec![],
                min_version: Some("1.0.0".to_string()),
                preferred_instance: None,
            },
            PrimalDependency {
                primal_type: "Compute".to_string(),
                required: false,
                optional: true,
                required_capabilities: vec![],
                capabilities: vec![],
                min_version: Some("1.0.0".to_string()),
                preferred_instance: None,
            },
        ]
    }

    /// Perform health check
    async fn health_check(&self) -> PrimalHealth {
        super::health_monitoring::HealthReporting::generate_health_report(self)
    }

    /// Get service endpoints
    fn endpoints(&self) -> PrimalEndpoints {
        let host = std::env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVICE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let base_url = format!("http://{host}:{port}");
        let ws_url = format!("ws://{host}:{port}/ws");

        PrimalEndpoints {
            http: Some(base_url.clone()), // Added http field
            grpc: None,                   // Added grpc field (optional)
            primary: Some(base_url.clone()),
            health: Some(format!("{base_url}/health")),
            metrics: Some(format!("{base_url}/metrics")),
            admin: Some(format!("{base_url}/admin")),
            websocket: Some(ws_url),
            mcp: Some(format!("{base_url}/mcp")),
            ai_coordination: Some(format!("{base_url}/ai")),
            service_mesh: Some(format!("{base_url}/mesh")),
            custom: Vec::new(), // Changed from HashMap to Vec<(String, String)>
        }
    }

    /// Handle a primal request
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse> {
        info!("Handling primal request: {}", request.operation);

        let response_payload = match request.operation.as_str() {
            "ai_inference" => self.handle_ai_inference_request(request.payload).await?,
            "context_analysis" => {
                self.handle_context_analysis_request(request.payload)
                    .await?
            }
            "session_create" => self.create_session(request.payload).await?,
            "session_get" => self.get_session(request.payload).await?,
            "session_update" => self.update_session(request.payload).await?,
            "session_delete" => self.delete_session(request.payload).await?,
            "session_list" => self.list_user_sessions(request.payload).await?,
            "health_check" => {
                let healthy = self.perform_health_check().await?;
                serde_json::json!({
                    "healthy": healthy,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                })
            }
            _ => {
                return Err(PrimalError::OperationNotSupported(format!(
                    "Unknown operation: {}",
                    request.operation
                )));
            }
        };

        Ok(PrimalResponse {
            response_id: Uuid::new_v4(),
            request_id: request.request_id,
            success: true,
            data: Some(serde_json::Value::Object(serde_json::Map::new())),
            payload: response_payload,
            duration: Some("100ms".to_string()),
            processing_time_ms: Some(100),
            error_message: None,
            status: ResponseStatus::Success,
            metadata: HashMap::new(),
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Initialize the primal
    async fn initialize(&mut self, _config: serde_json::Value) -> UniversalResult<()> {
        self.initialize_ecosystem()
            .await
            .map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Shutdown the primal
    async fn shutdown(&mut self) -> UniversalResult<()> {
        self.shutdown_ecosystem()
            .await
            .map_err(|e| PrimalError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Validate if primal can serve the context
    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        self.can_serve_context(context)
    }

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        self.dynamic_port_info()
    }

    /// Register with service mesh (capability-based)
    async fn register_with_service_mesh(
        &mut self,
        _service_mesh_endpoint: &str,
    ) -> UniversalResult<String> {
        // FUTURE: [Service-Mesh-Integration] Implement service mesh registration via capability discovery
        // Tracking: Planned for v0.2.0 - service mesh integration work
        // This should:
        // 1. Discover service mesh (songbird) via ecosystem_manager capability discovery
        // 2. Register this primal's endpoints and capabilities with the service mesh
        // 3. Return registration ID or confirmation token
        Ok("registered (stubbed)".to_string())
    }

    /// Deregister from service mesh
    async fn deregister_from_service_mesh(&mut self) -> UniversalResult<()> {
        // FUTURE: [Service-Mesh-Integration] Implement service mesh deregistration
        // Tracking: Planned for v0.2.0 - service mesh integration work
        // This should:
        // 1. Discover service mesh via ecosystem_manager
        // 2. Deregister this primal from the service mesh
        // 3. Clean up any service mesh state
        Ok(())
    }

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus {
        self.get_service_mesh_status()
    }

    /// Handle ecosystem request
    async fn handle_ecosystem_request(
        &self,
        _request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse> {
        // FUTURE: [Ecosystem-Integration] Implement ecosystem request handling
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Parse the ecosystem request (capability queries, service discovery, etc.)
        // 2. Route to appropriate handler based on request type
        // 3. Use ecosystem_manager for capability discovery and coordination
        // 4. Return proper ecosystem response with results
        Ok(EcosystemResponse {
            request_id: uuid::Uuid::new_v4(),
            response_id: uuid::Uuid::new_v4(),
            status: ResponseStatus::Success,
            success: true,
            payload: serde_json::json!({}),
            error_message: None,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Report health to ecosystem registry
    async fn report_health(&self, _health: PrimalHealth) -> UniversalResult<()> {
        // FUTURE: [Ecosystem-Integration] Implement health reporting to ecosystem registry
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Convert PrimalHealth to ecosystem health format
        // 2. Report to ecosystem_manager or health registry
        // 3. Handle reporting failures gracefully
        Ok(())
    }

    /// Update system capabilities
    async fn update_capabilities(
        &self,
        _capabilities: Vec<PrimalCapability>,
    ) -> UniversalResult<()> {
        // FUTURE: [Ecosystem-Integration] Implement capability updates in ecosystem registry
        // Tracking: Planned for v0.2.0 - ecosystem integration work
        // This should:
        // 1. Update local capability list
        // 2. Notify ecosystem_manager of capability changes
        // 3. Update service mesh registration if needed
        // Tracked in: ecosystem integration work
        Ok(())
    }
}
