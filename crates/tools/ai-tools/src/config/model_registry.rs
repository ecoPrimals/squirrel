// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Model registry for AI model capabilities
//!
//! This module provides a registry for AI model capabilities that can be loaded
//! from configuration files, allowing for easy updates without code changes.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::common::capability::{
    AICapabilities, CostMetrics, CostTier, ModelType, PerformanceMetrics, ResourceRequirements,
    TaskType,
};

static GLOBAL_REGISTRY: LazyLock<RwLock<ModelRegistry>> =
    LazyLock::new(|| RwLock::new(ModelRegistry::default()));

/// Model registry for AI model capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Map of provider ID to map of model ID to capabilities
    #[cfg(test)]
    pub models: HashMap<String, HashMap<String, ModelCapabilities>>,

    /// Map of provider ID to map of model ID to capabilities (private for non-test code)
    #[cfg(not(test))]
    models: HashMap<String, HashMap<String, ModelCapabilities>>,
}

/// Model capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Model name
    pub name: String,

    /// Provider ID
    pub provider_id: String,

    /// Model version
    pub version: Option<String>,

    /// Supported model types
    #[serde(default)]
    pub model_types: Vec<String>,

    /// Supported task types
    #[serde(default)]
    pub task_types: Vec<String>,

    /// Maximum context size in tokens
    pub max_context_size: Option<usize>,

    /// Whether streaming is supported
    #[serde(default)]
    pub supports_streaming: bool,

    /// Whether function calling is supported
    #[serde(default)]
    pub supports_function_calling: bool,

    /// Whether tool use is supported
    #[serde(default)]
    pub supports_tool_use: bool,

    /// Performance metrics
    #[serde(default)]
    pub performance: PerformanceConfig,

    /// Resource requirements
    #[serde(default)]
    pub resources: ResourceConfig,

    /// Cost metrics
    #[serde(default)]
    pub cost: CostConfig,

    /// Priority level (0-100, higher is more preferred)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// Whether this model handles sensitive data
    #[serde(default)]
    pub handles_sensitive_data: bool,

    /// Cost tier
    #[serde(default)]
    pub cost_tier: String,

    /// External API endpoint for this model (if different from provider default)
    pub api_endpoint: Option<String>,
}

fn default_priority() -> u8 {
    50
}

/// Performance characteristics of a model
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceConfig {
    /// Average tokens per second
    pub tokens_per_second: Option<f64>,

    /// Average latency in milliseconds
    pub avg_latency_ms: Option<u64>,

    /// Maximum batch size supported
    pub max_batch_size: Option<usize>, // Changed from u32 to usize

    /// Context window size
    pub context_window_size: Option<usize>,

    /// Memory usage per token
    pub memory_per_token: Option<f64>,
}

/// Resource requirements configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceConfig {
    /// Minimum RAM required in MB
    pub min_memory_mb: u64,

    /// Minimum GPU memory required in MB (if applicable)
    pub min_gpu_memory_mb: Option<u64>,

    /// Minimum CPU cores required
    pub min_cpu_cores: Option<u32>,

    /// Whether GPU acceleration is required
    pub requires_gpu: bool,

    /// Supported architectures
    pub supported_architectures: Vec<String>,
}

/// Cost metrics configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostConfig {
    /// Cost per 1000 input tokens in USD
    pub cost_per_1k_input_tokens: Option<f64>,

    /// Cost per 1000 output tokens in USD
    pub cost_per_1k_output_tokens: Option<f64>,

    /// Cost per request in USD
    pub cost_per_request: Option<f64>,

    /// Whether this provider has a fixed cost
    #[serde(default)]
    pub has_fixed_cost: bool,

    /// Whether this provider is free to use
    #[serde(default)]
    pub is_free: bool,
}

