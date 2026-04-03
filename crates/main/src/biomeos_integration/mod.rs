// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Squirrel biomeOS Integration
//!
//! This module provides integration with the biomeOS ecosystem, allowing squirrel
//! to register as an AI intelligence primal and provide MCP protocol services,
//! AI capabilities, and context state management for the ecosystem.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use crate::error::PrimalError;

pub(crate) const PRIMAL_TYPE: &str = "squirrel";
pub(crate) const API_VERSION: &str = "biomeOS/v1";
pub(crate) const STATUS_INITIALIZING: &str = "initializing";
pub(crate) const STATUS_STARTING: &str = "starting";
const STATUS_RUNNING: &str = "running";

pub mod agent_deployment;
pub mod ai_intelligence;
pub mod context_state;
pub mod manifest;
pub mod mcp_integration;
pub mod optimized_implementations;
pub mod types;

// Re-export types from the extracted types module
pub use types::*;

pub use optimized_implementations::{
    ContextData, OptimizedContextState, OptimizedServiceRegistration, SessionContext,
};

pub use agent_deployment::{
    AgentDeploymentConfig, AgentDeploymentManager, AgentStatus, DeployedAgent, DeploymentStatus,
    ResourceUtilization as AgentResourceUtilization,
};
pub use ai_intelligence::{
    AiIntelligence, IntelligenceEngine, OptimizationEngine, PredictionEngine,
    ResourceUtilization as AIResourceUtilization,
};
pub use context_state::*;
pub use manifest::{
    AgentManifest, AgentResourceLimits, AgentSpec, AuthenticationConfig as ManifestAuthConfig,
    BiomeManifest, BiomeManifestParser,
};
pub use mcp_integration::*;

/// Main biomeOS integration for squirrel AI
pub struct SquirrelBiomeOSIntegration {
    /// Unique identifier for this service instance.
    pub service_id: String,
    /// Identifier of the biome this service belongs to.
    pub biome_id: String,
    /// AI intelligence engine for predictions and optimization.
    pub ai_intelligence: AiIntelligence,
    /// MCP protocol integration layer.
    pub mcp_integration: McpIntegration,
    /// Context state management.
    pub context_state: ContextState,
    /// Manages agent deployment and lifecycle.
    pub agent_deployment: AgentDeploymentManager,
    /// Parses biome manifests.
    pub manifest_parser: BiomeManifestParser,
    /// Current health status of the integration.
    pub health_status: HealthStatus,
}

impl SquirrelBiomeOSIntegration {
    /// Create new biomeOS integration for squirrel AI
    #[must_use]
    pub fn new(biome_id: String) -> Self {
        let service_id = format!("primal-squirrel-ai-{}", uuid::Uuid::new_v4());

        let mcp_integration = McpIntegration::new();
        let ai_intelligence = AiIntelligence::new();
        let agent_deployment = AgentDeploymentManager::new(
            AgentDeploymentConfig::default(),
            Arc::new(mcp_integration.clone()),
            Arc::new(ai_intelligence.clone()),
        );

        Self {
            service_id,
            biome_id,
            ai_intelligence,
            mcp_integration,
            context_state: ContextState::new(),
            agent_deployment,
            manifest_parser: BiomeManifestParser::new(),
            health_status: HealthStatus::initializing(),
        }
    }

    /// Create an optimized version of the `BiomeOS` integration
    #[must_use]
    pub fn new_optimized() -> Self {
        Self::new("optimized-squirrel".to_string())
    }

    /// Migrate to optimized implementation
    pub async fn migrate_to_optimized(self) -> Result<Self, PrimalError> {
        let optimized = Self::new("migration-squirrel".to_string());

        // Migration logic would go here to transfer state
        // For now, we return a fresh optimized instance

        Ok(optimized)
    }

