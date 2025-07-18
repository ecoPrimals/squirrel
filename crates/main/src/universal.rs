use crate::error::PrimalError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal result type for all primal operations
pub type UniversalResult<T> = Result<T, PrimalError>;

/// Universal trait that ANY primal can implement for ecosystem integration
#[async_trait]
pub trait UniversalPrimalProvider: Send + Sync {
    fn primal_id(&self) -> &str;
    fn instance_id(&self) -> &str;
    fn context(&self) -> &PrimalContext;
    fn primal_type(&self) -> PrimalType;
    fn capabilities(&self) -> Vec<PrimalCapability>;
    fn dependencies(&self) -> Vec<PrimalDependency>;
    async fn health_check(&self) -> PrimalHealth;
    fn endpoints(&self) -> PrimalEndpoints;
    async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse>;
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String>;
    async fn handle_ecosystem_request(&self, request: EcosystemRequest) -> UniversalResult<EcosystemResponse>;
}

/// Alias for UniversalPrimalProvider for backward compatibility
pub type PrimalProvider = dyn UniversalPrimalProvider;

/// Universal primal capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalCapability {
    /// AI capabilities (Squirrel)
    ModelInference { models: Vec<String> },
    AgentFramework { mcp_support: bool },
    MachineLearning { training_support: bool },
    NaturalLanguage { languages: Vec<String> },
    
    /// Security capabilities (BearDog)
    Authentication { methods: Vec<String> },
    Encryption { algorithms: Vec<String> },
    KeyManagement { hsm_support: bool },
    ThreatDetection { ml_enabled: bool },
    
    /// Storage capabilities (NestGate)
    FileSystem { supports_zfs: bool },
    ObjectStorage { backends: Vec<String> },
    VolumeManagement { protocols: Vec<String> },
    BackupRestore { incremental: bool },
    
    /// Compute capabilities (ToadStool)
    ContainerRuntime { orchestrators: Vec<String> },
    ServerlessExecution { languages: Vec<String> },
    GpuAcceleration { cuda_support: bool },
    
    /// Network capabilities (Songbird)
    ServiceDiscovery { protocols: Vec<String> },
    LoadBalancing { algorithms: Vec<String> },
    CircuitBreaking { enabled: bool },
    
    /// OS capabilities (biomeOS)
    Orchestration { primals: Vec<String> },
    Manifests { formats: Vec<String> },
    Deployment { strategies: Vec<String> },
}

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

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Squirrel => write!(f, "squirrel"),
            PrimalType::Storage => write!(f, "storage"),
            PrimalType::Compute => write!(f, "compute"),
            PrimalType::Security => write!(f, "security"),
            PrimalType::ServiceMesh => write!(f, "service_mesh"),
            PrimalType::OS => write!(f, "os"),
            PrimalType::Generic => write!(f, "generic"),
        }
    }
}

/// Load balancing status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatus {
    /// Whether load balancing is healthy
    pub healthy: bool,
    /// Number of active connections
    pub active_connections: u32,
    /// Load balancing algorithm in use
    pub algorithm: String,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

/// Service mesh status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    pub connected: bool,
    pub registered: bool,
    pub songbird_endpoint: Option<String>,
    pub registration_time: Option<DateTime<Utc>>,
    pub last_registration: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub mesh_version: String,
    pub mesh_health: String,
    pub instance_id: String,
    pub load_balancing_enabled: bool,
    pub load_balancing: LoadBalancingStatus,
    pub active_connections: u32,
    pub circuit_breaker_status: CircuitBreakerStatus,
}

/// Network location information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkLocation {
    /// Local network
    Local,
    /// Cloud environment
    Cloud,
    /// Edge computing
    Edge,
    /// Hybrid environment
    Hybrid,
}

/// Universal security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityContext {
    pub user_id: String,
    pub session_id: String,
    pub permissions: Vec<String>,
    pub security_level: SecurityLevel,
}

impl Default for UniversalSecurityContext {
    fn default() -> Self {
        Self {
            user_id: "system".to_string(),
            session_id: "default".to_string(),
            permissions: vec![],
            security_level: SecurityLevel::Standard,
        }
    }
}

