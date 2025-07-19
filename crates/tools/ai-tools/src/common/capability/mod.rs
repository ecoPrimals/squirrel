//! AI Capability Management
//!
//! This module provides types and functions for managing AI provider capabilities,
//! which are used for intelligent routing of AI requests to suitable providers.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod registry;

/// Security level for AI operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

// Re-exports
pub use registry::ModelRegistry;

/// Types of AI models
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    /// Large language model for text generation
    LargeLanguageModel,
    /// Chat model (alias for LargeLanguageModel)
    ChatModel,
    /// Embedding model for generating vectors
    Embedding,
    /// Image generation model
    ImageGeneration,
    /// Image understanding model
    ImageUnderstanding,
    /// Audio transcription model
    AudioTranscription,
    /// Audio generation model
    AudioGeneration,
    /// Multi-modal model handling multiple input/output types
    MultiModal,
    /// Custom model type
    Custom(String),
}

/// Types of AI tasks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskType {
    /// Text generation (chat, completion)
    TextGeneration,
    /// Code generation and programming assistance
    CodeGeneration,
    /// Translation between languages
    Translation,
    /// Summarization of text
    Summarization,
    /// Question answering
    QuestionAnswering,
    /// Chat completion
    ChatCompletion,
    /// Function calling
    FunctionCalling,
    /// Image generation
    ImageGeneration,
    /// Image understanding or analysis
    ImageUnderstanding,
    /// Image analysis
    ImageAnalysis,
    /// Audio transcription
    AudioTranscription,
    /// Audio generation (text to speech)
    AudioGeneration,
    /// Speech synthesis
    SpeechSynthesis,
    /// Text embedding
    TextEmbedding,
    /// Embedding generation
    Embedding,
    /// Classification tasks
    Classification,
    /// Sentiment analysis
    Sentiment,
    /// Named entity recognition
    NamedEntityRecognition,
    /// Data analysis
    DataAnalysis,
    /// Function execution
    FunctionExecution,
    /// Other task types
    Other,
    /// Custom task type
    Custom(String),
}

/// Performance metrics for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    /// Average latency in milliseconds
    pub avg_latency_ms: Option<u64>,
    /// Requests per second capacity
    pub requests_per_second: Option<f64>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: Option<f64>,
    /// Average tokens per second for generation
    pub avg_tokens_per_second: Option<f64>,
    /// Maximum throughput in requests per second
    pub max_throughput_rps: Option<f64>,
    /// Maximum batch size supported
    pub max_batch_size: Option<usize>,
    /// Quality score (0-100)
    pub quality_score: Option<u8>,
}

/// Resource requirements for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceRequirements {
    /// Minimum memory required in MB
    pub min_memory_mb: u64,
    /// Minimum GPU memory required in MB
    pub min_gpu_memory_mb: Option<u64>,
    /// Minimum CPU cores required
    pub min_cpu_cores: Option<u32>,
    /// Whether GPU is required
    pub requires_gpu: bool,
    /// Whether internet connection is required
    pub requires_internet: bool,
    /// Model load time in milliseconds
    pub load_time_ms: Option<u64>,
    /// Whether specific hardware is required
    pub requires_specific_hardware: bool,
    /// Hardware requirements description
    pub hardware_requirements: Option<String>,
}

/// Cost tier for AI providers
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum CostTier {
    /// Free to use
    Free,
    /// Low cost
    Low,
    /// Medium cost
    #[default]
    Medium,
    /// High cost
    High,
}

/// Cost metrics for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostMetrics {
    /// Cost per 1000 input tokens in USD
    pub cost_per_1k_input_tokens: Option<f64>,
    /// Cost per 1000 output tokens in USD
    pub cost_per_1k_output_tokens: Option<f64>,
    /// Fixed cost per request in USD
    pub cost_per_request: Option<f64>,
    /// Whether there's a fixed cost component
    pub has_fixed_cost: bool,
    /// Whether the service is free
    pub is_free: bool,
}

/// Security requirements for AI requests
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Whether the request requires encryption
    pub requires_encryption: bool,
    /// Whether the request contains sensitive data
    pub contains_sensitive_data: bool,
    /// Required security level
    pub security_level: SecurityLevel,
    /// Whether the request requires audit logging
    pub requires_audit_logging: bool,
    /// Geographic restrictions
    pub geo_restrictions: Option<GeoConstraints>,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            requires_encryption: false,
            contains_sensitive_data: false,
            security_level: SecurityLevel::Medium,
            requires_audit_logging: false,
            geo_restrictions: None,
        }
    }
}

/// Geographic constraints for AI providers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoConstraints {
    /// Allowed regions
    pub allowed_regions: Option<Vec<String>>,
    /// Blocked regions
    pub blocked_regions: Option<Vec<String>>,
    /// Data residency requirements
    pub data_residency: Option<String>,
}

