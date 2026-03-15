// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Core traits for ecosystem integration
//!
//! This module contains the standardized traits that all primals in the
//! ecoPrimals ecosystem must implement for seamless integration.

use crate::error::{EcosystemError, UniversalResult};
use crate::types::{
    DynamicPortInfo, EcosystemRequest, EcosystemResponse, EcosystemServiceRegistration,
    HealthStatus, PrimalCapability, PrimalContext, PrimalDependency, PrimalEndpoints, PrimalHealth,
    PrimalRequest, PrimalResponse, PrimalType, SecurityConfig, ServiceCapabilities,
    ServiceMeshStatus,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Universal primal provider trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait defines the standard interface for all primals in the ecosystem.
/// It provides the foundation for service discovery, health monitoring, and
/// inter-primal communication.
#[async_trait]
pub trait UniversalPrimalProvider: Send + Sync {
    /// Unique primal identifier (e.g., "squirrel", "beardog", "nestgate")
    fn primal_id(&self) -> &str;

    /// Instance identifier for multi-instance support
    fn instance_id(&self) -> &str;

    /// User/device context this primal instance serves
    fn context(&self) -> &PrimalContext;

    /// Primal type category
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
    async fn handle_primal_request(
        &self,
        request: PrimalRequest,
    ) -> UniversalResult<PrimalResponse>;

    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: serde_json::Value) -> UniversalResult<()>;

    /// Shutdown the primal gracefully
    async fn shutdown(&mut self) -> UniversalResult<()>;

    /// Check if this primal can serve the given context
    fn can_serve_context(&self, context: &PrimalContext) -> bool;

    /// Get dynamic port information (managed by service mesh)
    fn dynamic_port_info(&self) -> Option<DynamicPortInfo>;

    /// Register with service mesh
    async fn register_with_service_mesh(
        &mut self,
        service_mesh_endpoint: &str,
    ) -> UniversalResult<String>;

    /// Deregister from service mesh
    async fn deregister_from_service_mesh(&mut self) -> UniversalResult<()>;

    /// Get service mesh status
    fn get_service_mesh_status(&self) -> ServiceMeshStatus;

    /// Handle ecosystem request (standardized format)
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> UniversalResult<EcosystemResponse>;

    /// Update capabilities dynamically
    async fn update_capabilities(&self, capabilities: Vec<PrimalCapability>)
    -> UniversalResult<()>;

    /// Report health status
    async fn report_health(&self, health: PrimalHealth) -> UniversalResult<()>;
}

/// Ecosystem integration trait - ALL PRIMALS MUST IMPLEMENT
///
/// This trait handles communication with the broader ecosystem through
/// the service mesh. It provides standardized request/response
/// handling and service lifecycle management.
#[async_trait]
pub trait EcosystemIntegration: Send + Sync {
    /// Register service with service mesh
    async fn register_with_service_mesh(&self) -> Result<String, EcosystemError>;

    /// Handle incoming requests from other services
    async fn handle_ecosystem_request(
        &self,
        request: EcosystemRequest,
    ) -> Result<EcosystemResponse, EcosystemError>;

    /// Report health status to Songbird
    async fn report_health(&self, health: HealthStatus) -> Result<(), EcosystemError>;

    /// Update service capabilities
    async fn update_capabilities(
        &self,
        capabilities: ServiceCapabilities,
    ) -> Result<(), EcosystemError>;

    /// Deregister from ecosystem
    async fn deregister(&self) -> Result<(), EcosystemError>;
}

/// Service mesh client trait for interacting with Songbird
///
/// This trait provides the interface for communicating with the Songbird
/// service mesh for service discovery, registration, and health reporting.
#[async_trait]
pub trait ServiceMeshClient: Send + Sync {
    /// Register a service with the service mesh
    async fn register_service(
        &self,
        endpoint: &str,
        registration: EcosystemServiceRegistration,
    ) -> UniversalResult<String>;

    /// Deregister a service from the service mesh
    async fn deregister_service(&self, service_id: &str) -> UniversalResult<()>;

    /// Discover services in the service mesh
    async fn discover_services(&self, query: ServiceQuery) -> UniversalResult<Vec<ServiceInfo>>;

