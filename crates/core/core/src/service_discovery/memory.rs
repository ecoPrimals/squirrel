// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! In-memory service discovery implementation
//!
//! This module provides a simple in-memory implementation of the service discovery
//! interface. It stores services in a `HashMap` and handles expiration based on
//! heartbeat timeouts.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::ServiceDiscovery;
// Removed: use squirrel_mcp_config::get_service_endpoints;
use crate::CoreResult;
use crate::error::CoreError;
use crate::service_discovery::types::{
    ServiceDefinition, ServiceHealthStatus, ServiceQuery, ServiceStats, ServiceType,
};

// Define ServiceInstance locally if not available elsewhere
/// Lightweight runtime view of a single registered service endpoint.
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    /// Stable service identifier matching the registry key.
    pub service_id: String,
    /// Primary contact URL or socket for this instance.
    pub endpoint: String,
    /// Last reported health classification.
    pub status: ServiceHealthStatus,
    /// Time of the last successful heartbeat or health update.
    pub last_heartbeat: chrono::DateTime<Utc>,
}

// Define ServiceRegistration locally if not available elsewhere
/// Record tying a service id to its full definition at registration time.
#[derive(Debug, Clone)]
pub struct ServiceRegistration {
    /// Service identifier used for lookup and updates.
    pub service_id: String,
    /// Complete service definition including endpoints and capabilities.
    pub definition: ServiceDefinition,
    /// Wall-clock time when this registration was recorded.
    pub timestamp: chrono::DateTime<Utc>,
}

/// In-memory service discovery implementation
///
/// This implementation stores services in memory using a `HashMap`. It provides
/// automatic cleanup of expired services based on heartbeat timeouts.
///
/// # Examples
///
/// ```rust
/// use squirrel_core::InMemoryServiceDiscovery;
/// use std::time::Duration;
///
/// // Create with default heartbeat timeout (30 seconds)
/// let discovery = InMemoryServiceDiscovery::new();
///
/// // Create with custom heartbeat timeout
/// let discovery = InMemoryServiceDiscovery::with_heartbeat_timeout(Duration::from_secs(60));
/// ```
pub struct InMemoryServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
    heartbeat_timeout: Duration,
}

