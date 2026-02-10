// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Universal AI Coordinator Types
//!
//! This module contains all the data structures, enums, and configuration types
//! used by the Universal AI Coordinator system.

use std::collections::HashMap;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};

use crate::error::types::Result;
use crate::enhanced::providers::ProviderType;

/// Routing strategies for universal AI coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Best fit based on capabilities
    BestFit,
    
    /// Lowest cost
    LowestCost,
    
    /// Cost optimized
    CostOptimized,
    
    /// Fastest response
    LowestLatency,
    
    /// Latency optimized
    LatencyOptimized,
    
    /// Highest quality
    HighestQuality,
    
    /// Local first (privacy)
    LocalFirst,
    
    /// Cloud first (performance)
    CloudFirst,
    
    /// Round robin
    RoundRobin,
    
    /// Weighted random
    WeightedRandom,
    
    /// Custom strategy
    Custom(String),
}

/// Routing rules for intelligent AI selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Rule priority (higher = more important)
    pub priority: u32,
    
    /// Condition to match
    pub condition: RuleCondition,
    
    /// Action to take
    pub action: RuleAction,
    
    /// Rule name/description
    pub name: String,
}

/// Rule conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Request contains sensitive data
    SensitiveData,
    
    /// Request requires high quality
    HighQuality,
    
    /// Request requires low latency
    LowLatency,
    
    /// Request requires low cost
    LowCost,
    
    /// Model type required
    ModelType(String),
    
    /// Task type required
    TaskType(String),
    
    /// Custom condition
    Custom(serde_json::Value),
}

/// Rule actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Prefer specific provider
    PreferProvider(String),
    
    /// Require local processing
    RequireLocal,
    
    /// Allow cloud processing
    AllowCloud,
    
    /// Set maximum cost
    MaxCost(f64),
    
    /// Set maximum latency
    MaxLatency(Duration),
    
    /// Custom action
    Custom(serde_json::Value),
}

/// Tool execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    /// Execution ID
    pub id: String,
    
    /// Tool name
    pub tool_name: String,
    
    /// AI model used
    pub ai_model: String,
    
    /// Parameters
    pub parameters: serde_json::Value,
    
    /// Result
    pub result: ToolResult,
    
    /// Execution time
    pub duration: Duration,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success/failure
    pub success: bool,
    
    /// Result data
    pub data: serde_json::Value,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// AI analysis of the result
    pub ai_analysis: Option<String>,
}

/// AI session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISession {
    /// Session ID
    pub id: String,
    
    /// Active models
    pub active_models: Vec<String>,
    
    /// Conversation history
    pub history: Vec<AIInteraction>,
    
    /// User preferences
    pub preferences: UserPreferences,
    
    /// Session metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// AI interaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIInteraction {
    /// Interaction ID
    pub id: String,
    
    /// Request
    pub request: UniversalAIRequest,
    
    /// Response
    pub response: UniversalAIResponse,
    
    /// Model used
    pub model_used: String,
    
    /// Duration
    pub duration: Duration,
    
    /// Cost
    pub cost: Option<f64>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// AI coordinator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICoordinatorConfig {
    // Cloud API keys
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    
    // Local model configuration (capability-based, vendor-agnostic)
    pub enable_local_server: bool,
    pub enable_native: bool,
    pub enable_model_hub: bool,
    
    pub local_server_config: LocalServerConfig,
    pub native_config: NativeConfig,
    pub model_hub_config: ModelHubConfig,
    
    // Custom providers
    pub custom_providers: HashMap<String, serde_json::Value>,
    
    // Routing configuration
    pub routing: RoutingConfig,
    
    // Performance settings
    pub max_concurrent_requests: usize,
    pub request_timeout: Duration,
    pub retry_attempts: u32,
}

/// Routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub default_strategy: RoutingStrategy,
    pub fallback_enabled: bool,
    pub cost_optimization: bool,
    pub latency_optimization: bool,
    pub quality_optimization: bool,
}

/// Local AI server configuration (vendor-agnostic)
///
/// Works with any OpenAI-compatible API server:
/// Ollama, llama.cpp, vLLM, LocalAI, text-generation-webui, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalServerConfig {
    /// Base URL of the local server (e.g., http://localhost:11434)
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Available models on this server
    pub models: Vec<String>,
    /// Optional: path to models directory (for servers that need it)
    pub models_path: Option<String>,
}

/// Backward-compatible type aliases for vendor-specific configs
pub type OllamaConfig = LocalServerConfig;
pub type LlamaCppConfig = LocalServerConfig;

/// Native model configuration (direct model loading without server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeConfig {
    pub models_directory: String,
    pub max_loaded_models: usize,
    pub use_gpu: bool,
}

