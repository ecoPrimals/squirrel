// # Ecosystem Registry Manager
use crate::ecosystem::registry::types::*;
//
// Pure service discovery and communication layer for ecosystem integration.
// This module provides standalone primal coordination through standardized APIs.
//
// ## Architecture Principles
// - Each primal is completely standalone
// - Communication happens through HTTP/REST APIs
// - Service discovery is handled through Songbird service mesh
// - No hard dependencies between primals
// - All primals can function independently

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tracing::{info, warn};

use crate::ecosystem::EcosystemServiceRegistration;
use crate::error::PrimalError;
use crate::monitoring::metrics::MetricsCollector;

// Import from registry module
use crate::ecosystem::registry::*;

/// Ecosystem registry manager - Pure service discovery and communication layer
pub struct EcosystemRegistryManager {
    config: EcosystemRegistryConfig,
    registry_state: Arc<RwLock<RegistryState>>,
    service_registry: Arc<RwLock<HashMap<String, DiscoveredService>>>,
    metrics_collector: Arc<MetricsCollector>,
    event_publisher: broadcast::Sender<EcosystemRegistryEvent>,
    shutdown_token: Arc<Mutex<Option<tokio_util::sync::CancellationToken>>>,
    http_client: reqwest::Client,
}

impl EcosystemRegistryManager {
    /// Create new ecosystem registry manager
    pub fn new(config: EcosystemRegistryConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        let (event_publisher, _) = broadcast::channel(1000);

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            registry_state: Arc::new(RwLock::new(RegistryState::default())),
            service_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector,
            event_publisher,
            shutdown_token: Arc::new(Mutex::new(None)),
            http_client,
        }
    }

    /// Initialize the ecosystem registry manager
    pub async fn initialize(&self) -> Result<(), PrimalError> {
        info!("Initializing ecosystem registry manager");

        // Start background tasks
        self.start_background_tasks().await?;

        // Perform initial discovery if enabled
        if self.config.discovery_config.enabled {
            if let Err(e) = self.discover_services().await {
                warn!(
                    "Initial service discovery failed (will retry in background): {}",
                    e
                );
            }
        }

        info!("Ecosystem registry manager initialized successfully");
        Ok(())
    }

    /// Start background tasks for service discovery and health monitoring
    async fn start_background_tasks(&self) -> Result<(), PrimalError> {
        info!("Starting background tasks for ecosystem registry manager");

        // Start service discovery task
        let discovery_task = async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                // Task implementation handled by DiscoveryOps
            }
        };

        tokio::spawn(discovery_task);

        info!("Background tasks started successfully");
        Ok(())
    }

    /// Discover services through delegated operations
    pub async fn discover_services(&self) -> Result<(), PrimalError> {
        DiscoveryOps::discover_services(
            &self.service_registry,
            &self.event_publisher,
            &self.http_client,
            &self.config.songbird_endpoint,
        )
        .await
    }

    /// Register Squirrel service with ecosystem (through Songbird)
    pub async fn register_squirrel_service(
        &self,
        registration: EcosystemServiceRegistration,
    ) -> Result<(), PrimalError> {
        let registration_arc = Arc::new(registration);
        let mut registry_state = self.registry_state.write().await;

        // Register the service (using Arc to avoid expensive clones)
        registry_state
            .registered_services
            .insert(registration_arc.service_id.clone(), registration_arc.clone());

        // Register with Songbird (clone the Arc contents for the call)
        self.register_with_songbird((*registration_arc).clone()).await?;

        info!("Squirrel service registered successfully");
        Ok(())
    }

    /// Register with Songbird service mesh
    pub async fn register_with_songbird(
        &self,
        registration: EcosystemServiceRegistration,
    ) -> Result<(), PrimalError> {
        let songbird_endpoint = &self.config.songbird_endpoint;
        info!("Registering with Songbird at: {}", songbird_endpoint);

        let url = format!("{}/api/v1/services/register", songbird_endpoint);
        let response = self
            .http_client
            .post(&url)
            .json(&registration)
            .send()
            .await
            .map_err(|e| {
                PrimalError::NetworkError(format!("Failed to register with Songbird: {}", e))
            })?;

        if response.status().is_success() {
            info!("Successfully registered with Songbird");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::RegistrationError(format!(
                "Songbird registration failed: {}",
                error_text
            )))
        }
    }

    /// Deregister from Songbird service mesh
    pub async fn deregister_from_songbird(&self, service_id: &str) -> Result<(), PrimalError> {
        let songbird_endpoint = &self.config.songbird_endpoint;
        info!("Deregistering from Songbird: {}", service_id);

        let url = format!(
            "{}/api/v1/services/deregister/{}",
            songbird_endpoint, service_id
        );
        let response = self.http_client.delete(&url).send().await.map_err(|e| {
            PrimalError::NetworkError(format!("Failed to deregister from Songbird: {}", e))
        })?;

        if response.status().is_success() {
            info!("Successfully deregistered from Songbird");
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PrimalError::RegistrationError(format!(
                "Songbird deregistration failed: {}",
                error_text
            )))
        }
    }

    /// Get discovered services
    pub async fn get_discovered_services(&self) -> Vec<DiscoveredService> {
        let service_registry = self.service_registry.read().await;
        service_registry.values().cloned().collect()
    }

    /// Get active integrations
    pub async fn get_active_integrations(&self) -> Vec<String> {
        let service_registry = self.service_registry.read().await;
        service_registry.keys().cloned().collect()
    }

    /// Get service statistics
    pub async fn get_service_stats(&self) -> ServiceStats {
        MetricsOps::get_service_stats(&self.service_registry).await
    }

    /// Check if a service is healthy
    pub async fn is_service_healthy(&self, service_id: &str) -> bool {
        HealthMonitor::is_service_healthy(&self.service_registry, service_id).await
    }

    /// Subscribe to registry events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<EcosystemRegistryEvent> {
        self.event_publisher.subscribe()
    }

    /// Find services by primal type
    pub async fn find_services_by_type(
        &self,
        primal_type: crate::ecosystem::EcosystemPrimalType,
    ) -> Vec<DiscoveredService> {
        let service_registry = self.service_registry.read().await;
        service_registry
            .values()
            .filter(|service| service.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Call primal API
    pub async fn call_primal_api(
        &self,
        request: PrimalApiRequest,
    ) -> Result<PrimalApiResponse, PrimalError> {
        let service_registry = self.service_registry.read().await;
        let service_arc = service_registry
            .values()
            .find(|s| s.primal_type == request.to_primal)
            .ok_or_else(|| {
                PrimalError::ResourceNotFound(format!(
                    "Service not found for primal type: {:?}",
                    request.to_primal
                ))
            })?;

        let url = format!("{}/api/v1/{}", service_arc.endpoint, request.operation);
        let start_time = std::time::Instant::now();

        let response = self
            .http_client
            .post(&url)
            .json(&request.payload)
            .timeout(request.timeout)
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("API call failed: {}", e)))?;

        let processing_time = start_time.elapsed();

        if response.status().is_success() {
            let data = response.json::<serde_json::Value>().await.ok();
            Ok(PrimalApiResponse {
                request_id: request.request_id,
                success: true,
                data,
                error: None,
                headers: HashMap::new(),
                processing_time,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Ok(PrimalApiResponse {
                request_id: request.request_id,
                success: false,
                data: None,
                error: Some(error_text),
                headers: HashMap::new(),
                processing_time,
            })
        }
    }

    /// Start coordination between services
    pub async fn start_coordination(
        &self,
        participants: Vec<String>,
        context: serde_json::Value,
    ) -> Result<String, PrimalError> {
        let coordination_id = uuid::Uuid::new_v4().to_string();

        // For now, just return a coordination ID
        // In a full implementation, this would orchestrate the coordination
        info!(
            "Starting coordination {} with {} participants",
            coordination_id,
            participants.len()
        );

        Ok(coordination_id)
    }

    /// Complete coordination between services
    pub async fn complete_coordination(
        &self,
        session_id: String,
        success: bool,
    ) -> Result<(), PrimalError> {
        info!(
            "Completing coordination {} with success: {}",
            session_id, success
        );

        // For now, just log the completion
        // In a full implementation, this would clean up coordination state

        Ok(())
    }

    /// Shutdown the registry manager
    pub async fn shutdown(&self) -> Result<(), PrimalError> {
        info!("Shutting down ecosystem registry manager");

        // Cancel all background tasks
        if let Some(token) = self.shutdown_token.lock().await.as_ref() {
            token.cancel();
        }

        // Deregister all services
        let services_to_deregister = {
            let registry_state = self.registry_state.read().await;
            registry_state
                .registered_services
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        };

        for service_id in services_to_deregister {
            let deregister_url = format!(
                "{}/api/v1/services/deregister/{}",
                &self.config.songbird_endpoint, service_id
            );
            let _ = self.http_client.delete(&deregister_url).send().await;
        }

        info!("Ecosystem registry manager shutdown completed");
        Ok(())
    }
}
