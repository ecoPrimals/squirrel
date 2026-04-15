// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service discovery client
//!
//! This module provides a client interface for interacting with service discovery
//! implementations. It offers high-level methods for finding services with various
//! criteria and load balancing support.

use std::sync::Arc;

use super::trait_::ServiceDiscovery;
use super::types::{ServiceDefinition, ServiceHealthStatus, ServiceQuery, ServiceType};
use crate::error::CoreResult;

/// Service discovery client for making requests
///
/// This client provides convenient methods for finding services based on various
/// criteria such as type, capability, and load balancing requirements.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient, ServiceType};
///
/// # async fn example() -> CoreResult<()> {
/// let discovery = Arc::new(InMemoryServiceDiscovery::new());
/// let client: ServiceDiscoveryClient<InMemoryServiceDiscovery> =
///     ServiceDiscoveryClient::new(discovery);
///
/// // Find a service by type
/// if let Some(service) = client.find_service_by_type(ServiceType::AI).await? {
///     println!("Found AI service: {}", service.name);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ServiceDiscoveryClient<D: ServiceDiscovery> {
    /// Backing discovery implementation used for register and query operations.
    pub discovery: Arc<D>,
}

impl<D: ServiceDiscovery> ServiceDiscoveryClient<D> {
    /// Create new client with discovery backend
    ///
    /// # Arguments
    ///
    /// * `discovery` - The service discovery implementation to use
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use squirrel_core::{ServiceDiscoveryClient, InMemoryServiceDiscovery};
    ///
    /// let discovery = Arc::new(InMemoryServiceDiscovery::new());
    /// let client = ServiceDiscoveryClient::new(discovery);
    /// ```
    pub const fn new(discovery: Arc<D>) -> Self {
        Self { discovery }
    }

