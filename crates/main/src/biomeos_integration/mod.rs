//! # Squirrel biomeOS Integration
//!
//! This module provides integration with the biomeOS ecosystem, allowing squirrel
//! to register as an AI intelligence primal and provide MCP protocol services,
//! AI capabilities, and context state management for the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tracing::warn;

use crate::error::PrimalError;
use squirrel_mcp_config::DefaultConfigManager;

// Constants to reduce string allocations
const PRIMAL_TYPE: &str = "squirrel";
const API_VERSION: &str = "biomeOS/v1";
const STATUS_INITIALIZING: &str = "initializing";
const STATUS_STARTING: &str = "starting";
const STATUS_RUNNING: &str = "running";

pub mod agent_deployment;
pub mod ai_intelligence;
pub mod context_state;
pub mod ecosystem_client;
pub mod manifest;
pub mod mcp_integration;
pub mod optimized_implementations;

// Re-export optimized implementations
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
pub use ecosystem_client::{
    AuthenticationConfig as EcosystemAuthConfig, EcosystemClient, HealthCheckResponse,
    PrimalStatus, RegistrationResponse,
};
pub use manifest::{
    AgentManifest, AgentResourceLimits, AgentSpec, AuthenticationConfig as ManifestAuthConfig,
    BiomeManifest, BiomeManifestParser,
};
pub use mcp_integration::*;

/// biomeOS Ecosystem Service Registration for Squirrel AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    pub service_id: String,
    pub primal_type: String,
    pub biome_id: String,
    pub version: String,
    pub api_version: String,
    pub registration_time: DateTime<Utc>,
    pub endpoints: EcosystemEndpoints,
    pub capabilities: EcosystemCapabilities,
    pub security: EcosystemSecurity,
    pub resource_requirements: ResourceRequirements,
    pub health_check: HealthCheckConfig,
    pub metadata: HashMap<String, String>,
}

/// Ecosystem endpoints for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEndpoints {
    pub ai_api: String,            // Primary AI intelligence API
    pub mcp_api: String,           // MCP protocol endpoint
    pub context_api: String,       // Context state management API
    pub health: String,            // Health check endpoint
    pub metrics: String,           // Metrics endpoint
    pub websocket: Option<String>, // Real-time AI updates
}

/// Ecosystem capabilities provided by squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemCapabilities {
    pub ai_capabilities: Vec<String>,          // Core AI services
    pub mcp_capabilities: Vec<String>,         // MCP protocol capabilities
    pub context_capabilities: Vec<String>,     // Context management capabilities
    pub integration_capabilities: Vec<String>, // Integration with other primals
}

/// Security configuration for ecosystem integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemSecurity {
    pub authentication_method: String,
    pub tls_enabled: bool,
    pub mtls_required: bool,
    pub trust_domain: String,
}

/// Resource requirements for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu: String,
    pub memory: String,
    pub storage: String,
    pub network: String,
    pub gpu: Option<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub grace_period: Duration,
}

/// Main biomeOS integration for squirrel AI
pub struct SquirrelBiomeOSIntegration {
    pub service_id: String,
    pub biome_id: String,
    pub ai_intelligence: AiIntelligence,
    pub mcp_integration: McpIntegration,
    pub context_state: ContextState,
    pub ecosystem_client: EcosystemClient,
    pub agent_deployment: AgentDeploymentManager,
    pub manifest_parser: BiomeManifestParser,
    pub health_status: HealthStatus,
}

/// Health status of the squirrel AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub ai_engine_status: String,
    pub mcp_server_status: String,
    pub context_manager_status: String,
    pub agent_deployment_status: String,
    pub active_sessions: u32,
    pub ai_requests_processed: u64,
    pub context_states_managed: u32,
    pub deployed_agents: u32,
}

