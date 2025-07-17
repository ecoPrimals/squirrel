//! Universal Primal Patterns for Squirrel AI Primal
//!
//! This module implements the universal, agnostic patterns that allow any primal
//! to be created, evolved, and integrated seamlessly within the ecoPrimals ecosystem.
//!
//! ## Universal Principles
//!
//! - **Agnostic**: Works across all computing environments and platforms
//! - **Extensible**: New primals can be added without breaking existing ones
//! - **Context-Aware**: Supports user/device-specific routing and multi-tenancy
//! - **Future-Proof**: Designed to evolve with new primal types and capabilities

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::PrimalError;

/// Universal result type for all primal operations
pub type UniversalResult<T> = Result<T, PrimalError>;

/// Universal trait that ANY primal can implement for ecosystem integration
#[async_trait]
pub trait UniversalPrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "squirrel", "beardog", "nestgate", "toadstool")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support (e.g., "squirrel-user123", "squirrel-device456")
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category (AI, Security, Storage, Compute, Network)
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
    async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> UniversalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information (managed by Songbird)
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;

    /// Register with Songbird service mesh
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String>;

    /// Deregister from Songbird service mesh
    async fn deregister_from_songbird(&mut self) -> UniversalResult<()>;

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;

    /// Handle ecosystem request (standardized format)
    async fn handle_ecosystem_request(&self, request: EcosystemRequest) -> UniversalResult<EcosystemResponse>;

    /// Report health status to Songbird
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;

    /// Update service capabilities
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>) -> UniversalResult<()>;
}

/// Context for user/device-specific primal routing and multi-tenancy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
    /// Biome identifier (for biomeOS integration)
    pub biome_id: Option<String>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

/// Network location information for context-aware routing
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

impl Default for NetworkLocation {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            subnet: None,
            network_id: None,
            geo_location: None,
        }
    }
}

/// Security level requirements (with proper ordering)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Public access
    Public,
    /// Basic security
    Basic,
    /// Standard security
    Standard,
    /// High security
    High,
    /// Critical security
    Critical,
    /// Maximum security
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// Universal primal type categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    /// AI primal (Squirrel)
    AI,
    /// Security primal (BearDog)
    Security,
    /// Storage primal (NestGate)
    Storage,
    /// Compute primal (ToadStool)
    Compute,
    /// Network primal (Songbird)
    Network,
    /// Operating System primal (biomeOS)
    OperatingSystem,
    /// Custom primal type
    Custom(String),
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::AI => write!(f, "AI"),
            PrimalType::Security => write!(f, "Security"),
            PrimalType::Storage => write!(f, "Storage"),
            PrimalType::Compute => write!(f, "Compute"),
            PrimalType::Network => write!(f, "Network"),
            PrimalType::OperatingSystem => write!(f, "OperatingSystem"),
            PrimalType::Custom(name) => write!(f, "Custom({name})"),
        }
    }
}