    /// Get service information by ID
    async fn get_service(&self, service_id: &str) -> UniversalResult<Option<ServiceInfo>>;

    /// Report health status
    async fn report_health(&self, service_id: &str, health: HealthStatus) -> UniversalResult<()>;

    /// Send heartbeat
    async fn heartbeat(&self, service_id: &str) -> UniversalResult<()>;

    /// Get service mesh status
    async fn get_mesh_status(&self) -> UniversalResult<ServiceMeshStatus>;
}

/// AI provider trait for Squirrel AI primal
///
/// This trait defines the interface for AI providers that can be registered
/// with the Squirrel AI coordinator for dynamic model access.
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Get AI capabilities
    async fn get_capabilities(&self) -> Vec<AICapability>;

    /// Health check for the provider
    async fn health_check(&self) -> ProviderHealth;

    /// Perform inference
    async fn inference(&self, request: InferenceRequest) -> Result<InferenceResponse, AIError>;

    /// Stream inference
    async fn stream_inference(&self, request: InferenceRequest)
    -> Result<InferenceStream, AIError>;

    /// Get provider name
    fn provider_name(&self) -> &str;

    /// Get provider type
    fn provider_type(&self) -> &str;
}

/// Service query for service discovery
#[derive(Debug, Clone, Default)]
pub struct ServiceQuery {
    /// Service type filter
    pub service_type: Option<String>,

    /// Primal type filter
    pub primal_type: Option<PrimalType>,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Health status filter
    pub health_status: Option<HealthStatus>,

    /// Metadata filters
    pub metadata: std::collections::HashMap<String, String>,
}

/// Service information from discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service ID
    pub id: String,

    /// Service name
    pub name: String,

    /// Service type
    pub service_type: String,

    /// Primal type
    pub primal_type: PrimalType,

    /// Service endpoint
    pub endpoint: String,

    /// Service capabilities
    pub capabilities: Vec<String>,

    /// Health status
    pub health_status: String,

    /// Service metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// AI capability enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AICapability {
    /// Text generation
    TextGeneration,
    /// Code generation
    CodeGeneration,
    /// Image generation
    ImageGeneration,
    /// Speech synthesis
    SpeechSynthesis,
    /// Language translation
    LanguageTranslation,
    /// Question answering
    QuestionAnswering,
    /// Summarization
    Summarization,
    /// Classification
    Classification,
    /// Sentiment analysis
    SentimentAnalysis,
    /// Multimodal processing
    MultiModal,
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Is the provider healthy?
    pub healthy: bool,

    /// Health status message
    pub message: String,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Error rate percentage
    pub error_rate: f64,

    /// Current load percentage
    pub load_percentage: f64,
}

impl ProviderHealth {
    /// Check if the provider is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }
}

/// AI inference request
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Request ID
    pub id: String,

    /// Input prompt or data
    pub input: String,

    /// Request parameters
    pub parameters: std::collections::HashMap<String, serde_json::Value>,

    /// Request context
    pub context: Option<String>,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,

    /// Temperature for generation
    pub temperature: Option<f32>,

    /// Top-p for generation
    pub top_p: Option<f32>,
}

/// AI inference response
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    /// Request ID
    pub request_id: String,

    /// Generated output
    pub output: String,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// Token usage information
    pub usage: TokenUsage,

    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,
}

/// AI inference stream
pub type InferenceStream =
    Box<dyn futures::Stream<Item = Result<InferenceChunk, AIError>> + Send + Unpin>;

/// AI inference chunk for streaming
#[derive(Debug, Clone)]
pub struct InferenceChunk {
    /// Request ID
    pub request_id: String,

    /// Chunk content
    pub content: String,

    /// Is this the final chunk?
    pub is_final: bool,

    /// Chunk metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// AI request for coordination
#[derive(Debug, Clone)]
pub struct AIRequest {
    /// Request ID
    pub id: String,

    /// Request prompt
    pub prompt: String,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Request context
    pub context: Option<String>,

    /// User preferences
    pub preferences: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// AI response from coordination
#[derive(Debug, Clone)]
pub struct AIResponse {
    /// Request ID
    pub request_id: String,

