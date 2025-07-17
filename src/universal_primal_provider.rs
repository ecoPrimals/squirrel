//! Universal Primal Provider Implementation
//!
//! This module implements a truly universal primal provider that is agnostic
//! to specific primal types and uses dynamic service discovery.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::PrimalError;
use crate::universal::{
    UniversalPrimalProvider, UniversalResult, PrimalContext, PrimalType, PrimalCapability,
    PrimalDependency, PrimalHealth, PrimalEndpoints, PrimalRequest, PrimalResponse,
    DynamicPortInfo, ServiceMeshStatus, EcosystemRequest, EcosystemResponse,
    UniversalSecurityContext, SecurityLevel, UniversalConfig, ResponseStatus,
};

// Import the universal service discovery system
use squirrel_core::{
    ServiceDiscovery, ServiceDiscoveryClient, ServiceRegistry, ServiceDefinition,
    ServiceType, ServiceEndpoint, HealthStatus, ServiceQuery, UniversalServiceConfig,
};

/// Universal Primal Provider that is agnostic to specific primal types
#[derive(Debug)]
pub struct UniversalSquirrelProvider {
    /// Unique primal identifier
    primal_id: String,
    /// Instance identifier for multi-instance support
    instance_id: String,
    /// Context this instance serves
    context: PrimalContext,
    /// Configuration
    config: UniversalConfig,
    /// Universal service configuration
    service_config: UniversalServiceConfig,
    /// Service discovery client
    discovery_client: Arc<ServiceDiscoveryClient>,
    /// Service registry
    service_registry: Arc<ServiceRegistry>,
    /// Current capabilities
    capabilities: Arc<RwLock<Vec<PrimalCapability>>>,
    /// Health status
    health_status: Arc<RwLock<PrimalHealth>>,
    /// Dynamic port information
    dynamic_port_info: Option<DynamicPortInfo>,
    /// Service mesh status
    service_mesh_status: Arc<RwLock<ServiceMeshStatus>>,
    /// Startup time
    startup_time: DateTime<Utc>,
}

impl UniversalSquirrelProvider {
    /// Create a new Universal Squirrel Provider
    pub fn new(
        config: UniversalConfig,
        service_config: UniversalServiceConfig,
        discovery: Arc<dyn ServiceDiscovery>,
    ) -> UniversalResult<Self> {
        let instance_id = config.service.instance_id.clone();
        let primal_id = config.service.name.clone();
        
        // Create default context
        let context = PrimalContext {
            user_id: "system".to_string(),
            device_id: "localhost".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: Default::default(),
            security_level: config.security.security_level.clone(),
            biome_id: None,
            metadata: HashMap::new(),
        };

        // Initialize service discovery and registry
        let discovery_client = Arc::new(ServiceDiscoveryClient::new(discovery.clone()));
        let service_registry = Arc::new(ServiceRegistry::new(discovery));

        let capabilities = Arc::new(RwLock::new(Self::default_capabilities()));
        let health_status = Arc::new(RwLock::new(PrimalHealth::Healthy));
        let service_mesh_status = Arc::new(RwLock::new(ServiceMeshStatus::default()));

        Ok(Self {
            primal_id,
            instance_id,
            context,
            config,
            service_config,
            discovery_client,
            service_registry,
            capabilities,
            health_status,
            dynamic_port_info: None,
            service_mesh_status,
            startup_time: Utc::now(),
        })
    }

    /// Get default capabilities for Squirrel
    fn default_capabilities() -> Vec<PrimalCapability> {
        vec![
            PrimalCapability {
                name: "ai_chat".to_string(),
                description: "AI chat capabilities".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
                requires_auth: true,
                security_level: SecurityLevel::User,
                metadata: HashMap::new(),
            },
            PrimalCapability {
                name: "mcp_protocol".to_string(),
                description: "Machine Context Protocol support".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
                requires_auth: true,
                security_level: SecurityLevel::User,
                metadata: HashMap::new(),
            },
            PrimalCapability {
                name: "plugin_execution".to_string(),
                description: "Plugin execution capabilities".to_string(),
                version: "1.0.0".to_string(),
                parameters: vec![],
                requires_auth: true,
                security_level: SecurityLevel::User,
                metadata: HashMap::new(),
            },
        ]
    }