/// Universal capabilities that any primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // AI capabilities (Squirrel)
    /// Model inference with supported models
    ModelInference { models: Vec<String> },
    /// Agent framework with MCP support
    AgentFramework { mcp_support: bool },
    /// Machine learning with training support
    MachineLearning { training_support: bool },
    /// Natural language processing with supported languages
    NaturalLanguage { languages: Vec<String> },
    /// Computer vision with supported models
    ComputerVision { models: Vec<String> },
    /// Knowledge management
    KnowledgeManagement { formats: Vec<String> },
    /// Reasoning and logic
    Reasoning { engines: Vec<String> },
    /// Context understanding
    ContextUnderstanding { max_context_length: usize },

    // Security capabilities (BearDog)
    /// Authentication with supported methods
    Authentication { methods: Vec<String> },
    /// Encryption with supported algorithms
    Encryption { algorithms: Vec<String> },
    /// Key management with HSM support
    KeyManagement { hsm_support: bool },
    /// Threat detection with ML capabilities
    ThreatDetection { ml_enabled: bool },
    /// Audit logging with compliance standards
    AuditLogging { compliance: Vec<String> },
    /// Authorization and access control
    Authorization { rbac_support: bool },

    // Storage capabilities (NestGate)
    /// File system support
    FileSystem { supports_zfs: bool },
    /// Object storage with backends
    ObjectStorage { backends: Vec<String> },
    /// Data replication with consistency model
    DataReplication { consistency: String },
    /// Backup capabilities
    Backup { incremental: bool },
    /// Data archiving with compression
    DataArchiving { compression: Vec<String> },
    /// Volume management
    VolumeManagement { protocols: Vec<String> },

    // Compute capabilities (ToadStool)
    /// Container runtime support
    ContainerRuntime { orchestrators: Vec<String> },
    /// Serverless execution
    ServerlessExecution { languages: Vec<String> },
    /// GPU acceleration
    GpuAcceleration { cuda_support: bool },
    /// Load balancing
    LoadBalancing { algorithms: Vec<String> },
    /// Auto-scaling
    AutoScaling { metrics: Vec<String> },
    /// Native execution
    NativeExecution { architectures: Vec<String> },
    /// WebAssembly execution
    WasmExecution { wasi_support: bool },

    // Network capabilities (Songbird)
    /// Service discovery
    ServiceDiscovery { protocols: Vec<String> },
    /// Network routing
    NetworkRouting { protocols: Vec<String> },
    /// Proxy services
    ProxyServices { types: Vec<String> },
    /// VPN capabilities
    VpnServices { protocols: Vec<String> },
    /// Circuit breaking
    CircuitBreaking { enabled: bool },

    // Operating System capabilities (biomeOS)
    /// Orchestration with supported primals
    Orchestration { primals: Vec<String> },
    /// Manifest support with formats
    Manifests { formats: Vec<String> },
    /// Deployment strategies
    Deployment { strategies: Vec<String> },
    /// Monitoring with metrics
    Monitoring { metrics: Vec<String> },
    /// BYOB (Bring Your Own Biome) support
    BYOB { supported: bool },

    // Generic capabilities
    /// Custom capability with attributes
    Custom { name: String, attributes: HashMap<String, String> },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PrimalCapability::ModelInference { models } => {
                "ModelInference".hash(state);
                models.hash(state);
            }
            PrimalCapability::AgentFramework { mcp_support } => {
                "AgentFramework".hash(state);
                mcp_support.hash(state);
            }
            PrimalCapability::MachineLearning { training_support } => {
                "MachineLearning".hash(state);
                training_support.hash(state);
            }
            PrimalCapability::NaturalLanguage { languages } => {
                "NaturalLanguage".hash(state);
                languages.hash(state);
            }
            PrimalCapability::ComputerVision { models } => {
                "ComputerVision".hash(state);
                models.hash(state);
            }
            PrimalCapability::KnowledgeManagement { formats } => {
                "KnowledgeManagement".hash(state);
                formats.hash(state);
            }
            PrimalCapability::Reasoning { engines } => {
                "Reasoning".hash(state);
                engines.hash(state);
            }
            PrimalCapability::ContextUnderstanding { max_context_length } => {
                "ContextUnderstanding".hash(state);
                max_context_length.hash(state);
            }
            PrimalCapability::Authentication { methods } => {
                "Authentication".hash(state);
                methods.hash(state);
            }
            PrimalCapability::Encryption { algorithms } => {
                "Encryption".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::KeyManagement { hsm_support } => {
                "KeyManagement".hash(state);
                hsm_support.hash(state);
            }
            PrimalCapability::ThreatDetection { ml_enabled } => {
                "ThreatDetection".hash(state);
                ml_enabled.hash(state);
            }
            PrimalCapability::AuditLogging { compliance } => {
                "AuditLogging".hash(state);
                compliance.hash(state);
            }
            PrimalCapability::Authorization { rbac_support } => {
                "Authorization".hash(state);
                rbac_support.hash(state);
            }
            PrimalCapability::FileSystem { supports_zfs } => {
                "FileSystem".hash(state);
                supports_zfs.hash(state);
            }
            PrimalCapability::ObjectStorage { backends } => {
                "ObjectStorage".hash(state);
                backends.hash(state);
            }
            PrimalCapability::DataReplication { consistency } => {
                "DataReplication".hash(state);
                consistency.hash(state);
            }
            PrimalCapability::Backup { incremental } => {
                "Backup".hash(state);
                incremental.hash(state);
            }
            PrimalCapability::DataArchiving { compression } => {
                "DataArchiving".hash(state);
                compression.hash(state);
            }
            PrimalCapability::VolumeManagement { protocols } => {
                "VolumeManagement".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::ContainerRuntime { orchestrators } => {
                "ContainerRuntime".hash(state);
                orchestrators.hash(state);
            }
            PrimalCapability::ServerlessExecution { languages } => {
                "ServerlessExecution".hash(state);
                languages.hash(state);
            }
            PrimalCapability::GpuAcceleration { cuda_support } => {
                "GpuAcceleration".hash(state);
                cuda_support.hash(state);
            }
            PrimalCapability::LoadBalancing { algorithms } => {
                "LoadBalancing".hash(state);
                algorithms.hash(state);
            }
            PrimalCapability::AutoScaling { metrics } => {
                "AutoScaling".hash(state);
                metrics.hash(state);
            }
            PrimalCapability::NativeExecution { architectures } => {
                "NativeExecution".hash(state);
                architectures.hash(state);
            }
            PrimalCapability::WasmExecution { wasi_support } => {
                "WasmExecution".hash(state);
                wasi_support.hash(state);
            }
            PrimalCapability::ServiceDiscovery { protocols } => {
                "ServiceDiscovery".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::NetworkRouting { protocols } => {
                "NetworkRouting".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::ProxyServices { types } => {
                "ProxyServices".hash(state);
                types.hash(state);
            }
            PrimalCapability::VpnServices { protocols } => {
                "VpnServices".hash(state);
                protocols.hash(state);
            }
            PrimalCapability::CircuitBreaking { enabled } => {
                "CircuitBreaking".hash(state);
                enabled.hash(state);
            }
            PrimalCapability::Orchestration { primals } => {
                "Orchestration".hash(state);
                primals.hash(state);
            }
            PrimalCapability::Manifests { formats } => {
                "Manifests".hash(state);
                formats.hash(state);
            }
            PrimalCapability::Deployment { strategies } => {
                "Deployment".hash(state);
                strategies.hash(state);
            }
            PrimalCapability::Monitoring { metrics } => {
                "Monitoring".hash(state);
                metrics.hash(state);
            }
            PrimalCapability::BYOB { supported } => {
                "BYOB".hash(state);
                supported.hash(state);
            }
            PrimalCapability::Custom { name, attributes: _ } => {
                "Custom".hash(state);
                name.hash(state);
                // Skip hashing attributes since HashMap doesn't implement Hash
            }
        }
    }
}

/// Dependencies that a primal needs from other primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalDependency {
    /// Requires authentication
    RequiresAuthentication { methods: Vec<String> },
    /// Requires encryption
    RequiresEncryption { algorithms: Vec<String> },
    /// Requires storage
    RequiresStorage { types: Vec<String> },
    /// Requires compute
    RequiresCompute { types: Vec<String> },
    /// Requires AI capabilities
    RequiresAI { capabilities: Vec<String> },
    /// Requires network services
    RequiresNetwork { services: Vec<String> },
    /// Requires orchestration
    RequiresOrchestration { features: Vec<String> },
    /// Custom dependency
    Custom { name: String, requirements: HashMap<String, String> },
}

