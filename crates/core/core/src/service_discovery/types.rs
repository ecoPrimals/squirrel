// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Core data types for service discovery
//!
//! This module contains all the fundamental data structures used throughout
//! the service discovery system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{CoreError, CoreResult};

/// Universal service definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceDefinition {
    /// Unique service identifier
    pub id: String,
    /// Human-readable service name
    pub name: String,
    /// Service type (e.g., "ai", "compute", "storage", "security")
    pub service_type: ServiceType,
    /// Available endpoints
    pub endpoints: Vec<ServiceEndpoint>,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service metadata and tags
    pub metadata: HashMap<String, String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,
}

impl ServiceDefinition {
    /// Create a new service definition with required fields
    pub fn new(
        id: String,
        name: String,
        service_type: ServiceType,
        endpoints: Vec<ServiceEndpoint>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            service_type,
            endpoints,
            capabilities: Vec::new(),
            metadata: HashMap::new(),
            health_status: HealthStatus::Healthy,
            registered_at: now,
            last_heartbeat: now,
        }
    }

    /// Add a capability to the service
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add multiple capabilities to the service
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Add metadata to the service
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set health status
    pub fn with_health_status(mut self, status: HealthStatus) -> Self {
        self.health_status = status;
        self
    }

    /// Check if service has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// Check if service matches metadata filter
    pub fn matches_metadata(&self, filters: &HashMap<String, String>) -> bool {
        for (key, value) in filters {
            if self.metadata.get(key) != Some(value) {
                return false;
            }
        }
        true
    }

    /// Get primary endpoint
    pub fn primary_endpoint(&self) -> Option<&ServiceEndpoint> {
        self.endpoints
            .iter()
            .find(|e| e.primary)
            .or_else(|| self.endpoints.first())
    }

    /// Validate service definition
    pub fn validate(&self) -> CoreResult<()> {
        if self.id.is_empty() {
            return Err(CoreError::InvalidServiceConfig(
                "Service ID cannot be empty".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(CoreError::InvalidServiceConfig(
                "Service name cannot be empty".to_string(),
            ));
        }

        if self.endpoints.is_empty() {
            return Err(CoreError::InvalidServiceConfig(
                "Service must have at least one endpoint".to_string(),
            ));
        }

        // Validate endpoints
        for endpoint in &self.endpoints {
            endpoint.validate()?;
        }

        // Ensure only one primary endpoint
        let primary_count = self.endpoints.iter().filter(|e| e.primary).count();
        if primary_count > 1 {
            return Err(CoreError::InvalidServiceConfig(
                "Service can have only one primary endpoint".to_string(),
            ));
        }

        Ok(())
    }
}

/// Service type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceType {
    /// AI/ML services
    AI,
    /// Compute services
    Compute,
    /// Storage services
    Storage,
    /// Security services
    Security,
    /// Communication services
    Communication,
    /// Discovery services
    Discovery,
    /// Monitoring services
    Monitoring,
    /// Gateway services
    Gateway,
    /// Custom service type
    Custom(String),
}

impl ServiceType {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            ServiceType::AI => "ai",
            ServiceType::Compute => "compute",
            ServiceType::Storage => "storage",
            ServiceType::Security => "security",
            ServiceType::Communication => "communication",
            ServiceType::Discovery => "discovery",
            ServiceType::Monitoring => "monitoring",
            ServiceType::Gateway => "gateway",
            ServiceType::Custom(s) => s,
        }
    }
}

impl std::str::FromStr for ServiceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "ai" => ServiceType::AI,
            "compute" => ServiceType::Compute,
            "storage" => ServiceType::Storage,
            "security" => ServiceType::Security,
            "communication" => ServiceType::Communication,
            "discovery" => ServiceType::Discovery,
            "monitoring" => ServiceType::Monitoring,
            "gateway" => ServiceType::Gateway,
            _ => ServiceType::Custom(s.to_string()),
        })
    }
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Protocol (http, https, tarpc, websocket)
    pub protocol: String,
    /// Port number
    pub port: u16,
    /// Whether this is the primary endpoint
    pub primary: bool,
    /// Health check URL
    pub health_check_url: Option<String>,
    /// Endpoint weight for load balancing
    pub weight: Option<f32>,
    /// Endpoint tags
    pub tags: Vec<String>,
}

impl ServiceEndpoint {
    /// Create a new service endpoint
    pub fn new(url: String, protocol: String, port: u16) -> Self {
        Self {
            url,
            protocol,
            port,
            primary: false,
            health_check_url: None,
            weight: None,
            tags: Vec::new(),
        }
    }

    /// Set as primary endpoint
    pub fn as_primary(mut self) -> Self {
        self.primary = true;
        self
    }

    /// Set health check URL
    pub fn with_health_check(mut self, url: String) -> Self {
        self.health_check_url = Some(url);
        self
    }

