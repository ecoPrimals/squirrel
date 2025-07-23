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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use crate::error::PrimalError;

/// Universal result type for all primal operations
pub type UniversalResult<T> = Result<T, PrimalError>;

/// Universal system version
pub const VERSION: &str = "1.0.0";

/// Initialize the universal system
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing::info!("Universal Primal System v{} initialized", VERSION);
    Ok(())
}

// ============================================================================
// CORE TYPES AND TRAITS
// ============================================================================

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
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse>;
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;
    async fn shutdown(&mut self) -> UniversalResult<()>;
    fn can_serve_context(&self, context: &PrimalContext) -> bool;
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String>;
    async fn deregister_from_songbird(&mut self) -> UniversalResult<()>;
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse>;
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>)
        -> UniversalResult<()>;
}

/// Context for user/device-specific primal routing and multi-tenancy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PrimalContext {
    pub user_id: String,
    pub device_id: String,
    pub session_id: String,
    pub network_location: NetworkLocation,
    pub security_level: SecurityLevel,
    pub biome_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Network location information for context-aware routing
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkLocation {
    pub ip_address: String,
    pub subnet: Option<String>,
    pub network_id: Option<String>,
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

/// Security levels for universal operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Public access - no authentication required  
    Public,
    /// Basic authentication required
    Basic,
    /// Standard security level
    Standard,
    /// High security level  
    High,
    /// Critical security level
    Critical,
    /// Maximum security level
    Maximum,
    /// Advanced authentication with MFA
    Advanced,
    /// Internal system access
    Internal,
    /// Administrative access
    Administrative,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Basic
    }
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::Public => write!(f, "public"),
            SecurityLevel::Basic => write!(f, "basic"),
            SecurityLevel::Standard => write!(f, "standard"),
            SecurityLevel::High => write!(f, "high"),
            SecurityLevel::Critical => write!(f, "critical"),
            SecurityLevel::Maximum => write!(f, "maximum"),
            SecurityLevel::Advanced => write!(f, "advanced"),
            SecurityLevel::Internal => write!(f, "internal"),
            SecurityLevel::Administrative => write!(f, "administrative"),
        }
    }
}

/// Load balancing status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStatus {
    /// Whether load balancing is enabled
    pub enabled: bool,
    /// Whether load balancing is healthy
    pub healthy: bool,
    /// Number of active connections
    pub active_connections: u32,
    /// Load balancing algorithm in use
    pub algorithm: String,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

/// Circuit breaker status for service resilience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Whether the circuit breaker is open
    pub open: bool,
    /// Number of recent failures
    pub failures: u32,
    /// Timestamp of the last failure
    pub last_failure: Option<DateTime<Utc>>,
    /// When to attempt the next retry
    pub next_retry: Option<DateTime<Utc>>,
}

impl Default for CircuitBreakerStatus {
    fn default() -> Self {
        Self {
            open: false,
            failures: 0,
            last_failure: None,
            next_retry: None,
        }
    }
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
    pub capabilities: Vec<PrimalCapability>,
}
/// Categories of primals in the ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    Squirrel,
    AI,
    Security,
    Storage,
    Compute,
    Network,
    OS,
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrimalType::Squirrel => write!(f, "squirrel"),
            PrimalType::AI => write!(f, "ai"),
            PrimalType::Security => write!(f, "security"),
            PrimalType::Storage => write!(f, "storage"),
            PrimalType::Compute => write!(f, "compute"),
            PrimalType::Network => write!(f, "network"),
            PrimalType::OS => write!(f, "os"),
        }
    }
}

