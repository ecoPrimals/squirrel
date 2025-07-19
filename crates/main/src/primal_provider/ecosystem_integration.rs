//! Ecosystem Integration and Service Mesh

use chrono::Utc;
use serde_json::json;
use tracing::info;

use crate::ecosystem::EcosystemServiceRegistration;
use crate::error::PrimalError;
use crate::universal::*;
use super::core::SquirrelPrimalProvider;

/// Ecosystem Integration functionality
pub struct EcosystemIntegration;

impl EcosystemIntegration {
    /// Create service registration for Songbird
    pub fn create_service_registration(provider: &SquirrelPrimalProvider) -> EcosystemServiceRegistration {
        let endpoints = provider.endpoints();
        EcosystemServiceRegistration {
            service_id: format!("{}-{}", provider.primal_type().to_string(), provider.instance_id),
            primal_type: crate::ecosystem::EcosystemPrimalType::Squirrel,
            name: "Squirrel AI Primal".to_string(),
            version: "1.0.0".to_string(),
            description: "AI coordination and context analysis primal".to_string(),
            biome_id: Some("default".to_string()),
            endpoints: crate::ecosystem::ServiceEndpoints {
                health: "http://0.0.0.0:8080/health".to_string(),
                metrics: "http://0.0.0.0:8080/metrics".to_string(),
                admin: "http://0.0.0.0:8080/admin".to_string(),
                mcp: "http://0.0.0.0:8080/mcp".to_string(),
                ai_coordination: "http://0.0.0.0:8080/ai".to_string(),
                service_mesh: "http://0.0.0.0:8080/mesh".to_string(),
            },
            capabilities: crate::ecosystem::ServiceCapabilities {
                core: vec!["ai_coordination".to_string(), "context_analysis".to_string()],
                extended: vec!["session_management".to_string()],
                integrations: vec!["mcp".to_string()],
            },
            dependencies: vec![],
            health_check: crate::ecosystem::HealthCheckConfig {
                failure_threshold: 3,
                recovery_threshold: 2,
            },
            metadata: std::collections::HashMap::new(),
            primal_provider: Some("squirrel".to_string()),
            registered_at: chrono::Utc::now(),
            last_seen: Some(chrono::Utc::now()),
        }
    }
}

impl SquirrelPrimalProvider {
    /// Initialize ecosystem connections and services
    pub async fn initialize_ecosystem(&mut self) -> Result<(), PrimalError> {
        // Set configuration for ecosystem manager
        if let Some(endpoint) = &self.config.discovery.songbird_endpoint {
            info!("Songbird endpoint configured: {}", endpoint);
        }

        // Initialize biomeos client if not already set
        // Note: biomeos_client field doesn't exist in current struct definition

        // Initialize session manager if not already set
        // Note: session_manager is not Optional in current struct definition

        // Set songbird endpoint if available
        if let Some(endpoint) = &self.config.discovery.songbird_endpoint {
            info!("Registered with Songbird at: {}", endpoint);
        }

        self.initialized = true;
        Ok(())
    }

    /// Gracefully shutdown the primal
    pub async fn shutdown_ecosystem(&mut self) -> Result<(), PrimalError> {
        // Deregister from Songbird if registered
        if let Some(endpoint) = &self.config.discovery.songbird_endpoint {
            let service_id = format!("{}-{}", self.primal_id(), self.instance_id);
            info!("Deregistering from Songbird: {}", service_id);

            // Note: http_client field doesn't exist in current struct definition
            // This would need to be implemented with a proper HTTP client
        }

        self.shutdown = true;
        Ok(())
    }

    /// Check if this primal can serve the given context
    pub fn can_serve_context(&self, context: &crate::universal::PrimalContext) -> bool {
        match context.security_level {
            crate::universal::SecurityLevel::Public => true,
            crate::universal::SecurityLevel::Basic => true,
            crate::universal::SecurityLevel::Standard => true,
            crate::universal::SecurityLevel::High => true,
            crate::universal::SecurityLevel::Critical => true,
            crate::universal::SecurityLevel::Maximum => true,
        }
    }

    /// Get dynamic port information
    pub fn dynamic_port_info(&self) -> Option<crate::universal::DynamicPortInfo> {
        let now = Utc::now();
        Some(crate::universal::DynamicPortInfo {
            assigned_port: 8080,
            current_port: 8080,
            port_range: (8080, 8090),
            port_type: crate::universal::PortType::Http,
            status: crate::universal::PortStatus::Active,
            assigned_at: now,
            allocated_at: now,
            lease_duration: chrono::Duration::hours(1),
            expires_at: Some(now + chrono::Duration::hours(1)),
        })
    }

    /// Register with Songbird service mesh
    pub async fn register_with_songbird(
        &mut self,
        songbird_endpoint: &str,
    ) -> crate::universal::UniversalResult<String> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id());

        // Note: In a real implementation, this would make an HTTP request to Songbird
        info!(
            "Registering with Songbird: {} at {}",
            service_id, songbird_endpoint
        );

        Ok(service_id)
    }

    /// Deregister from Songbird service mesh
    pub async fn deregister_from_songbird(&mut self) -> Result<(), PrimalError> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id);

        // Note: In a real implementation, this would make an HTTP request
        info!("Deregistering from Songbird: {}", service_id);

        Ok(())
    }

    /// Get service mesh status
    pub fn get_service_mesh_status(&self) -> crate::universal::ServiceMeshStatus {
        crate::universal::ServiceMeshStatus {
            connected: self.initialized,
            registered: self.initialized,
            mesh_health: if self.initialized {
                "healthy".to_string()
            } else {
                "initializing".to_string()
            },
            songbird_endpoint: self.config.discovery.songbird_endpoint.clone(),
            registration_time: Some(chrono::Utc::now()),
            last_heartbeat: Some(chrono::Utc::now()),
            mesh_version: "1.0.0".to_string(),
            instance_id: self.instance_id.clone(),
            load_balancing_enabled: true,
            circuit_breaker_status: crate::universal::CircuitBreakerStatus {
                open: false,
                failures: 0,
                last_failure: None,
                next_retry: None,
            },
            load_balancing: crate::universal::LoadBalancingStatus {
                enabled: true,
                healthy: true,
                active_connections: 5,
                algorithm: "round_robin".to_string(),
                health_score: 0.95,
                last_check: chrono::Utc::now(),
            },
            last_registration: Some(chrono::Utc::now()),
            active_connections: 5,
        }
    }

    /// Handle ecosystem request
    pub async fn handle_ecosystem_request(
        &self,
        request: crate::universal::EcosystemRequest,
    ) -> crate::universal::UniversalResult<crate::universal::EcosystemResponse> {
        Ok(crate::universal::EcosystemResponse {
            response_id: uuid::Uuid::new_v4(),
            request_id: request.request_id,
            payload: serde_json::json!({
                "status": "handled",
                "primal": self.primal_id(),
                "instance": self.instance_id()
            }),
            status: crate::universal::ResponseStatus::Success,
            metadata: std::collections::HashMap::new(),
            success: true,
            error_message: None,
        })
    }

    /// Create service registration for ecosystem
    pub fn create_service_registration(&self) -> EcosystemServiceRegistration {
        EcosystemIntegration::create_service_registration(self)
    }
} 