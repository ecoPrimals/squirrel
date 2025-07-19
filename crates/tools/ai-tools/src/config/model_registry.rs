//! Model registry for AI model capabilities
//!
//! This module provides a registry for AI model capabilities that can be loaded
//! from configuration files, allowing for easy updates without code changes.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::RwLock;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::common::capability::{
    AICapabilities, ModelType, TaskType, PerformanceMetrics, 
    ResourceRequirements, CostTier, CostMetrics,
};

lazy_static! {
    static ref GLOBAL_REGISTRY: RwLock<ModelRegistry> = RwLock::new(ModelRegistry::default());
}

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

/// Performance metrics configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Average latency in milliseconds
    pub avg_latency_ms: Option<u64>,
    
    /// Requests per second
    pub requests_per_second: Option<f64>,
    
    /// Success rate (0.0-1.0)
    pub success_rate: Option<f64>,
    
    /// Average token throughput (tokens/second)
    pub avg_tokens_per_second: Option<f64>,
    
    /// Maximum throughput in requests per second
    pub max_throughput_rps: Option<f64>,
    
    /// Maximum batch size for batch processing
    pub max_batch_size: Option<u32>,
    
    /// Quality score (0-100)
    pub quality_score: Option<u8>,
}

/// Resource requirements configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Minimum memory required in MB
    #[serde(default = "default_memory_mb")]
    pub min_memory_mb: u32,
    
    /// Minimum GPU memory required in MB (if GPU is used)
    pub min_gpu_memory_mb: Option<u32>,
    
    /// Minimum CPU cores required
    pub min_cpu_cores: Option<u32>,
    
    /// Whether GPU is required
    #[serde(default)]
    pub requires_gpu: bool,
    
    /// Whether internet access is required
    #[serde(default = "default_true")]
    pub requires_internet: bool,
    
    /// Estimated time required to load the model in milliseconds
    pub load_time_ms: Option<u64>,
    
    /// Whether specific hardware is required
    #[serde(default)]
    pub requires_specific_hardware: bool,
    
    /// Hardware requirements as a string description
    pub hardware_requirements: Option<String>,
}

fn default_memory_mb() -> u32 {
    256
}

fn default_true() -> bool {
    true
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
    
    /// Get the global model registry
    pub fn global() -> Self {
        GLOBAL_REGISTRY.read()
            .unwrap_or_else(|_| {
                tracing::error!("Global model registry lock poisoned, returning default registry");
                return Self::default();
            })
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
    pub fn get_model_capabilities(&self, provider_id: &str, model_id: &str) -> Option<AICapabilities> {
        self.models.get(provider_id).and_then(|models| {
            models.get(model_id).map(|capabilities| capabilities.to_ai_capabilities())
        })
    }
    
    /// Get all models for a provider
    pub fn get_provider_models(&self, provider_id: &str) -> Vec<String> {
        self.models.get(provider_id)
            .map(|models| models.keys().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Register model capabilities
    pub fn register_model(&mut self, capabilities: ModelCapabilities) {
        let provider_id = capabilities.provider_id.clone();
        let model_id = capabilities.name.clone();
        
        self.models
            .entry(provider_id)
            .or_insert_with(HashMap::new)
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
                requests_per_second: Some(20.0),
                success_rate: Some(0.99),
                avg_tokens_per_second: Some(40.0),
                max_throughput_rps: Some(20.0),
                max_batch_size: None,
                quality_score: Some(95),
            },
            resources: ResourceConfig {
                min_memory_mb: 512,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(2),
                requires_gpu: false,
                requires_internet: true,
                load_time_ms: None,
                requires_specific_hardware: false,
                hardware_requirements: None,
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
            task_types: vec!["TextGeneration".to_string(), "ImageUnderstanding".to_string()],
            max_context_size: Some(200000),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: true,
            performance: PerformanceConfig {
                avg_latency_ms: Some(1500),
                requests_per_second: Some(15.0),
                success_rate: Some(0.99),
                avg_tokens_per_second: Some(30.0),
                max_throughput_rps: Some(15.0),
                max_batch_size: None,
                quality_score: Some(98),
            },
            resources: ResourceConfig {
                min_memory_mb: 1024,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(2),
                requires_gpu: false,
                requires_internet: true,
                load_time_ms: None,
                requires_specific_hardware: false,
                hardware_requirements: None,
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
                requests_per_second: Some(30.0),
                success_rate: Some(0.98),
                avg_tokens_per_second: Some(45.0),
                max_throughput_rps: Some(30.0),
                max_batch_size: None,
                quality_score: Some(90),
            },
            resources: ResourceConfig {
                min_memory_mb: 512,
                min_gpu_memory_mb: None,
                min_cpu_cores: Some(1),
                requires_gpu: false,
                requires_internet: true,
                load_time_ms: None,
                requires_specific_hardware: false,
                hardware_requirements: None,
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
            requests_per_second: self.performance.requests_per_second,
            success_rate: self.performance.success_rate,
            avg_tokens_per_second: self.performance.avg_tokens_per_second,
            max_throughput_rps: self.performance.max_throughput_rps,
            max_batch_size: self.performance.max_batch_size,
            quality_score: self.performance.quality_score,
        };
        
        // Set resource requirements
        capabilities.resource_requirements = ResourceRequirements {
            min_memory_mb: self.resources.min_memory_mb,
            min_gpu_memory_mb: self.resources.min_gpu_memory_mb,
            min_cpu_cores: self.resources.min_cpu_cores,
            requires_gpu: self.resources.requires_gpu,
            requires_internet: self.resources.requires_internet,
            load_time_ms: self.resources.load_time_ms,
            requires_specific_hardware: self.resources.requires_specific_hardware,
            hardware_requirements: self.resources.hardware_requirements.clone(),
        };
        
        // Set cost metrics
        if self.cost.cost_per_1k_input_tokens.is_some() || 
           self.cost.cost_per_1k_output_tokens.is_some() || 
           self.cost.cost_per_request.is_some() {
            capabilities.cost_metrics = Some(CostMetrics {
                cost_per_1k_input_tokens: self.cost.cost_per_1k_input_tokens,
                cost_per_1k_output_tokens: self.cost.cost_per_1k_output_tokens,
                cost_per_request: self.cost.cost_per_request,
                has_fixed_cost: self.cost.has_fixed_cost,
                is_free: self.cost.is_free,
            });
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