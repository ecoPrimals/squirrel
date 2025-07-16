//! Universal Adapter for Squirrel AI Primal
//!
//! This module implements the universal adapter patterns that enable dynamic primal
//! evolution and seamless integration with the ecoPrimals ecosystem through Songbird.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::PrimalError;

/// Universal primal capabilities that Squirrel provides
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SquirrelCapability {
    /// AI coordination and intelligent routing
    AiCoordination,
    /// MCP protocol management
    McpProtocol,
    /// Context-aware processing
    ContextAwareness,
    /// Ecosystem intelligence
    EcosystemIntelligence,
    /// Session management
    SessionManagement,
    /// Tool orchestration
    ToolOrchestration,
    /// Biomeos integration
    BiomeosIntegration,
}

/// Universal primal types following ecosystem standards
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Security and authentication management
    Security,
    /// Orchestration and task management
    Orchestration,
    /// Data storage and retrieval
    Storage,
    /// Compute and processing
    Compute,
    /// AI primal (Squirrel)
    AI,
    /// Network primal
    Network,
    /// Custom/Other primal types
    Custom(String),
}

/// Universal primal health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and fully operational
    Healthy,
    /// Primal is operational but with some issues
    Degraded { issues: Vec<String> },
    /// Primal is not operational
    Unhealthy { reason: String },
}

/// Universal primal context for multi-instance support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier this primal instance serves
    pub user_id: String,
    /// Device identifier this primal instance serves
    pub device_id: String,
    /// Security level for this context
    pub security_level: SecurityLevel,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Security levels for context-aware routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Public access level
    Public,
    /// Standard authenticated access
    #[default]
    Standard,
    /// Elevated security requirements
    Elevated,
    /// Maximum security requirements
    Maximum,
}

/// Universal primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique request identifier
    pub request_id: Uuid,
    /// Source service identifier
    pub source_service: String,
    /// Target service identifier
    pub target_service: String,
    /// Request operation
    pub operation: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Universal primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request identifier this response corresponds to
    pub request_id: Uuid,
    /// Response status
    pub status: ResponseStatus,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error { code: String, message: String },
    /// Request is being processed
    Processing,
    /// Request timed out
    Timeout,
}

/// Universal primal endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// Admin/management endpoint
    pub admin: String,
    /// WebSocket endpoint for real-time communication
    pub websocket: Option<String>,
    /// MCP protocol endpoint
    pub mcp: String,
    /// AI coordination endpoint
    pub ai_coordination: String,
}

/// Dynamic port information for multi-instance support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Primary service port
    pub primary_port: u16,
    /// Health check port
    pub health_port: u16,
    /// Metrics port
    pub metrics_port: u16,
    /// WebSocket port (if applicable)
    pub websocket_port: Option<u16>,
    /// Port allocation timestamp
    pub allocated_at: DateTime<Utc>,
    /// Port lease duration
    pub lease_duration: std::time::Duration,
}

/// Universal primal dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Service identifier this primal depends on
    pub service_id: String,
    /// Required capabilities from the dependency
    pub required_capabilities: Vec<String>,
    /// Minimum version requirement
    pub min_version: Option<String>,
    /// Whether this dependency is optional
    pub optional: bool,
}

/// Universal primal provider trait for ecosystem integration
#[async_trait]
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "squirrel")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category
    fn primal_type(&self) -> PrimalType;

    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<SquirrelCapability>;

    /// What this primal needs from other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Health check for this primal
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> Result<PrimalResponse, PrimalError>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> Result<(), PrimalError>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> Result<(), PrimalError>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}

/// Result type for primal operations
pub type PrimalResult<T> = Result<T, PrimalError>;

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            device_id: "default".to_string(),
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        }
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Coordinator => write!(f, "coordinator"),
            PrimalType::Security => write!(f, "security"),
            PrimalType::Orchestration => write!(f, "orchestration"),
            PrimalType::Storage => write!(f, "storage"),
            PrimalType::Compute => write!(f, "compute"),
            PrimalType::AI => write!(f, "ai"),
            PrimalType::Network => write!(f, "network"),
            PrimalType::Custom(name) => write!(f, "custom-{name}"),
        }
    }
}

impl std::fmt::Display for SquirrelCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SquirrelCapability::AiCoordination => write!(f, "ai-coordination"),
            SquirrelCapability::McpProtocol => write!(f, "mcp-protocol"),
            SquirrelCapability::ContextAwareness => write!(f, "context-awareness"),
            SquirrelCapability::EcosystemIntelligence => write!(f, "ecosystem-intelligence"),
            SquirrelCapability::SessionManagement => write!(f, "session-management"),
            SquirrelCapability::ToolOrchestration => write!(f, "tool-orchestration"),
            SquirrelCapability::BiomeosIntegration => write!(f, "biomeos-integration"),
        }
    }
}
