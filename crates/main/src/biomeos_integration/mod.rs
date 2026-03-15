// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
// Migrated from deprecated DefaultConfigManager (ADR-008)

// Constants to reduce string allocations
const PRIMAL_TYPE: &str = "squirrel";
const API_VERSION: &str = "biomeOS/v1";
const STATUS_INITIALIZING: &str = "initializing";
const STATUS_STARTING: &str = "starting";
const STATUS_RUNNING: &str = "running";

pub mod agent_deployment;
pub mod ai_intelligence;
pub mod context_state;
// ecosystem_client removed - deprecated, unused, had reqwest dependency
// unix_socket_client removed - HTTP-based test utility
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
// ecosystem_client re-exports removed (module deleted)
pub use manifest::{
    AgentManifest, AgentResourceLimits, AgentSpec, AuthenticationConfig as ManifestAuthConfig,
    BiomeManifest, BiomeManifestParser,
};
pub use mcp_integration::*;

/// biomeOS Ecosystem Service Registration for Squirrel AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Unique identifier for this service instance.
    pub service_id: String,
    /// Type of primal (e.g., "squirrel").
    pub primal_type: String,
    /// Identifier of the biome this service belongs to.
    pub biome_id: String,
    /// Service version string.
    pub version: String,
    /// API version for compatibility.
    pub api_version: String,
    /// When the service was registered.
    pub registration_time: DateTime<Utc>,
    /// API endpoints for the service.
    pub endpoints: EcosystemEndpoints,
    /// Capabilities this service provides.
    pub capabilities: EcosystemCapabilities,
    /// Security configuration for the service.
    pub security: EcosystemSecurity,
    /// Resource requirements for deployment.
    pub resource_requirements: ResourceRequirements,
    /// Health check configuration.
    pub health_check: HealthCheckConfig,
    /// Arbitrary metadata key-value pairs.
    pub metadata: HashMap<String, String>,
}

/// Ecosystem endpoints for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEndpoints {
    /// Primary AI intelligence API endpoint.
    pub ai_api: String,
    /// MCP protocol endpoint.
    pub mcp_api: String,
    /// Context state management API endpoint.
    pub context_api: String,
    /// Health check endpoint.
    pub health: String,
    /// Metrics endpoint.
    pub metrics: String,
    /// Optional WebSocket endpoint for real-time AI updates.
    pub websocket: Option<String>,
}

/// Ecosystem capabilities provided by squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemCapabilities {
    /// Core AI service capabilities.
    pub ai_capabilities: Vec<String>,
    /// MCP protocol capabilities.
    pub mcp_capabilities: Vec<String>,
    /// Context management capabilities.
    pub context_capabilities: Vec<String>,
    /// Integration capabilities with other primals.
    pub integration_capabilities: Vec<String>,
}

/// Security configuration for ecosystem integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemSecurity {
    /// Authentication method (e.g., "bearer", "mtls").
    pub authentication_method: String,
    /// Whether TLS is enabled.
    pub tls_enabled: bool,
    /// Whether mutual TLS is required.
    pub mtls_required: bool,
    /// Trust domain for certificate validation.
    pub trust_domain: String,
}

/// Resource requirements for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirement (e.g., "500m", "1").
    pub cpu: String,
    /// Memory requirement (e.g., "512Mi").
    pub memory: String,
    /// Storage requirement.
    pub storage: String,
    /// Network requirement.
    pub network: String,
    /// Optional GPU requirement.
    pub gpu: Option<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Interval between health checks.
    pub interval: Duration,
    /// Timeout for each health check.
    pub timeout: Duration,
    /// Number of retries before marking unhealthy.
    pub retries: u32,
    /// Grace period before health checks start after startup.
    pub grace_period: Duration,
}

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

/// Health status of the squirrel AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status (e.g., "healthy", "degraded").
    pub status: String,
    /// When the status was last updated.
    pub timestamp: DateTime<Utc>,
    /// AI engine status.
    pub ai_engine_status: String,
    /// MCP server status.
    pub mcp_server_status: String,
    /// Context manager status.
    pub context_manager_status: String,
    /// Agent deployment status.
    pub agent_deployment_status: String,
    /// Number of active sessions.
    pub active_sessions: u32,
    /// Total AI requests processed.
    pub ai_requests_processed: u64,
    /// Number of context states being managed.
    pub context_states_managed: u32,
    /// Number of deployed agents.
    pub deployed_agents: u32,
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

    /// Create an optimized version of the `BiomeOS` integration
    #[must_use]
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

/// Request types for squirrel AI services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// Type of intelligence request.
    pub request_type: String,
    /// Optional target component.
    pub target_component: Option<String>,
    /// Request parameters.
    pub parameters: HashMap<String, serde_json::Value>,
    /// Optional context for the request.
    pub context: Option<HashMap<String, String>>,
}