    /// Register squirrel AI with biomeOS ecosystem
    #[expect(clippy::too_many_lines, reason = "Integration logic; refactor planned")]
    pub async fn register_with_biomeos(&mut self) -> Result<(), PrimalError> {
        let registration = EcosystemServiceRegistration {
            service_id: self.service_id.clone(),
            primal_type: PRIMAL_TYPE.to_string(),
            biome_id: self.biome_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: API_VERSION.to_string(),
            registration_time: Utc::now(),

            endpoints: {
                // Use environment-aware configuration for base URL
                let host = if std::env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .eq_ignore_ascii_case("production")
                {
                    "0.0.0.0"
                } else {
                    "127.0.0.1"
                };
                let port = std::env::var("SQUIRREL_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(8778);
                let base_url = format!("http://{host}:{port}");

                EcosystemEndpoints {
                    ai_api: format!("{base_url}/ai"),
                    mcp_api: format!("{base_url}/mcp"),
                    context_api: format!("{base_url}/context"),
                    health: format!("{base_url}/health"),
                    metrics: format!("{base_url}/metrics"),
                    websocket: None,
                }
            },

            capabilities: EcosystemCapabilities {
                ai_capabilities: vec![
                    "ecosystem_intelligence".to_string(),
                    "predictive_analytics".to_string(),
                    "optimization_recommendations".to_string(),
                    "anomaly_detection".to_string(),
                    "performance_analysis".to_string(),
                    "resource_prediction".to_string(),
                    "failure_prediction".to_string(),
                    "capacity_planning".to_string(),
                    "intelligent_routing".to_string(),
                    "adaptive_learning".to_string(),
                ],
                mcp_capabilities: vec![
                    "protocol_coordination".to_string(),
                    "message_routing".to_string(),
                    "session_management".to_string(),
                    "tool_orchestration".to_string(),
                    "resource_coordination".to_string(),
                    "multi_agent_coordination".to_string(),
                    "context_sharing".to_string(),
                    "state_synchronization".to_string(),
                ],
                context_capabilities: vec![
                    "state_management".to_string(),
                    "context_persistence".to_string(),
                    "cross_session_context".to_string(),
                    "contextual_recommendations".to_string(),
                    "context_analytics".to_string(),
                    "state_versioning".to_string(),
                    "context_migration".to_string(),
                    "contextual_search".to_string(),
                ],
                integration_capabilities: vec![
                    // Capability-based integration (no hardcoded primal names)
                    // Discovery happens at runtime via ServiceDiscoveryClient
                    "service_mesh_coordination".to_string(),
                    "workload_intelligence".to_string(),
                    "storage_optimization".to_string(),
                    "security_intelligence".to_string(),
                    "ecosystem_intelligence".to_string(),
                    "manifest_processing".to_string(),
                    "agent_deployment".to_string(),
                ],
            },

            security: EcosystemSecurity {
                authentication_method: "ecosystem_jwt".to_string(),
                tls_enabled: true,
                mtls_required: false, // Will be true when security primal is discovered
                trust_domain: "biome.local".to_string(),
            },

            resource_requirements: ResourceRequirements {
                cpu: "4".to_string(),
                memory: "8Gi".to_string(),
                storage: "20Gi".to_string(),
                network: "1Gbps".to_string(),
                gpu: Some("1".to_string()), // Optional GPU for AI workloads
            },

            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
                retries: 3,
                grace_period: Duration::from_secs(60),
            },

            metadata: {
                let mut meta = HashMap::new();
                meta.insert("role".to_string(), "ai_intelligence".to_string());
                meta.insert("specialization".to_string(), "mcp_ai_context".to_string());
                meta.insert("ai_engine".to_string(), "enhanced_coordinator".to_string());
                meta.insert("mcp_version".to_string(), "2.0".to_string());
                meta.insert("context_engine".to_string(), "advanced".to_string());
                meta.insert("federation_ready".to_string(), "true".to_string());
                meta.insert("manifest_support".to_string(), "biome_yaml".to_string());
                meta.insert("agent_deployment".to_string(), "enabled".to_string());
                meta
            },
        };

        // Registration is announced via the JSON-RPC discover_capabilities endpoint.
        // Other primals probe our socket and discover our capabilities at runtime.
        tracing::info!(
            "Service registration prepared for capability discovery: {:?}",
            registration.service_id
        );

        self.health_status.status = "registered".to_string();
        self.health_status.timestamp = Utc::now();

        Ok(())
    }