/// AI task description for routing decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AITask {
    /// Type of task to perform
    pub task_type: TaskType,
    /// Required model type (if specific)
    pub required_model_type: Option<ModelType>,
    /// Minimum context size required
    pub min_context_size: Option<usize>,
    /// Whether streaming is required
    pub requires_streaming: bool,
    /// Whether function calling is required
    pub requires_function_calling: bool,
    /// Whether tool use is required
    pub requires_tool_use: bool,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Task complexity score (0-100)
    pub complexity_score: Option<u8>,
    /// Task priority (0-100)
    pub priority: u8,
}

impl Default for AITask {
    fn default() -> Self {
        Self {
            task_type: TaskType::TextGeneration,
            required_model_type: None,
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: None,
            priority: 50,
        }
    }
}

/// Capabilities of an AI provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AICapabilities {
    /// Supported model types
    pub supported_model_types: HashSet<ModelType>,
    /// Supported task types
    pub supported_task_types: HashSet<TaskType>,
    /// Maximum context size in tokens
    pub max_context_size: usize,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether function calling is supported
    pub supports_function_calling: bool,
    /// Whether tool use is supported
    pub supports_tool_use: bool,
    /// Whether images are supported
    pub supports_images: bool,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Cost metrics
    pub cost_metrics: CostMetrics,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Routing preferences
    pub routing_preferences: RoutingPreferences,
}

impl AICapabilities {
    /// Create a new AICapabilities instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a supported model type
    pub fn add_model_type(&mut self, model_type: ModelType) {
        self.supported_model_types.insert(model_type);
    }

    /// Add a supported task type
    pub fn add_task_type(&mut self, task_type: TaskType) {
        self.supported_task_types.insert(task_type);
    }

    /// Set maximum context size
    pub fn with_max_context_size(&mut self, size: usize) -> &mut Self {
        self.max_context_size = size;
        self
    }

    /// Set streaming support
    pub fn with_streaming(&mut self, supports: bool) -> &mut Self {
        self.supports_streaming = supports;
        self
    }

    /// Set function calling support
    pub fn with_function_calling(&mut self, supports: bool) -> &mut Self {
        self.supports_function_calling = supports;
        self
    }

    /// Set tool use support
    pub fn with_tool_use(&mut self, supports: bool) -> &mut Self {
        self.supports_tool_use = supports;
        self
    }

    /// Set image support
    pub fn with_image_support(&mut self, supports: bool) -> &mut Self {
        self.supports_images = supports;
        self
    }

    /// Set cost metrics
    pub fn with_cost_metrics(&mut self, metrics: CostMetrics) -> &mut Self {
        self.cost_metrics = metrics;
        self
    }

    /// Set performance metrics
    pub fn with_performance_metrics(&mut self, metrics: PerformanceMetrics) -> &mut Self {
        self.performance_metrics = metrics;
        self
    }

    /// Set supported tasks
    pub fn with_supported_tasks(&mut self, tasks: Vec<TaskType>) -> &mut Self {
        self.supported_task_types = tasks.into_iter().collect();
        self
    }

    /// Set model type
    pub fn with_model_type(&mut self, model_type: ModelType) -> &mut Self {
        self.supported_model_types.insert(model_type);
        self
    }

    /// Set security features
    pub fn with_security_features(&mut self, features: SecurityRequirements) -> &mut Self {
        self.security_requirements = features;
        self
    }

    /// Set routing preferences
    pub fn with_routing_preferences(&mut self, preferences: RoutingPreferences) -> &mut Self {
        self.routing_preferences = preferences;
        self
    }

    /// Check if this provider supports a specific task
    pub fn supports_task(&self, task_type: &TaskType) -> bool {
        self.supported_task_types.contains(task_type)
    }

    /// Check if this provider supports a specific model type
    pub fn supports_model_type(&self, model_type: &ModelType) -> bool {
        self.supported_model_types.contains(model_type)
    }
}

/// Routing preferences for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPreferences {
    /// Priority level (0-100, higher is more preferred)
    pub priority: u8,
    /// Whether this provider allows forwarding requests to other providers
    pub allows_forwarding: bool,
    /// Whether this provider can handle sensitive data
    pub handles_sensitive_data: bool,
    /// Geographic constraints
    pub geo_constraints: Option<GeoConstraints>,
    /// Cost tier
    pub cost_tier: CostTier,
    /// Whether this provider prefers local processing
    pub prefers_local: bool,
    /// Cost sensitivity (0.0-1.0)
    pub cost_sensitivity: f64,
    /// Performance priority (0.0-1.0)
    pub performance_priority: f64,
}

impl Default for RoutingPreferences {
    fn default() -> Self {
        Self {
            priority: 50,
            allows_forwarding: true,
            handles_sensitive_data: false,
            geo_constraints: None,
            cost_tier: CostTier::Medium,
            prefers_local: false,
            cost_sensitivity: 0.5,
            performance_priority: 0.5,
        }
    }
}
