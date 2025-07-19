//! Universal Adapter for Squirrel AI Primal
//!
//! This module implements the universal adapter patterns that enable dynamic primal
//! evolution and seamless integration with the ecoPrimals ecosystem through Songbird.
//!
//! ## Ecosystem Integration Patterns
//!
//! All communication flows through Songbird service mesh:
//! ```
//! biomeOS → Songbird (Service Mesh) → All Primals
//!               ↓
//! ToadStool + BearDog + NestGate + Squirrel
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Service mesh integration
    ServiceMeshIntegration,
    /// Cross-primal communication
    CrossPrimalCommunication,
}

/// Universal primal types following ecosystem standards
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Service mesh and orchestration (Songbird)
    ServiceMesh,
    /// Security and authentication management (BearDog)
    Security,
    /// Data storage and retrieval (NestGate)
    Storage,
    /// Compute and processing (ToadStool)
    Compute,
    /// AI primal (Squirrel)
    AI,
    /// Universal OS (biomeOS)
    UniversalOS,
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
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Biome identifier (if applicable)
    pub biome_id: Option<String>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Security levels for primal operations
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Low security - public operations
    Public,
    /// Standard security - authenticated operations
    Standard,
    /// High security - privileged operations
    Privileged,
    /// Critical security - administrative operations
    Critical,
}

/// Primal dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Primal type required
    pub primal_type: PrimalType,
    /// Required capabilities
    pub capabilities: Vec<String>,
    /// Is this dependency optional?
    pub optional: bool,
    /// Minimum version required
    pub min_version: Option<String>,
}

/// Primal endpoints for service discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: String,
    /// Admin/management endpoint
    pub admin: String,
    /// WebSocket endpoint (if supported)
    pub websocket: Option<String>,
    /// MCP protocol endpoint
    pub mcp: String,
    /// AI coordination endpoint
    pub ai_coordination: String,
    /// Service mesh integration endpoint
    pub service_mesh: String,
}

/// Dynamic port information for Songbird-managed allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port range allocated by Songbird
    pub port_range: (u16, u16),
    /// Currently assigned port
    pub current_port: u16,
    /// Port allocation timestamp
    pub allocated_at: DateTime<Utc>,
    /// Port expiration (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Universal primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique request identifier
    pub id: String,
    /// Source primal information
    pub source: PrimalInfo,
    /// Target primal type
    pub target: PrimalType,
    /// Request method/operation
    pub method: String,
    /// Request parameters
    pub params: serde_json::Value,
    /// Request context
    pub context: PrimalContext,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Request timeout
    pub timeout: Option<chrono::Duration>,
}

/// Universal primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request identifier this responds to
    pub request_id: String,
    /// Response status
    pub status: ResponseStatus,
    /// Response data
    pub data: serde_json::Value,
    /// Error information (if any)
    pub error: Option<PrimalError>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Processing duration
    pub duration: chrono::Duration,
}

/// Response status codes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error,
    /// Request is being processed
    Processing,
    /// Request timed out
    Timeout,
    /// Request was cancelled
    Cancelled,
}

/// Primal information for identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal identifier
    pub id: String,
    /// Instance identifier
    pub instance_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Version information
    pub version: String,
    /// Capabilities provided
    pub capabilities: Vec<SquirrelCapability>,
}

/// Universal Primal Provider trait - Songbird-compatible
///
/// This trait defines the standard interface for all primals in the ecosystem.
/// It enables dynamic primal evolution and seamless integration through Songbird.
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

    /// Register with Songbird service mesh
    async fn register_with_songbird(&self, songbird_endpoint: &str) -> Result<(), PrimalError>;

    /// Deregister from Songbird service mesh
    async fn deregister_from_songbird(&self) -> Result<(), PrimalError>;

    /// Get service mesh status
    async fn service_mesh_status(&self) -> Result<ServiceMeshStatus, PrimalError>;
}

/// Service mesh status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    /// Registration status with Songbird
    pub registered: bool,
    /// Last registration timestamp
    pub last_registration: Option<DateTime<Utc>>,
    /// Service mesh health
    pub mesh_health: f64,
    /// Active connections
    pub active_connections: u32,
    /// Load balancing status
    pub load_balancing: LoadBalancingStatus,
}

/// Load balancing status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatus {
    /// Load balancing enabled
    pub enabled: bool,
    /// Current load factor (0.0 to 1.0)
    pub load_factor: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Average response time
    pub avg_response_time: chrono::Duration,
}

/// Result type for primal operations
pub type PrimalResult<T> = Result<T, PrimalError>;

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            device_id: "default".to_string(),
            security_level: SecurityLevel::Standard,
            biome_id: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::Standard
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Coordinator => write!(f, "coordinator"),
            PrimalType::ServiceMesh => write!(f, "service_mesh"),
            PrimalType::Security => write!(f, "security"),
            PrimalType::Storage => write!(f, "storage"),
            PrimalType::Compute => write!(f, "compute"),
            PrimalType::AI => write!(f, "ai"),
            PrimalType::UniversalOS => write!(f, "universal_os"),
            PrimalType::Network => write!(f, "network"),
            PrimalType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

impl std::fmt::Display for SquirrelCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SquirrelCapability::AiCoordination => write!(f, "ai_coordination"),
            SquirrelCapability::McpProtocol => write!(f, "mcp_protocol"),
            SquirrelCapability::ContextAwareness => write!(f, "context_awareness"),
            SquirrelCapability::EcosystemIntelligence => write!(f, "ecosystem_intelligence"),
            SquirrelCapability::SessionManagement => write!(f, "session_management"),
            SquirrelCapability::ToolOrchestration => write!(f, "tool_orchestration"),
            SquirrelCapability::BiomeosIntegration => write!(f, "biomeos_integration"),
            SquirrelCapability::ServiceMeshIntegration => write!(f, "service_mesh_integration"),
            SquirrelCapability::CrossPrimalCommunication => write!(f, "cross_primal_communication"),
        }
    }
}

/// Ecosystem service registration following Songbird standards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Service identifier
    pub service_id: String,
    /// Primal type
    pub primal_type: String,
    /// Service endpoint
    pub endpoint: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Health check endpoint
    pub health_endpoint: String,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
    /// Service version
    pub version: String,
}

/// Initialize ecosystem integration with Songbird patterns
pub async fn initialize_ecosystem_integration(
    config: crate::ecosystem::EcosystemConfig,
    metrics_collector: std::sync::Arc<crate::monitoring::metrics::MetricsCollector>,
) -> Result<crate::ecosystem::EcosystemManager, PrimalError> {
    tracing::info!("Initializing ecosystem integration with Songbird patterns");

    let manager = crate::ecosystem::EcosystemManager::new(config, metrics_collector);
    manager.initialize().await?;

    Ok(manager)
}
