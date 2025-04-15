//! AI Agent adapter type definitions
//!
//! This module provides type definitions for the AI Agent adapter.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use async_trait::async_trait;
use squirrel_ai_tools::common::{ChatRequest, ChatResponse, ChatResponseStream};

/// Resource limits for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    
    /// Maximum number of requests per minute
    pub max_requests_per_minute: u32,
    
    /// Maximum total tokens per minute
    pub max_tokens_per_minute: u32,
    
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    
    /// Maximum time per request in milliseconds
    pub max_request_time_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            max_requests_per_minute: 60,
            max_tokens_per_minute: 100000,
            max_concurrent_requests: 10,
            max_request_time_ms: 60000, // 1 minute
        }
    }
}

/// Capabilities granted to the AI agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    /// Permission to read files
    ReadFiles,
    
    /// Permission to write files
    WriteFiles,
    
    /// Permission to execute commands
    ExecuteCommands,
    
    /// Permission to access network
    AccessNetwork,
    
    /// Permission to access system information
    AccessSystemInfo,
    
    /// Permission to access user information
    AccessUserInfo,
}

/// Capabilities of an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// List of permissions granted to the agent
    pub permissions: Vec<Permission>,
    
    /// Access token for external services
    pub access_token: Option<String>,
    
    /// Resource limitations
    pub resource_limits: ResourceLimits,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            access_token: None,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Content format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentFormat {
    /// Plain text
    PlainText,
    
    /// Markdown
    Markdown,
    
    /// JSON
    Json,
    
    /// HTML
    Html,
    
    /// XML
    Xml,
    
    /// Binary
    Binary,
}

impl Default for ContentFormat {
    fn default() -> Self {
        Self::PlainText
    }
}

/// Content for AI processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    /// Content data
    pub data: String,
    
    /// Content format
    pub format: ContentFormat,
    
    /// Content metadata
    pub metadata: HashMap<String, Value>,
}

/// Context for AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Context ID
    pub id: Uuid,
    
    /// Working directory for file operations
    pub working_directory: Option<String>,
    
    /// User information
    pub user: Option<String>,
    
    /// Environment variables
    pub environment: HashMap<String, String>,
    
    /// Files available to the agent
    pub files: Vec<String>,
    
    /// Metadata for the context
    pub metadata: HashMap<String, String>,
    
    /// Agent capabilities
    pub capabilities: AgentCapabilities,
    
    /// Account ID
    pub account_id: String,
    
    /// Session ID
    pub session_id: String,
}

/// Prompt for AI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// System message
    pub system: Option<String>,
    
    /// User message
    pub user: String,
    
    /// Chat history
    pub history: Vec<String>,
}

/// Generation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    /// The model to use
    pub model: Option<String>,
    
    /// Temperature (0.0 - 1.0)
    pub temperature: Option<f32>,
    
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    
    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
    
    /// Top P
    pub top_p: Option<f32>,
    
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    
    /// Response format
    pub response_format: Option<ContentFormat>,
    
    /// Additional model parameters
    pub additional_parameters: HashMap<String, Value>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(1024),
            stop_sequences: None,
            top_p: Some(1.0),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
            response_format: Some(ContentFormat::PlainText),
            additional_parameters: HashMap::new(),
        }
    }
}

/// Analysis options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    /// The model to use
    pub model: Option<String>,
    
    /// Temperature (0.0 - 1.0)
    pub temperature: Option<f32>,
    
    /// Analysis type
    pub analysis_type: String,
    
    /// Target schema for structured output
    pub target_schema: Option<Value>,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            model: None,
            temperature: Some(0.2),
            analysis_type: "general".to_string(),
            target_schema: None,
        }
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Prompt tokens
    pub prompt_tokens: Option<u32>,
    
    /// Completion tokens
    pub completion_tokens: Option<u32>,
    
    /// Total tokens
    pub total_tokens: Option<u32>,
}

/// Usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Token usage
    pub tokens: UsageStatistics,
    
    /// API requests made
    pub requests: u32,
    
    /// Billable duration
    pub billable_duration_ms: Option<u64>,
}

/// Status of the AI Agent adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStatus {
    /// Is the adapter initialized
    pub initialized: bool,
    
    /// Is the adapter operational
    pub operational: bool,

/// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,
    
    /// Provider status
    pub provider_status: String,
    
    /// Resource usage (0.0 - 1.0)
    pub resource_usage: f64,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit breaker is closed (allowing requests)
    Closed,
    
    /// Circuit breaker is open (blocking requests)
    Open,
    
    /// Circuit breaker is half-open (testing if service is back)
    HalfOpen,
}

impl std::fmt::Display for CircuitBreakerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerState::Closed => write!(f, "Closed"),
            CircuitBreakerState::Open => write!(f, "Open"),
            CircuitBreakerState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold (0.0 - 1.0)
    pub failure_threshold: f64,
    
    /// Reset timeout in milliseconds
    pub reset_timeout: u64,
    
    /// Maximum number of calls in half-open state
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 0.5,
            reset_timeout: 30000, // 30 seconds
            half_open_max_calls: 5,
        }
    }
}

