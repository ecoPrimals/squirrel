//! Service discovery operations for the ecosystem registry manager

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, timeout};
use tracing::{debug, error, info, warn};

use super::types::{DiscoveredService, EcosystemRegistryEvent, ServiceHealthStatus};
use crate::ecosystem::EcosystemPrimalType;
use crate::error::PrimalError;

/// Service discovery operations
pub struct DiscoveryOps;

impl DiscoveryOps {
    /// Discover services through Songbird service mesh
    pub async fn discover_services(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        event_publisher: &broadcast::Sender<EcosystemRegistryEvent>,
        http_client: &reqwest::Client,
        songbird_endpoint: &str,
    ) -> Result<(), PrimalError> {
        debug!("Starting service discovery from Songbird");

        let discovery_url = format!("{}/api/v1/services/discover", songbird_endpoint);

        let response = timeout(
            Duration::from_secs(30),
            http_client.get(&discovery_url).send(),
        )
        .await
        .map_err(|_| PrimalError::NetworkError("Service discovery request timed out".to_string()))?
        .map_err(|e| PrimalError::NetworkError(format!("Failed to connect to Songbird: {}", e)))?;

        if !response.status().is_success() {
            return Err(PrimalError::NetworkError(format!(
                "Service discovery failed with status: {}",
                response.status()
            )));
        }

        let discovery_data: serde_json::Value = response.json().await.map_err(|e| {
            PrimalError::Internal(format!("Failed to parse discovery response: {}", e))
        })?;

        let services = discovery_data
            .get("services")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                PrimalError::Internal("Invalid discovery response format".to_string())
            })?;

        let mut discovered_count = 0;
        let mut updated_count = 0;

        for service_data in services {
            match Self::parse_discovered_service(service_data) {
                Ok(service) => {
                    let service_id = service.service_id.clone();
                    let primal_type = service.primal_type;
                    let endpoint = service.endpoint.clone();
                    let capabilities = service.capabilities.clone();

                    let mut registry = service_registry.write().await;
                    let is_new_service = !registry.contains_key(&service_id);

                    registry.insert(service_id.clone(), service);

                    if is_new_service {
                        discovered_count += 1;
                        let _ = event_publisher.send(EcosystemRegistryEvent::ServiceDiscovered {
                            service_id,
                            primal_type,
                            endpoint,
                            capabilities,
                        });
                    } else {
                        updated_count += 1;
                    }
                }
                Err(e) => {
                    warn!("Failed to parse discovered service: {}", e);
                }
            }
        }

        info!(
            "Service discovery completed: {} new services, {} updated services",
            discovered_count, updated_count
        );

        Ok(())
    }

    /// Parse discovered service data
    fn parse_discovered_service(
        data: &serde_json::Value,
    ) -> Result<DiscoveredService, PrimalError> {
        let service_id = data
            .get("service_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PrimalError::Internal("Missing service_id in discovery data".to_string())
            })?;

        let primal_type_str = data
            .get("primal_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PrimalError::Internal("Missing primal_type in discovery data".to_string())
            })?;

        let primal_type = EcosystemPrimalType::from_str(primal_type_str)
            .map_err(|e| PrimalError::Internal(format!("Invalid primal_type: {}", e)))?;

        let endpoint = data
            .get("endpoint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PrimalError::Internal("Missing endpoint in discovery data".to_string())
            })?;

        let health_endpoint = data
            .get("health_endpoint")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}/health", endpoint));

        let api_version = data
            .get("api_version")
            .and_then(|v| v.as_str())
            .unwrap_or("v1");

        let capabilities = data
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["basic".to_string()]);

        let metadata = data
            .get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        Ok(DiscoveredService {
            service_id: service_id.to_string(),
            primal_type,
            endpoint: endpoint.to_string(),
            health_endpoint,
            api_version: api_version.to_string(),
            capabilities,
            metadata,
            discovered_at: Utc::now(),
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
        })
    }

    /// Perform service discovery with error handling
    pub async fn perform_service_discovery(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        event_publisher: &broadcast::Sender<EcosystemRegistryEvent>,
        http_client: &reqwest::Client,
        songbird_endpoint: &str,
    ) -> Result<(), PrimalError> {
        match Self::discover_services(
            service_registry,
            event_publisher,
            http_client,
            songbird_endpoint,
        )
        .await
        {
            Ok(()) => {
                debug!("Service discovery completed successfully");
                Ok(())
            }
            Err(e) => {
                error!("Service discovery failed: {}", e);
                Err(e)
            }
        }
    }

    /// Start discovery background task
    pub async fn start_discovery_task(
        service_registry: Arc<RwLock<HashMap<String, DiscoveredService>>>,
        event_publisher: broadcast::Sender<EcosystemRegistryEvent>,
        http_client: reqwest::Client,
        songbird_endpoint: String,
        discovery_interval: Duration,
        shutdown_token: tokio_util::sync::CancellationToken,
    ) {
        let mut interval = interval(discovery_interval);

        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    debug!("Discovery task shutting down");
                    break;
                }
                _ = interval.tick() => {
                    if let Err(e) = Self::perform_service_discovery(
                        &service_registry,
                        &event_publisher,
                        &http_client,
                        &songbird_endpoint,
                    ).await {
                        error!("Service discovery failed: {}", e);
                    }
                }
            }
        }
    }
}