    /// Set weight for load balancing
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Validate endpoint
    pub fn validate(&self) -> CoreResult<()> {
        if self.url.is_empty() {
            return Err(CoreError::InvalidServiceConfig(
                "Endpoint URL cannot be empty".to_string(),
            ));
        }

        if self.protocol.is_empty() {
            return Err(CoreError::InvalidServiceConfig(
                "Endpoint protocol cannot be empty".to_string(),
            ));
        }

        if self.port == 0 {
            return Err(CoreError::InvalidServiceConfig(
                "Endpoint port must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy and available
    Healthy,
    /// Service is unhealthy but still available
    Unhealthy,
    /// Service is degraded
    Degraded,
    /// Service is unavailable
    Unavailable,
}

impl HealthStatus {
    /// Check if service is available
    pub fn is_available(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Unhealthy)
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unavailable => "unavailable",
        }
    }
}

/// Service query for filtering and sorting
#[derive(Debug, Clone, Default)]
pub struct ServiceQuery {
    /// Filter by service type
    pub service_type: Option<ServiceType>,
    /// Filter by capabilities
    pub capabilities: Vec<String>,
    /// Filter by metadata
    pub metadata: HashMap<String, String>,
    /// Filter by health status
    pub health_status: Option<HealthStatus>,
    /// Filter by tags
    pub tags: Vec<String>,
    /// Sort field
    pub sort_field: Option<SortField>,
    /// Sort order
    pub sort_order: SortOrder,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Sort field enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortField {
    /// Sort by service name
    Name,
    /// Sort by service type
    Type,
    /// Sort by registration time
    RegisteredAt,
    /// Sort by last heartbeat
    LastHeartbeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum SortOrder {
    /// Ascending order
    #[default]
    Asc,
    /// Descending order
    Desc,
}

impl ServiceQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by service type
    pub fn with_service_type(mut self, service_type: ServiceType) -> Self {
        self.service_type = Some(service_type);
        self
    }

    /// Filter by capability
    pub fn with_capability(mut self, capability: String) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Filter by metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Filter by health status
    pub fn with_health_status(mut self, status: HealthStatus) -> Self {
        self.health_status = Some(status);
        self
    }

    /// Filter by tag
    pub fn with_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }

    /// Sort by field
    pub fn sort_by(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_field = Some(field);
        self.sort_order = order;
        self
    }

    /// Limit results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset for pagination
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Check if service matches this query
    pub fn matches(&self, service: &ServiceDefinition) -> bool {
        // Service type filter
        if let Some(ref service_type) = self.service_type {
            if service.service_type != *service_type {
                return false;
            }
        }

        // Capabilities filter
        for capability in &self.capabilities {
            if !service.has_capability(capability) {
                return false;
            }
        }

        // Metadata filter
        if !service.matches_metadata(&self.metadata) {
            return false;
        }

        // Health status filter
        if let Some(ref health_status) = self.health_status {
            if service.health_status != *health_status {
                return false;
            }
        }

        // Tags filter
        for tag in &self.tags {
            let tag_exists = service
                .endpoints
                .iter()
                .any(|endpoint| endpoint.tags.contains(tag));
            if !tag_exists {
                return false;
            }
        }

        true
    }

    /// Sort services according to query
    pub fn sort_services(&self, services: &mut [ServiceDefinition]) {
        if let Some(ref sort_field) = self.sort_field {
            match sort_field {
                SortField::Name => {
                    services.sort_by(|a, b| match self.sort_order {
                        SortOrder::Asc => a.name.cmp(&b.name),
                        SortOrder::Desc => b.name.cmp(&a.name),
                    });
                }
                SortField::Type => {
                    services.sort_by(|a, b| match self.sort_order {
                        SortOrder::Asc => a.service_type.as_str().cmp(b.service_type.as_str()),
                        SortOrder::Desc => b.service_type.as_str().cmp(a.service_type.as_str()),
                    });
                }
                SortField::RegisteredAt => {
                    services.sort_by(|a, b| match self.sort_order {
                        SortOrder::Asc => a.registered_at.cmp(&b.registered_at),
                        SortOrder::Desc => b.registered_at.cmp(&a.registered_at),
                    });
                }
                SortField::LastHeartbeat => {
                    services.sort_by(|a, b| match self.sort_order {
                        SortOrder::Asc => a.last_heartbeat.cmp(&b.last_heartbeat),
                        SortOrder::Desc => b.last_heartbeat.cmp(&a.last_heartbeat),
                    });
                }
            }
        }
    }

    /// Apply pagination to services
    pub fn paginate_services(&self, services: Vec<ServiceDefinition>) -> Vec<ServiceDefinition> {
        let start = self.offset.unwrap_or(0);
        let services_slice = services.into_iter().skip(start);

        if let Some(limit) = self.limit {
            services_slice.take(limit).collect()
        } else {
            services_slice.collect()
        }
    }
}

/// Service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Total number of services
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
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl ServiceStats {
    /// Create new service statistics
    pub fn new() -> Self {
        Self {
            total_services: 0,
            healthy_services: 0,
            unhealthy_services: 0,
            degraded_services: 0,
            unavailable_services: 0,
            services_by_type: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    /// Update statistics from a list of services
    pub fn update_from_services(&mut self, services: &[ServiceDefinition]) {
        self.total_services = services.len();
        self.healthy_services = 0;
        self.unhealthy_services = 0;
        self.degraded_services = 0;
        self.unavailable_services = 0;
        self.services_by_type.clear();

        for service in services {
            match service.health_status {
                HealthStatus::Healthy => self.healthy_services += 1,
                HealthStatus::Unhealthy => self.unhealthy_services += 1,
                HealthStatus::Degraded => self.degraded_services += 1,
                HealthStatus::Unavailable => self.unavailable_services += 1,
            }

            *self
                .services_by_type
                .entry(service.service_type.as_str().to_string())
                .or_insert(0) += 1;
        }

        self.last_updated = Utc::now();
    }

    /// Get availability percentage
    pub fn availability_percentage(&self) -> f32 {
        if self.total_services == 0 {
            return 100.0;
        }

        (self.healthy_services as f32 / self.total_services as f32) * 100.0
    }
}

impl Default for ServiceStats {
    fn default() -> Self {
        Self::new()
    }
}
