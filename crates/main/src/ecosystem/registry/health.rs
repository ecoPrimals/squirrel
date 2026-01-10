//! Health monitoring operations for the ecosystem registry manager

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, timeout};
use tracing::debug;

use super::config::HealthConfig;
use super::types::{
    DiscoveredService, EcosystemRegistryEvent, HealthCheckResult, ServiceHealthStatus,
};

/// Health monitoring operations
pub struct HealthMonitor;

impl HealthMonitor {
    /// Perform health checks on discovered services
    pub async fn perform_health_checks(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        event_publisher: &broadcast::Sender<EcosystemRegistryEvent>,
        http_client: &reqwest::Client,
        health_config: &HealthConfig,
    ) {
        let services = {
            let registry = service_registry.read().await;
            registry.clone()
        };

        for (service_id, service) in services {
            let health_result =
                Self::check_service_health(http_client, &service, health_config).await;

            // Update service health status
            {
                let mut registry = service_registry.write().await;
                if let Some(service) = registry.get_mut(&service_id) {
                    let old_status = service.health_status.clone();
                    service.health_status = health_result.status;
                    service.last_health_check = Some(Utc::now());

                    if old_status != service.health_status {
                        let _ =
                            event_publisher.send(EcosystemRegistryEvent::ServiceHealthChanged {
                                service_id: service_id.clone().into(), // Convert String to Arc<str>
                                primal_type: service.primal_type,
                                old_status,
                                new_status: service.health_status.clone(),
                            });
                    }
                }
            }
        }
    }

    /// Check health of a single service
    pub async fn check_service_health(
        http_client: &reqwest::Client,
        service: &DiscoveredService,
        health_config: &HealthConfig,
    ) -> HealthCheckResult {
        let start_time = Instant::now();

        let result = timeout(
            health_config.timeout,
            http_client.get(&*service.health_endpoint).send(), // Dereference Arc<str> to &str
        )
        .await;

        let processing_time = start_time.elapsed();

        match result {
            Ok(Ok(response)) => {
                let status = if response.status().is_success() {
                    ServiceHealthStatus::Healthy
                } else {
                    ServiceHealthStatus::Unhealthy
                };

                HealthCheckResult {
                    status,
                    processing_time,
                    error: None,
                }
            }
            Ok(Err(e)) => HealthCheckResult {
                status: ServiceHealthStatus::Unhealthy,
                processing_time,
                error: Some(e.to_string()),
            },
            Err(_) => HealthCheckResult {
                status: ServiceHealthStatus::Unhealthy,
                processing_time,
                error: Some("Health check timed out".to_string()),
            },
        }
    }

    /// Start health monitoring background task
    pub async fn start_health_monitoring_task(
        service_registry: Arc<RwLock<HashMap<String, DiscoveredService>>>,
        event_publisher: broadcast::Sender<EcosystemRegistryEvent>,
        http_client: reqwest::Client,
        health_config: HealthConfig,
        shutdown_token: tokio_util::sync::CancellationToken,
    ) {
        let mut interval = interval(health_config.check_interval);

        loop {
            tokio::select! {
                () = shutdown_token.cancelled() => {
                    debug!("Health monitoring task shutting down");
                    break;
                }
                _ = interval.tick() => {
                    Self::perform_health_checks(
                        &service_registry,
                        &event_publisher,
                        &http_client,
                        &health_config,
                    ).await;
                }
            }
        }
    }

    /// Get health status of all services
    pub async fn get_health_status(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
    ) -> HashMap<String, ServiceHealthStatus> {
        let registry = service_registry.read().await;
        registry
            .iter()
            .map(|(id, service)| (id.clone(), service.health_status.clone()))
            .collect()
    }

    /// Get healthy services count
    pub async fn get_healthy_services_count(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
    ) -> usize {
        let registry = service_registry.read().await;
        registry
            .values()
            .filter(|s| s.health_status == ServiceHealthStatus::Healthy)
            .count()
    }

    /// Check if a specific service is healthy
    pub async fn is_service_healthy(
        service_registry: &Arc<RwLock<HashMap<String, DiscoveredService>>>,
        service_id: &str,
    ) -> bool {
        let registry = service_registry.read().await;
        registry
            .get(service_id)
            .is_some_and(|s| s.health_status == ServiceHealthStatus::Healthy)
    }
}