/// Capabilities that a primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // AI capabilities
    ModelInference {
        models: Vec<String>,
    },
    AgentFramework {
        mcp_support: bool,
    },
    MachineLearning {
        training_support: bool,
    },
    NaturalLanguage {
        languages: Vec<String>,
    },
    ComputerVision {
        models: Vec<String>,
    },

    // Security capabilities
    Authentication {
        methods: Vec<String>,
    },
    Encryption {
        algorithms: Vec<String>,
    },
    KeyManagement {
        hsm_support: bool,
    },

    // Storage capabilities
    FileSystem {
        supports_zfs: bool,
    },
    ObjectStorage {
        backends: Vec<String>,
    },
    DataReplication {
        consistency: String,
    },

    // Compute capabilities
    ContainerRuntime {
        orchestrators: Vec<String>,
    },
    ServerlessExecution {
        languages: Vec<String>,
    },
    GpuAcceleration {
        cuda_support: bool,
    },

    // Network capabilities
    ServiceDiscovery {
        protocols: Vec<String>,
    },
    NetworkRouting {
        protocols: Vec<String>,
    },
    CircuitBreaking {
        enabled: bool,
    },

    // Operating System capabilities
    Orchestration {
        primals: Vec<String>,
    },
    Manifests {
        formats: Vec<String>,
    },
    Deployment {
        strategies: Vec<String>,
    },

    // Generic capabilities
    Custom {
        name: String,
        attributes: HashMap<String, String>,
    },
}

impl Hash for PrimalCapability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            PrimalCapability::ModelInference { models } => models.hash(state),
            PrimalCapability::AgentFramework { mcp_support } => mcp_support.hash(state),
            PrimalCapability::MachineLearning { training_support } => training_support.hash(state),
            PrimalCapability::NaturalLanguage { languages } => languages.hash(state),
            PrimalCapability::ComputerVision { models } => models.hash(state),
            PrimalCapability::Authentication { methods } => methods.hash(state),
            PrimalCapability::Encryption { algorithms } => algorithms.hash(state),
            PrimalCapability::KeyManagement { hsm_support } => hsm_support.hash(state),
            PrimalCapability::FileSystem { supports_zfs } => supports_zfs.hash(state),
            PrimalCapability::ObjectStorage { backends } => backends.hash(state),
            PrimalCapability::DataReplication { consistency } => consistency.hash(state),
            PrimalCapability::ContainerRuntime { orchestrators } => orchestrators.hash(state),
            PrimalCapability::ServerlessExecution { languages } => languages.hash(state),
            PrimalCapability::GpuAcceleration { cuda_support } => cuda_support.hash(state),
            PrimalCapability::ServiceDiscovery { protocols } => protocols.hash(state),
            PrimalCapability::NetworkRouting { protocols } => protocols.hash(state),
            PrimalCapability::CircuitBreaking { enabled } => enabled.hash(state),
            PrimalCapability::Orchestration { primals } => primals.hash(state),
            PrimalCapability::Manifests { formats } => formats.hash(state),
            PrimalCapability::Deployment { strategies } => strategies.hash(state),
            PrimalCapability::Custom { name, attributes } => {
                name.hash(state);
                for (key, value) in attributes {
                    key.hash(state);
                    value.hash(state);
                }
            }
        }
    }
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

/// Dependencies that a primal requires from other primals
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimalDependency {
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub required: bool,
    pub min_version: Option<String>,
    pub preferred_instance: Option<String>,
}

/// Health status of a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    pub status: HealthStatus,
    pub details: HashMap<String, String>,
    pub last_check: DateTime<Utc>,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// Endpoint information for a primal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    pub primary: String,
    pub health: String,
    pub metrics: String,
    pub admin: String,
    pub websocket: Option<String>,
    pub mcp: String,
    pub ai_coordination: String,
    pub service_mesh: String,
    pub custom: HashMap<String, String>,
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        Self {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: "http://localhost:8080/metrics".to_string(),
            admin: "http://localhost:8080/admin".to_string(),
            websocket: Some("ws://localhost:8080/ws".to_string()),
            mcp: "http://localhost:8080/mcp".to_string(),
            ai_coordination: "http://localhost:8080/ai".to_string(),
            service_mesh: "http://localhost:8080/service-mesh".to_string(),
            custom: HashMap::new(),
        }
    }
}

// ============================================================================
// COMMUNICATION TYPES
// ============================================================================

