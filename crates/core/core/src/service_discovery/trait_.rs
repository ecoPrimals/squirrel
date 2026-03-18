// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service discovery trait and interface
//!
//! This module defines the core service discovery trait that all implementations
//! must follow.

use async_trait::async_trait;

use super::types::{
    ServiceDefinition, ServiceHealthStatus, ServiceQuery, ServiceStats, ServiceType,
};
use crate::error::CoreResult;

/// Service discovery trait
///
/// This trait defines the core interface for service discovery implementations.
/// It provides methods for service registration, discovery, health monitoring,
/// and statistics collection.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use squirrel_core::{CoreResult, InMemoryServiceDiscovery, ServiceDiscovery, ServiceDefinition, ServiceType};
///
/// # async fn example() -> CoreResult<()> {
/// let discovery = Arc::new(InMemoryServiceDiscovery::new());
///
/// // Register a service
/// let service = ServiceDefinition::new(
///     "ai-service-1".to_string(),
///     "AI Service".to_string(),
///     ServiceType::AI,
///     vec![]
/// );
///
/// discovery.register_service(service).await?;
///
/// // Query services
/// let services = discovery.get_active_services().await?;
/// println!("Found {} active services", services.len());
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Register a service
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
    /// # use squirrel_core::{CoreResult, ServiceDefinition, ServiceDiscovery, ServiceType};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let service = ServiceDefinition::new(
    ///     "my-service".to_string(),
    ///     "My Service".to_string(),
    ///     ServiceType::AI,
    ///     vec![]
    /// );
    /// discovery.register_service(service).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn register_service(&self, service: ServiceDefinition) -> CoreResult<()>;

    /// Deregister a service
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
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// discovery.deregister_service("my-service").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn deregister_service(&self, service_id: &str) -> CoreResult<()>;

    /// Discover services based on query
    ///
    /// # Arguments
    ///
    /// * `query` - The service query with filtering and sorting criteria
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - List of matching services
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery, ServiceQuery, ServiceType};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let query = ServiceQuery::new()
    ///     .with_service_type(ServiceType::AI)
    ///     .limit(10);
    /// let services = discovery.discover_services(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn discover_services(&self, query: ServiceQuery) -> CoreResult<Vec<ServiceDefinition>>;

    /// Get all active services
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - List of all active services
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let services = discovery.get_active_services().await?;
    /// println!("Found {} active services", services.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_active_services(&self) -> CoreResult<Vec<ServiceDefinition>>;

    /// Get service by ID
    ///
    /// # Arguments
    ///
    /// * `service_id` - The unique identifier of the service
    ///
    /// # Returns
    ///
    /// `CoreResult<Option<ServiceDefinition>>` - Service if found, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// if let Some(service) = discovery.get_service("my-service").await? {
    ///     println!("Found service: {}", service.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_service(&self, service_id: &str) -> CoreResult<Option<ServiceDefinition>>;

    /// Update service health
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
    /// # use squirrel_core::service_discovery::ServiceHealthStatus;
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// discovery.update_service_health("my-service", ServiceHealthStatus::Unhealthy).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealthStatus,
    ) -> CoreResult<()>;

    /// Send heartbeat for service
    ///
    /// This method updates the last heartbeat timestamp for a service,
    /// indicating that it's still active.
    ///
    /// # Arguments
    ///
    /// * `service_id` - The unique identifier of the service
    ///
    /// # Returns
    ///
    /// `CoreResult<()>` - Success or error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// discovery.heartbeat("my-service").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn heartbeat(&self, service_id: &str) -> CoreResult<()>;

    /// Get service statistics
    ///
    /// # Returns
    ///
    /// `CoreResult<ServiceStats>` - Service statistics
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let stats = discovery.get_service_stats().await?;
    /// println!("Total services: {}", stats.total_services);
    /// println!("Healthy services: {}", stats.healthy_services);
    /// println!("Availability: {:.2}%", stats.availability_percentage());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_service_stats(&self) -> CoreResult<ServiceStats>;

    /// Get services by type
    ///
    /// # Arguments
    ///
    /// * `service_type` - The service type to filter by
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - List of services of the specified type
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery, ServiceType};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let ai_services = discovery.get_services_by_type(ServiceType::AI).await?;
    /// println!("Found {} AI services", ai_services.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_services_by_type(
        &self,
        service_type: ServiceType,
    ) -> CoreResult<Vec<ServiceDefinition>>;

    /// Get services by capability
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to filter by
    ///
    /// # Returns
    ///
    /// `CoreResult<Vec<ServiceDefinition>>` - List of services with the specified capability
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use squirrel_core::{CoreResult, ServiceDiscovery};
    /// # async fn example(discovery: &dyn ServiceDiscovery) -> CoreResult<()> {
    /// let chat_services = discovery.get_services_by_capability("chat").await?;
    /// println!("Found {} services with chat capability", chat_services.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_services_by_capability(
        &self,
        capability: &str,
    ) -> CoreResult<Vec<ServiceDefinition>>;
}