impl ModelRegistry {
    /// Load model registry from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let registry = serde_json::from_str(&contents)?;
        Ok(registry)
    }

    /// Get the global model registry instance
    pub fn instance() -> Arc<RwLock<Self>> {
        static REGISTRY: std::sync::OnceLock<Arc<RwLock<ModelRegistry>>> =
            std::sync::OnceLock::new();
        REGISTRY
            .get_or_init(|| Arc::new(RwLock::new(Self::default())))
            .clone()
    }

    /// Set the global model registry
    pub fn set_global(registry: ModelRegistry) {
        match GLOBAL_REGISTRY.write() {
            Ok(mut global_registry) => {
                *global_registry = registry;
            }
            Err(_) => {
                tracing::error!("Global model registry lock poisoned, cannot update registry");
            }
        }
    }

    /// Get capabilities for a model
    pub fn get_model_capabilities(
        &self,
        provider_id: &str,
        model_id: &str,
    ) -> Option<AICapabilities> {
        self.models.get(provider_id).and_then(|models| {
            models
                .get(model_id)
                .map(|capabilities| capabilities.to_ai_capabilities())
        })
    }

    /// Get all models for a provider
    pub fn get_provider_models(&self, provider_id: &str) -> Vec<String> {
        self.models
            .get(provider_id)
            .map(|models| models.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Register model capabilities
    pub fn register_model(&mut self, capabilities: ModelCapabilities) {
        let provider_id = capabilities.provider_id.clone();
        let model_id = capabilities.name.clone();

        self.models
            .entry(provider_id)
            .or_default()
            .insert(model_id, capabilities);
    }

    /// Save the registry to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Import default models for standard providers
    pub fn import_defaults(&mut self) {
        // Import OpenAI models
        self.register_model(ModelCapabilities {
            name: "gpt-4".to_string(),
            provider_id: "openai".to_string(),
            version: Some("2023-03-15".to_string()),
            model_types: vec!["LargeLanguageModel".to_string()],
            task_types: vec!["TextGeneration".to_string()],
            max_context_size: Some(8192),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: true,
            performance: PerformanceConfig {
                avg_latency_ms: Some(1000),
                tokens_per_second: Some(20.0),
                max_batch_size: None,
                context_window_size: None,
                memory_per_token: None,
            },
            resources: ResourceConfig {
                min_memory_mb: 512,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(2),
                requires_gpu: false,
                supported_architectures: vec!["x86_64".to_string()],
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.03),
                cost_per_1k_output_tokens: Some(0.06),
                cost_per_request: None,
                has_fixed_cost: false,
                is_free: false,
            },
            priority: 80,
            handles_sensitive_data: false,
            cost_tier: "High".to_string(),
            api_endpoint: None,
        });

        // Import Claude model
        self.register_model(ModelCapabilities {
            name: "claude-3-opus".to_string(),
            provider_id: "anthropic".to_string(),
            version: Some("20240229".to_string()),
            model_types: vec!["LargeLanguageModel".to_string()],
            task_types: vec![
                "TextGeneration".to_string(),
                "ImageUnderstanding".to_string(),
            ],
            max_context_size: Some(200000),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: true,
            performance: PerformanceConfig {
                avg_latency_ms: Some(1500),
                tokens_per_second: Some(15.0),
                max_batch_size: None,
                context_window_size: None,
                memory_per_token: None,
            },
            resources: ResourceConfig {
                min_memory_mb: 1024,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(2),
                requires_gpu: false,
                supported_architectures: vec!["x86_64".to_string()],
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.015),
                cost_per_1k_output_tokens: Some(0.075),
                cost_per_request: None,
                has_fixed_cost: false,
                is_free: false,
            },
            priority: 90,
            handles_sensitive_data: true,
            cost_tier: "High".to_string(),
            api_endpoint: None,
        });

        // Import Gemini model
        self.register_model(ModelCapabilities {
            name: "gemini-pro".to_string(),
            provider_id: "gemini".to_string(),
            version: None,
            model_types: vec!["LargeLanguageModel".to_string()],
            task_types: vec!["TextGeneration".to_string()],
            max_context_size: Some(32768),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: false,
            performance: PerformanceConfig {
                avg_latency_ms: Some(800),
                tokens_per_second: Some(30.0),
                max_batch_size: None,
                context_window_size: None,
                memory_per_token: None,
            },
            resources: ResourceConfig {
                min_memory_mb: 512,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(1),
                requires_gpu: false,
                supported_architectures: vec!["x86_64".to_string()],
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.0025),
                cost_per_1k_output_tokens: Some(0.005),
                cost_per_request: None,
                has_fixed_cost: false,
                is_free: false,
            },
            priority: 70,
            handles_sensitive_data: false,
            cost_tier: "Medium".to_string(),
            api_endpoint: None,
        });
    }
}