/// Universal primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    pub request_id: Uuid,
    pub id: String,
    pub source_primal: String,
    pub target_primal: String,
    pub source: PrimalInfo,
    pub payload: serde_json::Value,
    pub target: PrimalType,
    pub operation: String,
    pub method: String,
    pub parameters: serde_json::Value,
    pub params: serde_json::Value,
    pub security_context: UniversalSecurityContext,
    pub metadata: HashMap<String, String>,
    pub context: PrimalContext,
    pub timestamp: DateTime<Utc>,
    pub timeout: Option<chrono::Duration>,
}
/// Universal primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub response_id: Uuid,
    pub request_id: Uuid,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub data: serde_json::Value,
    pub success: bool,
    pub error_message: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub duration: chrono::Duration,
    pub status: ResponseStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    PartialSuccess,
    Error,
    Timeout,
}

/// Standardized ecosystem request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemRequest {
    pub request_id: Uuid,
    pub source_service: String,
    pub target_service: String,
    pub operation: String,
    pub payload: serde_json::Value,
    pub security_context: UniversalSecurityContext,
    pub context: PrimalContext,
    pub timestamp: DateTime<Utc>,
    pub timeout: Option<chrono::Duration>,
    pub metadata: HashMap<String, String>,
}

/// Response from ecosystem operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    pub response_id: Uuid,
    pub request_id: Uuid,
    pub payload: serde_json::Value,
    pub status: ResponseStatus,
    pub metadata: HashMap<String, String>,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Universal security context for inter-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecurityContext {
    pub user_id: String,
    pub session_id: String,
    pub auth_token: Option<String>,
    pub security_level: SecurityLevel,
    pub permissions: Vec<String>,
}

impl Default for UniversalSecurityContext {
    fn default() -> Self {
        Self {
            user_id: "anonymous".to_string(),
            session_id: Uuid::new_v4().to_string(),
            auth_token: None,
            security_level: SecurityLevel::Basic,
            permissions: Vec::new(),
        }
    }
}

impl PrimalRequest {
    /// Create a new PrimalRequest with all required fields
    pub fn new(
        source_primal: impl Into<String>,
        target_primal: impl Into<String>,
        operation: impl Into<String>,
        payload: serde_json::Value,
        context: PrimalContext,
    ) -> Self {
        let operation_str = operation.into();
        Self {
            request_id: uuid::Uuid::new_v4(),
            id: uuid::Uuid::new_v4().to_string(),
            source_primal: source_primal.into(),
            target_primal: target_primal.into(),
            source: PrimalInfo {
                id: "squirrel".to_string(),
                instance_id: "squirrel-1".to_string(),
                primal_type: PrimalType::Squirrel,
                version: "1.0.0".to_string(),
                capabilities: vec![],
            },
            payload: payload.clone(),
            target: PrimalType::Squirrel, // Default, should be updated based on target_primal
            operation: operation_str.clone(),
            method: operation_str,
            parameters: payload.clone(),
            params: payload,
            security_context: UniversalSecurityContext::default(),
            metadata: std::collections::HashMap::new(),
            context,
            timestamp: chrono::Utc::now(),
            timeout: Some(chrono::Duration::seconds(30)),
        }
    }

    /// Set the target primal type
    pub fn with_target_type(mut self, target: PrimalType) -> Self {
        self.target = target;
        self
    }

    /// Set the source primal info
    pub fn with_source_info(mut self, source: PrimalInfo) -> Self {
        self.source = source;
        self
    }

    /// Set the security context
    pub fn with_security_context(mut self, security_context: UniversalSecurityContext) -> Self {
        self.security_context = security_context;
        self
    }

    /// Set the metadata
    pub fn with_metadata(mut self, metadata: std::collections::HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

// ============================================================================
// DYNAMIC PORT AND SERVICE MESH
// ============================================================================

/// Dynamic port assignment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    pub assigned_port: u16,
    pub current_port: u16,
    pub port_range: (u16, u16),
    pub port_type: PortType,
    pub status: PortStatus,
    pub assigned_at: DateTime<Utc>,
    pub allocated_at: DateTime<Utc>,
    pub lease_duration: chrono::Duration,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    Http,
    Https,
    WebSocket,
    Grpc,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    Active,
    Reserved,
    Releasing,
    Expired,
}

/// Service mesh status information
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
    pub circuit_breaker_status: CircuitBreakerStatus,
    pub active_connections: u32,
    pub load_balancing: LoadBalancingStatus,
}