    /// Response content
    pub content: String,

    /// Provider that handled the request
    pub provider: String,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// AI error types
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    /// Provider is not available
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    /// Provider is unhealthy
    #[error("Provider unhealthy: {0}")]
    ProviderUnhealthy(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Primal factory trait for creating primal instances
pub trait PrimalFactory: Send + Sync {
    /// Create a new primal instance
    fn create_primal(
        &self,
        config: UniversalConfig,
    ) -> UniversalResult<Box<dyn UniversalPrimalProvider>>;
}

/// Configuration trait for universal configuration management
pub trait ConfigProvider: Send + Sync {
    /// Load configuration from environment
    fn load_from_environment(&self) -> UniversalResult<UniversalConfig>;

    /// Validate configuration
    fn validate(&self, config: &UniversalConfig) -> UniversalResult<()>;

    /// Get configuration value
    fn get_value(&self, key: &str) -> Option<String>;

    /// Set configuration value
    fn set_value(&self, key: &str, value: String) -> UniversalResult<()>;
}

/// Universal configuration structure
#[derive(Debug, Clone)]
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
    pub primal_specific: std::collections::HashMap<String, serde_json::Value>,
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    pub description: String,

    /// Bind address
    pub bind_address: String,

    /// Port number
    pub port: u16,

    /// Log level
    pub log_level: String,

    /// Instance ID
    pub instance_id: String,
}

/// Songbird configuration
#[derive(Debug, Clone)]
pub struct SongbirdConfig {
    /// Discovery endpoint
    pub discovery_endpoint: String,

    /// Registration endpoint
    pub registration_endpoint: String,

    /// Health endpoint
    pub health_endpoint: String,

    /// Authentication token
    pub auth_token: Option<String>,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
}

/// Retry configuration for resilient operations
///
/// Simple `RetryConfig` for ecosystem-api (standalone crate)
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay in milliseconds before first retry
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds between retries
    pub max_delay_ms: u64,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles each retry)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Resource configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// CPU cores
    pub cpu_cores: Option<f64>,

    /// Memory in MB
    pub memory_mb: Option<u64>,

    /// Disk space in MB
    pub disk_mb: Option<u64>,

    /// Network bandwidth in Mbps
    pub network_bandwidth_mbps: Option<u64>,

    /// GPU count
    pub gpu_count: Option<u32>,
}

/// Feature flags
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    /// Development mode
    pub development_mode: bool,

    /// Debug logging
    pub debug_logging: bool,

    /// Metrics enabled
    pub metrics_enabled: bool,

    /// Tracing enabled
    pub tracing_enabled: bool,

    /// Experimental features
    pub experimental_features: Vec<String>,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network port
    pub port: u16,

    /// Max connections
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Read timeout in seconds
    pub read_timeout_secs: u64,

