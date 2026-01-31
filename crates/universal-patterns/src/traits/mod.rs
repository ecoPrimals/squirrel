//! Universal primal traits and interfaces
//!
//! This module provides the core traits and types for universal primal patterns,
//! designed for full compatibility with songbird's orchestration system.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use crate::config::PrimalConfig;

/// Core primal trait - foundational interface for all primals
#[async_trait]
pub trait Primal: Send + Sync {
    /// Get primal information
    fn info(&self) -> &PrimalInfo;

    /// Get current state of the primal
    async fn state(&self) -> PrimalState;

    /// Start the primal
    async fn start(&mut self) -> Result<(), PrimalError>;

    /// Stop the primal
    async fn stop(&mut self) -> Result<(), PrimalError>;

    /// Restart the primal
    async fn restart(&mut self) -> Result<(), PrimalError> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Check if the primal is healthy
    async fn health_check(&self) -> Result<HealthStatus, PrimalError>;

    /// Get configuration
    fn config(&self) -> &PrimalConfig;

    /// Update configuration
    async fn update_config(&mut self, config: PrimalConfig) -> Result<(), PrimalError>;

    /// Get metrics
    async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError>;

    /// Handle shutdown signal
    async fn shutdown(&mut self) -> Result<(), PrimalError>;
}

/// Songbird-compatible Universal Primal Provider trait
/// This trait enables full compatibility with songbird's orchestration system
#[async_trait]
pub trait PrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "beardog", "nestgate", "toadstool", "squirrel")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support (e.g., "beardog-user123", "beardog-device456")
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category (e.g., Security, Storage, Compute, AI)
    fn primal_type(&self) -> PrimalType;

    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<PrimalCapability>;

    /// What this primal needs from other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;

    /// Health check for this primal
    async fn health_check(&self) -> PrimalHealth;

    /// Get primal API endpoints
    fn endpoints(&self) -> PrimalEndpoints;

    /// Handle inter-primal communication
    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> PrimalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> PrimalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
}

/// Primal information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name
    pub name: String,

    /// Primal version
    pub version: String,

    /// Unique instance identifier
    pub instance_id: Uuid,

    /// Primal type
    pub primal_type: PrimalType,

    /// Description
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Capabilities
    pub capabilities: Vec<String>,
}

/// Primal type categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI coordination and MCP protocol management
    Coordinator,
    /// Security and authentication management (BearDog)
    Security,
    /// Orchestration and task management (Songbird)
    Orchestration,
    /// Data storage and retrieval (NestGate)
    Storage,
    /// Compute and processing (Toadstool)
    Compute,
    /// AI primal (Squirrel)
    AI,
    /// Network primal
    Network,
    /// Custom/Other primal types
    Custom(String),
}

/// Primal state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PrimalState {
    /// Primal is initializing
    Initializing,
    /// Primal is starting up
    Starting,
    /// Primal is running and healthy
    Running,
    /// Primal is stopping
    Stopping,
    /// Primal is stopped
    #[default]
    Stopped,
    /// Primal is in an error state
    Error(String),
    /// Primal is restarting
    Restarting,
    /// Primal is in maintenance mode
    Maintenance,
}

/// Health status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthState,

    /// Detailed health information
    pub details: HashMap<String, HealthDetail>,

    /// Timestamp of the health check
    pub timestamp: DateTime<Utc>,

    /// Time taken for the health check
    pub duration: Duration,
}

/// Health state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HealthState {
    /// Primal is healthy
    Healthy,
    /// Primal is degraded but functional
    Degraded,
    /// Primal is unhealthy
    Unhealthy,
    /// Health status is unknown
    #[default]
    Unknown,
}