    /// Start AI intelligence and MCP services
    pub async fn start_ecosystem_services(&mut self) -> Result<(), PrimalError> {
        // Initialize AI intelligence
        self.ai_intelligence.initialize().await?;
        self.health_status.ai_engine_status = STATUS_RUNNING.to_string();

        // Initialize MCP integration
        self.mcp_integration.initialize().await?;
        self.health_status.mcp_server_status = STATUS_RUNNING.to_string();

        // Initialize context state management
        self.context_state.initialize().await?;
        self.health_status.context_manager_status = STATUS_RUNNING.to_string();

        // Agent deployment is initialized by default
        self.health_status.agent_deployment_status = STATUS_RUNNING.to_string();

        // Start ecosystem AI services
        self.start_ecosystem_intelligence().await?;
        self.start_mcp_coordination().await?;
        self.start_context_management().await?;

        self.health_status.status = STATUS_RUNNING.to_string();
        self.health_status.timestamp = Utc::now();

        Ok(())
    }

    /// Deploy agents from a biome.yaml manifest file
    pub async fn deploy_agents_from_manifest_file(
        &mut self,
        manifest_path: &str,
    ) -> Result<Vec<String>, PrimalError> {
        let manifest = self.manifest_parser.parse_file(manifest_path).await?;
        self.deploy_agents_from_manifest(&manifest).await
    }

    /// Deploy agents from a biome.yaml manifest
    pub async fn deploy_agents_from_manifest(
        &mut self,
        manifest: &BiomeManifest,
    ) -> Result<Vec<String>, PrimalError> {
        let deployed_agents = self.agent_deployment.deploy_from_manifest(manifest).await?;

        // Update health status
        self.health_status.deployed_agents = deployed_agents.len() as u32;
        self.health_status.timestamp = Utc::now();

        Ok(deployed_agents)
    }

    /// Stop a deployed agent
    pub async fn stop_agent(&mut self, agent_id: &str) -> Result<(), PrimalError> {
        self.agent_deployment.stop_agent(agent_id).await?;

        // Update health status
        let agents = self.agent_deployment.list_agents().await;
        self.health_status.deployed_agents = agents.len() as u32;
        self.health_status.timestamp = Utc::now();

        Ok(())
    }

    /// List all deployed agents
    pub async fn list_deployed_agents(&self) -> Vec<DeployedAgent> {
        self.agent_deployment.list_agents().await
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self) -> DeploymentStatus {
        self.agent_deployment.get_deployment_status().await
    }

    /// Generate a biome.yaml manifest template
    #[must_use]
    pub fn generate_manifest_template(&self) -> BiomeManifest {
        BiomeManifestParser::generate_template()
    }

    /// Parse a biome.yaml manifest from content
    pub async fn parse_manifest_content(
        &self,
        content: &str,
    ) -> Result<BiomeManifest, PrimalError> {
        self.manifest_parser.parse_content(content).await
    }

