// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for biomeOS ecosystem integration — registration, capabilities,
//! security, health, and request/response envelopes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::{API_VERSION, PRIMAL_TYPE, STATUS_INITIALIZING, STATUS_STARTING};

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

impl HealthStatus {
    /// Create a new initializing health status.
    #[must_use]
    pub fn initializing() -> Self {
        Self {
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
        }
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

// -- Default implementations -------------------------------------------------

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
                    let port = std::env::var("BIOMEOS_PORT").unwrap_or_else(|_| {
                        universal_constants::network::DEFAULT_BIOMEOS_PORT.to_string()
                    });
                    format!("http://localhost:{port}/ai")
                }),
            mcp_api: std::env::var("BIOMEOS_MCP_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/mcp")))
                .unwrap_or_else(|_| {
                    let port = std::env::var("BIOMEOS_PORT").unwrap_or_else(|_| {
                        universal_constants::network::DEFAULT_BIOMEOS_PORT.to_string()
                    });
                    format!("http://localhost:{port}/mcp")
                }),
            context_api: std::env::var("BIOMEOS_CONTEXT_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/context")))
                .or_else(|_| {
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/context"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "BIOMEOS_CONTEXT_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_CONTEXT_API for production."
                    );
                    format!(
                        "http://localhost:{}/context",
                        universal_constants::network::DEFAULT_BIOMEOS_PORT
                    )
                }),
            health: std::env::var("BIOMEOS_HEALTH_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/health")))
                .or_else(|_| {
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/health"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "BIOMEOS_HEALTH_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_HEALTH_API for production."
                    );
                    format!(
                        "http://localhost:{}/health",
                        universal_constants::network::DEFAULT_BIOMEOS_PORT
                    )
                }),
            metrics: std::env::var("BIOMEOS_METRICS_API")
                .or_else(|_| std::env::var("BIOMEOS_ENDPOINT").map(|e| format!("{e}/metrics")))
                .or_else(|_| {
                    std::env::var("BIOMEOS_PORT")
                        .map(|port| format!("http://localhost:{port}/metrics"))
                })
                .unwrap_or_else(|_| {
                    tracing::warn!(
                        "BIOMEOS_METRICS_API not configured. \
                         Set BIOMEOS_ENDPOINT or BIOMEOS_METRICS_API for production."
                    );
                    format!(
                        "http://localhost:{}/metrics",
                        universal_constants::network::DEFAULT_BIOMEOS_PORT
                    )
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
                        "BIOMEOS_WEBSOCKET_URL not configured. \
                         Set BIOMEOS_WEBSOCKET_URL for production."
                    );
                    Some(format!(
                        "ws://localhost:{}/ws",
                        universal_constants::network::DEFAULT_BIOMEOS_PORT
                    ))
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