/// Health detail structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDetail {
    /// Status of this specific component
    pub status: HealthState,

    /// Human-readable message
    pub message: String,

    /// Additional data
    pub data: HashMap<String, serde_json::Value>,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer counter
    Counter(u64),
    /// Floating point gauge
    Gauge(f64),
    /// Histogram data
    Histogram {
        /// Number of observations
        count: u64,
        /// Sum of all observations
        sum: f64,
        /// Histogram buckets with (upper_bound, count) pairs
        buckets: Vec<(f64, u64)>,
    },
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Duration value
    Duration(Duration),
    /// Timestamp value
    Timestamp(DateTime<Utc>),
}

/// Primal error types
#[derive(Debug, thiserror::Error)]
pub enum PrimalError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    /// Security error
    #[error("Security error: {0}")]
    Security(String),
    /// Orchestration error
    #[error("Orchestration error: {0}")]
    Orchestration(String),
    /// State error
    #[error("State error: {0}")]
    State(String),
    /// Health check error
    #[error("Health check error: {0}")]
    HealthCheck(String),
    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),
    /// Shutdown error
    #[error("Shutdown error: {0}")]
    Shutdown(String),
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    /// Permission error
    #[error("Permission error: {0}")]
    Permission(String),
    /// Resource error
    #[error("Resource error: {0}")]
    Resource(String),
    /// Communication error
    #[error("Communication error: {0}")]
    Communication(String),
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    /// Service unavailable error
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for primal operations
pub type PrimalResult<T> = std::result::Result<T, PrimalError>;

/// Context for user/device-specific primal routing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalContext {
    /// User identifier
    pub user_id: String,
    /// Device identifier
    pub device_id: String,
    /// Session identifier
    pub session_id: String,
    /// Network location (IP, subnet, etc.)
    pub network_location: NetworkLocation,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkLocation {
    /// IP address
    pub ip_address: String,
    /// Subnet
    pub subnet: Option<String>,
    /// Local network identifier
    pub network_id: Option<String>,
    /// Geographic location
    pub geo_location: Option<String>,
}

/// Security level requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security
    Basic,
    /// Standard security
    Standard,
    /// High security
    High,
    /// Maximum security
    Maximum,
}

/// Dynamic port information for songbird-managed ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port assigned by songbird
    pub assigned_port: u16,
    /// Port type (HTTP, HTTPS, WebSocket, etc.)
    pub port_type: PortType,
    /// Port status
    pub status: PortStatus,
    /// Port assignment timestamp
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    /// Port lease duration
    pub lease_duration: chrono::Duration,
}

/// Port type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// HTTP port
    Http,
    /// HTTPS port
    Https,
    /// WebSocket port
    WebSocket,
    /// gRPC port
    Grpc,
    /// Custom port type
    Custom(String),
}

/// Port status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    /// Port is active and available
    Active,
    /// Port is reserved but not yet active
    Reserved,
    /// Port is being releasing
    Releasing,
    /// Port is expired and should be cleaned up
    Expired,
}