/// Response from `BiomeOS` intelligence services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceResponse {
    /// Request identifier matching the request.
    pub request_id: String,
    /// Type of intelligence result.
    pub intelligence_type: String,
    /// The intelligence result payload.
    pub result: serde_json::Value,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Processing time in milliseconds.
    pub processing_time_ms: u64,
    /// Additional metadata.
    pub metadata: std::collections::HashMap<String, String>,
}

/// MCP coordination request for multi-primal workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCoordinationRequest {
    /// Unique coordination session identifier.
    pub coordination_id: String,
    /// Type of coordination (e.g., "workflow", "consensus").
    pub coordination_type: String,
    /// Primal/service IDs participating in coordination.
    pub participants: Vec<String>,
    /// Coordination-specific data payload.
    pub coordination_data: HashMap<String, serde_json::Value>,
}

/// Response from MCP coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCoordinationResponse {
    /// Coordination session identifier.
    pub coordination_id: String,
    /// Current coordination status.
    pub status: String,
    /// Planned execution steps.
    pub coordination_plan: Vec<CoordinationStep>,
    /// Estimated completion timestamp.
    pub estimated_completion: DateTime<Utc>,
}

/// Request for context state operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStateRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// Session identifier.
    pub session_id: String,
    /// Type of context operation.
    pub request_type: String,
    /// Optional context data to store or merge.
    pub context_data: Option<HashMap<String, serde_json::Value>>,
    /// Optional query for context retrieval.
    pub query: Option<String>,
}

/// Response from context state operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStateResponse {
    /// Request identifier.
    pub request_id: String,
    /// Session identifier.
    pub session_id: String,
    /// Retrieved or updated context state.
    pub context_state: HashMap<String, serde_json::Value>,
    /// AI-generated recommendations.
    pub recommendations: Vec<String>,
    /// Related context identifiers.
    pub related_contexts: Vec<String>,
}

/// AI-generated prediction for ecosystem behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Unique prediction identifier.
    pub prediction_id: String,
    /// Type of prediction (e.g., "failure", "capacity").
    pub prediction_type: String,
    /// Confidence score (0.0–1.0).
    pub confidence: f64,
    /// Timeframe for the prediction.
    pub timeframe: String,
    /// Human-readable description.
    pub description: String,
    /// Recommended action to take.
    pub recommended_action: String,
}

/// AI-generated optimization recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    /// Unique optimization identifier.
    pub optimization_id: String,
    /// Type of optimization.
    pub optimization_type: String,
    /// Component to optimize.
    pub target_component: String,
    /// Expected improvement (0.0–1.0).
    pub improvement_potential: f64,
    /// Steps to implement the optimization.
    pub implementation_steps: Vec<String>,
}

/// A single step in a coordination plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationStep {
    /// Unique step identifier.
    pub step_id: String,
    /// Type of step.
    pub step_type: String,
    /// Participants in this step.
    pub participants: Vec<String>,
    /// Estimated duration.
    pub estimated_duration: Duration,
    /// Step IDs that must complete before this step.
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
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/ai")))
                .unwrap_or_else(|_| {
                    let port = std::env::var("BIOMEOS_PORT").unwrap_or_else(|_| "5000".to_string());
                    format!("http://localhost:{port}/ai")
                }),
            mcp_api: std::env::var("BIOMEOS_MCP_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/mcp")))
                .unwrap_or_else(|_| {
                    let port = std::env::var("BIOMEOS_PORT").unwrap_or_else(|_| "5000".to_string());
                    format!("http://localhost:{port}/mcp")
                }),
            context_api: std::env::var("BIOMEOS_CONTEXT_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/context")))
                .or_else(|_| {
                    // Try to construct from port
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/context"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "⚠️ BIOMEOS_CONTEXT_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_CONTEXT_API for production."
                    );
                    "http://localhost:5000/context".to_string()
                }),
            health: std::env::var("BIOMEOS_HEALTH_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/health")))
                .or_else(|_| {
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/health"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "⚠️ BIOMEOS_HEALTH_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_HEALTH_API for production."
                    );
                    "http://localhost:5000/health".to_string()
                }),
            metrics: std::env::var("BIOMEOS_METRICS_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/metrics")))
                .or_else(|_| {
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/metrics"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "⚠️ BIOMEOS_METRICS_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_METRICS_API for production."
                    );
                    "http://localhost:5000/metrics".to_string()
                }),
            websocket: std::env::var("BIOMEOS_WEBSOCKET_URL")
                .ok()
                .or_else(|| {
                    std::env::var("BIOMEOS_ENDPOINT")
                        .ok()
                        .map(|e| e.replace("http://", "ws://").replace("https://", "wss://"))
                        .map(|e| format!("{e}/ws"))
                })
                .or_else(|| {
                    std::env::var("BIOMEOS_PORT")
                        .ok()
                        .map(|port| format!("ws://localhost:{port}/ws"))
                })
                .or_else(|| {
                    tracing::warn!(
                        "⚠️ BIOMEOS_WEBSOCKET_URL not configured. \
                         Set BIOMEOS_WEBSOCKET_URL for production."
                    );
                    Some("ws://localhost:5000/ws".to_string())
                }),
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
}