/// Universal primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique request identifier
    pub request_id: Uuid,
    /// Request ID (alias for request_id)
    pub id: String,
    /// Source primal identifier
    pub source_primal: String,
    /// Target primal identifier
    pub target_primal: String,
    /// Source primal information
    pub source: PrimalInfo,
    /// Target primal type
    pub target: PrimalType,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request method/operation
    pub method: String,
    /// Request operation (alias for method)
    pub operation: String,
    /// Request parameters
    pub params: serde_json::Value,
    /// Request parameters (alias for params)
    pub parameters: serde_json::Value,
    /// Request context
    pub context: PrimalContext,
    /// Security context
    pub security_context: UniversalSecurityContext,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Request timeout
    pub timeout: Option<chrono::Duration>,
}

/// Universal primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub response_id: Uuid,
    pub request_id: Uuid,
    pub payload: serde_json::Value,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub status: ResponseStatus,
    pub success: bool,
    pub error_message: Option<String>,
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

/// Universal primal types following ecosystem standards
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management (Squirrel)
    Squirrel,
    /// Storage and data management (NestGate)
    Storage,
    /// Compute and processing (ToadStool)
    Compute,
    /// Security and authentication (BearDog)
    Security,
    /// Service mesh and orchestration (Songbird)
    ServiceMesh,
    /// Operating system and deployment (biomeOS)
    OS,
    /// Generic primal type
    Generic,
} 

/// Response status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request failed with error
    Error,
    /// Request timed out
    Timeout,
    /// Request was cancelled
    Cancelled,
    /// Request is still processing
    Processing,
}

/// Ecosystem response format (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    pub request_id: String,
    pub success: bool,
    pub payload: serde_json::Value,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
} 

/// Primal context for tracking request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalContext {
    /// Session identifier
    pub session_id: String,
    /// User identifier
    pub user_id: String,
    /// Request trace identifier
    pub trace_id: String,
    /// Environment information
    pub environment: String,
    /// Additional context data
    pub data: HashMap<String, serde_json::Value>,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            session_id: "default".to_string(),
            user_id: "system".to_string(),
            trace_id: uuid::Uuid::new_v4().to_string(),
            environment: "production".to_string(),
            data: HashMap::new(),
        }
    }
}

/// Circuit breaker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Whether circuit breaker is open
    pub open: bool,
    /// Number of failures
    pub failures: u32,
    /// Last failure time
    pub last_failure: Option<DateTime<Utc>>,
    /// Next retry time
    pub next_retry: Option<DateTime<Utc>>,
}

/// Primal dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Dependency identifier
    pub id: String,
    /// Dependency type
    pub primal_type: PrimalType,
    /// Whether dependency is required
    pub required: bool,
    /// Minimum version required
    pub min_version: Option<String>,
}

/// Primal health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Health status
    pub status: HealthStatus,
    /// Health score (0.0 to 1.0)
    pub score: f64,
    /// Last health check
    pub last_check: DateTime<Utc>,
    /// Health details
    pub details: HashMap<String, serde_json::Value>,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy
    Unhealthy,
    /// Health status unknown
    Unknown,
}

/// Primal endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// HTTP endpoint
    pub http: Option<String>,
    /// gRPC endpoint
    pub grpc: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Custom endpoints
    pub custom: HashMap<String, String>,
}

/// Ecosystem request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    /// Request identifier
    pub request_id: String,
    /// Source primal
    pub source_primal: String,
    /// Target primal
    pub target_primal: String,
    /// Request method
    pub method: String,
    /// Request payload
    pub payload: serde_json::Value,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Security level enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Low security level
    Low,
    /// Standard security level
    Standard,
    /// High security level
    High,
    /// Critical security level
    Critical,
}

/// Dynamic port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port number
    pub port: u16,
    /// Port protocol
    pub protocol: String,
    /// Port status
    pub status: String,
    /// Port description
    pub description: Option<String>,
} 