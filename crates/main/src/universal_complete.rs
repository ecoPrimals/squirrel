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
use std::time::Duration;

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
    async fn handle_primal_request(&self, request: PrimalRequest) -> UniversalResult<PrimalResponse>;
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;
    async fn shutdown(&mut self) -> UniversalResult<()>;
    fn can_serve_context(&self, context: &PrimalContext) -> bool;
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;
    async fn register_with_songbird(&mut self, songbird_endpoint: &str) -> UniversalResult<String>;
    async fn deregister_from_songbird(&mut self) -> UniversalResult<()>;
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;
    async fn handle_ecosystem_request(&self, request: EcosystemRequest) -> UniversalResult<EcosystemResponse>;
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>) -> UniversalResult<()>;
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

/// Security level requirements (with proper ordering)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    Public,
    Basic,
    Standard,
    High,
    Critical,
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Basic
    }
}

/// Categories of primals in the ecosystem
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalType {
    AI,
    Security,
    Storage,
    Compute,
    Network,
    OS,
}

/// Capabilities that a primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalCapability {
    // AI capabilities
    ModelInference { models: Vec<String> },
    AgentFramework { mcp_support: bool },
    MachineLearning { training_support: bool },
    NaturalLanguage { languages: Vec<String> },
    ComputerVision { models: Vec<String> },
    
    // Security capabilities
    Authentication { methods: Vec<String> },
    Encryption { algorithms: Vec<String> },
    KeyManagement { hsm_support: bool },
    
    // Storage capabilities
    FileSystem { supports_zfs: bool },
    ObjectStorage { backends: Vec<String> },
    DataReplication { consistency: String },
    
    // Compute capabilities
    ContainerRuntime { orchestrators: Vec<String> },
    ServerlessExecution { languages: Vec<String> },
    GpuAcceleration { cuda_support: bool },
    
    // Network capabilities
    ServiceDiscovery { protocols: Vec<String> },
    NetworkRouting { protocols: Vec<String> },
    CircuitBreaking { enabled: bool },
    
    // Operating System capabilities
    Orchestration { primals: Vec<String> },
    Manifests { formats: Vec<String> },
    Deployment { strategies: Vec<String> },
    
    // Generic capabilities
    Custom { name: String, attributes: HashMap<String, String> },
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
                attributes.hash(state);
            }
        }
    }
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

/// API endpoints for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    pub primary: String,
    pub health: String,
    pub metrics: String,
    pub service_mesh: String,
    pub custom: HashMap<String, String>,
}

impl Default for PrimalEndpoints {
    fn default() -> Self {
        Self {
            primary: "http://localhost:8080".to_string(),
            health: "http://localhost:8080/health".to_string(),
            metrics: "http://localhost:8080/metrics".to_string(),
            service_mesh: "http://localhost:8080/service-mesh".to_string(),
            custom: HashMap::new(),
        }
    }
}

// ============================================================================
// COMMUNICATION TYPES
// ============================================================================

/// Request structure for inter-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    pub request_id: Uuid,
    pub source_primal: String,
    pub target_primal: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

/// Response structure for inter-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub response_id: Uuid,
    pub request_id: Uuid,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
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
    pub metadata: HashMap<String, String>,
}

/// Standardized ecosystem response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemResponse {
    pub response_id: Uuid,
    pub request_id: Uuid,
    pub payload: serde_json::Value,
    pub status: ResponseStatus,
    pub metadata: HashMap<String, String>,
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

// ============================================================================
// DYNAMIC PORT AND SERVICE MESH
// ============================================================================

/// Dynamic port information for songbird-managed ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    pub assigned_port: u16,
    pub port_type: PortType,
    pub status: PortStatus,
    pub assigned_at: DateTime<Utc>,
    pub lease_duration: chrono::Duration,
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
    pub songbird_endpoint: Option<String>,
    pub registration_time: Option<DateTime<Utc>>,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub mesh_version: String,
    pub instance_id: String,
    pub load_balancing_enabled: bool,
    pub circuit_breaker_status: CircuitBreakerStatus,
}

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
        (PrimalCapability::ModelInference { models: p }, PrimalCapability::ModelInference { models: r }) => {
            r.iter().all(|req| p.contains(req))
        }
        (PrimalCapability::Authentication { methods: p }, PrimalCapability::Authentication { methods: r }) => {
            r.iter().all(|req| p.contains(req))
        }
        (PrimalCapability::Encryption { algorithms: p }, PrimalCapability::Encryption { algorithms: r }) => {
            r.iter().all(|req| p.contains(req))
        }
        (PrimalCapability::ObjectStorage { backends: p }, PrimalCapability::ObjectStorage { backends: r }) => {
            r.iter().all(|req| p.contains(req))
        }
        (PrimalCapability::ContainerRuntime { orchestrators: p }, PrimalCapability::ContainerRuntime { orchestrators: r }) => {
            r.iter().all(|req| p.contains(req))
        }
        (PrimalCapability::ServiceDiscovery { protocols: p }, PrimalCapability::ServiceDiscovery { protocols: r }) => {
            r.iter().all(|req| p.contains(req))
        }
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
        metadata: HashMap::new(),
    }
}

/// Create a successful ecosystem response
pub fn create_success_response(
    request_id: Uuid,
    payload: serde_json::Value,
) -> EcosystemResponse {
    EcosystemResponse {
        response_id: Uuid::new_v4(),
        request_id,
        payload,
        status: ResponseStatus::Success,
        metadata: HashMap::new(),
    }
}

/// Create an error ecosystem response
pub fn create_error_response(
    request_id: Uuid,
    error_message: &str,
) -> EcosystemResponse {
    EcosystemResponse {
        response_id: Uuid::new_v4(),
        request_id,
        payload: serde_json::json!({
            "error": error_message,
            "timestamp": Utc::now()
        }),
        status: ResponseStatus::Error,
        metadata: HashMap::new(),
    }
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
        
        let response = create_success_response(
            request.request_id,
            serde_json::json!({"result": "success"}),
        );
        
        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.status, ResponseStatus::Success);
        
        let error_response = create_error_response(
            request.request_id,
            "Test error message",
        );
        
        assert_eq!(error_response.request_id, request.request_id);
        assert_eq!(error_response.status, ResponseStatus::Error);
    }
} 