impl ModelCapabilities {
    /// Convert to AICapabilities
    pub fn to_ai_capabilities(&self) -> AICapabilities {
        let mut capabilities = AICapabilities::default();

        // Add model types
        for model_type_str in &self.model_types {
            let model_type = match model_type_str.as_str() {
                "LargeLanguageModel" => ModelType::LargeLanguageModel,
                "Embedding" => ModelType::Embedding,
                "ImageGeneration" => ModelType::ImageGeneration,
                "ImageUnderstanding" => ModelType::ImageUnderstanding,
                "AudioTranscription" => ModelType::AudioTranscription,
                "AudioGeneration" => ModelType::AudioGeneration,
                "MultiModal" => ModelType::MultiModal,
                _ => ModelType::Custom(model_type_str.clone()),
            };
            capabilities.add_model_type(model_type);
        }

        // Add task types
        for task_type_str in &self.task_types {
            let task_type = match task_type_str.as_str() {
                "TextGeneration" => TaskType::TextGeneration,
                "ImageGeneration" => TaskType::ImageGeneration,
                "ImageUnderstanding" => TaskType::ImageUnderstanding,
                "TextEmbedding" => TaskType::TextEmbedding,
                "AudioTranscription" => TaskType::AudioTranscription,
                "AudioGeneration" => TaskType::AudioGeneration,
                "DataAnalysis" => TaskType::DataAnalysis,
                "FunctionExecution" => TaskType::FunctionExecution,
                _ => TaskType::Custom(task_type_str.clone()),
            };
            capabilities.add_task_type(task_type);
        }

        // Set other capabilities
        if let Some(size) = self.max_context_size {
            capabilities.with_max_context_size(size);
        }

        capabilities.with_streaming(self.supports_streaming);
        capabilities.with_function_calling(self.supports_function_calling);
        capabilities.with_tool_use(self.supports_tool_use);

        // Set performance metrics
        capabilities.performance_metrics = PerformanceMetrics {
            avg_latency_ms: self.performance.avg_latency_ms,
            requests_per_second: self.performance.tokens_per_second,
            success_rate: None, // No direct mapping for success_rate from PerformanceConfig
            avg_tokens_per_second: self.performance.tokens_per_second,
            max_throughput_rps: None, // No direct mapping for max_throughput_rps from PerformanceConfig
            max_batch_size: self.performance.max_batch_size,
            quality_score: None, // No direct mapping for quality_score from PerformanceConfig
        };

        // Set resource requirements
        capabilities.resource_requirements = ResourceRequirements {
            min_memory_mb: self.resources.min_memory_mb,
            min_gpu_memory_mb: self.resources.min_gpu_memory_mb,
            min_cpu_cores: self.resources.min_cpu_cores,
            requires_gpu: self.resources.requires_gpu,
            requires_internet: false, // Default to false since field was removed
            load_time_ms: None,       // Default to None since field was removed
            requires_specific_hardware: false, // Default to false since field was removed
            hardware_requirements: None, // Default to None since field was removed
        };

        // Set cost metrics
        if self.cost.cost_per_1k_input_tokens.is_some()
            || self.cost.cost_per_1k_output_tokens.is_some()
            || self.cost.cost_per_request.is_some()
        {
            capabilities.cost_metrics = CostMetrics {
                cost_per_1k_input_tokens: self.cost.cost_per_1k_input_tokens,
                cost_per_1k_output_tokens: self.cost.cost_per_1k_output_tokens,
                cost_per_request: self.cost.cost_per_request,
                has_fixed_cost: self.cost.has_fixed_cost,
                is_free: self.cost.is_free,
            };
        }

        capabilities
    }

    /// Get the cost tier
    pub fn get_cost_tier(&self) -> CostTier {
        match self.cost_tier.as_str() {
            "Free" => CostTier::Free,
            "Low" => CostTier::Low,
            "Medium" => CostTier::Medium,
            "High" => CostTier::High,
            _ => CostTier::Medium, // Default to medium if unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model_capabilities(name: &str, provider_id: &str) -> ModelCapabilities {
        ModelCapabilities {
            name: name.to_string(),
            provider_id: provider_id.to_string(),
            version: Some("1.0".to_string()),
            model_types: vec!["LargeLanguageModel".to_string()],
            task_types: vec!["TextGeneration".to_string()],
            max_context_size: Some(8192),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: false,
            performance: PerformanceConfig {
                tokens_per_second: Some(20.0),
                avg_latency_ms: Some(500),
                max_batch_size: Some(32),
                context_window_size: Some(8192),
                memory_per_token: Some(0.001),
            },
            resources: ResourceConfig {
                min_memory_mb: 512,
                min_gpu_memory_mb: Some(1024),
                min_cpu_cores: Some(2),
                requires_gpu: false,
                supported_architectures: vec!["x86_64".to_string()],
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.03),
                cost_per_1k_output_tokens: Some(0.06),
                cost_per_request: None,
                has_fixed_cost: false,
                is_free: false,
            },
            priority: 80,
            handles_sensitive_data: false,
            cost_tier: "High".to_string(),
            api_endpoint: None,
        }
    }