impl Default for ServiceMeshStatus {
    fn default() -> Self {
        Self {
            connected: false,
            registered: false,
            songbird_endpoint: None,
            registration_time: None,
            last_registration: None,
            last_heartbeat: None,
            mesh_version: "1.0.0".to_string(),
            mesh_health: "unknown".to_string(),
            instance_id: Uuid::new_v4().to_string(),
            load_balancing_enabled: false,
            circuit_breaker_status: CircuitBreakerStatus::default(),
            active_connections: 0,
            load_balancing: LoadBalancingStatus {
                enabled: false,
                healthy: false,
                active_connections: 0,
                algorithm: "round_robin".to_string(),
                health_score: 0.0,
                last_check: chrono::Utc::now(),
            },
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Create a default primal context
pub fn create_default_context(user_id: &str, device_id: &str) -> PrimalContext {
    PrimalContext {
        user_id: user_id.to_string(),
        device_id: device_id.to_string(),
        session_id: Uuid::new_v4().to_string(),
        network_location: NetworkLocation::default(),
        security_level: SecurityLevel::Basic,
        biome_id: None,
        metadata: HashMap::new(),
    }
}

/// Create a default security context
pub fn create_default_security_context(user_id: &str) -> UniversalSecurityContext {
    UniversalSecurityContext {
        user_id: user_id.to_string(),
        session_id: Uuid::new_v4().to_string(),
        auth_token: None,
        security_level: SecurityLevel::Basic,
        permissions: Vec::new(),
    }
}

/// Validate primal capability compatibility
pub fn validate_capability_compatibility(
    provided: &PrimalCapability,
    required: &PrimalCapability,
) -> bool {
    match (provided, required) {
        (
            PrimalCapability::ModelInference { models: p },
            PrimalCapability::ModelInference { models: r },
        ) => r.iter().all(|req| p.contains(req)),
        (
            PrimalCapability::Authentication { methods: p },
            PrimalCapability::Authentication { methods: r },
        ) => r.iter().all(|req| p.contains(req)),
        (
            PrimalCapability::Encryption { algorithms: p },
            PrimalCapability::Encryption { algorithms: r },
        ) => r.iter().all(|req| p.contains(req)),
        (
            PrimalCapability::ObjectStorage { backends: p },
            PrimalCapability::ObjectStorage { backends: r },
        ) => r.iter().all(|req| p.contains(req)),
        (
            PrimalCapability::ContainerRuntime { orchestrators: p },
            PrimalCapability::ContainerRuntime { orchestrators: r },
        ) => r.iter().all(|req| p.contains(req)),
        (
            PrimalCapability::ServiceDiscovery { protocols: p },
            PrimalCapability::ServiceDiscovery { protocols: r },
        ) => r.iter().all(|req| p.contains(req)),
        _ => provided == required,
    }
}

/// Create an ecosystem request
pub fn create_ecosystem_request(
    source: &str,
    target: &str,
    operation: &str,
    payload: serde_json::Value,
    security_context: UniversalSecurityContext,
) -> EcosystemRequest {
    EcosystemRequest {
        request_id: Uuid::new_v4(),
        source_service: source.to_string(),
        target_service: target.to_string(),
        operation: operation.to_string(),
        payload,
        security_context,
        context: PrimalContext::default(),
        timestamp: chrono::Utc::now(),
        timeout: Some(chrono::Duration::seconds(30)),
        metadata: HashMap::new(),
    }
}

/// Create a successful ecosystem response
pub fn create_success_response(request_id: Uuid, payload: serde_json::Value) -> EcosystemResponse {
    EcosystemResponse {
        response_id: Uuid::new_v4(),
        request_id,
        payload,
        status: ResponseStatus::Success,
        metadata: HashMap::new(),
        success: true,
        error_message: None,
    }
}

/// Create an error ecosystem response
pub fn create_error_response(request_id: Uuid, error_message: &str) -> EcosystemResponse {
    EcosystemResponse {
        response_id: Uuid::new_v4(),
        request_id,
        payload: serde_json::json!({
            "error": error_message,
            "timestamp": Utc::now()
        }),
        status: ResponseStatus::Error,
        metadata: HashMap::new(),
        success: false,
        error_message: Some(error_message.to_string()),
    }
}

// ============================================================================
// SECURITY TYPES (Re-exported for compatibility)
// ============================================================================

/// Service capability for security providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceCapability {
    Authentication {
        methods: Vec<String>,
    },
    Authorization {
        features: Vec<String>,
    },
    Security {
        level: String,
        features: Vec<String>,
    },
    Encryption {
        algorithms: Vec<String>,
    },
    Auditing {
        capabilities: Vec<String>,
    },
    Custom {
        name: String,
        description: String,
        metadata: HashMap<String, serde_json::Value>,
    },
}

/// Service endpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub capabilities: Vec<ServiceCapability>,
    pub health_status: String,
}

/// Universal security provider trait
#[async_trait]
pub trait UniversalSecurityProvider: Send + Sync {
    /// Associated session type
    type Session;
    /// Associated error type
    type Error;