/// Model hub configuration (vendor-agnostic)
///
/// Works with any model hub: HuggingFace, ModelScope, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHubConfig {
    pub api_token: Option<String>,
    pub cache_directory: String,
    pub use_local_cache: bool,
}

/// Backward-compatible type alias
pub type HuggingFaceConfig = ModelHubConfig;

/// Custom provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    pub name: String,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_multimodal: bool,
    pub max_tokens: Option<usize>,
    pub supported_models: Vec<String>,
    pub provider_type: String,
}

/// Tool executor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutorConfig {
    pub max_concurrent_executions: usize,
    pub execution_timeout: Duration,
    pub retry_failed_executions: bool,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider: String,
    pub model_type: ProviderType,
    pub capabilities: Vec<String>,
    pub performance: Option<PerformanceMetrics>,
}

/// Model capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_tokens: Option<usize>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub cost_per_token: Option<f64>,
}

/// Request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Routing hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHints {
    pub prefer_local: bool,
    pub max_cost: Option<f64>,
    pub max_latency: Option<Duration>,
    pub quality_requirements: Vec<String>,
}

/// Quality requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub min_quality_score: Option<f64>,
    pub require_streaming: bool,
    pub require_tools: bool,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_providers: Vec<String>,
    pub privacy_level: PrivacyLevel,
    pub cost_sensitivity: CostSensitivity,
    pub quality_preference: QualityPreference,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub theme: Option<String>,
}

/// Privacy level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,
    Private,
    Sensitive,
    Classified,
}

/// Cost sensitivity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostSensitivity {
    Free,
    Low,
    Medium,
    High,
    Unlimited,
}

/// Quality preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityPreference {
    Fast,
    Balanced,
    HighQuality,
    BestAvailable,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub healthy: bool,
    pub latency: Option<Duration>,
    pub error_rate: Option<f64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Universal AI request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIRequest {
    /// Request ID
    pub id: String,
    
    /// Target model or system
    pub model: String,
    
    /// Request type
    pub request_type: AIRequestType,
    
    /// Request payload (flexible JSON)
    pub payload: serde_json::Value,
    
    /// Messages for chat-style requests
    pub messages: Vec<MessageContent>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    
    /// Context and metadata
    pub context: RequestContext,
    
    /// Routing hints
    pub hints: RoutingHints,
    
    /// Quality requirements
    pub requirements: QualityRequirements,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    /// Message role (user, assistant, system)
    pub role: String,
    
    /// Message content
    pub content: String,
}

/// AI request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIRequestType {
    /// Text generation/chat
    TextGeneration,
    
    /// Image generation
    ImageGeneration,
    
    /// Image understanding
    ImageAnalysis,
    
    /// Audio generation
    AudioGeneration,
    
    /// Audio transcription
    AudioTranscription,
    
    /// Video generation
    VideoGeneration,
    
    /// Video analysis
    VideoAnalysis,
    
    /// Embeddings
    Embeddings,
    
    /// Fine-tuning
    FineTuning,
    
    /// Model evaluation
    Evaluation,
    
    /// Custom function
    Custom(String),
    
    /// Future AI capabilities
    Future(String),
}

/// Universal AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAIResponse {
    /// Response ID
    pub id: String,
    
    /// Provider name
    pub provider: String,
    
    /// Model used
    pub model: String,
    
    /// Response type
    pub response_type: AIRequestType,
    
    /// Response content
    pub content: String,
    
    /// Cost
    pub cost: f64,
    
    /// Duration
    pub duration: Duration,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Universal AI stream type
pub type UniversalAIStream = Box<dyn futures::Stream<Item = Result<UniversalAIResponse>> + Send + Unpin>;

/// Cost estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Estimated cost
    pub estimated_cost: f64,
    
    /// Currency
    pub currency: String,
    
    /// Cost breakdown
    pub breakdown: HashMap<String, f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_latency: Duration,
    pub success_rate: f64,
    pub cost_per_request: Option<f64>,
    pub quality_score: Option<f64>,
}

/// AI requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequirements {
    pub min_model_size: Option<String>,
    pub required_capabilities: Vec<String>,
    pub max_cost: Option<f64>,
    pub max_latency: Option<Duration>,
}

/// AI metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_cost: f64,
    #[serde(with = "duration_serde")]
    pub avg_latency: Duration,
}

/// Duration serialization helper
mod duration_serde {
    use super::*;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl AIMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_cost: 0.0,
            avg_latency: Duration::from_millis(0),
        }
    }
}

/// Model capability registry
pub struct ModelCapabilityRegistry {
    // Implementation would store model capabilities
}

impl ModelCapabilityRegistry {
    pub fn new() -> Self {
        Self {}
    }
} 