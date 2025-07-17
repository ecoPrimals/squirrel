//! Universal Service Discovery System
//!
//! This module implements a universal service discovery system that replaces
//! hardcoded primal endpoints with dynamic discovery and registration.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::CoreResult;

/// Universal service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Health status
    pub health_status: HealthStatus,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,
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
    /// Custom service type
    Custom(String),
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Protocol (http, https, grpc, websocket)
    pub protocol: String,
    /// Port number
    pub port: u16,
    /// Whether this is the primary endpoint
    pub primary: bool,
    /// Health check URL
    pub health_check_url: Option<String>,
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Service status unknown
    Unknown,
}

/// Service discovery query parameters
#[derive(Debug, Clone, Default)]
pub struct ServiceQuery {
    /// Filter by service type
    pub service_type: Option<ServiceType>,
    /// Filter by capability
    pub capability: Option<String>,
    /// Filter by health status
    pub health_status: Option<HealthStatus>,
    /// Filter by metadata key-value pairs
    pub metadata_filters: HashMap<String, String>,
}

/// Service discovery trait
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Register a service
    async fn register_service(&self, service: ServiceDefinition) -> CoreResult<()>;
    
    /// Deregister a service
    async fn deregister_service(&self, service_id: &str) -> CoreResult<()>;
    
    /// Discover services based on query
    async fn discover_services(&self, query: ServiceQuery) -> CoreResult<Vec<ServiceDefinition>>;
    
    /// Get all active services
    async fn get_active_services(&self) -> CoreResult<Vec<ServiceDefinition>>;
    
    /// Get service by ID
    async fn get_service(&self, service_id: &str) -> CoreResult<Option<ServiceDefinition>>;
    
    /// Update service health
    async fn update_service_health(&self, service_id: &str, health: HealthStatus) -> CoreResult<()>;
    
    /// Send heartbeat for service
    async fn heartbeat(&self, service_id: &str) -> CoreResult<()>;
}

/// In-memory service discovery implementation
pub struct InMemoryServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
    heartbeat_timeout: Duration,
}

impl InMemoryServiceDiscovery {
    /// Create new in-memory service discovery
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: Duration::from_secs(30),
        }
    }
    
    /// Create with custom heartbeat timeout
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
        duration_since_heartbeat > chrono::Duration::from_std(self.heartbeat_timeout).unwrap_or_default()
    }
    
    /// Clean up expired services
    pub async fn cleanup_expired_services(&self) -> CoreResult<Vec<String>> {
        let mut services = self.services.write().await;
        let mut expired_services = Vec::new();
        
        services.retain(|id, service| {
            if self.is_service_expired(service) {
                expired_services.push(id.clone());
                false
            } else {
                true
            }
        });
        
        Ok(expired_services)
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
        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service);
        Ok(())
    }
    
    async fn deregister_service(&self, service_id: &str) -> CoreResult<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);
        Ok(())
    }
    
    async fn discover_services(&self, query: ServiceQuery) -> CoreResult<Vec<ServiceDefinition>> {
        let services = self.services.read().await;
        let mut results = Vec::new();
        
        for service in services.values() {
            // Skip expired services
            if self.is_service_expired(service) {
                continue;
            }
            
            // Apply filters
            if let Some(ref service_type) = query.service_type {
                if service.service_type != *service_type {
                    continue;
                }
            }
            
            if let Some(ref capability) = query.capability {
                if !service.capabilities.contains(capability) {
                    continue;
                }
            }
            
            if let Some(ref health_status) = query.health_status {
                if service.health_status != *health_status {
                    continue;
                }
            }
            
            // Apply metadata filters
            let mut metadata_match = true;
            for (key, value) in &query.metadata_filters {
                if service.metadata.get(key) != Some(value) {
                    metadata_match = false;
                    break;
                }
            }
            
            if metadata_match {
                results.push(service.clone());
            }
        }
        
        Ok(results)
    }
    
    async fn get_active_services(&self) -> CoreResult<Vec<ServiceDefinition>> {
        let services = self.services.read().await;
        let results = services
            .values()
            .filter(|service| !self.is_service_expired(service))
            .cloned()
            .collect();
        Ok(results)
    }
    
    async fn get_service(&self, service_id: &str) -> CoreResult<Option<ServiceDefinition>> {
        let services = self.services.read().await;
        Ok(services.get(service_id).cloned())
    }
    
    async fn update_service_health(&self, service_id: &str, health: HealthStatus) -> CoreResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.health_status = health;
            service.last_heartbeat = Utc::now();
        }
        Ok(())
    }
    
    async fn heartbeat(&self, service_id: &str) -> CoreResult<()> {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.last_heartbeat = Utc::now();
        }
        Ok(())
    }
}