/// Health status of a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but operational
    Degraded { issues: Vec<String> },
    /// Primal is unhealthy and not operational
    Unhealthy { reason: String },
    /// Health status unknown
    Unknown,
}

/// Primal API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Service mesh integration endpoint
    pub service_mesh: String,
    /// Additional custom endpoints
    pub custom: HashMap<String, String>,
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        Self {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: Some("http://localhost:8080/metrics".to_string()),
            admin: Some("http://localhost:8080/admin".to_string()),
            websocket: Some("ws://localhost:8080/ws".to_string()),
            service_mesh: "http://localhost:8080/service-mesh".to_string(),
            custom: HashMap::new(),
        }
    }
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
    pub assigned_at: DateTime<Utc>,
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
    /// Port is being released
    Releasing,
    /// Port is expired and should be cleaned up
    Expired,
}

/// Service mesh status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshStatus {
    pub connected: bool,
    pub songbird_endpoint: Option<String>,
    pub registration_time: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub mesh_version: String,
    pub instance_id: String,
    pub load_balancing_enabled: bool,
    pub circuit_breaker_status: CircuitBreakerStatus,
}

/// Circuit breaker status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerStatus {
    Closed,
    Open,
    HalfOpen,
}

impl Default for ServiceMeshStatus {
    fn default() -> Self {
        Self {
            connected: false,
            songbird_endpoint: None,
            registration_time: None,
            last_heartbeat: None,
            mesh_version: "1.0.0".to_string(),
            instance_id: Uuid::new_v4().to_string(),
            load_balancing_enabled: false,
            circuit_breaker_status: CircuitBreakerStatus::Closed,
        }
    }
}