    /// Register this service with the service discovery system
    pub async fn register_service(&self) -> UniversalResult<()> {
        let service_definition = ServiceDefinition {
            id: self.instance_id.clone(),
            name: self.primal_id.clone(),
            service_type: ServiceType::AI,
            endpoints: vec![
                ServiceEndpoint {
                    url: format!("http://localhost:{}", self.config.network.port),
                    protocol: "http".to_string(),
                    port: self.config.network.port,
                    primary: true,
                    health_check_url: Some(format!("http://localhost:{}/health", self.config.network.port)),
                }
            ],
            capabilities: self.capabilities.read().await.iter().map(|c| c.name.clone()).collect(),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("type".to_string(), "ai".to_string());
                metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
                metadata.insert("primal_type".to_string(), "squirrel".to_string());
                metadata
            },
            health_status: HealthStatus::Healthy,
            registered_at: Utc::now(),
            last_heartbeat: Utc::now(),
        };

        self.service_registry.register_local_service(service_definition).await
            .map_err(|e| PrimalError::ServiceRegistration(e.to_string()))?;

        // Start heartbeat loop
        self.service_registry.start_heartbeat_loop().await
            .map_err(|e| PrimalError::ServiceRegistration(e.to_string()))?;

        Ok(())
    }

    /// Deregister this service from the service discovery system
    pub async fn deregister_service(&self) -> UniversalResult<()> {
        self.service_registry.deregister_local_service(&self.instance_id).await
            .map_err(|e| PrimalError::ServiceDeregistration(e.to_string()))?;

        // Update service mesh status
        let mut status = self.service_mesh_status.write().await;
        status.connected = false;
        status.registration_time = None;
        status.last_heartbeat = None;

        Ok(())
    }

    /// Find service by type using service discovery
    pub async fn find_service_by_type(&self, service_type: ServiceType) -> UniversalResult<Option<ServiceDefinition>> {
        self.discovery_client.find_service_by_type(service_type).await
            .map_err(|e| PrimalError::ServiceDiscovery(e.to_string()))
    }

    /// Find service by capability using service discovery
    pub async fn find_service_by_capability(&self, capability: &str) -> UniversalResult<Option<ServiceDefinition>> {
        self.discovery_client.find_service_by_capability(capability).await
            .map_err(|e| PrimalError::ServiceDiscovery(e.to_string()))
    }

    /// Route request to appropriate service based on capability
    pub async fn route_to_service(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
        // Determine required capability from request
        let required_capability = self.determine_required_capability(&request);
        
        // Find service with required capability
        let service = self.find_service_by_capability(&required_capability).await?;
        
        if let Some(service) = service {
            // Route to the discovered service
            self.send_request_to_service(&service, request).await
        } else {
            // No service found, return error
            Err(PrimalError::ServiceNotFound(format!("No service found for capability: {}", required_capability)))
        }
    }

    /// Determine required capability from request
    fn determine_required_capability(&self, request: &PrimalRequest) -> String {
        // Analyze request to determine required capability
        match request.operation.as_str() {
            "chat" | "ai_chat" => "ai_chat".to_string(),
            "compute" | "execute" => "compute".to_string(),
            "store" | "save" => "storage".to_string(),
            "secure" | "encrypt" => "security".to_string(),
            _ => "generic".to_string(),
        }
    }

    /// Send request to a discovered service
    async fn send_request_to_service(&self, service: &ServiceDefinition, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
        // Get primary endpoint
        let endpoint = service.endpoints.iter()
            .find(|e| e.primary)
            .or_else(|| service.endpoints.first())
            .ok_or_else(|| PrimalError::ServiceEndpointNotFound(service.id.clone()))?;

        // Create HTTP client and send request
        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/process", endpoint.url);
        
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let primal_response: PrimalResponse = response.json().await
                .map_err(|e| PrimalError::SerializationError(e.to_string()))?;
            Ok(primal_response)
        } else {
            Err(PrimalError::ServiceError(format!("Service returned error: {}", response.status())))
        }
    }

    /// Get all available services
    pub async fn get_available_services(&self) -> UniversalResult<Vec<ServiceDefinition>> {
        self.discovery_client.discovery.get_active_services().await
            .map_err(|e| PrimalError::ServiceDiscovery(e.to_string()))
    }

    /// Update health status
    async fn update_health_status(&self) -> UniversalResult<()> {
        let mut health = self.health_status.write().await;
        *health = PrimalHealth::Healthy;
        Ok(())
    }
}