/// Service discovery client for making requests
pub struct ServiceDiscoveryClient {
    discovery: Arc<dyn ServiceDiscovery>,
}

impl ServiceDiscoveryClient {
    /// Create new client with discovery backend
    pub fn new(discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self { discovery }
    }
    
    /// Find service by type
    pub async fn find_service_by_type(&self, service_type: ServiceType) -> CoreResult<Option<ServiceDefinition>> {
        let query = ServiceQuery {
            service_type: Some(service_type),
            health_status: Some(HealthStatus::Healthy),
            ..Default::default()
        };
        
        let services = self.discovery.discover_services(query).await?;
        Ok(services.into_iter().next())
    }
    
    /// Find service by capability
    pub async fn find_service_by_capability(&self, capability: &str) -> CoreResult<Option<ServiceDefinition>> {
        let query = ServiceQuery {
            capability: Some(capability.to_string()),
            health_status: Some(HealthStatus::Healthy),
            ..Default::default()
        };
        
        let services = self.discovery.discover_services(query).await?;
        Ok(services.into_iter().next())
    }
    
    /// Find services with load balancing
    pub async fn find_services_for_load_balancing(&self, service_type: ServiceType) -> CoreResult<Vec<ServiceDefinition>> {
        let query = ServiceQuery {
            service_type: Some(service_type),
            health_status: Some(HealthStatus::Healthy),
            ..Default::default()
        };
        
        self.discovery.discover_services(query).await
    }
}

/// Service registry for managing service lifecycle
pub struct ServiceRegistry {
    discovery: Arc<dyn ServiceDiscovery>,
    local_services: Arc<RwLock<HashMap<String, ServiceDefinition>>>,
}

impl ServiceRegistry {
    /// Create new service registry
    pub fn new(discovery: Arc<dyn ServiceDiscovery>) -> Self {
        Self {
            discovery,
            local_services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register local service
    pub async fn register_local_service(&self, service: ServiceDefinition) -> CoreResult<()> {
        // Register with discovery
        self.discovery.register_service(service.clone()).await?;
        
        // Store locally
        let mut local_services = self.local_services.write().await;
        local_services.insert(service.id.clone(), service);
        
        Ok(())
    }
    
    /// Deregister local service
    pub async fn deregister_local_service(&self, service_id: &str) -> CoreResult<()> {
        // Deregister from discovery
        self.discovery.deregister_service(service_id).await?;
        
        // Remove locally
        let mut local_services = self.local_services.write().await;
        local_services.remove(service_id);
        
        Ok(())
    }
    
    /// Start heartbeat for all local services
    pub async fn start_heartbeat_loop(&self) -> CoreResult<()> {
        let discovery = self.discovery.clone();
        let local_services = self.local_services.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                let services = local_services.read().await;
                for service_id in services.keys() {
                    if let Err(e) = discovery.heartbeat(service_id).await {
                        eprintln!("Failed to send heartbeat for service {}: {}", service_id, e);
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_service_discovery_registration() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        
        let service = ServiceDefinition {
            id: "test-service".to_string(),
            name: "Test Service".to_string(),
            service_type: ServiceType::AI,
            endpoints: vec![ServiceEndpoint {
                url: "http://localhost:8080".to_string(),
                protocol: "http".to_string(),
                port: 8080,
                primary: true,
                health_check_url: Some("http://localhost:8080/health".to_string()),
            }],
            capabilities: vec!["chat".to_string()],
            metadata: HashMap::new(),
            health_status: HealthStatus::Healthy,
            registered_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        
        discovery.register_service(service.clone()).await.unwrap();
        
        let retrieved = discovery.get_service(&service.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, service.id);
        assert_eq!(retrieved.name, service.name);
    }
    
    #[tokio::test]
    async fn test_service_discovery_query() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        
        let service = ServiceDefinition {
            id: "ai-service".to_string(),
            name: "AI Service".to_string(),
            service_type: ServiceType::AI,
            endpoints: vec![],
            capabilities: vec!["chat".to_string()],
            metadata: HashMap::new(),
            health_status: HealthStatus::Healthy,
            registered_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        
        discovery.register_service(service).await.unwrap();
        
        let query = ServiceQuery {
            service_type: Some(ServiceType::AI),
            capability: Some("chat".to_string()),
            ..Default::default()
        };
        
        let results = discovery.discover_services(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "ai-service");
    }
} 