/// Universal capabilities that any primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Security capabilities (BearDog)
    /// Authentication with supported methods
    Authentication {
        /// List of supported authentication methods
        methods: Vec<String>,
    },
    /// Encryption with supported algorithms
    Encryption {
        /// List of supported encryption algorithms
        algorithms: Vec<String>,
    },
    /// Key management with HSM support
    KeyManagement {
        /// Whether HSM (Hardware Security Module) is supported
        hsm_support: bool,
    },
    /// Threat detection with ML capabilities
    ThreatDetection {
        /// Whether machine learning is enabled for threat detection
        ml_enabled: bool,
    },
    /// Audit logging with compliance standards
    AuditLogging {
        /// List of supported compliance standards
        compliance: Vec<String>,
    },
    /// Authorization and access control
    Authorization {
        /// Whether RBAC (Role-Based Access Control) is supported
        rbac_support: bool,
    },

    // Storage capabilities (NestGate)
    /// File system support
    FileSystem {
        /// Whether ZFS file system is supported
        supports_zfs: bool,
    },
    /// Object storage with backends
    ObjectStorage {
        /// List of supported storage backends
        backends: Vec<String>,
    },
    /// Data replication
    DataReplication {
        /// Consistency model for data replication
        consistency: String,
    },
    /// Backup capabilities
    Backup {
        /// Whether incremental backups are supported
        incremental: bool,
    },
    /// Data archiving
    DataArchiving {
        /// List of supported compression algorithms
        compression: Vec<String>,
    },

    // Compute capabilities (Toadstool)
    /// Container runtime support
    ContainerRuntime {
        /// List of supported container orchestrators
        orchestrators: Vec<String>,
    },
    /// Serverless execution
    ServerlessExecution {
        /// List of supported programming languages
        languages: Vec<String>,
    },
    /// GPU acceleration
    GpuAcceleration {
        /// Whether CUDA is supported
        cuda_support: bool,
    },
    /// Load balancing
    LoadBalancing {
        /// List of supported load balancing algorithms
        algorithms: Vec<String>,
    },
    /// Auto-scaling
    AutoScaling {
        /// List of supported scaling metrics
        metrics: Vec<String>,
    },

    // AI capabilities (Squirrel)
    /// Model inference
    ModelInference {
        /// List of supported AI models
        models: Vec<String>,
    },
    /// Agent framework
    AgentFramework {
        /// Whether MCP (Model Context Protocol) is supported
        mcp_support: bool,
    },
    /// Machine learning
    MachineLearning {
        /// Whether training is supported
        training_support: bool,
    },
    /// Natural language processing
    NaturalLanguage {
        /// List of supported languages
        languages: Vec<String>,
    },
    /// Computer vision
    ComputerVision {
        /// List of supported computer vision models
        models: Vec<String>,
    },

    // Networking capabilities
    /// Service discovery
    ServiceDiscovery {
        /// List of supported discovery protocols
        protocols: Vec<String>,
    },
    /// Network routing
    NetworkRouting {
        /// List of supported routing protocols
        protocols: Vec<String>,
    },
    /// Proxy services
    ProxyServices {
        /// List of supported proxy types
        types: Vec<String>,
    },
    /// VPN capabilities
    VpnServices {
        /// List of supported VPN protocols
        protocols: Vec<String>,
    },

    // Generic capabilities
    /// Custom capability
    Custom {
        /// Name of the custom capability
        name: String,
        /// Custom attributes for the capability
        attributes: String, // Changed from HashMap to String to fix Hash issues
    },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimalCapability::Authentication { methods } => {
                "authentication".hash(state);
                methods.hash(state);
            }
            PrimalCapability::Encryption { algorithms } => {
                "encryption".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::KeyManagement { hsm_support } => {
                "key_management".hash(state);
                hsm_support.hash(state);
            }
            PrimalCapability::ThreatDetection { ml_enabled } => {
                "threat_detection".hash(state);
                ml_enabled.hash(state);
            }
            PrimalCapability::AuditLogging { compliance } => {
                "audit_logging".hash(state);
                compliance.hash(state);
            }
            PrimalCapability::Authorization { rbac_support } => {
                "authorization".hash(state);
                rbac_support.hash(state);
            }
            PrimalCapability::FileSystem { supports_zfs } => {
                "file_system".hash(state);
                supports_zfs.hash(state);
            }
            PrimalCapability::ObjectStorage { backends } => {
                "object_storage".hash(state);
                backends.hash(state);
            }
            PrimalCapability::DataReplication { consistency } => {
                "data_replication".hash(state);
                consistency.hash(state);
            }
            PrimalCapability::Backup { incremental } => {
                "backup".hash(state);
                incremental.hash(state);
            }
            PrimalCapability::DataArchiving { compression } => {
                "data_archiving".hash(state);
                compression.hash(state);
            }
            PrimalCapability::ContainerRuntime { orchestrators } => {
                "container_runtime".hash(state);
                orchestrators.hash(state);
            }
            PrimalCapability::ServerlessExecution { languages } => {
                "serverless_execution".hash(state);
                languages.hash(state);
            }
            PrimalCapability::GpuAcceleration { cuda_support } => {
                "gpu_acceleration".hash(state);
                cuda_support.hash(state);
            }
            PrimalCapability::LoadBalancing { algorithms } => {
                "load_balancing".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::AutoScaling { metrics } => {
                "auto_scaling".hash(state);
                metrics.hash(state);
            }
            PrimalCapability::ModelInference { models } => {
                "model_inference".hash(state);
                models.hash(state);
            }
            PrimalCapability::AgentFramework { mcp_support } => {
                "agent_framework".hash(state);
                mcp_support.hash(state);
            }
            PrimalCapability::MachineLearning { training_support } => {
                "machine_learning".hash(state);
                training_support.hash(state);
            }
            PrimalCapability::NaturalLanguage { languages } => {
                "natural_language".hash(state);
                languages.hash(state);
            }
            PrimalCapability::ComputerVision { models } => {
                "computer_vision".hash(state);
                models.hash(state);
            }
            PrimalCapability::ServiceDiscovery { protocols } => {
                "service_discovery".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::NetworkRouting { protocols } => {
                "network_routing".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::ProxyServices { types } => {
                "proxy_services".hash(state);
                types.hash(state);
            }
            PrimalCapability::VpnServices { protocols } => {
                "vpn_services".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::Custom { name, attributes } => {
                "custom".hash(state);
                name.hash(state);
                attributes.hash(state);
            }
        }
    }
}

/// Primal dependencies enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalDependency {
    /// Requires authentication
    RequiresAuthentication {
        /// List of required authentication methods
        methods: Vec<String>,
    },
    /// Requires encryption
    RequiresEncryption {
        /// List of required encryption algorithms
        algorithms: Vec<String>,
    },
    /// Requires storage
    RequiresStorage {
        /// List of required storage types
        types: Vec<String>,
    },
    /// Requires compute
    RequiresCompute {
        /// List of required compute types
        types: Vec<String>,
    },
    /// Requires AI
    RequiresAI {
        /// List of required AI capabilities
        capabilities: Vec<String>,
    },
    /// Custom dependency
    Custom {
        /// Name of the custom dependency
        name: String,
        /// Custom requirements for the dependency
        requirements: String, // Changed from HashMap to String to fix Hash issues
    },
}

/// Primal health status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but operational
    Degraded {
        /// List of issues causing degradation
        issues: Vec<String>,
    },
    /// Primal is unhealthy and not operational
    Unhealthy {
        /// Reason why the primal is unhealthy
        reason: String,
    },
}