    async fn authenticate(&self, credentials: Value) -> Result<Self::Session, Self::Error>;
    async fn authorize(
        &self,
        session_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, Self::Error>;
    async fn health_check(&self) -> Result<bool, Self::Error>;

    /// Get session by ID
    async fn get_session(&self, session_id: &str) -> Result<Option<Self::Session>, Self::Error>;

    /// Revoke a session
    async fn revoke_session(&self, session_id: &str) -> Result<(), Self::Error>;

    /// Get provider capabilities
    async fn get_capabilities(&self) -> Result<Vec<ServiceCapability>, Self::Error>;
}

/// Universal security session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalSecuritySession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub capabilities: Vec<ServiceCapability>,
    pub metadata: HashMap<String, String>,
}

/// Universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    pub service_id: String,
    pub service_name: String,
    pub endpoints: Vec<ServiceEndpoint>,
    pub capabilities: Vec<ServiceCapability>,
    pub metadata: HashMap<String, String>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_system_initialization() {
        let result = init();
        assert!(result.is_ok());
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_context_creation() {
        let context = create_default_context("test-user", "test-device");
        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.device_id, "test-device");
        assert_eq!(context.security_level, SecurityLevel::Basic);
    }

    #[test]
    fn test_security_context_creation() {
        let security_context = create_default_security_context("test-user");
        assert_eq!(security_context.user_id, "test-user");
        assert_eq!(security_context.security_level, SecurityLevel::Basic);
    }

    #[test]
    fn test_capability_validation() {
        let provided = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string(), "claude-3".to_string()],
        };

        let required = PrimalCapability::ModelInference {
            models: vec!["gpt-4".to_string()],
        };

        assert!(validate_capability_compatibility(&provided, &required));

        let required = PrimalCapability::ModelInference {
            models: vec!["gpt-5".to_string()],
        };

        assert!(!validate_capability_compatibility(&provided, &required));
    }

    #[test]
    fn test_ecosystem_requests() {
        let security_context = create_default_security_context("test-user");
        let request = create_ecosystem_request(
            "source-service",
            "target-service",
            "test-operation",
            serde_json::json!({"data": "test"}),
            security_context,
        );

        assert_eq!(request.source_service, "source-service");
        assert_eq!(request.target_service, "target-service");
        assert_eq!(request.operation, "test-operation");

        let response =
            create_success_response(request.request_id, serde_json::json!({"result": "success"}));

        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.status, ResponseStatus::Success);
        assert!(response.success);
        assert!(response.error_message.is_none());

        let error_response = create_error_response(request.request_id, "Test error message");

        assert_eq!(error_response.request_id, request.request_id);
        assert_eq!(error_response.status, ResponseStatus::Error);
        assert!(!error_response.success);
        assert_eq!(error_response.error_message.unwrap(), "Test error message");
    }
}