    /// Write timeout in seconds
    pub write_timeout_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- RetryConfig tests ---
    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_config_custom() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 500,
            max_delay_ms: 60000,
            backoff_multiplier: 3.0,
        };
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 60000);
        assert!((config.backoff_multiplier - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.max_retries, config.max_retries);
        assert_eq!(cloned.initial_delay_ms, config.initial_delay_ms);
    }

    // --- ServiceQuery tests ---
    #[test]
    fn test_service_query_default() {
        let query = ServiceQuery::default();
        assert!(query.service_type.is_none());
        assert!(query.primal_type.is_none());
        assert!(query.capabilities.is_empty());
        assert!(query.health_status.is_none());
        assert!(query.metadata.is_empty());
    }

    #[test]
    fn test_service_query_with_filters() {
        let query = ServiceQuery {
            service_type: Some("ai".to_string()),
            primal_type: Some(PrimalType::Squirrel),
            capabilities: vec!["inference".to_string()],
            health_status: None,
            metadata: std::collections::HashMap::new(),
        };
        assert_eq!(query.service_type.as_deref(), Some("ai"));
        assert!(matches!(query.primal_type, Some(PrimalType::Squirrel)));
        assert_eq!(query.capabilities.len(), 1);
    }

    // --- ServiceInfo tests ---
    #[test]
    fn test_service_info_serde() {
        let info = ServiceInfo {
            id: "svc-1".to_string(),
            name: "test-service".to_string(),
            service_type: "ai".to_string(),
            primal_type: PrimalType::Squirrel,
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["inference".to_string()],
            health_status: "healthy".to_string(),
            metadata: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ServiceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "svc-1");
        assert_eq!(deserialized.name, "test-service");
        assert_eq!(deserialized.endpoint, "http://localhost:8080");
        assert_eq!(deserialized.capabilities.len(), 1);
    }

    #[test]
    fn test_service_info_clone() {
        let info = ServiceInfo {
            id: "svc-2".to_string(),
            name: "cloned-service".to_string(),
            service_type: "compute".to_string(),
            primal_type: PrimalType::ToadStool,
            endpoint: "http://localhost:9090".to_string(),
            capabilities: vec![],
            health_status: "degraded".to_string(),
            metadata: std::collections::HashMap::new(),
        };
        let cloned = info.clone();
        assert_eq!(cloned.id, info.id);
        assert_eq!(cloned.health_status, "degraded");
    }

    // --- AICapability tests ---
    #[test]
    fn test_ai_capability_variants() {
        let caps = vec![
            AICapability::TextGeneration,
            AICapability::CodeGeneration,
            AICapability::ImageGeneration,
            AICapability::SpeechSynthesis,
            AICapability::LanguageTranslation,
            AICapability::QuestionAnswering,
            AICapability::Summarization,
            AICapability::Classification,
            AICapability::SentimentAnalysis,
            AICapability::MultiModal,
        ];
        assert_eq!(caps.len(), 10);
    }

    #[test]
    fn test_ai_capability_eq() {
        assert_eq!(AICapability::TextGeneration, AICapability::TextGeneration);
        assert_ne!(AICapability::TextGeneration, AICapability::CodeGeneration);
    }

    #[test]
    fn test_ai_capability_clone() {
        let cap = AICapability::MultiModal;
        let cloned = cap.clone();
        assert_eq!(cap, cloned);
    }

    // --- ProviderHealth tests ---
    #[test]
    fn test_provider_health_healthy() {
        let health = ProviderHealth {
            healthy: true,
            message: "OK".to_string(),
            response_time_ms: 50,
            error_rate: 0.0,
            load_percentage: 25.0,
        };
        assert!(health.is_healthy());
        assert_eq!(health.response_time_ms, 50);
    }

    #[test]
    fn test_provider_health_unhealthy() {
        let health = ProviderHealth {
            healthy: false,
            message: "High error rate".to_string(),
            response_time_ms: 5000,
            error_rate: 50.0,
            load_percentage: 95.0,
        };
        assert!(!health.is_healthy());
        assert_eq!(health.message, "High error rate");
    }

    // --- AIError tests ---
    #[test]
    fn test_ai_error_display() {
        let err = AIError::ProviderUnavailable("test-provider".to_string());
        assert_eq!(err.to_string(), "Provider unavailable: test-provider");

        let err = AIError::RateLimitExceeded("too many requests".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: too many requests");
    }

    #[test]
    fn test_ai_error_variants() {
        let errors: Vec<AIError> = vec![
            AIError::ProviderUnavailable("a".to_string()),
            AIError::ProviderUnhealthy("b".to_string()),
            AIError::InvalidRequest("c".to_string()),
            AIError::RateLimitExceeded("d".to_string()),
            AIError::AuthenticationFailed("e".to_string()),
            AIError::NetworkError("f".to_string()),
            AIError::InternalError("g".to_string()),
        ];
        assert_eq!(errors.len(), 7);
        // All should display correctly
        for err in &errors {
            assert!(!err.to_string().is_empty());
        }
    }

    // --- TokenUsage tests ---
    #[test]
    fn test_token_usage() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
        };
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_clone() {
        let usage = TokenUsage {
            input_tokens: 200,
            output_tokens: 100,
            total_tokens: 300,
        };
        let cloned = usage.clone();
        assert_eq!(cloned.total_tokens, 300);
    }

    // --- InferenceRequest tests ---
    #[test]
    fn test_inference_request() {
        let req = InferenceRequest {
            id: "req-1".to_string(),
            input: "Hello world".to_string(),
            parameters: std::collections::HashMap::new(),
            context: Some("test context".to_string()),
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
        };
        assert_eq!(req.id, "req-1");
        assert_eq!(req.max_tokens, Some(100));
        assert!(req.context.is_some());
        assert!(req.top_p.is_none());
    }

    // --- InferenceResponse tests ---
    #[test]
    fn test_inference_response() {
        let resp = InferenceResponse {
            request_id: "req-1".to_string(),
            output: "Generated text".to_string(),
            metadata: std::collections::HashMap::new(),
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
                total_tokens: 30,
            },
            response_time_ms: 150,
        };
        assert_eq!(resp.request_id, "req-1");
        assert_eq!(resp.response_time_ms, 150);
        assert_eq!(resp.usage.total_tokens, 30);
    }

    // --- InferenceChunk tests ---
    #[test]
    fn test_inference_chunk() {
        let chunk = InferenceChunk {
            request_id: "req-1".to_string(),
            content: "partial output".to_string(),
            is_final: false,
            metadata: std::collections::HashMap::new(),
        };
        assert!(!chunk.is_final);
        assert_eq!(chunk.content, "partial output");
    }

    // --- AIRequest/AIResponse tests ---
    #[test]
    fn test_ai_request() {
        let req = AIRequest {
            id: "ai-req-1".to_string(),
            prompt: "What is Rust?".to_string(),
            capabilities: vec!["text_generation".to_string()],
            context: None,
            preferences: None,
        };
        assert_eq!(req.capabilities.len(), 1);
        assert!(req.context.is_none());
    }

    #[test]
    fn test_ai_response() {
        let resp = AIResponse {
            request_id: "ai-req-1".to_string(),
            content: "Rust is a systems programming language".to_string(),
            provider: "test-provider".to_string(),
            metadata: std::collections::HashMap::new(),
            response_time_ms: 200,
        };
        assert_eq!(resp.provider, "test-provider");
        assert_eq!(resp.response_time_ms, 200);
    }

    // --- ServiceConfig tests ---
    #[test]
    fn test_service_config() {
        let config = ServiceConfig {
            name: "squirrel".to_string(),
            version: "1.0.0".to_string(),
            description: "AI Primal".to_string(),
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
            log_level: "info".to_string(),
            instance_id: "inst-1".to_string(),
        };
        assert_eq!(config.name, "squirrel");
        assert_eq!(config.port, 8080);
    }

    // --- FeatureFlags tests ---
    #[test]
    fn test_feature_flags() {
        let flags = FeatureFlags {
            development_mode: true,
            debug_logging: false,
            metrics_enabled: true,
            tracing_enabled: true,
            experimental_features: vec!["feature_a".to_string()],
        };
        assert!(flags.development_mode);
        assert!(!flags.debug_logging);
        assert_eq!(flags.experimental_features.len(), 1);
    }

    // --- ResourceConfig tests ---
    #[test]
    fn test_resource_config() {
        let config = ResourceConfig {
            cpu_cores: Some(4.0),
            memory_mb: Some(8192),
            disk_mb: Some(100_000),
            network_bandwidth_mbps: Some(1000),
            gpu_count: Some(1),
        };
        assert_eq!(config.cpu_cores, Some(4.0));
        assert_eq!(config.gpu_count, Some(1));
    }

    #[test]
    fn test_resource_config_empty() {
        let config = ResourceConfig {
            cpu_cores: None,
            memory_mb: None,
            disk_mb: None,
            network_bandwidth_mbps: None,
            gpu_count: None,
        };
        assert!(config.cpu_cores.is_none());
        assert!(config.gpu_count.is_none());
    }

    // --- NetworkConfig tests ---
    #[test]
    fn test_network_config() {
        let config = NetworkConfig {
            port: 443,
            max_connections: 1000,
            connection_timeout_secs: 30,
            read_timeout_secs: 60,
            write_timeout_secs: 60,
        };
        assert_eq!(config.port, 443);
        assert_eq!(config.max_connections, 1000);
    }
}