    /// Find service by type
    ///
    /// Returns the first service found of the specified type.
    ///
    /// # Arguments
    ///
    /// * `service_type` - The type of service to find
    ///
    /// # Returns
    ///
    /// `CoreResult<Option<ServiceDefinition>>` - The first service of the specified type, if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient, ServiceType};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// if let Some(service) = client.find_service_by_type(ServiceType::AI).await? {
    ///     println!("Found AI service: {}", service.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_service_by_type(
        &self,
        service_type: ServiceType,
    ) -> CoreResult<Option<ServiceDefinition>> {
        let services = self.discovery.get_services_by_type(service_type).await?;
        Ok(services.into_iter().next())
    }

    /// Find service by capability
    ///
    /// Returns the first service found with the specified capability.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// `CoreResult<Option<ServiceDefinition>>` - The first service with the specified capability, if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// if let Some(service) = client.find_service_by_capability("chat").await? {
    ///     println!("Found chat service: {}", service.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_service_by_capability(
        &self,
        capability: &str,
    ) -> CoreResult<Option<ServiceDefinition>> {
        let services = self
            .discovery
            .get_services_by_capability(capability)
            .await?;
        Ok(services.into_iter().next())
    }

    /// Find services with load balancing
    ///
    /// Returns all healthy services of the specified type, suitable for load balancing.
    ///
    /// # Arguments
    ///
    /// * `service_type` - The type of service to find
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - All healthy services of the specified type
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient, ServiceType};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// let services = client.find_services_for_load_balancing(ServiceType::AI).await?;
    /// println!("Found {} AI services for load balancing", services.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_services_for_load_balancing(
        &self,
        service_type: ServiceType,
    ) -> CoreResult<Vec<ServiceDefinition>> {
        let query = ServiceQuery::new()
            .with_service_type(service_type)
            .with_health_status(ServiceHealthStatus::Healthy);

        self.discovery.discover_services(query).await
    }

    /// Find best service for capability (highest weight or most recent)
    ///
    /// Returns the best service with the specified capability, based on endpoint
    /// weight (if available) or last heartbeat time.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for
    ///
    /// # Returns
    ///
    /// `CoreResult<Option<ServiceDefinition>>` - The best service with the specified capability, if found
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// if let Some(service) = client.find_best_service_for_capability("chat").await? {
    ///     println!("Found best chat service: {}", service.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_best_service_for_capability(
        &self,
        capability: &str,
    ) -> CoreResult<Option<ServiceDefinition>> {
        let services = self
            .discovery
            .get_services_by_capability(capability)
            .await?;

        if services.is_empty() {
            return Ok(None);
        }

        // Sort by weight (if available) or by last heartbeat
        let mut sorted_services = services;
        sorted_services.sort_by(|a, b| {
            // First, compare by primary endpoint weight
            let a_weight = a.primary_endpoint().and_then(|e| e.weight).unwrap_or(0.0);
            let b_weight = b.primary_endpoint().and_then(|e| e.weight).unwrap_or(0.0);

            if (a_weight - b_weight).abs() > f32::EPSILON {
                return b_weight
                    .partial_cmp(&a_weight)
                    .unwrap_or(std::cmp::Ordering::Equal);
            }

            // If weights are equal, compare by last heartbeat (more recent is better)
            b.last_heartbeat.cmp(&a.last_heartbeat)
        });

        Ok(sorted_services.into_iter().next())
    }

    /// Find services with metadata filter
    ///
    /// Returns services that match the specified metadata filter.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Key-value pairs that services must match
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - Services matching the metadata filter
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient};
    /// # use std::collections::HashMap;
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// let mut metadata = HashMap::new();
    /// metadata.insert("region".to_string(), "us-east-1".to_string());
    ///
    /// let services = client.find_services_with_metadata(metadata).await?;
    /// println!("Found {} services in us-east-1", services.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_services_with_metadata(
        &self,
        metadata: std::collections::HashMap<String, String>,
    ) -> CoreResult<Vec<ServiceDefinition>> {
        let mut query = ServiceQuery::new();
        for (key, value) in metadata {
            query = query.with_metadata(key, value);
        }

        self.discovery.discover_services(query).await
    }

    /// Find services with multiple capabilities
    ///
    /// Returns services that have all the specified capabilities.
    ///
    /// # Arguments
    ///
    /// * `capabilities` - List of capabilities that services must have
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - Services with all specified capabilities
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// let capabilities = vec!["chat".to_string(), "translate".to_string()];
    /// let services = client.find_services_with_capabilities(capabilities).await?;
    /// println!("Found {} services with chat and translate capabilities", services.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if the discovery backend returns an error.
    pub async fn find_services_with_capabilities(
        &self,
        capabilities: Vec<String>,
    ) -> CoreResult<Vec<ServiceDefinition>> {
        let mut query = ServiceQuery::new();
        for capability in capabilities {
            query = query.with_capability(capability);
        }

        self.discovery.discover_services(query).await
    }

    /// Get service statistics
    ///
    /// Returns statistics about all services in the discovery system.
    ///
    /// # Returns
    ///
    /// `CoreResult<ServiceStats>` - Service statistics
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscoveryClient};
    /// # async fn example(client: &ServiceDiscoveryClient<InMemoryServiceDiscovery>) -> CoreResult<()> {
    /// let stats = client.get_stats().await?;
    /// println!("Total services: {}", stats.total_services);
    /// println!("Healthy services: {}", stats.healthy_services);
    /// println!("Availability: {:.2}%", stats.availability_percentage());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CoreError`] if statistics cannot be read.
    pub async fn get_stats(&self) -> CoreResult<super::types::ServiceStats> {
        self.discovery.get_service_stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::super::memory::InMemoryServiceDiscovery;
    use super::super::types::ServiceEndpoint;
    use super::*;

    use std::collections::HashMap;

    #[tokio::test]
    async fn test_find_service_by_type() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let service = ServiceDefinition::new(
            "ai-service".to_string(),
            "AI Service".to_string(),
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

        let found = client
            .find_service_by_type(ServiceType::AI)
            .await
            .expect("should succeed");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").id, "ai-service");
    }

    #[tokio::test]
    async fn test_find_service_by_capability() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let service = ServiceDefinition::new(
            "chat-service".to_string(),
            "Chat Service".to_string(),
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

        let found = client
            .find_service_by_capability("chat")
            .await
            .expect("should succeed");
        assert!(found.is_some());
        assert_eq!(found.expect("should succeed").id, "chat-service");
    }

    #[tokio::test]
    async fn test_find_services_for_load_balancing() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let service1 = ServiceDefinition::new(
            "ai-service-1".to_string(),
            "AI Service 1".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        );
        let service2 = ServiceDefinition::new(
            "ai-service-2".to_string(),
            "AI Service 2".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8081".to_string(),
                "http".to_string(),
                8081,
            )],
        );
        discovery
            .register_service(service1)
            .await
            .expect("should succeed");
        discovery
            .register_service(service2)
            .await
            .expect("should succeed");

        let found = client
            .find_services_for_load_balancing(ServiceType::AI)
            .await
            .expect("should succeed");
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_find_best_service_for_capability() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let endpoint1 =
            ServiceEndpoint::new("http://service1".to_string(), "http".to_string(), 8080)
                .with_weight(0.5);

        let endpoint2 =
            ServiceEndpoint::new("http://service2".to_string(), "http".to_string(), 8080)
                .with_weight(0.8);

        let service1 = ServiceDefinition::new(
            "chat-service-1".to_string(),
            "Chat Service 1".to_string(),
            ServiceType::AI,
            vec![endpoint1],
        )
        .with_capability("chat".to_string());

        let service2 = ServiceDefinition::new(
            "chat-service-2".to_string(),
            "Chat Service 2".to_string(),
            ServiceType::AI,
            vec![endpoint2],
        )
        .with_capability("chat".to_string());

        discovery
            .register_service(service1)
            .await
            .expect("should succeed");
        discovery
            .register_service(service2)
            .await
            .expect("should succeed");

        let best = client
            .find_best_service_for_capability("chat")
            .await
            .expect("should succeed");
        assert!(best.is_some());
        assert_eq!(best.expect("should succeed").id, "chat-service-2"); // Higher weight
    }

    #[tokio::test]
    async fn test_find_services_with_metadata() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let service = ServiceDefinition::new(
            "regional-service".to_string(),
            "Regional Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        )
        .with_metadata("region".to_string(), "us-east-1".to_string());

        discovery
            .register_service(service)
            .await
            .expect("should succeed");

        let mut metadata = HashMap::new();
        metadata.insert("region".to_string(), "us-east-1".to_string());
        let services = client
            .find_services_with_metadata(metadata)
            .await
            .expect("should succeed");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "regional-service");
    }

    #[tokio::test]
    async fn test_find_services_with_capabilities() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

        let service = ServiceDefinition::new(
            "multi-service".to_string(),
            "Multi Service".to_string(),
            ServiceType::AI,
            vec![ServiceEndpoint::new(
                "http://localhost:8080".to_string(),
                "http".to_string(),
                8080,
            )],
        )
        .with_capability("chat".to_string())
        .with_capability("translate".to_string());

        discovery
            .register_service(service)
            .await
            .expect("should succeed");

        let capabilities = vec!["chat".to_string(), "translate".to_string()];
        let services = client
            .find_services_with_capabilities(capabilities)
            .await
            .expect("should succeed");
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "multi-service");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let client = ServiceDiscoveryClient::new(discovery.clone());

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

        let stats = client.get_stats().await.expect("should succeed");
        assert_eq!(stats.total_services, 1);
        assert_eq!(stats.healthy_services, 1);
    }
}