/// Standardized ecosystem request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
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
    /// Security context
    pub security_context: UniversalSecurityContext,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Standardized ecosystem response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    /// Request ID this response is for
    pub request_id: Uuid,
    /// Response status
    pub status: ResponseStatus,
    /// Response payload
    pub payload: serde_json::Value,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// Processing time
    pub processing_time_ms: u64,
}

/// Response status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { code: String, message: String },
    Timeout,
    ServiceUnavailable,
    AuthenticationRequired,
    AuthorizationFailed,
    RateLimited,
}

/// Universal security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,
    /// User/service identity
    pub identity: String,
    /// Permissions/capabilities
    pub permissions: Vec<String>,
    /// Security level required
    pub security_level: SecurityLevel,
    /// Session information
    pub session_id: Option<String>,
    /// Request signature (for crypto-lock)
    pub signature: Option<String>,
    /// Context creation timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for UniversalSecurityContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            identity: "anonymous".to_string(),
            permissions: Vec::new(),
            security_level: SecurityLevel::Public,
            session_id: None,
            signature: None,
            created_at: Utc::now(),
        }
    }
}

/// Inter-primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request ID
    pub id: Uuid,
    /// Source primal
    pub source: String,
    /// Target primal
    pub target: String,
    /// Operation to perform
    pub operation: String,
    /// Request data
    pub data: serde_json::Value,
    /// Security context
    pub security: UniversalSecurityContext,
    /// Request context
    pub context: PrimalContext,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Inter-primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Request ID this response is for
    pub request_id: Uuid,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Processing time
    pub processing_time: std::time::Duration,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Result of primal integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    /// Success status
    pub success: bool,
    /// Integration ID
    pub integration_id: String,
    /// Shared capabilities after integration
    pub shared_capabilities: Vec<PrimalCapability>,
    /// Configuration updates needed
    pub configuration_updates: Option<serde_json::Value>,
    /// Error message if integration failed
    pub error_message: Option<String>,
}

/// Universal configuration that works across all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalConfig {
    /// Service configuration
    pub service: ServiceConfig,
    /// Songbird integration settings
    pub songbird: SongbirdConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Resource limits and requirements
    pub resources: ResourceConfig,
    /// Feature flags
    pub features: FeatureFlags,
    /// Primal-specific configuration
    pub primal_specific: HashMap<String, serde_json::Value>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub bind_address: String,
    pub port: u16,
    pub log_level: String,
    pub instance_id: String,
}

/// Songbird configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub discovery_endpoint: String,
    pub registration_endpoint: String,
    pub health_endpoint: String,
    pub auth_token: Option<String>,
    pub retry_config: RetryConfig,
    pub heartbeat_interval_secs: u64,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub auth_method: String,
    pub tls_enabled: bool,
    pub mtls_required: bool,
    pub trust_domain: String,
    pub security_level: SecurityLevel,
    pub crypto_lock_enabled: bool,
}

/// Resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub disk_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u64>,
    pub gpu_count: Option<u32>,
}

/// Feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub development_mode: bool,
    pub debug_logging: bool,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub experimental_features: Vec<String>,
}