/// Primal API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary API endpoint
    pub primary: String,
    /// Health check endpoint
    pub health: String,
    /// Metrics endpoint
    pub metrics: Option<String>,
    /// Admin endpoint
    pub admin: Option<String>,
    /// WebSocket endpoint
    pub websocket: Option<String>,
    /// Additional custom endpoints
    pub custom: String, // Changed from HashMap to String to fix Hash issues
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        // Multi-tier primal endpoint resolution
        // 1. PRIMAL_ENDPOINT (full endpoint)
        // 2. PRIMAL_PORT (port override)
        // 3. Default: http://localhost:8080
        let port = std::env::var("PRIMAL_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080); // Default primal port
        let primary = std::env::var("PRIMAL_ENDPOINT")
            .unwrap_or_else(|_| format!("http://localhost:{}", port));
        let health = format!("{}/health", primary);

        Self {
            primary,
            health,
            metrics: None,
            admin: None,
            websocket: None,
            custom: String::new(),
        }
    }
}

/// Universal request structure for primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Unique identifier for this request
    pub id: uuid::Uuid,
    /// Type of request being made
    pub request_type: PrimalRequestType,
    /// Request payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when request was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User context making the request
    pub context: Option<String>,
    /// Priority level for request processing
    pub priority: Option<u8>,
    /// Security classification of the request
    pub security_level: Option<String>,
}

/// Universal response structure from primal services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response corresponds to
    pub request_id: uuid::Uuid,
    /// Type of response being returned
    pub response_type: PrimalResponseType,
    /// Response payload data
    pub payload: HashMap<String, serde_json::Value>,
    /// Timestamp when response was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if request failed
    pub error_message: Option<String>,
    /// Additional metadata about the response
    pub metadata: Option<HashMap<String, String>>,
}