impl InMemoryServiceDiscovery {
    /// Create new in-memory service discovery
    ///
    /// Uses a default heartbeat timeout of 30 seconds.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel_core::InMemoryServiceDiscovery;
    ///
    /// let discovery = InMemoryServiceDiscovery::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: Duration::from_secs(30),
        }
    }

    /// Create with custom heartbeat timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - The heartbeat timeout duration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use squirrel_core::InMemoryServiceDiscovery;
    /// use std::time::Duration;
    ///
    /// let discovery = InMemoryServiceDiscovery::with_heartbeat_timeout(Duration::from_secs(60));
    /// ```
    #[must_use]
    pub fn with_heartbeat_timeout(timeout: Duration) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: timeout,
        }
    }

    /// Check if service is expired based on heartbeat
    fn is_service_expired(&self, service: &ServiceDefinition) -> bool {
        let now = Utc::now();
        let duration_since_heartbeat = now.signed_duration_since(service.last_heartbeat);
        duration_since_heartbeat
            > chrono::Duration::from_std(self.heartbeat_timeout).unwrap_or_default()
    }

    /// Clean up expired services
    ///
    /// This method removes services that haven't sent a heartbeat within the
    /// configured timeout period.
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<String>>` - List of expired service IDs that were removed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery};
    /// # async fn example() -> CoreResult<()> {
    /// let discovery = InMemoryServiceDiscovery::new();
    /// let expired_services = discovery.cleanup_expired_services().await?;
    /// println!("Removed {} expired services", expired_services.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the registry lock cannot be acquired or cleanup fails.
    pub async fn cleanup_expired_services(&self) -> CoreResult<Vec<String>> {
        let expired_ids: Vec<String> = {
            let services = self.services.read().await;
            services
                .iter()
                .filter_map(|(id, service)| self.is_service_expired(service).then_some(id.clone()))
                .collect()
        };

        let mut expired_services = Vec::new();
        {
            let mut services = self.services.write().await;
            for id in expired_ids {
                if services.remove(&id).is_some() {
                    warn!("Service {} expired, removing from registry", id);
                    expired_services.push(id);
                }
            }
        }

        if !expired_services.is_empty() {
            info!("Cleaned up {} expired services", expired_services.len());
        }

        Ok(expired_services)
    }

    /// Apply query filters and sorting
    fn apply_query_filters(
        services: Vec<ServiceDefinition>,
        query: &ServiceQuery,
    ) -> Vec<ServiceDefinition> {
        let mut filtered: Vec<ServiceDefinition> = services
            .into_iter()
            .filter(|service| query.matches(service))
            .collect();

        // Apply sorting
        query.sort_services(&mut filtered);

        // Apply pagination
        query.paginate_services(filtered)
    }
}

impl Default for InMemoryServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceDiscovery for InMemoryServiceDiscovery {
    async fn register_service(&self, service: ServiceDefinition) -> CoreResult<()> {
        // Validate service definition
        service.validate()?;

        {
            let mut services = self.services.write().await;

            // Check if service already exists
            if services.contains_key(&service.id) {
                warn!("Service {} already exists, updating", service.id);
            } else {
                info!("Registering new service: {} ({})", service.name, service.id);
            }

            services.insert(service.id.clone(), service);
        }
        Ok(())
    }

    async fn deregister_service(&self, service_id: &str) -> CoreResult<()> {
        {
            let mut services = self.services.write().await;

            if services.remove(service_id).is_some() {
                info!("Deregistered service: {}", service_id);
            } else {
                warn!(
                    "Attempted to deregister non-existent service: {}",
                    service_id
                );
            }
        }

        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> CoreResult<Vec<ServiceDefinition>> {
        debug!("Discovering services with query: {:?}", query);

        let results: Vec<ServiceDefinition> = {
            let services = self.services.read().await;
            services
                .values()
                .filter(|service| {
                    if self.is_service_expired(service) {
                        debug!("Skipping expired service: {}", service.id);
                        false
                    } else {
                        true
                    }
                })
                .cloned()
                .collect()
        };

        let filtered = Self::apply_query_filters(results, &query);

        debug!("Found {} services matching query", filtered.len());
        Ok(filtered)
    }

    async fn get_active_services(&self) -> CoreResult<Vec<ServiceDefinition>> {
        let results: Vec<ServiceDefinition> = {
            let services = self.services.read().await;
            services
                .values()
                .filter(|service| !self.is_service_expired(service))
                .cloned()
                .collect()
        };

        debug!("Found {} active services", results.len());
        Ok(results)
    }

    async fn get_service(&self, service_id: &str) -> CoreResult<Option<ServiceDefinition>> {
        let service = {
            let services = self.services.read().await;
            services.get(service_id).cloned()
        };

        if let Some(ref service) = service
            && self.is_service_expired(service)
        {
            debug!("Service {} is expired", service_id);
            return Ok(None);
        }

        Ok(service)
    }

    async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> CoreResult<()> {
        {
            let mut services = self.services.write().await;

            if let Some(service) = services.get_mut(service_id) {
                let old_status = service.health_status.clone();
                service.health_status = health.clone();
                service.last_heartbeat = Utc::now();

                if old_status != health {
                    info!(
                        "Service {} health changed from {:?} to {:?}",
                        service_id, old_status, health
                    );
                }
            } else {
                return Err(CoreError::ServiceNotFound(service_id.to_string()));
            }
        }

        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> CoreResult<()> {
        {
            let mut services = self.services.write().await;

            if let Some(service) = services.get_mut(service_id) {
                service.last_heartbeat = Utc::now();
                debug!("Updated heartbeat for service: {}", service_id);
            } else {
                return Err(CoreError::ServiceNotFound(service_id.to_string()));
            }
        }

        Ok(())
    }

    async fn get_service_stats(&self) -> CoreResult<ServiceStats> {
        let services = self.services.read().await;
        let active_services: Vec<ServiceDefinition> = services
            .values()
            .filter(|service| !self.is_service_expired(service))
            .cloned()
            .collect();
        drop(services);

        let mut stats = ServiceStats::new();
        stats.update_from_services(&active_services);

        Ok(stats)
    }

    async fn get_services_by_type(
        &self,
        service_type: ServiceType,
    ) -> CoreResult<Vec<ServiceDefinition>> {
        let query = ServiceQuery::new().with_service_type(service_type);
        self.discover_services(query).await
    }

    async fn get_services_by_capability(
        &self,
        capability: &str,
    ) -> CoreResult<Vec<ServiceDefinition>> {
        let query = ServiceQuery::new().with_capability(capability.to_string());
        self.discover_services(query).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::ServiceEndpoint;
    use super::*;

    use std::time::Duration;

    #[tokio::test]
    async fn test_register_and_get_service() {
        let discovery = InMemoryServiceDiscovery::new();
        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                std::env::var("MCP_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:8444".to_string()),
                "http".to_string(),
                std::env::var("MCP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8080),
            )],
        );

        discovery
            .register_service(service.clone())
            .await
            .expect("should succeed");

        let retrieved = discovery
            .get_service("test-service")
            .await
            .expect("should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("should succeed").id, "test-service");
    }

    #[tokio::test]
    async fn test_deregister_service() {
        let discovery = InMemoryServiceDiscovery::new();
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

        discovery
            .register_service(service)
            .await
            .expect("should succeed");
        discovery
            .deregister_service("test-service")
            .await
            .expect("should succeed");

        let retrieved = discovery
            .get_service("test-service")
            .await
            .expect("should succeed");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let discovery = InMemoryServiceDiscovery::new();
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

        discovery
            .register_service(service)
            .await
            .expect("should succeed");
        discovery
            .heartbeat("test-service")
            .await
            .expect("should succeed");

        let retrieved = discovery
            .get_service("test-service")
            .await
            .expect("should succeed");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_service_expiration() {
        let discovery =
            InMemoryServiceDiscovery::with_heartbeat_timeout(Duration::from_millis(100));
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

        discovery
            .register_service(service)
            .await
            .expect("should succeed");

        // Wait for service to expire
        tokio::time::sleep(Duration::from_millis(200)).await;

        let retrieved = discovery
            .get_service("test-service")
            .await
            .expect("should succeed");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired_services() {
        let discovery =
            InMemoryServiceDiscovery::with_heartbeat_timeout(Duration::from_millis(100));
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

        discovery
            .register_service(service)
            .await
            .expect("should succeed");

        // Wait for service to expire
        tokio::time::sleep(Duration::from_millis(200)).await;

        let expired = discovery
            .cleanup_expired_services()
            .await
            .expect("should succeed");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0], "test-service");
    }

    #[tokio::test]
    async fn test_get_services_by_type() {
        let discovery = InMemoryServiceDiscovery::new();
        let ai_service = ServiceDefinition::new(
            "ai-service".to_string(),
            "AI Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );
        let compute_service = ServiceDefinition::new(
            "compute-service".to_string(),
            "Compute Service".to_string(),
            ServiceType::Compute,
            vec![ServiceEndpoint::new(
                "http://localhost:8081".to_string(),
                "http".to_string(),
                8081,
            )],
        );

        discovery
            .register_service(ai_service)
            .await
            .expect("should succeed");
        discovery
            .register_service(compute_service)
            .await
            .expect("should succeed");

        let ai_services = discovery
            .get_services_by_type(ServiceType::AI)
            .await
            .expect("should succeed");
        assert_eq!(ai_services.len(), 1);
        assert_eq!(ai_services[0].id, "ai-service");
    }

    #[tokio::test]
    async fn test_get_services_by_capability() {
        let discovery = InMemoryServiceDiscovery::new();
        let service = ServiceDefinition::new(
            "test-service".to_string(),
            "Test Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        )
        .with_capability("chat".to_string());

        discovery
            .register_service(service)
            .await
            .expect("should succeed");

        let chat_services = discovery
            .get_services_by_capability("chat")
            .await
            .expect("should succeed");
        assert_eq!(chat_services.len(), 1);
        assert_eq!(chat_services[0].id, "test-service");
    }

    #[tokio::test]
    async fn test_service_stats() {
        let discovery = InMemoryServiceDiscovery::new();
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
        .with_health_status(ServiceHealthStatus::Unhealthy);

        discovery
            .register_service(service1)
            .await
            .expect("should succeed");
        discovery
            .register_service(service2)
            .await
            .expect("should succeed");

        let stats = discovery.get_service_stats().await.expect("should succeed");
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.healthy_services, 1);
        assert_eq!(stats.unhealthy_services, 1);
    }
}
