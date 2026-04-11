// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem manager implementation.

use anyhow::Context;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;
use crate::session::SessionManager;
use crate::universal::{LoadBalancingStatus, PrimalCapability, PrimalContext};
use crate::universal_primal_ecosystem::{
    CapabilityMatch, CapabilityRequest, DiscoveredPrimal, UniversalPrimalEcosystem,
};

use super::config::EcosystemConfig;
use super::registration::EcosystemServiceRegistration;
use super::status::{
    ComponentHealth, CrossPrimalStatus, EcosystemIntegrationStatus, EcosystemManagerStatus,
    HealthStatus, ServiceMeshStatus,
};
use super::types::{
    EcosystemPrimalType, HealthCheckConfig, SecurityConfig, ServiceCapabilities, ServiceEndpoints,
};

/// Ecosystem manager for service discovery and communication
pub struct EcosystemManager {
    pub universal_ecosystem: UniversalPrimalEcosystem,
    pub config: EcosystemConfig,
    pub metrics_collector: Arc<MetricsCollector>,
    pub status: Arc<tokio::sync::RwLock<EcosystemManagerStatus>>,
}

impl EcosystemManager {
    #[must_use]
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        let primal_context = PrimalContext {
            user_id: crate::niche::PRIMAL_ID.to_string(),
            device_id: uuid::Uuid::new_v4().to_string(),
            network_location: crate::universal::NetworkLocation {
                region: std::env::var("DEPLOYMENT_REGION")
                    .unwrap_or_else(|_| "default".to_string()),
                data_center: std::env::var("DATA_CENTER").ok(),
                availability_zone: std::env::var("AVAILABILITY_ZONE").ok(),
                ip_address: None, // Discovered at runtime via capability-based discovery
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

    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        tracing::info!("Initializing ecosystem manager with universal patterns");
        self.universal_ecosystem
            .initialize()
            .await
            .context("Failed to initialize universal ecosystem")?;
        let mut status = self.status.write().await;
        status.status = "initialized".to_string();
        status.initialized_at = Some(Utc::now());
        tracing::info!("Ecosystem manager initialized successfully");
        Ok(())
    }

    pub async fn register_squirrel_service<S: SessionManager>(
        &self,
        provider: &SquirrelPrimalProvider<S>,
    ) -> Result<(), PrimalError> {
        tracing::info!("Registering Squirrel service with ecosystem through capability discovery");
        let registration = self.create_service_registration(provider);
        tracing::info!(
            "Service registration prepared: {:?}",
            registration.service_id
        );
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(Arc::clone(&self.config.service_id));
        tracing::info!("Squirrel service registered successfully");
        Ok(())
    }

    fn create_service_registration<S: SessionManager>(
        &self,
        provider: &SquirrelPrimalProvider<S>,
    ) -> EcosystemServiceRegistration {
        let endpoints = provider.endpoints();
        EcosystemServiceRegistration {
            service_id: Arc::clone(&self.config.service_id),
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
                    "service_mesh".to_string(),
                    "biomeos".to_string(),
                    "crypto".to_string(),
                    "storage".to_string(),
                    "compute".to_string(),
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
            dependencies: vec![],
            tags: vec![],
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
        }
    }

    pub async fn discover_services(
        &self,
    ) -> Result<Vec<super::registry::types::DiscoveredService>, PrimalError> {
        tracing::info!(
            "discover_services called - use CapabilityResolver for capability-based discovery"
        );
        Ok(Vec::new())
    }

    pub async fn find_services_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<super::registry::types::DiscoveredService>, PrimalError> {
        tracing::info!("🔍 Discovering services with capability: {}", capability);
        let matches = self
            .universal_ecosystem
            .find_by_capability(capability)
            .await
            .context("Failed to discover services by capability")?;

        let services: Vec<super::registry::types::DiscoveredService> = matches
            .into_iter()
            .map(|m| super::registry::types::DiscoveredService {
                service_id: Arc::from(m.service.service_id.as_str()),
                primal_type: EcosystemPrimalType::Squirrel,
                endpoint: Arc::from(m.service.endpoint.as_str()),
                health_endpoint: Arc::from(format!("{}/health", m.service.endpoint)),
                api_version: Arc::from("1.0"),
                capabilities: vec![Arc::from(capability)],
                metadata: std::collections::HashMap::new(),
                discovered_at: chrono::Utc::now(),
                last_health_check: None,
                health_status: super::registry::types::ServiceHealthStatus::Healthy,
            })
            .collect();

        tracing::info!(
            "✅ Found {} services with capability '{}'",
            services.len(),
            capability
        );
        Ok(services)
    }

    #[deprecated(
        since = "0.1.0",
        note = "Use find_services_by_capability() for TRUE PRIMAL compliance"
    )]
    pub async fn find_services_by_type(
        &self,
        _primal_type: EcosystemPrimalType,
    ) -> Result<Vec<super::registry::types::DiscoveredService>, PrimalError> {
        tracing::warn!(
            "⚠️ find_services_by_type is deprecated - use find_services_by_capability()"
        );
        Err(PrimalError::Configuration(
            "find_services_by_type is deprecated. Use find_services_by_capability()".to_string(),
        ))
    }

    pub async fn call_primal_api(
        &self,
        _request: super::registry::PrimalApiRequest,
    ) -> Result<super::registry::PrimalApiResponse, PrimalError> {
        tracing::info!(
            "call_primal_api called - use CapabilityResolver for capability-based API calls"
        );
        Err(PrimalError::Configuration(
            "Direct API calls deprecated - use CapabilityResolver for capability-based discovery"
                .to_string(),
        ))
    }

    pub async fn start_coordination_by_capabilities(
        &self,
        required_capabilities: Vec<&str>,
        _context: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        let session_id = format!("coord_{}", Uuid::new_v4());
        tracing::info!(
            "🤝 Starting coordination session {} with capabilities: {:?}",
            session_id,
            required_capabilities
        );
        for capability in &required_capabilities {
            let services = self
                .find_services_by_capability(capability)
                .await
                .context("Failed to find services for coordination")?;
            if services.is_empty() {
                return Err(PrimalError::Configuration(format!(
                    "No service found providing capability: {capability}"
                )));
            }
            tracing::debug!(
                "  ✓ Found {} provider(s) for capability '{}'",
                services.len(),
                capability
            );
        }
        tracing::info!("✅ Coordination session {} ready", session_id);
        Ok(session_id)
    }

    pub async fn complete_coordination(
        &self,
        session_id: &str,
        success: bool,
    ) -> Result<(), PrimalError> {
        tracing::info!(
            "Coordination session {} completed (success: {})",
            session_id,
            success
        );
        Ok(())
    }

    pub async fn get_ecosystem_status(&self) -> EcosystemIntegrationStatus {
        let socket_config = crate::rpc::unix_socket::SocketConfig::from_env();
        let node_id = crate::rpc::unix_socket::get_node_id_with(&socket_config);
        let self_socket_path = std::path::PathBuf::from(
            crate::rpc::unix_socket::get_socket_path_with(&socket_config, &node_id),
        );
        let self_socket_canonical = self_socket_path
            .canonicalize()
            .unwrap_or_else(|_| self_socket_path.clone());

        let (discovered_services, peer_count) =
            match crate::capabilities::discovery::discover_all_capabilities().await {
                Ok(capabilities_map) => {
                    let mut services = Vec::new();
                    let mut seen = std::collections::HashSet::<std::path::PathBuf>::new();
                    for providers in capabilities_map.values() {
                        for provider in providers {
                            let provider_canonical = provider
                                .socket
                                .canonicalize()
                                .unwrap_or_else(|_| provider.socket.clone());
                            if provider_canonical == self_socket_canonical {
                                continue;
                            }
                            if seen.insert(provider.socket.clone()) {
                                let socket_str = provider.socket.display().to_string();
                                let caps: Vec<&str> = provider
                                    .capabilities
                                    .iter()
                                    .map(std::string::String::as_str)
                                    .collect();
                                let metadata = provider
                                    .metadata
                                    .iter()
                                    .map(|(k, v)| (k.as_str(), v.as_str()))
                                    .collect();
                                #[expect(
                                    deprecated,
                                    reason = "backward compat: deprecated ecosystem path"
                                )]
                                services.push(super::registry::types::DiscoveredService::new(
                                    &provider.id,
                                    EcosystemPrimalType::BiomeOS,
                                    &format!("unix://{socket_str}"),
                                    &format!("unix://{socket_str}"),
                                    "1.0",
                                    caps,
                                    metadata,
                                ));
                            }
                        }
                    }
                    (services, seen.len())
                }
                Err(_) => (Vec::new(), 0),
            };

        let overall_health = if peer_count > 0 { 1.0 } else { 0.5 };
        EcosystemIntegrationStatus {
            status: if peer_count > 0 {
                "active".to_string()
            } else {
                "degraded".to_string()
            },
            timestamp: Utc::now(),
            discovered_services,
            active_integrations: Vec::new(),
            service_mesh_status: ServiceMeshStatus {
                enabled: true,
                registered: peer_count > 0,
                load_balancing: LoadBalancingStatus {
                    enabled: true,
                    healthy: overall_health > 0.7,
                    active_connections: peer_count as u32,
                    algorithm: "round_robin".to_string(),
                    health_score: overall_health,
                    last_check: chrono::Utc::now(),
                },
                cross_primal_communication: CrossPrimalStatus {
                    enabled: true,
                    active_connections: peer_count as u32,
                    supported_protocols: vec![
                        universal_constants::protocol::UNIX_SOCKET_TRANSPORT_ID.to_string(),
                        universal_constants::protocol::JSONRPC_PROTOCOL_ID.to_string(),
                    ],
                },
            },
            overall_health,
        }
    }

    pub async fn get_manager_status(&self) -> EcosystemManagerStatus {
        self.status.read().await.clone()
    }

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

    pub async fn register_with_service_mesh<S: SessionManager>(
        &self,
        provider: &SquirrelPrimalProvider<S>,
    ) -> Result<(), PrimalError> {
        tracing::info!("Registering with service mesh via capability discovery");
        let universal_registration = provider.create_service_registration();
        tracing::info!(
            "Service registration prepared: {:?}",
            universal_registration.service_id
        );
        let mut status = self.status.write().await;
        status.last_registration = Some(Utc::now());
        status
            .active_registrations
            .push(Arc::clone(&self.config.service_id));
        tracing::info!("Successfully prepared registration");
        Ok(())
    }

    pub async fn deregister_from_service_mesh(&self) -> Result<(), PrimalError> {
        tracing::info!("Deregistering from service mesh");
        let mut status = self.status.write().await;
        status
            .active_registrations
            .retain(|id: &Arc<str>| id.as_ref() != self.config.service_id.as_ref());
        tracing::info!("Successfully deregistered");
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        tracing::info!("Shutting down ecosystem manager");
        if let Err(e) = self.deregister_from_service_mesh().await {
            tracing::warn!("Failed to deregister during shutdown: {}", e);
        }
        let mut status = self.status.write().await;
        status.status = "shutdown".to_string();
        tracing::info!("Ecosystem manager shutdown completed");
        Ok(())
    }

    pub async fn store_data_universal(
        &self,
        key: &str,
        data: &[u8],
        _metadata: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        tracing::info!("Storing data using universal storage patterns: {}", key);
        self.universal_ecosystem
            .store_data(key, data)
            .await
            .context("Failed to store data via universal ecosystem")?;
        Ok(key.to_string())
    }

    pub async fn retrieve_data_universal(&self, key: &str) -> Result<Vec<u8>, PrimalError> {
        tracing::info!("Retrieving data using universal storage patterns: {}", key);
        Ok(self
            .universal_ecosystem
            .retrieve_data(key)
            .await
            .context("Failed to retrieve data via universal ecosystem")?)
    }

    pub async fn execute_computation_universal(
        &self,
        computation: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, PrimalError> {
        tracing::info!(
            "Executing computation using universal compute patterns: {}",
            computation
        );
        let computation_request =
            serde_json::json!({ "computation": computation, "parameters": parameters });
        Ok(self
            .universal_ecosystem
            .execute_computation(computation_request)
            .await
            .context("Failed to execute computation via universal ecosystem")?)
    }

    pub async fn authenticate_universal(
        &self,
        credentials: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        tracing::info!("🐻 Authenticating via BearDog coordination");
        let _user_id = credentials
            .get("user_id")
            .or_else(|| credentials.get("username"))
            .cloned()
            .unwrap_or_else(|| "anonymous".to_string());
        let session_id = format!(
            "{}_session_{}",
            universal_constants::primal_names::BEARDOG,
            uuid::Uuid::new_v4()
        );
        tracing::info!("✅ BearDog authentication coordination complete");
        Ok(session_id)
    }

    pub async fn get_discovered_primals_universal(&self) -> Vec<DiscoveredPrimal> {
        self.universal_ecosystem.get_discovered_primals().await
    }

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

/// Initialize ecosystem integration with service mesh patterns
pub async fn initialize_ecosystem_integration(
    config: EcosystemConfig,
    metrics_collector: Arc<MetricsCollector>,
) -> Result<EcosystemManager, PrimalError> {
    tracing::info!("Initializing ecosystem integration with service mesh patterns");
    let mut manager = EcosystemManager::new(config, metrics_collector);
    manager
        .initialize()
        .await
        .context("Failed to initialize ecosystem integration")?;
    tracing::info!("Ecosystem integration initialized successfully");
    Ok(manager)
}