/// Types of requests that can be made to primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalRequestType {
    /// Authentication request
    Authenticate,
    /// Encryption request
    Encrypt,
    /// Decryption request
    Decrypt,
    /// Authorization check request
    Authorize,
    /// Audit logging request
    AuditLog,
    /// Threat detection request
    ThreatDetection,
    /// Health check request
    HealthCheck,
    /// Store data request
    Store,
    /// Retrieve data request
    Retrieve,
    /// Compute request
    Compute,
    /// AI inference request
    Infer,
    /// Custom request type
    Custom(String),
}

/// Types of responses that can be returned from primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalResponseType {
    /// Authentication response
    Authentication,
    /// Encryption response
    Encryption,
    /// Decryption response
    Decryption,
    /// Authorization response
    Authorization,
    /// Audit response
    Audit,
    /// Threat detection response
    ThreatDetection,
    /// Health check response
    HealthCheck,
    /// Storage response
    Storage,
    /// Retrieval response
    Retrieval,
    /// Compute response
    Compute,
    /// AI inference response
    Inference,
    /// Custom response type
    Custom(String),
}

/// Credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    /// Username and password
    Password {
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
    },
    /// API key
    ApiKey {
        /// The API key
        key: String,
        /// Service ID for the API key
        service_id: String,
    },
    /// Bearer token
    Bearer {
        /// Bearer token string
        token: String,
    },
    /// JWT token
    Token {
        /// JWT token string
        token: String,
    },
    /// Certificate
    Certificate {
        /// Certificate data
        cert: Vec<u8>,
    },
    /// Service account credentials
    ServiceAccount {
        /// Service ID for the service account
        service_id: String,
        /// API key for the service account
        api_key: String,
    },
    /// Bootstrap credentials
    Bootstrap {
        /// Service ID for bootstrap
        service_id: String,
    },
    /// Test credentials
    Test {
        /// Service ID for testing
        service_id: String,
    },
    /// Custom credentials
    Custom(HashMap<String, String>),
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Authenticated principal
    pub principal: Principal,

    /// Authentication token
    pub token: String,

    /// Token expiration time
    pub expires_at: DateTime<Utc>,

    /// Granted permissions
    pub permissions: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Principal (authenticated user/service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Principal ID
    pub id: String,

    /// Principal name
    pub name: String,

    /// Principal type
    pub principal_type: PrincipalType,

    /// Roles
    pub roles: Vec<String>,

    /// Permissions
    pub permissions: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Type of principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrincipalType {
    /// Human user
    User,
    /// Service account
    Service,
    /// API client
    Client,
    /// System account
    System,
}

impl Default for PrimalContext {
    fn default() -> Self {
        Self {
            user_id: "default".to_string(),
            device_id: "default".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            network_location: NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: SecurityLevel::Standard,
            metadata: HashMap::new(),
        }
    }
}

impl std::fmt::Display for PrimalState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalState::Initializing => write!(f, "Initializing"),
            PrimalState::Starting => write!(f, "Starting"),
            PrimalState::Running => write!(f, "Running"),
            PrimalState::Stopping => write!(f, "Stopping"),
            PrimalState::Stopped => write!(f, "Stopped"),
            PrimalState::Error(msg) => write!(f, "Error: {msg}"),
            PrimalState::Restarting => write!(f, "Restarting"),
            PrimalState::Maintenance => write!(f, "Maintenance"),
        }
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Coordinator => write!(f, "Coordinator"),
            PrimalType::Security => write!(f, "Security"),
            PrimalType::Orchestration => write!(f, "Orchestration"),
            PrimalType::Storage => write!(f, "Storage"),
            PrimalType::Compute => write!(f, "Compute"),
            PrimalType::AI => write!(f, "AI"),
            PrimalType::Network => write!(f, "Network"),
            PrimalType::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}
