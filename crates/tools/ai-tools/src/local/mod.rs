//! Local AI Model Client
//!
//! This module provides a client for running AI models locally, supporting
//! both native Rust implementations and external local model servers.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::common::capability::{
    AICapabilities, CostTier, ModelType, RoutingPreferences, TaskType,
};
use crate::common::{AIClient, ChatRequest, ChatResponse, ChatResponseStream};
use crate::error::Error;
use crate::Result;

pub mod config;
pub mod native;
pub mod ollama;

pub use config::{LocalAIConfig, OllamaConfig};

/// Local AI client that can dispatch to different local model implementations
#[derive(Debug)]
pub struct LocalAIClient {
    /// Configuration
    config: LocalAIConfig,

    /// Available local model implementations
    implementations: Arc<RwLock<HashMap<String, Arc<dyn LocalModelProvider>>>>,

    /// Model registry for local models
    model_registry: Arc<RwLock<HashMap<String, LocalModelInfo>>>,

    /// Default model ID
    default_model_id: String,
}

/// Information about a local model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    /// Model ID
    pub id: String,

    /// Model name
    pub name: String,

    /// Model path or identifier
    pub path: String,

    /// Implementation type (ollama, native, etc.)
    pub implementation: String,

    /// Model capabilities
    pub capabilities: AICapabilities,

    /// Resource requirements
    pub resource_requirements: ResourceRequirements,

    /// Whether the model is currently loaded
    pub is_loaded: bool,

    /// Load time in milliseconds
    pub load_time_ms: Option<u64>,
}

/// Resource requirements for a local model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum RAM required in MB
    pub min_memory_mb: u64,

    /// Minimum GPU memory required in MB (if applicable)
    pub min_gpu_memory_mb: Option<u64>,

    /// Whether GPU is required
    pub requires_gpu: bool,

    /// Minimum CPU cores
    pub min_cpu_cores: Option<u32>,

    /// Estimated load time in milliseconds
    pub estimated_load_time_ms: u64,
}

/// Trait for local model providers (Ollama, native implementations, etc.)
#[async_trait]
pub trait LocalModelProvider: Send + Sync + std::fmt::Debug {
    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// List available models for this provider
    async fn list_models(&self) -> Result<Vec<LocalModelInfo>>;

    /// Load a model
    async fn load_model(&self, model_id: &str) -> Result<()>;

    /// Unload a model
    async fn unload_model(&self, model_id: &str) -> Result<()>;

    /// Check if a model is loaded
    async fn is_model_loaded(&self, model_id: &str) -> Result<bool>;

    /// Send a chat request to a loaded model
    async fn chat(&self, model_id: &str, request: ChatRequest) -> Result<ChatResponse>;

    /// Send a streaming chat request to a loaded model
    async fn chat_stream(&self, model_id: &str, request: ChatRequest)
        -> Result<ChatResponseStream>;

    /// Get model information
    async fn get_model_info(&self, model_id: &str) -> Result<LocalModelInfo>;

    /// Get provider capabilities
    fn capabilities(&self) -> AICapabilities;
}

impl LocalAIClient {
    /// Create a new local AI client
    pub fn new(config: LocalAIConfig) -> Result<Self> {
        let default_model_id = config.default_model.clone();

        Ok(Self {
            config,
            implementations: Arc::new(RwLock::new(HashMap::new())),
            model_registry: Arc::new(RwLock::new(HashMap::new())),
            default_model_id,
        })
    }

    /// Register a local model provider
    pub async fn register_provider(&self, provider: Arc<dyn LocalModelProvider>) -> Result<()> {
        let provider_name = provider.provider_name().to_string();
        info!("Registering local model provider: {}", provider_name);

        // Add to implementations
        {
            let mut implementations = self.implementations.write().await;
            implementations.insert(provider_name.clone(), provider.clone());
        }

        // Discover and register models from this provider
        match provider.list_models().await {
            Ok(models) => {
                let mut registry = self.model_registry.write().await;
                for model in models {
                    debug!(
                        "Registering local model: {} from provider {}",
                        model.id, provider_name
                    );
                    registry.insert(model.id.clone(), model);
                }
                info!(
                    "Registered {} models from provider {}",
                    registry.len(),
                    provider_name
                );
            }
            Err(e) => {
                warn!(
                    "Failed to discover models from provider {}: {}",
                    provider_name, e
                );
            }
        }

        Ok(())
    }