/// Operation type for agent requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    /// Generate text
    Generate,
    
    /// Execute a task
    Execute,
    
    /// Analyze content
    Analyze,
    
    /// Edit content
    Edit,
    
    /// Summarize content
    Summarize,
}

impl Default for OperationType {
    fn default() -> Self {
        Self::Generate
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Generate => write!(f, "generate"),
            OperationType::Execute => write!(f, "execute"),
            OperationType::Analyze => write!(f, "analyze"),
            OperationType::Edit => write!(f, "edit"),
            OperationType::Summarize => write!(f, "summarize"),
        }
    }
}

/// Request to the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    /// Request ID
    pub id: Uuid,
    
    /// Prompt for the AI
    pub prompt: String,
    
    /// System message
    pub system_message: Option<String>,
    
    /// Generation options
    pub options: GenerationOptions,
    
    /// Agent context
    pub context: Option<AgentContext>,
    
    /// Operation type
    pub operation_type: OperationType,
    
    /// Content to process
    pub content: Option<Content>,
}

/// Response from the AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// Response ID
    pub id: Uuid,
    
    /// Response text
    pub text: String,
    
    /// Request ID
    pub request_id: Uuid,
    
    /// Completion time in milliseconds
    pub completion_time: u64,
    
    /// Response format
    pub format: ContentFormat,
    
    /// Usage information
    pub usage: Usage,
    
    /// Additional metadata
    pub metadata: HashMap<String, Value>,
}

/// AIClientV2 trait with improved thread safety
///
/// This version provides explicit Send + Sync bounds and uses callbacks
/// instead of direct adapter references to avoid potential Send/Sync issues.
#[async_trait]
pub trait AIClientV2: Send + Sync + std::fmt::Debug + 'static {
    /// Get the provider name
    fn provider_name(&self) -> &str;
    
    /// Get the default model name
    fn default_model(&self) -> &str;
    
    /// Get available models
    async fn list_models(&self) -> anyhow::Result<Vec<String>>;
    
    /// Send a chat request and get a chat response
    async fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;
    
    /// Send a chat request and get a streaming response
    async fn chat_stream(&self, request: ChatRequest) -> anyhow::Result<ChatResponseStream>;
    
    /// Register callbacks for adapter interaction
    fn register_callbacks(&mut self, callbacks: AIClientCallbacks) {
        // Default empty implementation
        let _ = callbacks; // Suppress unused variable warning
    }
}

/// Callbacks for AIClientV2
pub struct AIClientCallbacks {
    /// Callback for accessing MCP service
    pub mcp_service: Option<Box<dyn Fn(&str) -> anyhow::Result<String> + Send + Sync>>,
    
    /// Callback for logging
    pub log_event: Option<Box<dyn Fn(&str, &str) -> anyhow::Result<()> + Send + Sync>>,
    
    /// Callback for telemetry/usage tracking
    pub track_usage: Option<Box<dyn Fn(i32, i32, i32) -> anyhow::Result<()> + Send + Sync>>,
    
    /// Callback for rate limiting
    pub check_rate_limit: Option<Box<dyn Fn() -> anyhow::Result<bool> + Send + Sync>>,
}

impl Default for AIClientCallbacks {
    fn default() -> Self {
        Self {
            mcp_service: None,
            log_event: None,
            track_usage: None,
            check_rate_limit: None,
        }
    }
}

// Helper struct to adapt AIClientV2 to AIClient for backward compatibility
#[derive(Debug)]
pub struct AIClientWrapper<T: AIClientV2> {
    inner: T,
}

impl<T: AIClientV2> AIClientWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T: AIClientV2 + 'static> squirrel_ai_tools::common::AIClient for AIClientWrapper<T> {
    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }
    
    fn default_model(&self) -> &str {
        self.inner.default_model()
    }
    
    async fn list_models(&self) -> std::result::Result<Vec<String>, squirrel_ai_tools::Error> {
        self.inner.list_models().await.map_err(|e| squirrel_ai_tools::Error::Service(e.to_string()))
    }
    
    async fn chat(&self, request: ChatRequest) -> std::result::Result<ChatResponse, squirrel_ai_tools::Error> {
        self.inner.chat(request).await.map_err(|e| squirrel_ai_tools::Error::Service(e.to_string()))
    }
    
    async fn chat_stream(&self, request: ChatRequest) -> std::result::Result<ChatResponseStream, squirrel_ai_tools::Error> {
        self.inner.chat_stream(request).await.map_err(|e| squirrel_ai_tools::Error::Service(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_state_display() {
        assert_eq!(CircuitBreakerState::Open.to_string(), "Open");
        assert_eq!(CircuitBreakerState::HalfOpen.to_string(), "HalfOpen");
        assert_eq!(CircuitBreakerState::Closed.to_string(), "Closed");
    }
    
    #[test]
    fn test_operation_type_display() {
        assert_eq!(OperationType::Generate.to_string(), "generate");
        assert_eq!(OperationType::Execute.to_string(), "execute");
        assert_eq!(OperationType::Analyze.to_string(), "analyze");
        assert_eq!(OperationType::Edit.to_string(), "edit");
        assert_eq!(OperationType::Summarize.to_string(), "summarize");
    }
    
    #[test]
    fn test_operation_type_default() {
        assert_eq!(OperationType::default(), OperationType::Generate);
    }
} 