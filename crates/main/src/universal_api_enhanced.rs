//! Universal API Module
//!
//! This module provides a universal API that uses service discovery
//! instead of hardcoded primal endpoints.

use std::collections::HashMap;
use std::sync::Arc;
use warp::{Filter, Reply, Rejection};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use squirrel_core::{
    ServiceDiscovery, ServiceDiscoveryClient, ServiceDefinition, ServiceType, HealthStatus,
};

use crate::ecosystem::EcosystemManager;
use crate::error::PrimalError;

/// Universal ecosystem status response
#[derive(Debug, Serialize, Deserialize)]
pub struct UniversalEcosystemStatusResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub active_services: Vec<ActiveServiceInfo>,
    pub service_discovery: String,
    pub metadata: HashMap<String, String>,
}

/// Active service information
#[derive(Debug, Serialize, Deserialize)]
pub struct ActiveServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub status: String,
    pub endpoints: Vec<String>,
    pub capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub health_status: String,
    pub last_heartbeat: DateTime<Utc>,
}

/// Universal service status response
#[derive(Debug, Serialize, Deserialize)]
pub struct UniversalServiceStatusResponse {
    pub services: Vec<ActiveServiceInfo>,
    pub total_count: usize,
    pub healthy_count: usize,
    pub unhealthy_count: usize,
    pub timestamp: DateTime<Utc>,
}

/// Universal API router
pub struct UniversalApiRouter {
    discovery_client: Arc<ServiceDiscoveryClient>,
    ecosystem_manager: Arc<EcosystemManager>,
}

impl UniversalApiRouter {
    /// Create new universal API router
    pub fn new(
        discovery: Arc<dyn ServiceDiscovery>,
        ecosystem_manager: Arc<EcosystemManager>,
    ) -> Self {
        let discovery_client = Arc::new(ServiceDiscoveryClient::new(discovery));
        
        Self {
            discovery_client,
            ecosystem_manager,
        }
    }
    
    /// Create warp routes for universal API
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let discovery_client = self.discovery_client.clone();
        let ecosystem_manager = self.ecosystem_manager.clone();
        
        let ecosystem_status = warp::path("ecosystem")
            .and(warp::path("status"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_discovery(discovery_client.clone()))
            .and(with_ecosystem(ecosystem_manager.clone()))
            .and_then(handle_universal_ecosystem_status);
        
        let services_list = warp::path("services")
            .and(warp::path::end())
            .and(warp::get())
            .and(with_discovery(discovery_client.clone()))
            .and_then(handle_universal_services_list);
        
        let service_by_type = warp::path("services")
            .and(warp::path("type"))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::get())
            .and(with_discovery(discovery_client.clone()))
            .and_then(handle_services_by_type);
        
        let service_by_capability = warp::path("services")
            .and(warp::path("capability"))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::get())
            .and(with_discovery(discovery_client.clone()))
            .and_then(handle_services_by_capability);
        
        let service_health = warp::path("services")
            .and(warp::path("health"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_discovery(discovery_client.clone()))
            .and_then(handle_service_health);
        
        ecosystem_status
            .or(services_list)
            .or(service_by_type)
            .or(service_by_capability)
            .or(service_health)
    }
}

// Helper function to inject discovery client
fn with_discovery(
    client: Arc<ServiceDiscoveryClient>,
) -> impl Filter<Extract = (Arc<ServiceDiscoveryClient>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || client.clone())
}

// Helper function to inject ecosystem manager
fn with_ecosystem(
    manager: Arc<EcosystemManager>,
) -> impl Filter<Extract = (Arc<EcosystemManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || manager.clone())
}

/// Handle universal ecosystem status
async fn handle_universal_ecosystem_status(
    discovery_client: Arc<ServiceDiscoveryClient>,
    _ecosystem_manager: Arc<EcosystemManager>,
) -> Result<impl Reply, Rejection> {
    // Get all active services from discovery
    let active_services = match discovery_client.discovery.get_active_services().await {
        Ok(services) => services,
        Err(e) => {
            eprintln!("Failed to get active services: {}", e);
            vec![]
        }
    };
    
    // Convert to API format
    let service_info: Vec<ActiveServiceInfo> = active_services
        .into_iter()
        .map(|service| convert_service_to_info(service))
        .collect();
    
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    metadata.insert("discovery_type".to_string(), "universal".to_string());
    metadata.insert("service_count".to_string(), service_info.len().to_string());
    
    let response = UniversalEcosystemStatusResponse {
        status: "active".to_string(),
        timestamp: Utc::now(),
        active_services: service_info,
        service_discovery: "active".to_string(),
        metadata,
    };
    
    Ok(warp::reply::json(&response))
}

/// Handle universal services list
async fn handle_universal_services_list(
    discovery_client: Arc<ServiceDiscoveryClient>,
) -> Result<impl Reply, Rejection> {
    // Get all active services from discovery
    let active_services = match discovery_client.discovery.get_active_services().await {
        Ok(services) => services,
        Err(e) => {
            eprintln!("Failed to get active services: {}", e);
            vec![]
        }
    };
    
    // Convert to API format
    let service_info: Vec<ActiveServiceInfo> = active_services
        .into_iter()
        .map(|service| convert_service_to_info(service))
        .collect();
    
    let healthy_count = service_info.iter()
        .filter(|s| s.health_status == "healthy")
        .count();
    
    let unhealthy_count = service_info.len() - healthy_count;
    
    let response = UniversalServiceStatusResponse {
        services: service_info.clone(),
        total_count: service_info.len(),
        healthy_count,
        unhealthy_count,
        timestamp: Utc::now(),
    };
    
    Ok(warp::reply::json(&response))
}

