// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tracing::{info, warn};

use super::{AICapabilities, CostMetrics, CostTier, ModelType, TaskType};

// Global registry singleton
static GLOBAL_REGISTRY: LazyLock<RwLock<ModelRegistry>> =
    LazyLock::new(|| RwLock::new(ModelRegistry::default()));

/// Model registry for AI model capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Map of provider ID to map of model ID to capabilities
    models: HashMap<String, HashMap<String, ModelCapabilities>>,

    /// Default search path for configuration files
    #[serde(skip)]
    config_paths: Vec<PathBuf>,
}

/// Model capabilities configuration
#[expect(
    clippy::struct_excessive_bools,
    reason = "Serde-backed model manifest; flat bool fields match JSON schema"
)]
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

const fn default_priority() -> u8 {
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

const fn default_memory_mb() -> u32 {
    256
}

const fn default_true() -> bool {
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

/// Custom model registry serialization struct for cleaner JSON output
#[derive(Serialize)]
struct ModelRegistryOutput {
    models: HashMap<String, HashMap<String, ModelCapabilities>>,
}

impl ModelRegistry {
    /// Create a new empty model registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            config_paths: vec![],
        }
    }

    /// Get the global model registry instance
    pub fn global() -> Self {
        GLOBAL_REGISTRY.read().map_or_else(
            |_| {
                warn!("Global model registry is poisoned, returning default instance");
                Self::new()
            },
            |registry| registry.clone(),
        )
    }

    /// Set the global model registry instance
    pub fn set_global(registry: Self) {
        match GLOBAL_REGISTRY.write() {
            Ok(mut global_registry) => {
                *global_registry = registry;
            }
            Err(_) => {
                warn!("Failed to set global model registry - global registry is poisoned");
                // Cannot recover from this, but we don't panic
            }
        }
    }

    /// Update the global model registry
    pub fn update_global<F>(f: F)
    where
        F: FnOnce(&mut Self),
    {
        match GLOBAL_REGISTRY.write() {
            Ok(mut registry) => {
                f(&mut registry);
            }
            Err(_) => {
                warn!("Failed to update global model registry - global registry is poisoned");
                // Cannot recover from this, but we don't panic
            }
        }
    }

    /// Initialize the registry with default search paths and load configurations
    ///
    /// # Errors
    ///
    /// Propagates I/O and deserialization errors from [`Self::load_from_available_paths`].
    pub fn initialize() -> anyhow::Result<()> {
        let mut registry = Self::new();

        // Add standard configuration paths
        registry.add_config_path(PathBuf::from("./config/ai-models.json"));

        // Check for user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let user_config = config_dir.join("squirrel-ai-tools").join("ai-models.json");
            registry.add_config_path(user_config);
        }

        // Load from available paths
        registry.load_from_available_paths()?;

        // Set as global
        Self::set_global(registry);

        Ok(())
    }

    /// Add a configuration path to search
    pub fn add_config_path(&mut self, path: PathBuf) {
        self.config_paths.push(path);
    }

    /// Load from the first available config path
    ///
    /// # Errors
    ///
    /// Propagates errors from the first successful path's [`Self::load_from_file`].
    pub fn load_from_available_paths(&mut self) -> anyhow::Result<()> {
        let paths = self.config_paths.clone();
        for path in &paths {
            if path.exists() {
                info!("Loading model registry from {}", path.display());
                match self.load_from_file(path) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        warn!(
                            "Failed to load model registry from {}: {}",
                            path.display(),
                            e
                        );
                        // Continue to next path
                    }
                }
            }
        }

        // If no configs were found, import defaults
        info!("No model registry config found, importing defaults");
        self.import_defaults();

        Ok(())
    }

    /// Load model registry from a file
    ///
    /// # Errors
    ///
    /// Propagates I/O and JSON/TOML deserialization errors.
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path_ref = path.as_ref(); // Get a reference to avoid moving path
        let contents = fs::read_to_string(path_ref)?;

        // Special handling for json files with a models key
        if path_ref.extension().is_some_and(|ext| ext == "json") {
            let json: serde_json::Value = serde_json::from_str(&contents)?;

            if let Some(models_obj) = json.get("models").and_then(|m| m.as_object()) {
                let mut registry = Self::new();

                // Process each provider
                for (_provider_id, provider_models) in models_obj {
                    if let Some(models_map) = provider_models.as_object() {
                        // Process each model in this provider
                        for (_model_id, model_data) in models_map {
                            if let Ok(capabilities) =
                                serde_json::from_value::<ModelCapabilities>(model_data.clone())
                            {
                                registry.register_model(capabilities);
                            } else {
                                tracing::warn!("Failed to parse capabilities for model");
                            }
                        }
                    }
                }

                return Ok(());
            }
        }

        // Standard parsing (direct model registry format)
        let registry: Self = serde_json::from_str(&contents)?;
        self.models = registry.models;
        Ok(())
    }

    /// Save the registry to a file
    ///
    /// # Errors
    ///
    /// Propagates serialization and I/O errors.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        // Format depends on file extension
        let is_json = path.as_ref().extension().is_some_and(|ext| ext == "json");

        let contents = if is_json {
            // For JSON files, use a nested structure with "models" top-level key
            let output = ModelRegistryOutput {
                models: self.models.clone(),
            };
            serde_json::to_string_pretty(&output)?
        } else {
            // For other files, use the direct registry format
            serde_json::to_string_pretty(self)?
        };

        // Ensure directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, contents)?;
        Ok(())
    }

    /// Get capabilities for a model
    #[must_use]
    pub fn get_model_capabilities(
        &self,
        provider_id: &str,
        model_id: &str,
    ) -> Option<AICapabilities> {
        self.models.get(provider_id).and_then(|models| {
            models
                .get(model_id)
                .map(ModelCapabilities::to_ai_capabilities)
        })
    }

    /// Get all models for a provider
    #[must_use]
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

    /// Get all registered providers
    #[must_use]
    pub fn get_providers(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    /// Import default models for standard providers
    #[expect(
        clippy::too_many_lines,
        reason = "Single catalog of bundled defaults; splitting would obscure provider groupings"
    )]
    pub fn import_defaults(&mut self) {
        // OpenAI models
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
                avg_latency_ms: Some(2000),
                quality_score: Some(95),
                ..Default::default()
            },
            resources: ResourceConfig {
                requires_internet: true,
                ..Default::default()
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.03),
                cost_per_1k_output_tokens: Some(0.06),
                ..Default::default()
            },
            priority: 80,
            handles_sensitive_data: false,
            cost_tier: "High".to_string(),
            api_endpoint: None,
        });

        self.register_model(ModelCapabilities {
            name: "gpt-4-turbo".to_string(),
            provider_id: "openai".to_string(),
            version: Some("2024-04-09".to_string()),
            model_types: vec!["LargeLanguageModel".to_string()],
            task_types: vec!["TextGeneration".to_string()],
            max_context_size: Some(128_000),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: true,
            performance: PerformanceConfig {
                avg_latency_ms: Some(1800),
                quality_score: Some(90),
                ..Default::default()
            },
            resources: ResourceConfig {
                requires_internet: true,
                ..Default::default()
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.01),
                cost_per_1k_output_tokens: Some(0.03),
                ..Default::default()
            },
            priority: 85,
            handles_sensitive_data: false,
            cost_tier: "Medium".to_string(),
            api_endpoint: None,
        });

        // Anthropic models
        self.register_model(ModelCapabilities {
            name: "claude-3-opus-20240229".to_string(),
            provider_id: "anthropic".to_string(),
            version: Some("2024-02-29".to_string()),
            model_types: vec!["LargeLanguageModel".to_string(), "MultiModal".to_string()],
            task_types: vec![
                "TextGeneration".to_string(),
                "ImageUnderstanding".to_string(),
            ],
            max_context_size: Some(200_000),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: true,
            performance: PerformanceConfig {
                avg_latency_ms: Some(2500),
                quality_score: Some(98),
                ..Default::default()
            },
            resources: ResourceConfig {
                requires_internet: true,
                ..Default::default()
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.015),
                cost_per_1k_output_tokens: Some(0.075),
                ..Default::default()
            },
            priority: 95,
            handles_sensitive_data: false,
            cost_tier: "High".to_string(),
            api_endpoint: None,
        });

        // Gemini models
        self.register_model(ModelCapabilities {
            name: "gemini-1.5-pro".to_string(),
            provider_id: "gemini".to_string(),
            version: Some("2024-05".to_string()),
            model_types: vec!["LargeLanguageModel".to_string(), "MultiModal".to_string()],
            task_types: vec![
                "TextGeneration".to_string(),
                "ImageUnderstanding".to_string(),
            ],
            max_context_size: Some(1_000_000),
            supports_streaming: true,
            supports_function_calling: true,
            supports_tool_use: false,
            performance: PerformanceConfig {
                avg_latency_ms: Some(1500),
                quality_score: Some(88),
                ..Default::default()
            },
            resources: ResourceConfig {
                requires_internet: true,
                ..Default::default()
            },
            cost: CostConfig {
                cost_per_1k_input_tokens: Some(0.0025),
                cost_per_1k_output_tokens: Some(0.0075),
                ..Default::default()
            },
            priority: 75,
            handles_sensitive_data: false,
            cost_tier: "Medium".to_string(),
            api_endpoint: None,
        });
    }

    /// Create a model registry from a file
    ///
    /// # Errors
    ///
    /// Propagates the same errors as [`Self::load_from_file`] for the given path format.
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path_ref = path.as_ref(); // Get a reference to avoid moving path
        let contents = fs::read_to_string(path_ref)?;

        // Special handling for json files with a models key
        if path_ref.extension().is_some_and(|ext| ext == "json") {
            let json: serde_json::Value = serde_json::from_str(&contents)?;

            if let Some(models_obj) = json.get("models").and_then(|m| m.as_object()) {
                let mut registry = Self::new();

                // Process each provider
                for (_provider_id, provider_models) in models_obj {
                    if let Some(models_map) = provider_models.as_object() {
                        // Process each model in this provider
                        for (_model_id, model_data) in models_map {
                            if let Ok(capabilities) =
                                serde_json::from_value::<ModelCapabilities>(model_data.clone())
                            {
                                registry.register_model(capabilities);
                            } else {
                                tracing::warn!("Failed to parse capabilities for model");
                            }
                        }
                    }
                }

                return Ok(registry);
            }
        }

        // Standard parsing (direct model registry format)
        let registry: Self = serde_json::from_str(&contents)?;
        Ok(registry)
    }
}

