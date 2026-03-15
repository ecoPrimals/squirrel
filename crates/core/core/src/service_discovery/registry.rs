// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service registry for managing service lifecycle
//!
//! This module provides a registry implementation that manages the lifecycle of
//! local services, including registration, health updates, and heartbeat management.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use super::trait_::ServiceDiscovery;
use super::types::{HealthStatus, ServiceDefinition};
use crate::error::CoreResult;

/// Service registry for managing service lifecycle
///
/// This registry manages local services and their lifecycle, including automatic
/// heartbeat management and graceful shutdown.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use std::time::Duration;
/// use squirrel_core::{ServiceRegistry, InMemoryServiceDiscovery, ServiceDefinition, ServiceType};
///
/// # async fn example() -> CoreResult<()> {
/// let discovery = Arc::new(InMemoryServiceDiscovery::new());
/// let registry = ServiceRegistry::new(discovery);
///
/// // Register a local service
/// let service = ServiceDefinition::new(
///     "my-service".to_string(),
///     "My Service".to_string(),
///     ServiceType::AI,
///     vec![]
/// );
/// registry.register_local_service(service).await?;
///
/// // Start heartbeat loop
/// registry.start_heartbeat_loop().await?;
/// # Ok(())
/// # }
/// ```
pub struct ServiceRegistry {
    discovery: Arc<dyn ServiceDiscovery>,
    local_services: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
    heartbeat_interval: Duration,
}

