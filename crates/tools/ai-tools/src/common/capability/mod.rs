// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! AI Capability Management
//!
//! This module provides types and functions for managing AI provider capabilities,
//! which are used for intelligent routing of AI requests to suitable providers.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Model registry for capability discovery and routing.
pub mod registry;

/// Security level for AI operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Low security - no special handling required.
    Low,
    /// Medium security - standard data handling.
    Medium,
    /// High security - sensitive data, requires encryption.
    High,
    /// Critical security - maximum protection, audit logging required.
    Critical,
}

// Re-exports
pub use registry::ModelRegistry;

/// Types of AI models
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    /// Large language model for text generation
    LargeLanguageModel,
    /// Chat model (alias for `LargeLanguageModel`)
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeoConstraints {
    /// Allowed regions
    pub allowed_regions: Option<Vec<String>>,
    /// Blocked regions
    pub blocked_regions: Option<Vec<String>>,
    /// Data residency requirements
    pub data_residency: Option<String>,
}

/// AI task description for routing decisions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[expect(
    clippy::struct_excessive_bools,
    reason = "Provider capability flags; flat fields for ergonomics and serde"
)]
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
    /// Create a new `AICapabilities` instance
    #[must_use]
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
    pub const fn with_max_context_size(&mut self, size: usize) -> &mut Self {
        self.max_context_size = size;
        self
    }

    /// Set streaming support
    pub const fn with_streaming(&mut self, supports: bool) -> &mut Self {
        self.supports_streaming = supports;
        self
    }

    /// Set function calling support
    pub const fn with_function_calling(&mut self, supports: bool) -> &mut Self {
        self.supports_function_calling = supports;
        self
    }

    /// Set tool use support
    pub const fn with_tool_use(&mut self, supports: bool) -> &mut Self {
        self.supports_tool_use = supports;
        self
    }

    /// Set image support
    pub const fn with_image_support(&mut self, supports: bool) -> &mut Self {
        self.supports_images = supports;
        self
    }

    /// Set cost metrics
    pub const fn with_cost_metrics(&mut self, metrics: CostMetrics) -> &mut Self {
        self.cost_metrics = metrics;
        self
    }

    /// Set performance metrics
    pub const fn with_performance_metrics(&mut self, metrics: PerformanceMetrics) -> &mut Self {
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
    #[must_use]
    pub fn supports_task(&self, task_type: &TaskType) -> bool {
        self.supported_task_types.contains(task_type)
    }

    /// Check if this provider supports a specific model type
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- SecurityLevel tests ---
    #[test]
    fn test_security_level_serde() {
        let levels = vec![
            SecurityLevel::Low,
            SecurityLevel::Medium,
            SecurityLevel::High,
            SecurityLevel::Critical,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).expect("should succeed");
            let deserialized: SecurityLevel = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, level);
        }
    }

    #[test]
    fn test_security_level_hash() {
        let mut set = HashSet::new();
        set.insert(SecurityLevel::Low);
        set.insert(SecurityLevel::High);
        set.insert(SecurityLevel::Low); // duplicate
        assert_eq!(set.len(), 2);
    }

    // --- ModelType tests ---
    #[test]
    fn test_model_type_serde() {
        let types = vec![
            ModelType::LargeLanguageModel,
            ModelType::ChatModel,
            ModelType::Embedding,
            ModelType::ImageGeneration,
            ModelType::ImageUnderstanding,
            ModelType::AudioTranscription,
            ModelType::AudioGeneration,
            ModelType::MultiModal,
            ModelType::Custom("my-model".to_string()),
        ];
        for mt in types {
            let json = serde_json::to_string(&mt).expect("should succeed");
            let deserialized: ModelType = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, mt);
        }
    }

    // --- TaskType tests ---
    #[test]
    fn test_task_type_serde() {
        let types = vec![
            TaskType::TextGeneration,
            TaskType::CodeGeneration,
            TaskType::Translation,
            TaskType::Summarization,
            TaskType::QuestionAnswering,
            TaskType::ChatCompletion,
            TaskType::FunctionCalling,
            TaskType::ImageGeneration,
            TaskType::Other,
            TaskType::Custom("my-task".to_string()),
        ];
        for tt in types {
            let json = serde_json::to_string(&tt).expect("should succeed");
            let deserialized: TaskType = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, tt);
        }
    }

    // --- CostTier tests ---
    #[test]
    fn test_cost_tier_default() {
        let tier = CostTier::default();
        assert_eq!(tier, CostTier::Medium);
    }

    #[test]
    fn test_cost_tier_ordering() {
        assert!(CostTier::Free < CostTier::Low);
        assert!(CostTier::Low < CostTier::Medium);
        assert!(CostTier::Medium < CostTier::High);
    }

    #[test]
    fn test_cost_tier_serde() {
        let tiers = vec![
            CostTier::Free,
            CostTier::Low,
            CostTier::Medium,
            CostTier::High,
        ];
        for tier in tiers {
            let json = serde_json::to_string(&tier).expect("should succeed");
            let deserialized: CostTier = serde_json::from_str(&json).expect("should succeed");
            assert_eq!(deserialized, tier);
        }
    }

    // --- PerformanceMetrics tests ---
    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert!(metrics.avg_latency_ms.is_none());
        assert!(metrics.requests_per_second.is_none());
        assert!(metrics.success_rate.is_none());
    }

    #[test]
    fn test_performance_metrics_serde() {
        let metrics = PerformanceMetrics {
            avg_latency_ms: Some(100),
            requests_per_second: Some(50.0),
            success_rate: Some(0.99),
            avg_tokens_per_second: Some(100.0),
            max_throughput_rps: Some(200.0),
            max_batch_size: Some(32),
            quality_score: Some(95),
        };
        let json = serde_json::to_string(&metrics).expect("should succeed");
        let deserialized: PerformanceMetrics = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.avg_latency_ms, Some(100));
        assert_eq!(deserialized.quality_score, Some(95));
    }

    // --- ResourceRequirements tests ---
    #[test]
    fn test_resource_requirements_default() {
        let req = ResourceRequirements::default();
        assert_eq!(req.min_memory_mb, 0);
        assert!(!req.requires_gpu);
        assert!(!req.requires_internet);
    }

    #[test]
    fn test_resource_requirements_serde() {
        let req = ResourceRequirements {
            min_memory_mb: 4096,
            min_gpu_memory_mb: Some(8192),
            min_cpu_cores: Some(4),
            requires_gpu: true,
            requires_internet: false,
            load_time_ms: Some(5000),
            requires_specific_hardware: true,
            hardware_requirements: Some("NVIDIA A100".to_string()),
        };
        let json = serde_json::to_string(&req).expect("should succeed");
        let deserialized: ResourceRequirements =
            serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.min_memory_mb, 4096);
        assert!(deserialized.requires_gpu);
    }

    // --- CostMetrics tests ---
    #[test]
    fn test_cost_metrics_default() {
        let metrics = CostMetrics::default();
        assert!(!metrics.has_fixed_cost);
        assert!(!metrics.is_free);
    }

    #[test]
    fn test_cost_metrics_serde() {
        let metrics = CostMetrics {
            cost_per_1k_input_tokens: Some(0.01),
            cost_per_1k_output_tokens: Some(0.03),
            cost_per_request: Some(0.001),
            has_fixed_cost: true,
            is_free: false,
        };
        let json = serde_json::to_string(&metrics).expect("should succeed");
        let deserialized: CostMetrics = serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.has_fixed_cost);
        assert!(!deserialized.is_free);
    }

    // --- SecurityRequirements tests ---
    #[test]
    fn test_security_requirements_default() {
        let req = SecurityRequirements::default();
        assert!(!req.requires_encryption);
        assert!(!req.contains_sensitive_data);
        assert_eq!(req.security_level, SecurityLevel::Medium);
        assert!(!req.requires_audit_logging);
        assert!(req.geo_restrictions.is_none());
    }

    #[test]
    fn test_security_requirements_serde() {
        let req = SecurityRequirements {
            requires_encryption: true,
            contains_sensitive_data: true,
            security_level: SecurityLevel::Critical,
            requires_audit_logging: true,
            geo_restrictions: Some(GeoConstraints {
                allowed_regions: Some(vec!["US".to_string(), "EU".to_string()]),
                blocked_regions: None,
                data_residency: Some("US".to_string()),
            }),
        };
        let json = serde_json::to_string(&req).expect("should succeed");
        let deserialized: SecurityRequirements =
            serde_json::from_str(&json).expect("should succeed");
        assert!(deserialized.requires_encryption);
        assert_eq!(deserialized.security_level, SecurityLevel::Critical);
        assert!(deserialized.geo_restrictions.is_some());
    }

    // --- GeoConstraints tests ---
    #[test]
    fn test_geo_constraints_serde() {
        let geo = GeoConstraints {
            allowed_regions: Some(vec!["US".to_string()]),
            blocked_regions: Some(vec!["CN".to_string()]),
            data_residency: Some("EU".to_string()),
        };
        let json = serde_json::to_string(&geo).expect("should succeed");
        let deserialized: GeoConstraints = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(
            deserialized
                .allowed_regions
                .as_ref()
                .expect("should succeed")
                .len(),
            1
        );
    }

    // --- AITask tests ---
    #[test]
    fn test_ai_task_default() {
        let task = AITask::default();
        assert_eq!(task.task_type, TaskType::TextGeneration);
        assert!(task.required_model_type.is_none());
        assert!(!task.requires_streaming);
        assert!(!task.requires_function_calling);
        assert_eq!(task.priority, 50);
    }

    #[test]
    fn test_ai_task_serde() {
        let task = AITask {
            task_type: TaskType::CodeGeneration,
            required_model_type: Some(ModelType::LargeLanguageModel),
            min_context_size: Some(4096),
            requires_streaming: true,
            requires_function_calling: true,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: Some(75),
            priority: 90,
        };
        let json = serde_json::to_string(&task).expect("should succeed");
        let deserialized: AITask = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.task_type, TaskType::CodeGeneration);
        assert_eq!(deserialized.priority, 90);
        assert!(deserialized.requires_streaming);
    }

    // --- AICapabilities tests ---
    #[test]
    fn test_ai_capabilities_default() {
        let caps = AICapabilities::default();
        assert!(caps.supported_model_types.is_empty());
        assert!(caps.supported_task_types.is_empty());
        assert_eq!(caps.max_context_size, 0);
        assert!(!caps.supports_streaming);
    }

    #[test]
    fn test_ai_capabilities_new() {
        let caps = AICapabilities::new();
        assert!(caps.supported_model_types.is_empty());
    }

    #[test]
    fn test_ai_capabilities_add_model_type() {
        let mut caps = AICapabilities::new();
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.add_model_type(ModelType::Embedding);
        assert!(caps.supports_model_type(&ModelType::LargeLanguageModel));
        assert!(caps.supports_model_type(&ModelType::Embedding));
        assert!(!caps.supports_model_type(&ModelType::ImageGeneration));
    }

    #[test]
    fn test_ai_capabilities_add_task_type() {
        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_task_type(TaskType::CodeGeneration);
        assert!(caps.supports_task(&TaskType::TextGeneration));
        assert!(caps.supports_task(&TaskType::CodeGeneration));
        assert!(!caps.supports_task(&TaskType::Translation));
    }

    #[test]
    fn test_ai_capabilities_builder_methods() {
        let mut caps = AICapabilities::new();
        caps.with_max_context_size(128_000)
            .with_streaming(true)
            .with_function_calling(true)
            .with_tool_use(true)
            .with_image_support(true);

        assert_eq!(caps.max_context_size, 128_000);
        assert!(caps.supports_streaming);
        assert!(caps.supports_function_calling);
        assert!(caps.supports_tool_use);
        assert!(caps.supports_images);
    }

    #[test]
    fn test_ai_capabilities_with_cost_metrics() {
        let mut caps = AICapabilities::new();
        let cost_metrics = CostMetrics {
            cost_per_1k_input_tokens: Some(0.01),
            cost_per_1k_output_tokens: Some(0.03),
            cost_per_request: None,
            has_fixed_cost: false,
            is_free: false,
        };
        caps.with_cost_metrics(cost_metrics);
        assert!(!caps.cost_metrics.is_free);
    }

    #[test]
    fn test_ai_capabilities_with_supported_tasks() {
        let mut caps = AICapabilities::new();
        caps.with_supported_tasks(vec![
            TaskType::TextGeneration,
            TaskType::CodeGeneration,
            TaskType::Summarization,
        ]);
        assert_eq!(caps.supported_task_types.len(), 3);
        assert!(caps.supports_task(&TaskType::Summarization));
    }

    #[test]
    fn test_ai_capabilities_serde() {
        let mut caps = AICapabilities::new();
        caps.add_model_type(ModelType::LargeLanguageModel);
        caps.add_task_type(TaskType::TextGeneration);
        caps.with_max_context_size(4096).with_streaming(true);

        let json = serde_json::to_string(&caps).expect("should succeed");
        let deserialized: AICapabilities = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.max_context_size, 4096);
        assert!(deserialized.supports_streaming);
    }

    // --- RoutingPreferences tests ---
    #[test]
    fn test_routing_preferences_default() {
        let prefs = RoutingPreferences::default();
        assert_eq!(prefs.priority, 50);
        assert!(prefs.allows_forwarding);
        assert!(!prefs.handles_sensitive_data);
        assert!(prefs.geo_constraints.is_none());
        assert_eq!(prefs.cost_tier, CostTier::Medium);
        assert!(!prefs.prefers_local);
        assert!((prefs.cost_sensitivity - 0.5).abs() < f64::EPSILON);
        assert!((prefs.performance_priority - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_routing_preferences_serde() {
        let prefs = RoutingPreferences {
            priority: 90,
            allows_forwarding: false,
            handles_sensitive_data: true,
            geo_constraints: None,
            cost_tier: CostTier::High,
            prefers_local: true,
            cost_sensitivity: 0.1,
            performance_priority: 0.9,
        };
        let json = serde_json::to_string(&prefs).expect("should succeed");
        let deserialized: RoutingPreferences = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.priority, 90);
        assert!(deserialized.prefers_local);
        assert_eq!(deserialized.cost_tier, CostTier::High);
    }
}