impl ModelCapabilities {
    /// Convert to `AICapabilities` structure
    #[must_use]
    pub fn to_ai_capabilities(&self) -> AICapabilities {
        let mut capabilities = AICapabilities::new();

        // Convert model types
        for model_type_str in &self.model_types {
            capabilities.add_model_type(Self::str_to_model_type(model_type_str));
        }

        // Convert task types
        for task_type_str in &self.task_types {
            capabilities.add_task_type(Self::str_to_task_type(task_type_str));
        }

        // Set other capabilities
        capabilities.with_max_context_size(self.max_context_size.unwrap_or(0));
        capabilities.with_streaming(self.supports_streaming);
        capabilities.with_function_calling(self.supports_function_calling);
        capabilities.with_tool_use(self.supports_tool_use);

        // Set performance metrics
        if let Some(latency) = self.performance.avg_latency_ms {
            capabilities.performance_metrics.avg_latency_ms = Some(latency);
        }
        if let Some(rps) = self.performance.requests_per_second {
            capabilities.performance_metrics.requests_per_second = Some(rps);
        }
        if let Some(success) = self.performance.success_rate {
            capabilities.performance_metrics.success_rate = Some(success);
        }
        if let Some(tokens) = self.performance.avg_tokens_per_second {
            capabilities.performance_metrics.avg_tokens_per_second = Some(tokens);
        }
        if let Some(throughput) = self.performance.max_throughput_rps {
            capabilities.performance_metrics.max_throughput_rps = Some(throughput);
        }
        if let Some(batch) = self.performance.max_batch_size {
            capabilities.performance_metrics.max_batch_size = Some(batch as usize);
        }
        if let Some(quality) = self.performance.quality_score {
            capabilities.performance_metrics.quality_score = Some(quality);
        }

        // Set resource requirements
        capabilities.resource_requirements.min_memory_mb = u64::from(self.resources.min_memory_mb);
        capabilities.resource_requirements.min_gpu_memory_mb =
            self.resources.min_gpu_memory_mb.map(u64::from);
        capabilities.resource_requirements.min_cpu_cores = self.resources.min_cpu_cores;
        capabilities.resource_requirements.requires_gpu = self.resources.requires_gpu;
        capabilities.resource_requirements.requires_internet = self.resources.requires_internet;
        capabilities.resource_requirements.load_time_ms = self.resources.load_time_ms;
        capabilities
            .resource_requirements
            .requires_specific_hardware = self.resources.requires_specific_hardware;
        capabilities
            .resource_requirements
            .hardware_requirements
            .clone_from(&self.resources.hardware_requirements);

        // Set cost metrics
        let cost_metrics = CostMetrics {
            cost_per_1k_input_tokens: self.cost.cost_per_1k_input_tokens,
            cost_per_1k_output_tokens: self.cost.cost_per_1k_output_tokens,
            cost_per_request: self.cost.cost_per_request,
            has_fixed_cost: self.cost.has_fixed_cost,
            is_free: self.cost.is_free,
        };
        capabilities.cost_metrics = cost_metrics;

        capabilities
    }