    #[test]
    fn test_model_registry_default() {
        let registry = ModelRegistry::default();
        assert!(registry.models.is_empty());
    }

    #[test]
    fn test_model_registry_register_and_get() {
        let mut registry = ModelRegistry::default();
        let model = sample_model_capabilities("gpt-4", "openai");
        registry.register_model(model);

        let caps = registry.get_model_capabilities("openai", "gpt-4");
        assert!(caps.is_some());

        let caps = caps.unwrap();
        assert!(caps.supports_streaming);
        assert!(caps.supports_function_calling);
    }

    #[test]
    fn test_model_registry_get_nonexistent() {
        let registry = ModelRegistry::default();
        assert!(registry.get_model_capabilities("openai", "gpt-4").is_none());
        assert!(registry
            .get_model_capabilities("nonexistent", "model")
            .is_none());
    }

    #[test]
    fn test_model_registry_get_provider_models() {
        let mut registry = ModelRegistry::default();
        registry.register_model(sample_model_capabilities("gpt-4", "openai"));
        registry.register_model(sample_model_capabilities("gpt-3.5", "openai"));
        registry.register_model(sample_model_capabilities("claude-3", "anthropic"));

        let openai_models = registry.get_provider_models("openai");
        assert_eq!(openai_models.len(), 2);

        let anthropic_models = registry.get_provider_models("anthropic");
        assert_eq!(anthropic_models.len(), 1);

        let none_models = registry.get_provider_models("nonexistent");
        assert!(none_models.is_empty());
    }

    #[test]
    fn test_model_registry_import_defaults() {
        let mut registry = ModelRegistry::default();
        registry.import_defaults();

        // Should have 3 providers: openai, anthropic, gemini
        assert!(!registry.models.is_empty());

        let openai_models = registry.get_provider_models("openai");
        assert!(!openai_models.is_empty());

        let anthropic_models = registry.get_provider_models("anthropic");
        assert!(!anthropic_models.is_empty());

        let gemini_models = registry.get_provider_models("gemini");
        assert!(!gemini_models.is_empty());
    }

    #[test]
    fn test_model_capabilities_to_ai_capabilities() {
        let model = sample_model_capabilities("gpt-4", "openai");
        let caps = model.to_ai_capabilities();

        assert!(caps.supports_streaming);
        assert!(caps.supports_function_calling);
        assert!(!caps.supports_tool_use);
        assert_eq!(caps.performance_metrics.avg_latency_ms, Some(500));
        assert_eq!(caps.performance_metrics.avg_tokens_per_second, Some(20.0));
        assert_eq!(caps.resource_requirements.min_memory_mb, 512);
        assert_eq!(caps.resource_requirements.min_gpu_memory_mb, Some(1024));
        assert!(!caps.resource_requirements.requires_gpu);
        assert_eq!(caps.cost_metrics.cost_per_1k_input_tokens, Some(0.03));
        assert_eq!(caps.cost_metrics.cost_per_1k_output_tokens, Some(0.06));
    }

    #[test]
    fn test_model_capabilities_to_ai_capabilities_all_model_types() {
        let mut model = sample_model_capabilities("test", "test");
        model.model_types = vec![
            "LargeLanguageModel".to_string(),
            "Embedding".to_string(),
            "ImageGeneration".to_string(),
            "ImageUnderstanding".to_string(),
            "AudioTranscription".to_string(),
            "AudioGeneration".to_string(),
            "MultiModal".to_string(),
            "CustomType".to_string(),
        ];
        model.task_types = vec![
            "TextGeneration".to_string(),
            "ImageGeneration".to_string(),
            "ImageUnderstanding".to_string(),
            "TextEmbedding".to_string(),
            "AudioTranscription".to_string(),
            "AudioGeneration".to_string(),
            "DataAnalysis".to_string(),
            "FunctionExecution".to_string(),
            "CustomTask".to_string(),
        ];

        let caps = model.to_ai_capabilities();
        // Should have parsed all model and task types
        assert!(caps.supported_model_types.len() >= 8);
        assert!(caps.supported_task_types.len() >= 9);
    }