impl Default for UniversalConfig {
    fn default() -> Self {
        Self {
            service: ServiceConfig {
                name: "squirrel".to_string(),
                version: "1.0.0".to_string(),
                description: "Squirrel Universal AI Primal".to_string(),
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                log_level: "info".to_string(),
                instance_id: Uuid::new_v4().to_string(),
            },
            songbird: SongbirdConfig {
                discovery_endpoint: "http://localhost:8081/discovery".to_string(),
                registration_endpoint: "http://localhost:8081/register".to_string(),
                health_endpoint: "http://localhost:8081/health".to_string(),
                auth_token: None,
                retry_config: RetryConfig {
                    max_retries: 3,
                    initial_delay_ms: 1000,
                    max_delay_ms: 10000,
                    backoff_multiplier: 2.0,
                },
                heartbeat_interval_secs: 30,
            },
            security: SecurityConfig {
                auth_method: "beardog".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "ecoprimals.local".to_string(),
                security_level: SecurityLevel::Standard,
                crypto_lock_enabled: false,
            },
            resources: ResourceConfig {
                cpu_cores: Some(2.0),
                memory_mb: Some(4096),
                disk_mb: Some(10240),
                network_bandwidth_mbps: Some(1000),
                gpu_count: None,
            },
            features: FeatureFlags {
                development_mode: false,
                debug_logging: false,
                metrics_enabled: true,
                tracing_enabled: true,
                experimental_features: Vec::new(),
            },
            primal_specific: HashMap::new(),
        }
    }
}

/// Trait for creating new primals dynamically
pub trait PrimalFactory: Send + Sync {
    /// Create a new primal instance
    fn create_primal(&self, config: UniversalConfig) -> UniversalResult<Box<dyn UniversalPrimalProvider>>;
    
    /// Get supported primal types
    fn supported_types(&self) -> Vec<PrimalType>;
    
    /// Validate configuration for a primal type
    fn validate_config(&self, primal_type: PrimalType, config: &UniversalConfig) -> UniversalResult<()>;
}

/// Registry for managing multiple primal instances
pub struct PrimalRegistry {
    primals: HashMap<String, Box<dyn UniversalPrimalProvider>>,
    factories: HashMap<PrimalType, Box<dyn PrimalFactory>>,
}

impl PrimalRegistry {
    /// Create a new primal registry
    pub fn new() -> Self {
        Self {
            primals: HashMap::new(),
            factories: HashMap::new(),
        }
    }
    
    /// Register a primal factory
    pub fn register_factory(&mut self, primal_type: PrimalType, factory: Box<dyn PrimalFactory>) {
        self.factories.insert(primal_type, factory);
    }
    
    /// Create and register a new primal instance
    pub fn create_primal(&mut self, primal_type: PrimalType, config: UniversalConfig) -> UniversalResult<String> {
        let factory = self.factories.get(&primal_type)
            .ok_or_else(|| PrimalError::General(format!("No factory registered for primal type: {}", primal_type)))?;
        
        let primal = factory.create_primal(config)?;
        let instance_id = primal.instance_id().to_string();
        
        self.primals.insert(instance_id.clone(), primal);
        Ok(instance_id)
    }
    
    /// Get a primal instance by ID
    pub fn get_primal(&self, instance_id: &str) -> Option<&dyn UniversalPrimalProvider> {
        self.primals.get(instance_id).map(|p| p.as_ref())
    }
    
    /// Get all primal instances
    pub fn get_all_primals(&self) -> Vec<&dyn UniversalPrimalProvider> {
        self.primals.values().map(|p| p.as_ref()).collect()
    }
    
    /// Remove a primal instance
    pub fn remove_primal(&mut self, instance_id: &str) -> UniversalResult<()> {
        if let Some(mut primal) = self.primals.remove(instance_id) {
            // Shutdown the primal gracefully
            tokio::spawn(async move {
                let _ = primal.shutdown().await;
            });
        }
        Ok(())
    }
}