    /// Convert string to `ModelType`
    fn str_to_model_type(model_type_str: &str) -> ModelType {
        match model_type_str {
            "LargeLanguageModel" => ModelType::LargeLanguageModel,
            "Embedding" => ModelType::Embedding,
            "ImageGeneration" => ModelType::ImageGeneration,
            "ImageUnderstanding" => ModelType::ImageUnderstanding,
            "AudioTranscription" => ModelType::AudioTranscription,
            "AudioGeneration" => ModelType::AudioGeneration,
            "MultiModal" => ModelType::MultiModal,
            custom => ModelType::Custom(custom.to_string()),
        }
    }

    /// Convert string to `TaskType`
    fn str_to_task_type(task_type_str: &str) -> TaskType {
        match task_type_str {
            "TextGeneration" => TaskType::TextGeneration,
            "ImageGeneration" => TaskType::ImageGeneration,
            "ImageUnderstanding" => TaskType::ImageUnderstanding,
            "TextEmbedding" => TaskType::TextEmbedding,
            "AudioTranscription" => TaskType::AudioTranscription,
            "AudioGeneration" => TaskType::AudioGeneration,
            "DataAnalysis" => TaskType::DataAnalysis,
            "FunctionExecution" => TaskType::FunctionExecution,
            custom => TaskType::Custom(custom.to_string()),
        }
    }

    /// Get cost tier as enum
    #[must_use]
    pub fn get_cost_tier(&self) -> CostTier {
        match self.cost_tier.to_lowercase().as_str() {
            "free" => CostTier::Free,
            "low" => CostTier::Low,
            "medium" => CostTier::Medium,
            _ => CostTier::High,
        }
    }
}

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