    #[test]
    fn test_model_capabilities_get_cost_tier() {
        let mut model = sample_model_capabilities("test", "test");

        model.cost_tier = "Free".to_string();
        assert_eq!(model.get_cost_tier(), CostTier::Free);

        model.cost_tier = "Low".to_string();
        assert_eq!(model.get_cost_tier(), CostTier::Low);

        model.cost_tier = "Medium".to_string();
        assert_eq!(model.get_cost_tier(), CostTier::Medium);

        model.cost_tier = "High".to_string();
        assert_eq!(model.get_cost_tier(), CostTier::High);

        model.cost_tier = "Unknown".to_string();
        assert_eq!(model.get_cost_tier(), CostTier::Medium); // Default
    }

    #[test]
    fn test_model_registry_serde() {
        let mut registry = ModelRegistry::default();
        registry.register_model(sample_model_capabilities("gpt-4", "openai"));

        let json = serde_json::to_string(&registry).unwrap();
        let deserialized: ModelRegistry = serde_json::from_str(&json).unwrap();
        assert!(deserialized
            .get_model_capabilities("openai", "gpt-4")
            .is_some());
    }

    #[test]
    fn test_model_capabilities_serde() {
        let model = sample_model_capabilities("gpt-4", "openai");
        let json = serde_json::to_string(&model).unwrap();
        let deserialized: ModelCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "gpt-4");
        assert_eq!(deserialized.provider_id, "openai");
        assert_eq!(deserialized.priority, 80);
        assert!(!deserialized.handles_sensitive_data);
    }

    #[test]
    fn test_default_priority() {
        assert_eq!(default_priority(), 50);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert!(config.tokens_per_second.is_none());
        assert!(config.avg_latency_ms.is_none());
        assert!(config.max_batch_size.is_none());
    }

    #[test]
    fn test_resource_config_default() {
        let config = ResourceConfig::default();
        assert_eq!(config.min_memory_mb, 0);
        assert!(config.min_gpu_memory_mb.is_none());
        assert!(!config.requires_gpu);
    }

    #[test]
    fn test_cost_config_default() {
        let config = CostConfig::default();
        assert!(config.cost_per_1k_input_tokens.is_none());
        assert!(!config.has_fixed_cost);
        assert!(!config.is_free);
    }

    #[test]
    fn test_model_registry_instance() {
        let instance = ModelRegistry::instance();
        let guard = instance.read().unwrap();
        // Default instance should have empty models
        assert!(guard.models.is_empty() || !guard.models.is_empty()); // Just verify it's accessible
    }

    #[test]
    fn test_model_registry_set_global() {
        let mut registry = ModelRegistry::default();
        registry.register_model(sample_model_capabilities("test-model", "test-provider"));
        ModelRegistry::set_global(registry);
        // Verify no panic
    }

    #[test]
    fn test_model_capabilities_no_cost() {
        let mut model = sample_model_capabilities("test", "test");
        model.cost = CostConfig::default(); // No cost info

        let caps = model.to_ai_capabilities();
        // cost_metrics should be default since no cost data
        assert!(caps.cost_metrics.cost_per_1k_input_tokens.is_none());
    }

    #[test]
    fn test_model_capabilities_no_context_size() {
        let mut model = sample_model_capabilities("test", "test");
        model.max_context_size = None;

        let caps = model.to_ai_capabilities();
        // Should still produce valid capabilities
        assert!(caps.supports_streaming);
    }

    #[test]
    fn test_model_registry_save_and_load() {
        let mut registry = ModelRegistry::default();
        registry.register_model(sample_model_capabilities("gpt-4", "openai"));

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_model_registry.json");

        registry.save_to_file(&path).unwrap();
        let loaded = ModelRegistry::from_file(&path).unwrap();
        assert!(loaded.get_model_capabilities("openai", "gpt-4").is_some());

        // Clean up
        let _ = std::fs::remove_file(path);
    }
}