impl Default for PrimalRegistry {
    fn default() -> Self {
        Self::new()
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_universal_system_integration() {
        // Test complete universal system workflow
        let config = UniversalServiceConfig::new()
            .add_discovery_endpoint("http://localhost:8500".to_string())
            .unwrap()
            .with_default_timeout(Duration::from_secs(30))
            .unwrap()
            .add_service(
                "ai-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["chat".to_string()],
                    weight: Some(0.8),
                    tags: vec!["ai".to_string()],
                    priority: Some(1),
                    required: true,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        // Create universal primal provider
        let provider = UniversalPrimalProvider::new(config);
        
        // Test service query
        let primal_query = PrimalQuery::new("ai".to_string())
            .with_capability("chat".to_string())
            .with_metadata("type".to_string(), "ai".to_string());
        
        let result = provider.query_primal(primal_query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "ai-service");
        assert_eq!(primals[0].primary_endpoint, "http://localhost:8080");
        assert!(primals[0].has_capability("chat"));
        assert!(primals[0].has_tag("ai"));
        
        // Test API
        let api = UniversalApi::new(Arc::new(provider));
        
        let response = api.get_primals().await;
        assert!(response.is_ok());
        
        let primals = response.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "ai-service");
        assert_eq!(primals[0].service_type, "ai");
        assert_eq!(primals[0].capabilities.len(), 1);
        assert_eq!(primals[0].capabilities[0], "chat");
    }
    
    #[tokio::test]
    async fn test_service_discovery_with_config() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "test-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["test".to_string()],
                    weight: Some(0.5),
                    tags: vec!["test".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test getting service by name
        let service = provider.get_service("test-service").await;
        assert!(service.is_ok());
        
        let service = service.unwrap();
        assert!(service.is_some());
        
        let service = service.unwrap();
        assert_eq!(service.name, "test-service");
        assert_eq!(service.primary_endpoint, "http://localhost:8080");
        assert!(service.has_capability("test"));
        assert!(service.has_tag("test"));
    }
    
    #[tokio::test]
    async fn test_load_balancing_integration() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "balanced-service".to_string(),
                ServiceConfig {
                    endpoints: vec![
                        "http://localhost:8080".to_string(),
                        "http://localhost:8081".to_string(),
                    ],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["balance".to_string()],
                    weight: Some(0.8),
                    tags: vec!["balanced".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test load balancing
        let query = PrimalQuery::new("balanced-service".to_string())
            .with_capability("balance".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "balanced-service");
        assert_eq!(primals[0].endpoints.len(), 2);
    }
    
    #[tokio::test]
    async fn test_health_check_integration() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "health-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["health".to_string()],
                    weight: Some(0.8),
                    tags: vec!["healthy".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test health check
        let result = provider.check_health("health-service").await;
        assert!(result.is_ok());
        
        let health = result.unwrap();
        assert!(health.is_some());
        
        let health = health.unwrap();
        assert_eq!(health.service_name, "health-service");
        assert_eq!(health.status, "healthy");
        assert!(health.endpoints.len() > 0);
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let config = UniversalServiceConfig::new()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test non-existent service
        let result = provider.get_service("non-existent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        
        // Test health check for non-existent service
        let result = provider.check_health("non-existent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        
        // Test query with no results
        let query = PrimalQuery::new("non-existent".to_string())
            .with_capability("non-existent".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_api_endpoints() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "api-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["api".to_string()],
                    weight: Some(0.8),
                    tags: vec!["api".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        let api = UniversalApi::new(Arc::new(provider));
        
        // Test get primals
        let result = api.get_primals().await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "api-service");
        
        // Test get primal by name
        let result = api.get_primal("api-service").await;
        assert!(result.is_ok());
        
        let primal = result.unwrap();
        assert!(primal.is_some());
        assert_eq!(primal.unwrap().name, "api-service");
        
        // Test health check
        let result = api.check_health("api-service").await;
        assert!(result.is_ok());
        
        let health = result.unwrap();
        assert!(health.is_some());
        assert_eq!(health.unwrap().service_name, "api-service");
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "concurrent-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["concurrent".to_string()],
                    weight: Some(0.8),
                    tags: vec!["concurrent".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = Arc::new(UniversalPrimalProvider::new(config));
        let mut handles = Vec::new();
        
        // Test concurrent access
        for i in 0..10 {
            let provider = provider.clone();
            let handle = tokio::spawn(async move {
                let query = PrimalQuery::new("concurrent-service".to_string())
                    .with_capability("concurrent".to_string());
                
                let result = provider.query_primal(query).await;
                assert!(result.is_ok());
                
                let primals = result.unwrap();
                assert_eq!(primals.len(), 1);
                assert_eq!(primals[0].name, "concurrent-service");
                
                i
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
    
    #[tokio::test]
    async fn test_configuration_validation() {
        // Test invalid service configuration
        let result = UniversalServiceConfig::new()
            .add_service(
                "invalid-service".to_string(),
                ServiceConfig {
                    endpoints: vec![], // Invalid: no endpoints
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["invalid".to_string()],
                    weight: Some(0.8),
                    tags: vec!["invalid".to_string()],
                    priority: Some(1),
                    required: false,
                },
            );
        
        assert!(result.is_err());
        
        // Test invalid endpoint URL
        let result = UniversalServiceConfig::new()
            .add_service(
                "invalid-url-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["invalid-url".to_string()], // Invalid URL
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["invalid".to_string()],
                    weight: Some(0.8),
                    tags: vec!["invalid".to_string()],
                    priority: Some(1),
                    required: false,
                },
            );
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_capability_filtering() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "multi-cap-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["chat".to_string(), "search".to_string()],
                    weight: Some(0.8),
                    tags: vec!["multi".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test capability filtering
        let query = PrimalQuery::new("multi-cap-service".to_string())
            .with_capability("chat".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 1);
        assert!(primals[0].has_capability("chat"));
        assert!(primals[0].has_capability("search"));
        
        // Test non-matching capability
        let query = PrimalQuery::new("multi-cap-service".to_string())
            .with_capability("non-existent".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 0);
    }
    
    #[tokio::test]
    async fn test_metadata_filtering() {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "ai".to_string());
        metadata.insert("version".to_string(), "1.0".to_string());
        
        let config = UniversalServiceConfig::new()
            .add_service(
                "metadata-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: metadata,
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["metadata".to_string()],
                    weight: Some(0.8),
                    tags: vec!["metadata".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test metadata filtering
        let query = PrimalQuery::new("metadata-service".to_string())
            .with_metadata("type".to_string(), "ai".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].metadata.get("type").unwrap(), "ai");
        assert_eq!(primals[0].metadata.get("version").unwrap(), "1.0");
        
        // Test non-matching metadata
        let query = PrimalQuery::new("metadata-service".to_string())
            .with_metadata("type".to_string(), "compute".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 0);
    }
    
    #[tokio::test]
    async fn test_priority_ordering() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "high-priority".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["priority".to_string()],
                    weight: Some(0.8),
                    tags: vec!["high".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .add_service(
                "low-priority".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8081".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8081/health".to_string()),
                    capabilities: vec!["priority".to_string()],
                    weight: Some(0.8),
                    tags: vec!["low".to_string()],
                    priority: Some(10),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test priority ordering
        let query = PrimalQuery::new("".to_string())
            .with_capability("priority".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 2);
        
        // Check that high priority comes first
        assert_eq!(primals[0].name, "high-priority");
        assert_eq!(primals[1].name, "low-priority");
    }
    
    #[tokio::test]
    async fn test_weight_based_selection() {
        let config = UniversalServiceConfig::new()
            .add_service(
                "heavy-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8080".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8080/health".to_string()),
                    capabilities: vec!["weight".to_string()],
                    weight: Some(0.9),
                    tags: vec!["heavy".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .add_service(
                "light-service".to_string(),
                ServiceConfig {
                    endpoints: vec!["http://localhost:8081".to_string()],
                    timeout: Some(Duration::from_secs(10)),
                    metadata: HashMap::new(),
                    health_check_url: Some("http://localhost:8081/health".to_string()),
                    capabilities: vec!["weight".to_string()],
                    weight: Some(0.1),
                    tags: vec!["light".to_string()],
                    priority: Some(1),
                    required: false,
                },
            )
            .unwrap()
            .build()
            .unwrap();
        
        let provider = UniversalPrimalProvider::new(config);
        
        // Test weight-based selection
        let query = PrimalQuery::new("".to_string())
            .with_capability("weight".to_string());
        
        let result = provider.query_primal(query).await;
        assert!(result.is_ok());
        
        let primals = result.unwrap();
        assert_eq!(primals.len(), 2);
        
        // Check that services are ordered by weight (descending)
        assert_eq!(primals[0].name, "heavy-service");
        assert_eq!(primals[0].weight, Some(0.9));
        assert_eq!(primals[1].name, "light-service");
        assert_eq!(primals[1].weight, Some(0.1));
    }
} 