    /// Get a model provider for a given model ID
    async fn get_provider_for_model(&self, model_id: &str) -> Result<Arc<dyn LocalModelProvider>> {
        let registry = self.model_registry.read().await;
        let model_info = registry
            .get(model_id)
            .ok_or_else(|| Error::Model(format!("Model not found: {model_id}")))?;

        let implementations = self.implementations.read().await;
        let provider = implementations
            .get(&model_info.implementation)
            .ok_or_else(|| {
                Error::Model(format!("Provider not found: {}", model_info.implementation))
            })?;

        Ok(provider.clone())
    }

    /// Ensure a model is loaded
    async fn ensure_model_loaded(&self, model_id: &str) -> Result<()> {
        let provider = self.get_provider_for_model(model_id).await?;

        if !provider.is_model_loaded(model_id).await? {
            info!("Loading model: {}", model_id);
            provider.load_model(model_id).await?;
            info!("Model loaded successfully: {}", model_id);
        }

        Ok(())
    }

    /// Get aggregated capabilities from all providers
    fn get_aggregated_capabilities(&self) -> AICapabilities {
        // For now, return default capabilities
        // In a real implementation, this would aggregate capabilities from all providers
        let mut capabilities = AICapabilities::default();

        // Local models typically support these features
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.with_streaming(true);
        capabilities.with_function_calling(true);
        capabilities.with_tool_use(false); // Most local models don't support tool use yet
        capabilities.with_max_context_size(8192); // Conservative default

        capabilities
    }
}

#[async_trait]
impl AIClient for LocalAIClient {
    fn provider_name(&self) -> &str {
        "local"
    }

    fn default_model(&self) -> &str {
        &self.default_model_id
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let registry = self.model_registry.read().await;
        Ok(registry.keys().cloned().collect())
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model_id = request.model.as_deref().unwrap_or(&self.default_model_id);

        // Ensure model is loaded
        self.ensure_model_loaded(model_id).await?;

        // Get provider for this model
        let provider = self.get_provider_for_model(model_id).await?;

        // Clone the request to avoid borrow issues
        let request_clone = ChatRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            parameters: request.parameters.clone(),
            tools: request.tools.clone(),
        };

        provider.chat(model_id, request_clone).await
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let model_id = request.model.as_deref().unwrap_or(&self.default_model_id);

        // Ensure model is loaded
        self.ensure_model_loaded(model_id).await?;

        // Get provider for this model
        let provider = self.get_provider_for_model(model_id).await?;

        // Clone the request to avoid borrow issues
        let request_clone = ChatRequest {
            model: request.model.clone(),
            messages: request.messages.clone(),
            parameters: request.parameters.clone(),
            tools: request.tools.clone(),
        };

        provider.chat_stream(model_id, request_clone).await
    }

    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities> {
        // Try to get capabilities from a specific provider
        if let Ok(provider) = self.get_provider_for_model(model).await {
            return Ok(provider.capabilities());
        }

        // Fall back to aggregated capabilities
        Ok(self.get_aggregated_capabilities())
    }

    async fn is_available(&self) -> bool {
        // Check if any providers are available
        let implementations = self.implementations.read().await;
        !implementations.is_empty()
    }

    fn capabilities(&self) -> AICapabilities {
        self.get_aggregated_capabilities()
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        RoutingPreferences {
            priority: 90,                 // High priority for local models (privacy, cost)
            allows_forwarding: false,     // Local models don't forward
            handles_sensitive_data: true, // Local models are great for sensitive data
            geo_constraints: None,
            cost_tier: CostTier::Free, // Local models are free to use
            prefers_local: true,
            cost_sensitivity: 0.9,
            performance_priority: 0.6,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory function to create a local AI client with common providers
pub async fn create_local_ai_client(config: LocalAIConfig) -> Result<Arc<LocalAIClient>> {
    let client = Arc::new(LocalAIClient::new(config.clone())?);

    // Register Ollama provider if enabled
    if config.enable_ollama {
        if let Ok(ollama_provider) = ollama::OllamaProvider::new(config.ollama.clone()).await {
            client.register_provider(Arc::new(ollama_provider)).await?;
        } else {
            warn!("Failed to initialize Ollama provider, continuing without it");
        }
    }

    // Register native provider if enabled
    if config.enable_native {
        if let Ok(native_provider) = native::NativeProvider::new(config.native.clone()).await {
            client.register_provider(Arc::new(native_provider)).await?;
        } else {
            warn!("Failed to initialize native provider, continuing without it");
        }
    }

    Ok(client)
}

/// Convenience function to create a local AI client with default configuration
pub async fn create_default_local_client() -> Result<Arc<LocalAIClient>> {
    let config = LocalAIConfig::default();
    create_local_ai_client(config).await
}