#[async_trait]
impl UniversalPrimalProvider for UniversalSquirrelProvider {
    fn primal_id(&self) -> &str {
        &self.primal_id
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn primal_type(&self) -> PrimalType {
        PrimalType::AI
    }

    async fn get_capabilities(&self) -> UniversalResult<Vec<PrimalCapability>> {
        Ok(self.capabilities.read().await.clone())
    }

    async fn get_health(&self) -> UniversalResult<PrimalHealth> {
        Ok(self.health_status.read().await.clone())
    }

    async fn get_endpoints(&self) -> UniversalResult<PrimalEndpoints> {
        Ok(PrimalEndpoints {
            http: Some(format!("http://localhost:{}", self.config.network.port)),
            websocket: Some(format!("ws://localhost:{}/ws", self.config.network.port)),
            grpc: None,
            custom: HashMap::new(),
        })
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
        // Check if we can handle this request locally
        let required_capability = self.determine_required_capability(&request);
        let capabilities = self.capabilities.read().await;
        
        if capabilities.iter().any(|c| c.name == required_capability) {
            // Handle locally
            self.handle_local_request(request).await
        } else {
            // Route to another service
            self.route_to_service(request).await
        }
    }

    async fn handle_local_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse> {
        // Handle request locally based on operation
        match request.operation.as_str() {
            "ai_chat" => {
                // Handle AI chat request
                Ok(PrimalResponse {
                    id: request.id,
                    success: true,
                    data: Some(serde_json::json!({
                        "message": "Hello from Squirrel AI!",
                        "timestamp": Utc::now().to_rfc3339(),
                    })),
                    error: None,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
            "mcp_protocol" => {
                // Handle MCP protocol request
                Ok(PrimalResponse {
                    id: request.id,
                    success: true,
                    data: Some(serde_json::json!({
                        "protocol": "mcp",
                        "version": "1.0.0",
                        "status": "active",
                    })),
                    error: None,
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
            _ => {
                // Unknown operation
                Ok(PrimalResponse {
                    id: request.id,
                    success: false,
                    data: None,
                    error: Some(format!("Unknown operation: {}", request.operation)),
                    metadata: HashMap::new(),
                    timestamp: Utc::now(),
                })
            }
        }
    }

    async fn startup(&mut self) -> UniversalResult<()> {
        // Register with service discovery
        self.register_service().await?;

        // Update health status
        self.update_health_status().await?;

        Ok(())
    }

    async fn shutdown(&mut self) -> UniversalResult<()> {
        // Deregister from service discovery
        self.deregister_service().await?;

        // Update health status
        let mut health = self.health_status.write().await;
        *health = PrimalHealth::Unhealthy {
            reason: "Shutting down".to_string(),
        };

        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        // Check if we can serve this context based on security level and other factors
        context.security_level <= self.config.security.security_level
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        self.dynamic_port_info.clone()
    }

    fn get_service_mesh_status(&self) -> ServiceMeshStatus {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.service_mesh_status.read().await.clone()
            })
        })
    }

    async fn handle_ecosystem_request(&self, request: EcosystemRequest) -> UniversalResult<EcosystemResponse> {
        let start_time = std::time::Instant::now();
        
        // Convert ecosystem request to primal request
        let primal_request = PrimalRequest {
            id: request.request_id,
            source: request.source_service,
            target: request.target_service,
            operation: request.operation,
            data: request.payload,
            security: request.security_context,
            context: self.context.clone(),
            timestamp: request.timestamp,
        };

        // Handle the request
        let primal_response = self.handle_primal_request(primal_request).await?;

        // Convert response
        let status = if primal_response.success {
            ResponseStatus::Success
        } else {
            ResponseStatus::Error {
                code: "PROCESSING_ERROR".to_string(),
                message: primal_response.error.unwrap_or("Unknown error".to_string()),
            }
        };

        Ok(EcosystemResponse {
            request_id: request.request_id,
            source_service: self.primal_id.clone(),
            target_service: request.source_service,
            payload: primal_response.data,
            status,
            processing_time: start_time.elapsed(),
            timestamp: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_core::InMemoryServiceDiscovery;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_universal_provider_creation() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let config = UniversalConfig::default();
        let service_config = UniversalServiceConfig::new();
        
        let provider = UniversalSquirrelProvider::new(config, service_config, discovery).unwrap();
        
        assert_eq!(provider.primal_type(), PrimalType::AI);
        assert!(!provider.primal_id().is_empty());
    }

    #[tokio::test]
    async fn test_service_registration() {
        let discovery = Arc::new(InMemoryServiceDiscovery::new());
        let config = UniversalConfig::default();
        let service_config = UniversalServiceConfig::new();
        
        let provider = UniversalSquirrelProvider::new(config, service_config, discovery.clone()).unwrap();
        
        // Register service
        provider.register_service().await.unwrap();
        
        // Check if service is registered
        let services = discovery.get_active_services().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, provider.primal_id());
    }
} 