impl ServiceRegistry {
    /// Create new service registry
    ///
    /// Uses a default heartbeat interval of 10 seconds.
    ///
    /// # Arguments
    ///
    /// * `discovery` - The service discovery implementation to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use squirrel_core::{ServiceRegistry, InMemoryServiceDiscovery};
    ///
    /// let discovery = Arc::new(InMemoryServiceDiscovery::new());
    /// let registry = ServiceRegistry::new(discovery);
    /// ```
    pub fn new(discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            discovery,
            local_services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_interval: Duration::from_secs(10),
        }
    }

    /// Create with custom heartbeat interval
    ///
    /// # Arguments
    ///
    /// * `discovery` - The service discovery implementation to use
    /// * `interval` - The heartbeat interval duration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use squirrel_core::{ServiceRegistry, InMemoryServiceDiscovery};
    ///
    /// let discovery = Arc::new(InMemoryServiceDiscovery::new());
    /// let registry = ServiceRegistry::with_heartbeat_interval(discovery, Duration::from_secs(5));
    /// ```
    pub fn with_heartbeat_interval(
        discovery: Arc<dyn ServiceDiscovery>,
        interval: Duration,
    ) -> Self {
        Self {
            discovery,
            local_services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_interval: interval,
        }
    }

    /// Register local service
    ///
    /// Registers a service both with the discovery system and locally for heartbeat management.
    ///
    /// # Arguments
    ///
    /// * `service` - The service definition to register
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{ServiceRegistry, ServiceDefinition, ServiceType};
    /// # async fn example(registry: &ServiceRegistry) -> CoreResult<()> {
    /// let service = ServiceDefinition::new(
    ///     "my-service".to_string(),
    ///     "My Service".to_string(),
    ///     ServiceType::AI,
    ///     vec![]
    /// );
    /// registry.register_local_service(service).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_local_service(&self, service: ServiceDefinition) -> CoreResult<()> {
        // Validate service before registration
        service.validate()?;

        // Register with discovery
        self.discovery.register_service(service.clone()).await?;

        // Store locally
        let mut local_services = self.local_services.write().await;
        local_services.insert(service.id.clone(), service);

        Ok(())
    }

    /// Deregister local service
    ///
    /// Removes a service from both the discovery system and local registry.
    ///
    /// # Arguments
    ///
    /// * `service_id` - The unique identifier of the service to deregister
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::ServiceRegistry;
    /// # async fn example(registry: &ServiceRegistry) -> CoreResult<()> {
    /// registry.deregister_local_service("my-service").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn deregister_local_service(&self, service_id: &str) -> CoreResult<()> {
        // Deregister from discovery
        self.discovery.deregister_service(service_id).await?;

        // Remove locally
        let mut local_services = self.local_services.write().await;
        local_services.remove(service_id);

        Ok(())
    }

    /// Update local service health
    ///
    /// Updates the health status of a local service both in the discovery system
    /// and in the local registry.
    ///
    /// # Arguments
    ///
    /// * `service_id` - The unique identifier of the service
    /// * `health` - The new health status
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{ServiceRegistry, HealthStatus};
    /// # async fn example(registry: &ServiceRegistry) -> CoreResult<()> {
    /// registry.update_local_service_health("my-service", HealthStatus::Unhealthy).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_local_service_health(
        &self,
        service_id: &str,
        health: HealthStatus,
    ) -> CoreResult<()> {
        // Update in discovery
        self.discovery
            .update_service_health(service_id, health.clone())
            .await?;

        // Update locally
        let mut local_services = self.local_services.write().await;
        if let Some(service) = local_services.get_mut(service_id) {
            service.health_status = health;
            service.last_heartbeat = Utc::now();
        }

        Ok(())
    }

    /// Get local services
    ///
    /// Returns a list of all locally registered services.
    ///
    /// # Returns
    ///
    /// `Vec<ServiceDefinition>` - List of local services
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::ServiceRegistry;
    /// # async fn example(registry: &ServiceRegistry) {
    /// let services = registry.get_local_services().await;
    /// println!("Found {} local services", services.len());
    /// # }
    /// ```
    pub async fn get_local_services(&self) -> Vec<ServiceDefinition> {
        let services = self.local_services.read().await;
        services.values().cloned().collect()
    }

    /// Start heartbeat for all local services
    ///
    /// Starts a background task that sends periodic heartbeats for all locally
    /// registered services to keep them active in the discovery system.
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::ServiceRegistry;
    /// # async fn example(registry: &ServiceRegistry) -> CoreResult<()> {
    /// registry.start_heartbeat_loop().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_heartbeat_loop(&self) -> CoreResult<()> {
        let discovery = self.discovery.clone();
        let local_services = self.local_services.clone();
        let interval = self.heartbeat_interval;

        info!("Starting heartbeat loop with interval: {:?}", interval);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            interval_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval_timer.tick().await;

                let services = local_services.read().await;
                let service_ids: Vec<String> = services.keys().cloned().collect();
                drop(services);

                for service_id in service_ids {
                    if let Err(e) = discovery.heartbeat(&service_id).await {
                        error!("Failed to send heartbeat for service {}: {}", service_id, e);
                    } else {
                        debug!("Sent heartbeat for service: {}", service_id);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop heartbeat and deregister all local services
    ///
    /// Gracefully shuts down the registry by deregistering all local services
    /// from the discovery system.
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::ServiceRegistry;
    /// # async fn example(registry: &ServiceRegistry) -> CoreResult<()> {
    /// registry.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&self) -> CoreResult<()> {
        info!("Shutting down service registry");

        let services = self.local_services.read().await;
        let service_ids: Vec<String> = services.keys().cloned().collect();
        drop(services);

        for service_id in service_ids {
            if let Err(e) = self.deregister_local_service(&service_id).await {
                error!(
                    "Failed to deregister service {} during shutdown: {}",
                    service_id, e
                );
            }
        }

        Ok(())
    }

    /// Get registry statistics
    ///
    /// Returns statistics about the local registry, including the number of
    /// registered services and their health status distribution.
    ///
    /// # Returns
    ///
    /// `RegistryStats` - Registry statistics
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::ServiceRegistry;
    /// # async fn example(registry: &ServiceRegistry) {
    /// let stats = registry.get_registry_stats().await;
    /// println!("Local services: {}", stats.total_services);
    /// println!("Healthy services: {}", stats.healthy_services);
    /// # }
    /// ```
    pub async fn get_registry_stats(&self) -> RegistryStats {
        let services = self.local_services.read().await;
        let all_services: Vec<ServiceDefinition> = services.values().cloned().collect();

        RegistryStats::from_services(&all_services)
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of local services
    pub total_services: usize,
    /// Number of healthy services
    pub healthy_services: usize,
    /// Number of unhealthy services
    pub unhealthy_services: usize,
    /// Number of degraded services
    pub degraded_services: usize,
    /// Number of unavailable services
    pub unavailable_services: usize,
    /// Services by type
    pub services_by_type: HashMap<String, usize>,
}

impl RegistryStats {
    /// Create statistics from a list of services
    pub fn from_services(services: &[ServiceDefinition]) -> Self {
        let total_services = services.len();
        let mut healthy_services = 0;
        let mut unhealthy_services = 0;
        let mut degraded_services = 0;
        let mut unavailable_services = 0;
        let mut services_by_type = HashMap::new();

        for service in services {
            match service.health_status {
                HealthStatus::Healthy => healthy_services += 1,
                HealthStatus::Unhealthy => unhealthy_services += 1,
                HealthStatus::Degraded => degraded_services += 1,
                HealthStatus::Unavailable => unavailable_services += 1,
            }

            *services_by_type
                .entry(service.service_type.as_str().to_string())
                .or_insert(0) += 1;
        }

        Self {
            total_services,
            healthy_services,
            unhealthy_services,
            degraded_services,
            unavailable_services,
            services_by_type,
        }
    }

    /// Get availability percentage
    pub fn availability_percentage(&self) -> f32 {
        if self.total_services == 0 {
            return 100.0;
        }

        (self.healthy_services as f32 / self.total_services as f32) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::super::memory::InMemoryServiceDiscovery;
    use super::super::types::{ServiceEndpoint, ServiceType};
    use super::*;

    use std::time::Duration;

    #[tokio::test]
    async fn test_register_local_service() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry = ServiceRegistry::new(discovery.clone());

        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        registry.register_local_service(service).await.unwrap();

        let local_services = registry.get_local_services().await;
        assert_eq!(local_services.len(), 1);
        assert_eq!(local_services[0].id, "test-service");

        // Verify it's also in discovery
        let discovered = discovery.get_service("test-service").await.unwrap();
        assert!(discovered.is_some());
    }

    #[tokio::test]
    async fn test_deregister_local_service() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry = ServiceRegistry::new(discovery.clone());

        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        registry.register_local_service(service).await.unwrap();
        registry
            .deregister_local_service("test-service")
            .await
            .unwrap();

        let local_services = registry.get_local_services().await;
        assert_eq!(local_services.len(), 0);

        // Verify it's removed from discovery
        let discovered = discovery.get_service("test-service").await.unwrap();
        assert!(discovered.is_none());
    }

    #[tokio::test]
    async fn test_update_local_service_health() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry = ServiceRegistry::new(discovery.clone());

        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        registry.register_local_service(service).await.unwrap();
        registry
            .update_local_service_health("test-service", HealthStatus::Unhealthy)
            .await
            .unwrap();

        let local_services = registry.get_local_services().await;
        assert_eq!(local_services[0].health_status, HealthStatus::Unhealthy);

        // Verify it's updated in discovery
        let discovered = discovery.get_service("test-service").await.unwrap();
        assert_eq!(discovered.unwrap().health_status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_get_registry_stats() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry = ServiceRegistry::new(discovery);

        let service1 = ServiceDefinition::new(
            "service1".to_string(),
            "Service 1".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        let service2 = ServiceDefinition::new(
            "service2".to_string(),
            "Service 2".to_string(),
            ServiceType::Compute,
            vec![ServiceEndpoint::new(
                "http://localhost:8081".to_string(),
                "http".to_string(),
                8081,
            )],
        )
        .with_health_status(HealthStatus::Unhealthy);

        registry.register_local_service(service1).await.unwrap();
        registry.register_local_service(service2).await.unwrap();

        let stats = registry.get_registry_stats().await;
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.healthy_services, 1);
        assert_eq!(stats.unhealthy_services, 1);
        assert_eq!(stats.services_by_type.get("ai").unwrap(), &1);
        assert_eq!(stats.services_by_type.get("compute").unwrap(), &1);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry = ServiceRegistry::new(discovery.clone());

        let service1 = ServiceDefinition::new(
            "service1".to_string(),
            "Service 1".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        let service2 = ServiceDefinition::new(
            "service2".to_string(),
            "Service 2".to_string(),
            ServiceType::Compute,
            vec![ServiceEndpoint::new(
                "http://localhost:8081".to_string(),
                "http".to_string(),
                8081,
            )],
        );

        registry.register_local_service(service1).await.unwrap();
        registry.register_local_service(service2).await.unwrap();

        registry.shutdown().await.unwrap();

        let local_services = registry.get_local_services().await;
        assert_eq!(local_services.len(), 0);

        // Verify services are removed from discovery
        let discovered1 = discovery.get_service("service1").await.unwrap();
        let discovered2 = discovery.get_service("service2").await.unwrap();
        assert!(discovered1.is_none());
        assert!(discovered2.is_none());
    }

    #[tokio::test]
    async fn test_heartbeat_loop() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let registry =
            ServiceRegistry::with_heartbeat_interval(discovery.clone(), Duration::from_millis(50));

        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        registry.register_local_service(service).await.unwrap();
        registry.start_heartbeat_loop().await.unwrap();

        // Wait for a few heartbeats
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Verify service is still active
        let discovered = discovery.get_service("test-service").await.unwrap();
        assert!(discovered.is_some());
    }
}
