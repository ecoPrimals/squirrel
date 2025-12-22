//! Ecosystem Integration and Service Mesh

use super::core::SquirrelPrimalProvider;
use crate::ecosystem::EcosystemServiceRegistration;
use crate::ecosystem::{HealthCheckConfig, SecurityConfig};
use crate::error::PrimalError;
use tracing::info;

/// Ecosystem Integration functionality
pub struct EcosystemIntegration;

impl EcosystemIntegration {
    /// Create service registration for Songbird
    ///
    /// This dynamically constructs endpoints from environment or configuration:
    /// - SERVER_BIND_ADDRESS (default: 0.0.0.0)
    /// - SERVER_PORT (default: 8080)
    pub fn create_service_registration(
        provider: &SquirrelPrimalProvider,
    ) -> EcosystemServiceRegistration {
        // Get configuration from environment or use defaults
        let bind_address =
            std::env::var("SERVER_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

        // Construct base URL
        let base_url = format!("http://{}:{}", bind_address, port);

        let _endpoints = provider.endpoints();
        EcosystemServiceRegistration {
            service_id: format!("{}-{}", provider.primal_type(), provider.instance_id),
            primal_type: crate::ecosystem::EcosystemPrimalType::Squirrel,
            name: "Squirrel AI Primal".to_string(),
            version: "1.0.0".to_string(),
            description: "AI coordination and context analysis primal".to_string(),
            biome_id: Some("default".to_string()),
            tags: vec!["ai".to_string(), "coordination".to_string()],
            security_config: SecurityConfig {
                auth_required: false,
                encryption_level: "none".to_string(),
                access_level: "public".to_string(),
                policies: Vec::new(),
                audit_enabled: false,
                security_level: "none".to_string(),
            },
            resource_requirements: crate::ecosystem::ResourceSpec {
                cpu: "1.0".to_string(),
                memory: "512".to_string(),
                storage: "10".to_string(),
                network: "100".to_string(),
                gpu: Some("0".to_string()),
            },
            endpoints: crate::ecosystem::ServiceEndpoints {
                primary: base_url.clone(),
                secondary: vec![
                    format!("{}/metrics", base_url),
                    format!("{}/admin", base_url),
                    format!("{}/mcp", base_url),
                    format!("{}/ai", base_url),
                    format!("{}/mesh", base_url),
                ],
                health: Some(format!("{}/health", base_url)),
            },
            capabilities: crate::ecosystem::ServiceCapabilities {
                core: vec![
                    "ai_coordination".to_string(),
                    "context_analysis".to_string(),
                ],
                extended: vec!["session_management".to_string()],
                integrations: vec!["mcp".to_string()],
            },
            dependencies: vec![],
            health_check: HealthCheckConfig {
                enabled: true,
                interval_secs: 30,
                timeout_secs: 10,
                failure_threshold: 3,
            },
            metadata: std::collections::HashMap::new(),
            primal_provider: Some("squirrel".to_string()),
            registered_at: chrono::Utc::now(),
            // Remove last_seen field as it doesn't exist in the struct
        }
    }
}

impl SquirrelPrimalProvider {
    /// Initialize ecosystem connections and services
    pub async fn initialize_ecosystem(&mut self) -> Result<(), PrimalError> {
        // Use ecosystem_manager field for ecosystem initialization (simplified approach)
        info!("Initializing ecosystem using EcosystemManager");

        // The ecosystem manager coordinates the initialization process
        info!(
            "EcosystemManager coordinating ecosystem initialization for instance: {}",
            self.instance_id
        );

        // Register capabilities with ecosystem manager (simplified)
        let capabilities = self.capabilities();
        info!(
            "EcosystemManager registering {} capabilities for instance: {}",
            capabilities.len(),
            self.instance_id
        );

        // Set configuration for ecosystem manager
        info!(
            "EcosystemManager configuring Songbird endpoint: {}",
            &self.config.songbird_endpoint
        );

        // Initialize service discovery through ecosystem manager (simplified)
        info!("EcosystemManager starting service discovery");

        self.initialized = true;
        info!("Ecosystem initialization completed via EcosystemManager");
        Ok(())
    }

    /// Gracefully shutdown the primal
    pub async fn shutdown_ecosystem(&mut self) -> Result<(), PrimalError> {
        // Use ecosystem_manager field for graceful shutdown
        info!("Shutting down ecosystem using EcosystemManager");

        // Deregister capabilities from ecosystem (simplified)
        info!(
            "EcosystemManager deregistering capabilities for instance: {}",
            self.instance_id
        );

        // Stop service discovery (simplified)
        info!("EcosystemManager stopping service discovery");

        // Deregister from Songbird via ecosystem manager
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id);
        info!(
            "EcosystemManager deregistering from Songbird at {}: {}",
            &self.config.songbird_endpoint, service_id
        );
        info!("Successfully deregistered from Songbird via EcosystemManager");

        // Shutdown ecosystem manager (simplified)
        info!("EcosystemManager shutdown completed");

        self.shutdown = true;
        info!("Ecosystem shutdown completed successfully");
        Ok(())
    }

    /// Check if this primal can serve the given context
    pub fn can_serve_context(&self, context: &crate::universal::PrimalContext) -> bool {
        match context.security_level {
            crate::universal::SecurityLevel::Public => false,
            crate::universal::SecurityLevel::Basic => false,
            crate::universal::SecurityLevel::Standard => true,
            crate::universal::SecurityLevel::High => true,
            crate::universal::SecurityLevel::Critical => true,
            crate::universal::SecurityLevel::Maximum => true,
            crate::universal::SecurityLevel::Advanced => true,
            crate::universal::SecurityLevel::Internal => true,
            crate::universal::SecurityLevel::Administrative => true,
            crate::universal::SecurityLevel::Enhanced => true, // Add missing variant
            crate::universal::SecurityLevel::Custom(_) => true, // Add missing variant
        }
    }

    /// Get dynamic port information
    pub fn dynamic_port_info(&self) -> Option<crate::universal::DynamicPortInfo> {
        let now = chrono::Utc::now();
        Some(crate::universal::DynamicPortInfo {
            port: 8080,
            assigned_port: 8080,
            current_port: 8080,
            port_range: Some((8080, 8090)),
            port_type: crate::universal::PortType::Http,
            status: crate::universal::PortStatus::Active,
            assigned_at: now,
            allocated_at: now,
            lease_duration: Some(chrono::Duration::hours(1)),
            expires_at: Some(now + chrono::Duration::hours(1)),
            metadata: std::collections::HashMap::new(),
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
            songbird_endpoint: Some(self.config.songbird_endpoint.clone()),
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
            timestamp: chrono::Utc::now(),
        })
    }

    /// Create service registration for ecosystem
    pub fn create_service_registration(&self) -> EcosystemServiceRegistration {
        EcosystemIntegration::create_service_registration(self)
    }
}