/// Handle services by type
async fn handle_services_by_type(
    service_type: String,
    discovery_client: Arc<ServiceDiscoveryClient>,
) -> Result<impl Reply, Rejection> {
    let service_type_enum = match service_type.to_lowercase().as_str() {
        "ai" => ServiceType::AI,
        "compute" => ServiceType::Compute,
        "storage" => ServiceType::Storage,
        "security" => ServiceType::Security,
        "communication" => ServiceType::Communication,
        "discovery" => ServiceType::Discovery,
        _ => ServiceType::Custom(service_type.clone()),
    };
    
    let service = match discovery_client.find_service_by_type(service_type_enum).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Failed to find service by type: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Service discovery failed",
                    "message": e.to_string()
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    
    if let Some(service) = service {
        let service_info = convert_service_to_info(service);
        Ok(warp::reply::with_status(
            warp::reply::json(&service_info),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Service not found",
                "message": format!("No service found for type: {}", service_type)
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

/// Handle services by capability
async fn handle_services_by_capability(
    capability: String,
    discovery_client: Arc<ServiceDiscoveryClient>,
) -> Result<impl Reply, Rejection> {
    let service = match discovery_client.find_service_by_capability(&capability).await {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Failed to find service by capability: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Service discovery failed",
                    "message": e.to_string()
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    
    if let Some(service) = service {
        let service_info = convert_service_to_info(service);
        Ok(warp::reply::with_status(
            warp::reply::json(&service_info),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Service not found",
                "message": format!("No service found for capability: {}", capability)
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

/// Handle service health check
async fn handle_service_health(
    discovery_client: Arc<ServiceDiscoveryClient>,
) -> Result<impl Reply, Rejection> {
    let active_services = match discovery_client.discovery.get_active_services().await {
        Ok(services) => services,
        Err(e) => {
            eprintln!("Failed to get active services: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Service discovery failed",
                    "message": e.to_string()
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    
    let mut health_report = HashMap::new();
    let mut healthy_count = 0;
    let mut unhealthy_count = 0;
    
    for service in active_services {
        let health_status = match service.health_status {
            HealthStatus::Healthy => {
                healthy_count += 1;
                "healthy"
            }
            HealthStatus::Degraded => {
                unhealthy_count += 1;
                "degraded"
            }
            HealthStatus::Unhealthy => {
                unhealthy_count += 1;
                "unhealthy"
            }
            HealthStatus::Unknown => {
                unhealthy_count += 1;
                "unknown"
            }
        };
        
        health_report.insert(service.id, serde_json::json!({
            "name": service.name,
            "status": health_status,
            "last_heartbeat": service.last_heartbeat,
            "endpoints": service.endpoints.iter().map(|e| &e.url).collect::<Vec<_>>(),
        }));
    }
    
    let response = serde_json::json!({
        "services": health_report,
        "summary": {
            "total": healthy_count + unhealthy_count,
            "healthy": healthy_count,
            "unhealthy": unhealthy_count,
            "timestamp": Utc::now()
        }
    });
    
    Ok(warp::reply::json(&response))
}

/// Convert service definition to API info
fn convert_service_to_info(service: ServiceDefinition) -> ActiveServiceInfo {
    let status = match service.health_status {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Unhealthy => "unhealthy",
        HealthStatus::Unknown => "unknown",
    };
    
    let service_type = match service.service_type {
        ServiceType::AI => "ai",
        ServiceType::Compute => "compute",
        ServiceType::Storage => "storage",
        ServiceType::Security => "security",
        ServiceType::Communication => "communication",
        ServiceType::Discovery => "discovery",
        ServiceType::Custom(ref s) => s,
    };
    
    ActiveServiceInfo {
        id: service.id,
        name: service.name,
        service_type: service_type.to_string(),
        status: status.to_string(),
        endpoints: service.endpoints.iter().map(|e| e.url.clone()).collect(),
        capabilities: service.capabilities,
        metadata: service.metadata,
        health_status: status.to_string(),
        last_heartbeat: service.last_heartbeat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_core::InMemoryServiceDiscovery;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_universal_api_router() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let ecosystem_manager = Arc::new(EcosystemManager::new());
        
        let router = UniversalApiRouter::new(discovery, ecosystem_manager);
        let routes = router.routes();
        
        // Test that routes are created without panicking
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_service_conversion() {
        let service = ServiceDefinition {
            id: "test-service".to_string(),
            name: "Test Service".to_string(),
            service_type: ServiceType::AI,
            endpoints: vec![],
            capabilities: vec!["chat".to_string()],
            metadata: HashMap::new(),
            health_status: HealthStatus::Healthy,
            registered_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };
        
        let info = convert_service_to_info(service);
        assert_eq!(info.service_type, "ai");
        assert_eq!(info.health_status, "healthy");
        assert_eq!(info.capabilities.len(), 1);
    }
} 