impl SquirrelBiomeOSIntegration {
    /// Create new biomeOS integration for squirrel AI
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
            ecosystem_client: EcosystemClient::new(),
            agent_deployment,
            manifest_parser: BiomeManifestParser::new(),
            health_status: HealthStatus {
                status: STATUS_INITIALIZING.to_string(),
                timestamp: Utc::now(),
                ai_engine_status: STATUS_STARTING.to_string(),
                mcp_server_status: STATUS_STARTING.to_string(),
                context_manager_status: STATUS_STARTING.to_string(),
                agent_deployment_status: STATUS_STARTING.to_string(),
                active_sessions: 0,
                ai_requests_processed: 0,
                context_states_managed: 0,
                deployed_agents: 0,
            },
        }
    }

    /// Create an optimized version of the BiomeOS integration
    pub fn new_optimized() -> SquirrelBiomeOSIntegration {
        SquirrelBiomeOSIntegration::new("optimized-squirrel".to_string())
    }

    /// Migrate to optimized implementation
    pub async fn migrate_to_optimized(self) -> Result<SquirrelBiomeOSIntegration, PrimalError> {
        let optimized = SquirrelBiomeOSIntegration::new("migration-squirrel".to_string());

        // Migration logic would go here to transfer state
        // For now, we return a fresh optimized instance

        Ok(optimized)
    }

    /// Register squirrel AI with biomeOS ecosystem
    pub async fn register_with_biomeos(&mut self) -> Result<(), PrimalError> {
        let config_manager = DefaultConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        let registration = EcosystemServiceRegistration {
            service_id: self.service_id.clone(),
            primal_type: PRIMAL_TYPE.to_string(),
            biome_id: self.biome_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: API_VERSION.to_string(),
            registration_time: Utc::now(),

            endpoints: {
                use squirrel_mcp_config::core::{DevelopmentConfig, NetworkEndpointConfig};
                let network_config = NetworkEndpointConfig::default();
                let dev_config = DevelopmentConfig::default();
                let base_url = network_config.get_http_url(&dev_config);

                EcosystemEndpoints {
                    ai_api: _endpoints
                        .get("ai_api")
                        .cloned()
                        .unwrap_or_else(|| format!("{}/ai", base_url)),
                    mcp_api: _endpoints
                        .get("mcp_api")
                        .cloned()
                        .unwrap_or_else(|| format!("{}/mcp", base_url)),
                    context_api: _endpoints
                        .get("context_api")
                        .cloned()
                        .unwrap_or_else(|| format!("{}/context", base_url)),
                    health: _endpoints
                        .get("health")
                        .cloned()
                        .unwrap_or_else(|| format!("{}/health", base_url)),
                    metrics: _endpoints
                        .get("metrics")
                        .cloned()
                        .unwrap_or_else(|| format!("{}/metrics", base_url)),
                    websocket: _endpoints.get("websocket").cloned(),
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
                    "songbird_ai_coordination".to_string(),
                    "toadstool_workload_intelligence".to_string(),
                    "nestgate_storage_optimization".to_string(),
                    "beardog_security_intelligence".to_string(),
                    "biomeos_ecosystem_intelligence".to_string(),
                    "biome_manifest_processing".to_string(),
                    "agent_deployment".to_string(),
                ],
            },

            security: EcosystemSecurity {
                authentication_method: "ecosystem_jwt".to_string(),
                tls_enabled: true,
                mtls_required: false, // Will be true when BearDog is integrated
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

        // Register with songbird (service registry)
        self.ecosystem_client
            .register_service_with_songbird(registration)
            .await?;
        self.health_status.status = "registered".to_string();
        self.health_status.timestamp = Utc::now();

        Ok(())
    }

    /// Start AI intelligence and MCP services
    pub async fn start_ecosystem_services(&mut self) -> Result<(), PrimalError> {
        let config_manager = DefaultConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

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
        let config_manager = DefaultConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        // Start AI intelligence background task
        let ai_intelligence = self.ai_intelligence.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = ai_intelligence.provide_ecosystem_intelligence().await {
                    warn!("AI intelligence error: {}", e);
                }
                let interval = std::env::var("AI_INTELLIGENCE_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(30);
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });

        Ok(())
    }

    /// Start MCP coordination services
    async fn start_mcp_coordination(&mut self) -> Result<(), PrimalError> {
        let config_manager = DefaultConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        // Start MCP coordination background task
        let mcp_integration = self.mcp_integration.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = mcp_integration.coordinate_with_ecosystem().await {
                    warn!("MCP coordination error: {}", e);
                }
                let interval = std::env::var("MCP_COORDINATION_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(45);
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });

        Ok(())
    }

    /// Start context management services
    async fn start_context_management(&mut self) -> Result<(), PrimalError> {
        let config_manager = DefaultConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        // Start context management background task
        let context_state = self.context_state.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = context_state.manage_ecosystem_context().await {
                    warn!("Context management error: {}", e);
                }
                let interval = std::env::var("CONTEXT_MANAGEMENT_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(45);
                tokio::time::sleep(Duration::from_secs(interval)).await;
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
    pub fn get_health_status(&self) -> &HealthStatus {
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
            self.health_status.ai_engine_status = format!("unhealthy: {}", e);
        } else {
            self.health_status.ai_engine_status = STATUS_RUNNING.to_string();
        }

        // Check MCP integration health
        if let Err(e) = self.mcp_integration.health_check().await {
            self.health_status.mcp_server_status = format!("unhealthy: {}", e);
        } else {
            self.health_status.mcp_server_status = STATUS_RUNNING.to_string();
        }

        // Check context state health
        if let Err(e) = self.context_state.health_check().await {
            self.health_status.context_manager_status = format!("unhealthy: {}", e);
        } else {
            self.health_status.context_manager_status = STATUS_RUNNING.to_string();
        }

        // Check agent deployment health
        if let Err(e) = self.agent_deployment.health_check().await {
            self.health_status.agent_deployment_status = format!("unhealthy: {}", e);
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

/// Request types for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceRequest {
    pub request_id: String,
    pub request_type: String,
    pub target_component: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub context: Option<HashMap<String, String>>,
}

/// Response from BiomeOS intelligence services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceResponse {
    pub request_id: String,
    pub intelligence_type: String,
    pub result: serde_json::Value,
    pub confidence: f64,
    pub processing_time_ms: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCoordinationRequest {
    pub coordination_id: String,
    pub coordination_type: String,
    pub participants: Vec<String>,
    pub coordination_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCoordinationResponse {
    pub coordination_id: String,
    pub status: String,
    pub coordination_plan: Vec<CoordinationStep>,
    pub estimated_completion: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStateRequest {
    pub request_id: String,
    pub session_id: String,
    pub request_type: String,
    pub context_data: Option<HashMap<String, serde_json::Value>>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStateResponse {
    pub request_id: String,
    pub session_id: String,
    pub context_state: HashMap<String, serde_json::Value>,
    pub recommendations: Vec<String>,
    pub related_contexts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub prediction_id: String,
    pub prediction_type: String,
    pub confidence: f64,
    pub timeframe: String,
    pub description: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub optimization_id: String,
    pub optimization_type: String,
    pub target_component: String,
    pub improvement_potential: f64,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationStep {
    pub step_id: String,
    pub step_type: String,
    pub participants: Vec<String>,
    pub estimated_duration: Duration,
    pub dependencies: Vec<String>,
}

// Default implementations
impl Default for EcosystemServiceRegistration {
    fn default() -> Self {
        Self {
            service_id: "primal-squirrel-ai-default".to_string(),
            primal_type: PRIMAL_TYPE.to_string(),
            biome_id: "default-biome".to_string(),
            version: "1.0.0".to_string(),
            api_version: API_VERSION.to_string(),
            registration_time: Utc::now(),
            endpoints: EcosystemEndpoints::default(),
            capabilities: EcosystemCapabilities::default(),
            security: EcosystemSecurity::default(),
            resource_requirements: ResourceRequirements::default(),
            health_check: HealthCheckConfig::default(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for EcosystemEndpoints {
    fn default() -> Self {
        Self {
            ai_api: std::env::var("BIOMEOS_AI_API")
                .unwrap_or_else(|_| "http://localhost:5000/ai".to_string()),
            mcp_api: std::env::var("BIOMEOS_MCP_API")
                .unwrap_or_else(|_| "http://localhost:5000/mcp".to_string()),
            context_api: std::env::var("BIOMEOS_CONTEXT_API")
                .unwrap_or_else(|_| "http://localhost:5000/context".to_string()),
            health: std::env::var("BIOMEOS_HEALTH_API")
                .unwrap_or_else(|_| "http://localhost:5000/health".to_string()),
            metrics: std::env::var("BIOMEOS_METRICS_API")
                .unwrap_or_else(|_| "http://localhost:5000/metrics".to_string()),
            websocket: std::env::var("BIOMEOS_WEBSOCKET_URL")
                .ok()
                .or_else(|| Some("ws://localhost:5000/ws".to_string())),
        }
    }
}

impl Default for EcosystemCapabilities {
    fn default() -> Self {
        Self {
            ai_capabilities: vec![
                "ecosystem_intelligence".to_string(),
                "predictive_analytics".to_string(),
            ],
            mcp_capabilities: vec![
                "protocol_coordination".to_string(),
                "session_management".to_string(),
            ],
            context_capabilities: vec![
                "state_management".to_string(),
                "context_persistence".to_string(),
            ],
            integration_capabilities: vec!["biomeos_ecosystem_intelligence".to_string()],
        }
    }
}

impl Default for EcosystemSecurity {
    fn default() -> Self {
        Self {
            authentication_method: "ecosystem_jwt".to_string(),
            tls_enabled: true,
            mtls_required: false,
            trust_domain: "biome.local".to_string(),
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu: "4".to_string(),
            memory: "8Gi".to_string(),
            storage: "20Gi".to_string(),
            network: "1Gbps".to_string(),
            gpu: Some("1".to_string()),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            retries: 3,
            grace_period: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
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
        assert!(registration
            .capabilities
            .ai_capabilities
            .contains(&"ecosystem_intelligence".to_string()));
    }

    #[tokio::test]
    async fn test_health_status_update() {
        let mut integration = SquirrelBiomeOSIntegration::new("test-biome".to_string());
        let original_timestamp = integration.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        integration.update_health_status("running");
        assert!(integration.health_status.timestamp > original_timestamp);
    }
}
