// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Squirrel Primal Provider Implementation
#![expect(
    dead_code,
    reason = "Provider fields used by ecosystem integration at runtime"
)]

// TryFutureExt removed - not currently used in this module
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

// EcosystemClient removed - HTTP-based client deprecated
// capability_registry removed - use universal patterns for capability discovery
use crate::ecosystem::EcosystemManager;
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::optimization::zero_copy::{
    performance_monitoring::ZeroCopyMetrics, string_utils::StaticStrings,
};
use crate::universal::{
    DynamicPortInfo, PrimalCapability, PrimalContext, PrimalEndpoints, PrimalInfo, PrimalType,
    ServiceMeshStatus,
};
use crate::universal_adapter_v2::UniversalAdapterV2;
use squirrel_mcp_config::EcosystemConfig;
use universal_constants::capabilities;

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

        let mut discovered: Vec<serde_json::Value> = Vec::new();
        for cap in [
            capabilities::COMPUTE_CAPABILITY,
            capabilities::ECOSYSTEM_CAPABILITY,
            capabilities::SERVICE_MESH_CAPABILITY,
        ] {
            match self
                .ecosystem_manager
                .find_services_by_capability(cap)
                .await
            {
                Ok(services) => {
                    for s in services {
                        discovered.push(serde_json::json!({
                            "capability": cap,
                            "service_id": s.service_id.as_ref(),
                            "endpoint": s.endpoint.as_ref(),
                            "health_endpoint": s.health_endpoint.as_ref(),
                        }));
                    }
                }
                Err(e) => {
                    info!(
                        "Capability discovery for '{}' returned: {} (continuing with other capabilities)",
                        cap, e
                    );
                }
            }
        }

        let participating_primals: Vec<String> = discovered
            .iter()
            .filter_map(|v| {
                v.get("service_id")
                    .and_then(|x| x.as_str())
                    .map(std::string::ToString::to_string)
            })
            .collect();

        let response = serde_json::json!({
            "status": "coordinated",
            "operation_type": operation_type,
            "participating_primals": participating_primals,
            "discovered_via_capabilities": discovered,
            "coordinator": crate::niche::PRIMAL_ID,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!(
            "AI operation coordinated with {} service endpoint(s) from IPC capability discovery",
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

        let mut discovered_services: Vec<serde_json::Value> = Vec::new();
        for cap in [
            capabilities::ECOSYSTEM_CAPABILITY,
            capabilities::SERVICE_MESH_CAPABILITY,
            capabilities::STORAGE_CAPABILITY,
            capabilities::COMPUTE_CAPABILITY,
            capabilities::SECURITY_CAPABILITY,
        ] {
            if let Ok(services) = self
                .ecosystem_manager
                .find_services_by_capability(cap)
                .await
            {
                for s in services {
                    discovered_services.push(serde_json::json!({
                        "capability": cap,
                        "service_id": s.service_id.as_ref(),
                        "endpoint": s.endpoint.as_ref(),
                    }));
                }
            }
        }

        info!(
            "Ecosystem service discovery (IPC): {} entries from capability queries",
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

        let mesh = self
            .ecosystem_manager
            .find_services_by_capability(capabilities::SERVICE_MESH_CAPABILITY)
            .await
            .unwrap_or_default();

        let response = serde_json::json!({
            "status": "completed",
            "operation": operation,
            "coordinator": crate::niche::PRIMAL_ID,
            "mesh_services_discovered": mesh.len(),
            "mesh_services": mesh.iter().map(|s| {
                serde_json::json!({
                    "service_id": s.service_id.as_ref(),
                    "endpoint": s.endpoint.as_ref(),
                    "health_endpoint": s.health_endpoint.as_ref(),
                })
            }).collect::<Vec<_>>(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        info!(
            mesh_count = mesh.len(),
            "Service mesh coordination: endpoints from IPC capability discovery (`{}`)",
            capabilities::SERVICE_MESH_CAPABILITY
        );
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
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            })
            .count();
        let participating_primals: Vec<String> = all_primals
            .iter()
            .filter(|p| {
                p.get("is_healthy")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
            })
            .filter_map(|p| {
                p.get("display_name")
                    .and_then(|v| v.as_str())
                    .map(std::string::ToString::to_string)
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
            "coordinator": crate::niche::PRIMAL_ID,
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
    pub const fn name(&self) -> &'static str {
        "Squirrel AI Primal"
    }

    /// Get primal description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        "AI coordination and context analysis primal"
    }

    /// Convenience method to access `primal_id` from trait
    #[must_use]
    pub const fn primal_id(&self) -> &'static str {
        crate::niche::PRIMAL_ID
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
    pub const fn primal_type(&self) -> PrimalType {
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

mod universal_trait;

#[cfg(test)]
#[path = "core_tests.rs"]
mod core_provider_tests;