    /// Start ecosystem intelligence services
    async fn start_ecosystem_intelligence(&mut self) -> Result<(), PrimalError> {
        // Start AI intelligence background task
        let ai_intelligence = self.ai_intelligence.clone();
        tokio::spawn(async move {
            // Evolution: Use interval ticker instead of loop+sleep
            let interval_secs = std::env::var("AI_INTELLIGENCE_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(30);

            let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                if let Err(e) = ai_intelligence.provide_ecosystem_intelligence().await {
                    warn!("AI intelligence error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start MCP coordination services
    async fn start_mcp_coordination(&mut self) -> Result<(), PrimalError> {
        // Start MCP coordination background task
        let mcp_integration = self.mcp_integration.clone();
        tokio::spawn(async move {
            // Evolution: Use interval ticker instead of loop+sleep
            let interval_secs = std::env::var("MCP_COORDINATION_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(45);

            let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                if let Err(e) = mcp_integration.coordinate_with_ecosystem().await {
                    warn!("MCP coordination error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start context management services
    async fn start_context_management(&mut self) -> Result<(), PrimalError> {
        // Start context management background task
        let context_state = self.context_state.clone();
        tokio::spawn(async move {
            // Evolution: Use interval ticker instead of loop+sleep
            let interval_secs = std::env::var("CONTEXT_MANAGEMENT_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(45);

            let mut ticker = tokio::time::interval(Duration::from_secs(interval_secs));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                ticker.tick().await;

                if let Err(e) = context_state.manage_ecosystem_context().await {
                    warn!("Context management error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Provide AI intelligence for ecosystem optimization
    pub async fn provide_ecosystem_intelligence(
        &self,
        request: IntelligenceRequest,
    ) -> Result<IntelligenceResponse, PrimalError> {
        self.ai_intelligence
            .process_intelligence_request(request)
            .await
    }

    /// Handle MCP protocol coordination
    pub async fn handle_mcp_coordination(
        &self,
        request: McpCoordinationRequest,
    ) -> Result<McpCoordinationResponse, PrimalError> {
        self.mcp_integration
            .handle_coordination_request(request)
            .await
    }

    /// Manage context state for sessions
    pub async fn manage_context_state(
        &self,
        request: ContextStateRequest,
    ) -> Result<ContextStateResponse, PrimalError> {
        self.context_state.handle_state_request(request).await
    }

    /// Get current health status
    #[must_use]
    pub const fn get_health_status(&self) -> &HealthStatus {
        &self.health_status
    }

    /// Update health status
    pub fn update_health_status(&mut self, status: &str) {
        self.health_status.status = status.to_string();
        self.health_status.timestamp = Utc::now();
    }

    /// Perform health check on all components
    pub async fn health_check(&mut self) -> Result<(), PrimalError> {
        // Check AI intelligence health
        if let Err(e) = self.ai_intelligence.health_check().await {
            self.health_status.ai_engine_status = format!("unhealthy: {e}");
        } else {
            self.health_status.ai_engine_status = STATUS_RUNNING.to_string();
        }

        // Check MCP integration health
        if let Err(e) = self.mcp_integration.health_check().await {
            self.health_status.mcp_server_status = format!("unhealthy: {e}");
        } else {
            self.health_status.mcp_server_status = STATUS_RUNNING.to_string();
        }

        // Check context state health
        if let Err(e) = self.context_state.health_check().await {
            self.health_status.context_manager_status = format!("unhealthy: {e}");
        } else {
            self.health_status.context_manager_status = STATUS_RUNNING.to_string();
        }

        // Check agent deployment health
        if let Err(e) = self.agent_deployment.health_check().await {
            self.health_status.agent_deployment_status = format!("unhealthy: {e}");
        } else {
            self.health_status.agent_deployment_status = STATUS_RUNNING.to_string();
        }

        // Update overall health status
        let all_healthy = self.health_status.ai_engine_status == STATUS_RUNNING
            && self.health_status.mcp_server_status == STATUS_RUNNING
            && self.health_status.context_manager_status == STATUS_RUNNING
            && self.health_status.agent_deployment_status == STATUS_RUNNING;

        if all_healthy {
            self.health_status.status = STATUS_RUNNING.to_string();
        } else {
            self.health_status.status = "degraded".to_string();
        }

        self.health_status.timestamp = Utc::now();

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Integration tests use expect on parse/deploy Result paths"
)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_squirrel_biomeos_integration_creation() {
        let integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());
        assert_eq!(integration.biome_id, "test-biome");
        assert_eq!(integration.health_status.status, STATUS_INITIALIZING);
    }

    #[tokio::test]
    async fn test_ecosystem_service_registration() {
        let registration = EcosystemServiceRegistration::default();
        assert_eq!(registration.primal_type, PRIMAL_TYPE);
        assert_eq!(registration.api_version, API_VERSION);
        assert!(
            registration
                .capabilities
                .ai_capabilities
                .contains(&"ecosystem_intelligence".to_string())
        );
    }

    #[tokio::test]
    async fn test_health_status_update() {
        let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());
        let original_timestamp = integration.health_status.timestamp;

        // Minimal wait for chrono::Utc::now() to advance (1ms is sufficient)
        tokio::time::sleep(Duration::from_millis(1)).await;

        integration.update_health_status("running");
        assert!(integration.health_status.timestamp >= original_timestamp);
    }

    #[tokio::test]
    async fn test_register_with_biomeos() {
        let mut integration = SquirrelBiomeOSIntegration::new("register-test".to_string());
        let result = integration.register_with_biomeos().await;
        assert!(result.is_ok());
        assert_eq!(integration.health_status.status, "registered");
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut integration = SquirrelBiomeOSIntegration::new("health-test".to_string());
        let result = integration.health_check().await;
        assert!(result.is_ok());
        assert!(
            integration.health_status.status == STATUS_RUNNING
                || integration.health_status.status == "degraded"
        );
    }

    #[tokio::test]
    async fn test_generate_manifest_template() {
        let integration = SquirrelBiomeOSIntegration::new("manifest-test".to_string());
        let manifest = integration.generate_manifest_template();
        assert!(!manifest.metadata.name.is_empty());
        assert!(!manifest.metadata.version.is_empty());
    }

    #[tokio::test]
    async fn test_parse_manifest_content() {
        let integration = SquirrelBiomeOSIntegration::new("parse-test".to_string());
        let yaml = r#"
metadata:
  name: parsed-biome
  description: Parsed test
  version: 2.0.0
  biomeos_version: 0.2.0
  created_at: 2024-01-01T00:00:00Z
  updated_at: 2024-01-01T00:00:00Z
  author: test
  labels: {}
agents: []
services: {}
resources:
  limits: { memory_gb: 8.0, cpu_cores: 4.0, storage_gb: 100.0, network_bandwidth_mbps: 1000.0 }
  reservations: { memory_gb: 2.0, cpu_cores: 1.0, storage_gb: 10.0 }
  policies: { memory_over_commit: false, cpu_over_commit: true, storage_over_commit: false, resource_quotas: {} }
security:
  authentication: { enabled: true, method: oauth2, providers: [github] }
  authorization: { enabled: true, method: rbac, policies: [default] }
  encryption: { enabled: true, algorithm: AES256, key_size: 256, at_rest: true, in_transit: true }
  tokens: { enabled: true, expiration_seconds: 3600, refresh_enabled: true }
  policies: { network_policies: [], pod_security_policies: [], rbac_policies: [] }
storage: { volumes: [], volume_claim_templates: [], storage_classes: [] }
networking:
  ingress: { enabled: true, host: test.example.com, tls_enabled: true, annotations: {} }
  network_policies: []
  dns: { enabled: true, servers: ["8.8.8.8"], search_domains: [] }
primals: {}
"#;
        let result = integration.parse_manifest_content(yaml).await;
        assert!(result.is_ok());
        let manifest = result.expect("parse manifest");
        assert_eq!(manifest.metadata.name, "parsed-biome");
        assert_eq!(manifest.metadata.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_list_deployed_agents() {
        let integration = SquirrelBiomeOSIntegration::new("agents-test".to_string());
        let agents = integration.list_deployed_agents().await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_get_deployment_status() {
        let integration = SquirrelBiomeOSIntegration::new("deploy-test".to_string());
        let status = integration.get_deployment_status().await;
        let _ = status.total_agents; // usize is always >= 0
    }

    #[tokio::test]
    async fn test_deploy_agents_from_manifest() {
        let mut integration = SquirrelBiomeOSIntegration::new("deploy-manifest-test".to_string());
        let manifest = integration.generate_manifest_template();
        let result = integration.deploy_agents_from_manifest(&manifest).await;
        assert!(result.is_ok());
        let deployed = result.expect("deploy agents");
        let _ = deployed.len(); // usize is always >= 0
    }

    #[tokio::test]
    async fn test_migrate_to_optimized() {
        let integration = SquirrelBiomeOSIntegration::new("migrate-test".to_string());
        let result = integration.migrate_to_optimized().await;
        assert!(result.is_ok());
        let optimized = result.expect("migrate optimized");
        assert_eq!(optimized.biome_id, "migration-squirrel");
    }

    #[tokio::test]
    async fn test_new_optimized() {
        let integration = SquirrelBiomeOSIntegration::new_optimized();
        assert_eq!(integration.biome_id, "optimized-squirrel");
    }

    #[tokio::test]
    async fn test_ecosystem_endpoints_default() {
        let endpoints = EcosystemEndpoints::default();
        assert!(!endpoints.ai_api.is_empty());
        assert!(!endpoints.health.is_empty());
        assert!(endpoints.ai_api.contains("/ai"));
    }

    #[tokio::test]
    async fn test_resource_requirements_default() {
        let req = ResourceRequirements::default();
        assert_eq!(req.cpu, "4");
        assert_eq!(req.memory, "8Gi");
        assert!(req.gpu.is_some());
    }
}
