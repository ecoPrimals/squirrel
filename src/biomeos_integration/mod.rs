//! # Squirrel biomeOS Integration
//!
//! This module provides integration with the biomeOS ecosystem, allowing squirrel
//! to register as an AI intelligence primal and provide MCP protocol services,
//! AI capabilities, and context state management for the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::PrimalError;
use config::ConfigManager;
use tokio::time::sleep;
use tracing::warn;

// Constants to reduce string allocations
const PRIMAL_TYPE: &str = "squirrel";
const API_VERSION: &str = "biomeOS/v1";
const STATUS_INITIALIZING: &str = "initializing";
const STATUS_STARTING: &str = "starting";
const STATUS_RUNNING: &str = "running";
const STATUS_SHUTTING_DOWN: &str = "shutting_down";

pub mod ai_intelligence;
pub mod context_state;
pub mod ecosystem_client;
pub mod mcp_integration;

pub use ai_intelligence::*;
pub use context_state::*;
pub use ecosystem_client::*;
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
    pub active_sessions: u32,
    pub ai_requests_processed: u64,
    pub context_states_managed: u32,
}

impl SquirrelBiomeOSIntegration {
    /// Create new biomeOS integration for squirrel AI
    pub fn new(biome_id: String) -> Self {
        let service_id = format!("primal-squirrel-ai-{}", uuid::Uuid::new_v4());

        Self {
            service_id,
            biome_id,
            ai_intelligence: AiIntelligence::new(),
            mcp_integration: McpIntegration::new(),
            context_state: ContextState::new(),
            ecosystem_client: EcosystemClient::new(),
            health_status: HealthStatus {
                status: STATUS_INITIALIZING.to_string(),
                timestamp: Utc::now(),
                ai_engine_status: STATUS_STARTING.to_string(),
                mcp_server_status: STATUS_STARTING.to_string(),
                context_manager_status: STATUS_STARTING.to_string(),
                active_sessions: 0,
                ai_requests_processed: 0,
                context_states_managed: 0,
            },
        }
    }

    /// Register squirrel AI with biomeOS ecosystem
    pub async fn register_with_biomeos(&mut self) -> Result<(), PrimalError> {
        let config_manager = ConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        let registration = EcosystemServiceRegistration {
            service_id: self.service_id.clone(),
            primal_type: PRIMAL_TYPE.to_string(),
            biome_id: self.biome_id.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            api_version: API_VERSION.to_string(),
            registration_time: Utc::now(),

            endpoints: EcosystemEndpoints {
                ai_api: _endpoints.ai_api,
                mcp_api: _endpoints.mcp_api,
                context_api: _endpoints.context_api,
                health: _endpoints.health,
                metrics: _endpoints.metrics,
                websocket: _endpoints.websocket,
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
        let config_manager = ConfigManager::new();
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

        // Start ecosystem AI services
        self.start_ecosystem_intelligence().await?;
        self.start_mcp_coordination().await?;
        self.start_context_management().await?;

        self.health_status.status = STATUS_RUNNING.to_string();
        self.health_status.timestamp = Utc::now();

        Ok(())
    }

    /// Start ecosystem intelligence services
    async fn start_ecosystem_intelligence(&mut self) -> Result<(), PrimalError> {
        let config_manager = ConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        // Start intelligence background task
        let ai_intelligence = self.ai_intelligence.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = ai_intelligence.analyze_ecosystem().await {
                    warn!("Ecosystem intelligence analysis error: {}", e);
                }
                let interval = std::env::var("AI_INTELLIGENCE_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(60);
                sleep(Duration::from_secs(interval)).await;
            }
        });

        Ok(())
    }

    /// Start MCP coordination services
    async fn start_mcp_coordination(&mut self) -> Result<(), PrimalError> {
        let config_manager = ConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        // Start MCP coordination background task
        let mcp_integration = self.mcp_integration.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = mcp_integration.coordinate_mcp_services().await {
                    warn!("MCP coordination error: {}", e);
                }
                let interval = std::env::var("MCP_COORDINATION_INTERVAL_SECS")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(30);
                sleep(Duration::from_secs(interval)).await;
            }
        });

        Ok(())
    }

    /// Start context management services
    async fn start_context_management(&mut self) -> Result<(), PrimalError> {
        let config_manager = ConfigManager::new();
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
                sleep(Duration::from_secs(interval)).await;
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
    pub fn update_health_status(&mut self) {
        self.health_status.timestamp = Utc::now();
        self.health_status.active_sessions = self.context_state.get_active_sessions();
        self.health_status.ai_requests_processed = self.ai_intelligence.get_requests_processed();
        self.health_status.context_states_managed = self.context_state.get_managed_states();
    }

    /// Gracefully shutdown ecosystem services
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        let config_manager = ConfigManager::new();
        let _endpoints = config_manager.get_biomeos_endpoints();

        self.health_status.status = STATUS_SHUTTING_DOWN.to_string();
        self.health_status.timestamp = Utc::now();

        // Shutdown AI intelligence
        self.ai_intelligence.shutdown().await?;

        // Shutdown MCP integration
        self.mcp_integration.shutdown().await?;

        // Shutdown context state management
        self.context_state.shutdown().await?;

        // Deregister from ecosystem (via songbird)
        self.ecosystem_client
            .deregister_service_from_songbird(&self.service_id)
            .await?;

        self.health_status.status = "shutdown".to_string();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceResponse {
    pub request_id: String,
    pub response_type: String,
    pub recommendations: Vec<String>,
    pub predictions: Vec<Prediction>,
    pub optimizations: Vec<Optimization>,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
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
    pub session_id: String,
    pub request_type: String,
    pub context_data: Option<HashMap<String, serde_json::Value>>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStateResponse {
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

        integration.update_health_status();
        assert!(integration.health_status.timestamp > original_timestamp);
    }
}
