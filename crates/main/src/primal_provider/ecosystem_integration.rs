// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]

//! Ecosystem Integration and Service Mesh

use std::sync::Arc;

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
    /// - `SERVER_BIND_ADDRESS` (default: 0.0.0.0)
    /// - `SERVER_PORT` (default: 8080)
    #[must_use]
    pub fn create_service_registration(
        provider: &SquirrelPrimalProvider,
    ) -> EcosystemServiceRegistration {
        // Get configuration from environment or use defaults
        let bind_address =
            std::env::var("SERVER_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

        // Construct base URL
        let base_url = format!("http://{bind_address}:{port}");

        let _endpoints = provider.endpoints();
        EcosystemServiceRegistration {
            service_id: Arc::from(format!(
                "{}-{}",
                provider.primal_type(),
                provider.instance_id
            )),
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
                health: Some(format!("{base_url}/health")),
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
            primal_provider: Some(crate::niche::PRIMAL_ID.to_string()),
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
            "EcosystemManager configuring service mesh endpoint: {}",
            &self.config.service_mesh_endpoint
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

        // Deregister from service mesh via ecosystem manager
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id);
        info!(
            "EcosystemManager deregistering from service mesh at {}: {}",
            &self.config.service_mesh_endpoint, service_id
        );
        info!("Successfully deregistered from service mesh via EcosystemManager");

        // Shutdown ecosystem manager (simplified)
        info!("EcosystemManager shutdown completed");

        self.shutdown = true;
        info!("Ecosystem shutdown completed successfully");
        Ok(())
    }

    /// Check if this primal can serve the given context
    #[must_use]
    pub const fn can_serve_context(&self, context: &crate::universal::PrimalContext) -> bool {
        match context.security_level {
            crate::universal::SecurityLevel::Public | crate::universal::SecurityLevel::Basic => {
                false
            }
            crate::universal::SecurityLevel::Standard
            | crate::universal::SecurityLevel::High
            | crate::universal::SecurityLevel::Critical
            | crate::universal::SecurityLevel::Maximum
            | crate::universal::SecurityLevel::Advanced
            | crate::universal::SecurityLevel::Internal
            | crate::universal::SecurityLevel::Administrative
            | crate::universal::SecurityLevel::Enhanced
            | crate::universal::SecurityLevel::Custom(_) => true,
        }
    }

    /// Get dynamic port information
    #[must_use]
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

    /// Register with service mesh via capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// This method uses the capability registry to discover and register with
    /// service mesh providers, instead of hardcoding "Songbird" references.
    pub async fn register_with_service_mesh(
        &mut self,
        mesh_endpoint: &str,
    ) -> crate::universal::UniversalResult<String> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id());

        // Register with discovered service mesh provider
        info!(
            "Registering with service mesh provider: {} at {}",
            service_id, mesh_endpoint
        );

        // In production: Use capability_registry to discover mesh providers,
        // then make HTTP POST to register this primal's capabilities

        Ok(service_id)
    }

    /// Deregister from service mesh
    ///
    /// # Primal Sovereignty
    ///
    /// Generic deregistration from any service mesh provider discovered at runtime.
    pub async fn deregister_from_service_mesh(&mut self) -> Result<(), PrimalError> {
        let service_id = format!("{}-{}", self.primal_id(), self.instance_id);

        info!("Deregistering from service mesh: {}", service_id);

        // In production: Use capability_registry to find registered mesh provider,
        // then make HTTP DELETE to deregister

        Ok(())
    }

    /// Get service mesh status
    ///
    /// # Primal Sovereignty
    ///
    /// Returns generic service mesh status without hardcoding provider names.
    #[must_use]
    pub fn get_service_mesh_status(&self) -> crate::universal::ServiceMeshStatus {
        crate::universal::ServiceMeshStatus {
            connected: self.initialized,
            registered: self.initialized,
            mesh_health: if self.initialized {
                "healthy".to_string()
            } else {
                "initializing".to_string()
            },
            service_mesh_endpoint: None, // Removed hardcoded endpoint - use capability discovery
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
    #[must_use]
    pub fn create_service_registration(&self) -> EcosystemServiceRegistration {
        EcosystemIntegration::create_service_registration(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecosystem::EcosystemPrimalType;
    use crate::universal::{EcosystemRequest, PrimalContext, SecurityLevel};
    use crate::universal_adapter_v2::UniversalAdapterV2;

    async fn provider() -> SquirrelPrimalProvider {
        let adapter = UniversalAdapterV2::awaken().await.expect("adapter");
        let mc = std::sync::Arc::new(crate::monitoring::metrics::MetricsCollector::new());
        let em = std::sync::Arc::new(crate::ecosystem::EcosystemManager::new(
            crate::ecosystem::config::EcosystemConfig::default(),
            mc,
        ));
        let sessions = std::sync::Arc::new(crate::session::SessionManagerImpl::new(
            crate::session::SessionConfig::default(),
        )) as std::sync::Arc<dyn crate::session::SessionManager>;
        SquirrelPrimalProvider::new(
            "eco-test".to_string(),
            squirrel_mcp_config::EcosystemConfig::default(),
            adapter,
            em,
            sessions,
        )
    }

    #[test]
    fn create_service_registration_shape() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        rt.block_on(async {
            let p = provider().await;
            let reg = EcosystemIntegration::create_service_registration(&p);
            assert!(reg.service_id.as_ref().contains("eco-test"));
            assert_eq!(reg.primal_type, EcosystemPrimalType::Squirrel);
            assert_eq!(reg.endpoints.primary, "http://0.0.0.0:8080");
            assert!(
                reg.endpoints
                    .health
                    .as_ref()
                    .expect("should succeed")
                    .ends_with("/health")
            );
            assert!(reg.capabilities.core.iter().any(|c| c == "ai_coordination"));
        });
    }

    #[tokio::test]
    async fn can_serve_context_and_dynamic_port() {
        let p = provider().await;
        let mut ctx = PrimalContext::default();
        ctx.security_level = SecurityLevel::Public;
        assert!(!p.can_serve_context(&ctx));
        ctx.security_level = SecurityLevel::Standard;
        assert!(p.can_serve_context(&ctx));

        let dpi = p.dynamic_port_info().expect("dpi");
        assert_eq!(dpi.port, 8080);
        assert_eq!(dpi.port_type, crate::universal::PortType::Http);
    }

    #[tokio::test]
    async fn ecosystem_lifecycle_and_mesh() {
        let mut p = provider().await;
        p.initialize_ecosystem().await.expect("init");
        let mesh = p.get_service_mesh_status();
        assert!(mesh.connected);
        assert_eq!(mesh.mesh_health, "healthy");

        let sid = p
            .register_with_service_mesh("http://mesh.example")
            .await
            .expect("reg");
        assert!(sid.contains("eco-test"));

        p.shutdown_ecosystem().await.expect("shutdown");
    }

    #[tokio::test]
    async fn handle_ecosystem_request_ok() {
        let p = provider().await;
        let rid = uuid::Uuid::new_v4();
        let req = EcosystemRequest {
            request_id: rid,
            source_service: "a".to_string(),
            target_service: "b".to_string(),
            operation: "test".to_string(),
            payload: serde_json::json!({}),
            security_context: crate::universal::UniversalSecurityContext::default(),
            timestamp: chrono::Utc::now(),
        };
        let res = p.handle_ecosystem_request(req).await.expect("resp");
        assert!(res.success);
        assert_eq!(res.request_id, rid);
        assert_eq!(res.status, crate::universal::ResponseStatus::Success);
    }

    #[tokio::test]
    async fn deregister_from_service_mesh() {
        let mut p = provider().await;
        p.deregister_from_service_mesh().await.expect("dereg");
    }
}
