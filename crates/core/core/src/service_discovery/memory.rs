//! In-memory service discovery implementation
//!
//! This module provides a simple in-memory implementation of the service discovery
//! interface. It stores services in a HashMap and handles expiration based on
//! heartbeat timeouts.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::ServiceDiscovery;
use crate::error::CoreError;
use crate::service_discovery::types::{
    HealthStatus, ServiceDefinition, ServiceQuery, ServiceStats, ServiceType,
};
use crate::CoreResult;

// Define ServiceInstance locally if not available elsewhere
#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub service_id: String,
    pub endpoint: String,
    pub status: HealthStatus,
    pub last_heartbeat: chrono::DateTime<Utc>,
}

// Define ServiceRegistration locally if not available elsewhere
#[derive(Debug, Clone)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub definition: ServiceDefinition,
    pub timestamp: chrono::DateTime<Utc>,
}

/// In-memory service discovery implementation
///
/// This implementation stores services in memory using a HashMap. It provides
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
    /// # use squirrel_core::InMemoryServiceDiscovery;
    /// # async fn example() -> CoreResult<()> {
    /// let discovery = InMemoryServiceDiscovery::new();
    /// let expired_services = discovery.cleanup_expired_services().await?;
    /// println!("Removed {} expired services", expired_services.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cleanup_expired_services(&self) -> CoreResult<Vec<String>> {
        let mut services = self.services.write().await;
        let mut expired_services = Vec::new();

        services.retain(|id, service| {
            if self.is_service_expired(service) {
                expired_services.push(id.clone());
                warn!("Service {} expired, removing from registry", id);
                false
            } else {
                true
            }
        });

        if !expired_services.is_empty() {
            info!("Cleaned up {} expired services", expired_services.len());
        }

        Ok(expired_services)
    }

    /// Apply query filters and sorting
    fn apply_query_filters(
        &self,
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

        let mut services = self.services.write().await;

        // Check if service already exists
        if services.contains_key(&service.id) {
            warn!("Service {} already exists, updating", service.id);
        } else {
            info!("Registering new service: {} ({})", service.name, service.id);
        }

        services.insert(service.id.clone(), service);
        Ok(())
    }

    async fn deregister_service(&self, service_id: &str) -> CoreResult<()> {
        let mut services = self.services.write().await;

        if services.remove(service_id).is_some() {
            info!("Deregistered service: {}", service_id);
        } else {
            warn!(
                "Attempted to deregister non-existent service: {}",
                service_id
            );
        }

        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> CoreResult<Vec<ServiceDefinition>> {
        let services = self.services.read().await;
        let mut results = Vec::new();

        debug!("Discovering services with query: {:?}", query);

        for service in services.values() {
            // Skip expired services
            if self.is_service_expired(service) {
                debug!("Skipping expired service: {}", service.id);
                continue;
            }

            results.push(service.clone());
        }

        let filtered = self.apply_query_filters(results, &query);

        debug!("Found {} services matching query", filtered.len());
        Ok(filtered)
    }

    async fn get_active_services(&self) -> CoreResult<Vec<ServiceDefinition>> {
        let services = self.services.read().await;
        let results: Vec<ServiceDefinition> = services
            .values()
            .filter(|service| !self.is_service_expired(service))
            .cloned()
            .collect();

        debug!("Found {} active services", results.len());
        Ok(results)
    }

    async fn get_service(&self, service_id: &str) -> CoreResult<Option<ServiceDefinition>> {
        let services = self.services.read().await;
        let service = services.get(service_id).cloned();

        if let Some(ref service) = service {
            if self.is_service_expired(service) {
                debug!("Service {} is expired", service_id);
                return Ok(None);
            }
        }

        Ok(service)
    }

    async fn update_service_health(
        &self,
        service_id: &str,
        health: HealthStatus,
    ) -> CoreResult<()> {
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

        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> CoreResult<()> {
        let mut services = self.services.write().await;

        if let Some(service) = services.get_mut(service_id) {
            service.last_heartbeat = Utc::now();
            debug!("Updated heartbeat for service: {}", service_id);
        } else {
            return Err(CoreError::ServiceNotFound(service_id.to_string()));
        }

        Ok(())
    }

    async fn get_service_stats(&self) -> CoreResult<ServiceStats> {
        let services = self.services.read().await;
        let active_services: Vec<&ServiceDefinition> = services
            .values()
            .filter(|service| !self.is_service_expired(service))
            .collect();

        let mut stats = ServiceStats::new();
        let active_services_vec: Vec<ServiceDefinition> =
            active_services.into_iter().cloned().collect();

        stats.update_from_services(&active_services_vec);

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
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );

        discovery.register_service(service.clone()).await.unwrap();

        let retrieved = discovery.get_service("test-service").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-service");
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

        discovery.register_service(service).await.unwrap();
        discovery.deregister_service("test-service").await.unwrap();

        let retrieved = discovery.get_service("test-service").await.unwrap();
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

        discovery.register_service(service).await.unwrap();
        discovery.heartbeat("test-service").await.unwrap();

        let retrieved = discovery.get_service("test-service").await.unwrap();
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

        discovery.register_service(service).await.unwrap();

        // Wait for service to expire
        tokio::time::sleep(Duration::from_millis(200)).await;

        let retrieved = discovery.get_service("test-service").await.unwrap();
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

        discovery.register_service(service).await.unwrap();

        // Wait for service to expire
        tokio::time::sleep(Duration::from_millis(200)).await;

        let expired = discovery.cleanup_expired_services().await.unwrap();
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

        discovery.register_service(ai_service).await.unwrap();
        discovery.register_service(compute_service).await.unwrap();

        let ai_services = discovery
            .get_services_by_type(ServiceType::AI)
            .await
            .unwrap();
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

        discovery.register_service(service).await.unwrap();

        let chat_services = discovery.get_services_by_capability("chat").await.unwrap();
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
        .with_health_status(HealthStatus::Unhealthy);

        discovery.register_service(service1).await.unwrap();
        discovery.register_service(service2).await.unwrap();

        let stats = discovery.get_service_stats().await.unwrap();
        assert_eq!(stats.total_services, 2);
        assert_eq!(stats.healthy_services, 1);
        assert_eq!(stats.unhealthy_services, 1);
    }
}
