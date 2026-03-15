// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal AI Coordinator Module
//!
//! This module provides the main AICoordinator that manages ALL AI systems
//! through a unified interface with intelligent routing and comprehensive
//! provider support.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Mutex};
use uuid::Uuid;
use tracing::{info, debug, warn, error, instrument};

use crate::error::types::{MCPError, Result};
use crate::enhanced::events::EventBroadcaster;
use crate::enhanced::providers::UniversalAIProvider;

// Re-export all the modules
pub mod types;
pub mod router;
pub mod providers;
pub mod tools;

// Re-export commonly used types
pub use types::*;
pub use router::AIRouter;
pub use providers::*;
pub use tools::*;

/// Universal AI Coordinator - manages ALL AI systems
pub struct AICoordinator {
    /// Provider registry for all AI systems
    providers: Arc<RwLock<HashMap<String, Arc<dyn UniversalAIProvider>>>>,
    
    /// Routing engine for intelligent model selection
    router: Arc<AIRouter>,
    
    /// Capability registry for all known models
    capabilities: Arc<RwLock<ModelCapabilityRegistry>>,
    
    /// Active sessions and their contexts
    sessions: Arc<RwLock<HashMap<String, AISession>>>,
    
    /// Event broadcaster
    event_broadcaster: Arc<EventBroadcaster>,
    
    /// Configuration
    config: AICoordinatorConfig,
    
    /// Metrics
    metrics: Arc<Mutex<AIMetrics>>,
    
    /// Plugin manager
    plugin_manager: Arc<dyn PluginManagerInterface>,
    
    /// Tool manager
    tool_manager: Arc<dyn ToolManagerInterface>,
}

impl AICoordinator {
    /// Create a new AI coordinator
    pub async fn new(config: AICoordinatorConfig) -> Result<Self> {
        let router = Arc::new(AIRouter::new(config.routing.clone()));
        let capabilities = Arc::new(RwLock::new(ModelCapabilityRegistry::new()));
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let event_broadcaster = Arc::new(EventBroadcaster::new());
        let metrics = Arc::new(Mutex::new(AIMetrics::new()));
        let plugin_manager = Arc::new(PluginManager::new());
        let tool_manager = Arc::new(ToolManager::new());
        
        let coordinator = Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            router,
            capabilities,
            sessions,
            event_broadcaster,
            config,
            metrics,
            plugin_manager,
            tool_manager,
        };
        
        // Initialize standard providers
        coordinator.initialize_standard_providers().await?;
        
        Ok(coordinator)
    }
    
    /// Initialize standard providers
    async fn initialize_standard_providers(&self) -> Result<()> {
        info!("Initializing standard AI providers");
        
        // Register cloud providers
        self.register_cloud_providers().await?;
        
        // Register local providers
        self.register_local_providers().await?;
        
        // Register aggregator providers
        self.register_aggregator_providers().await?;
        
        // Register hub providers
        self.register_hub_providers().await?;
        
        // Register custom providers
        self.register_custom_providers().await?;
        
        Ok(())
    }
    
    /// Register cloud providers
    async fn register_cloud_providers(&self) -> Result<()> {
        if let Some(api_key) = &self.config.openai_api_key {
            let provider = Arc::new(OpenAIProvider::new(api_key.clone()));
            self.register_provider("openai", provider).await?;
        }
        
        if let Some(api_key) = &self.config.anthropic_api_key {
            let provider = Arc::new(AnthropicProvider::new(api_key.clone()));
            self.register_provider("anthropic", provider).await?;
        }
        
        if let Some(api_key) = &self.config.gemini_api_key {
            let provider = Arc::new(GeminiProvider::new(api_key.clone()));
            self.register_provider("gemini", provider).await?;
        }
        
        Ok(())
    }
    
    /// Register local providers (capability-based, vendor-agnostic)
    async fn register_local_providers(&self) -> Result<()> {
        if self.config.enable_local_server {
            let provider = Arc::new(LocalServerProvider::new(self.config.local_server_config.clone()));
            self.register_provider("local", provider).await?;
        }
        
        if self.config.enable_native {
            let provider = Arc::new(NativeProvider::new(self.config.native_config.clone()));
            self.register_provider("native", provider).await?;
        }
        
        Ok(())
    }
    
    /// Register aggregator providers
    async fn register_aggregator_providers(&self) -> Result<()> {
        if let Some(api_key) = &self.config.openrouter_api_key {
            let provider = Arc::new(OpenRouterProvider::new(api_key.clone()));
            self.register_provider("openrouter", provider).await?;
        }
        
        Ok(())
    }
    
    /// Register hub providers
    async fn register_hub_providers(&self) -> Result<()> {
        if self.config.enable_huggingface {
            let provider = Arc::new(HuggingFaceProvider::new(self.config.huggingface_config.clone()));
            self.register_provider("huggingface", provider).await?;
        }
        
        Ok(())
    }
    
    /// Register custom providers
    async fn register_custom_providers(&self) -> Result<()> {
        for (name, config) in &self.config.custom_providers {
            let provider = Arc::new(CustomProvider::new(name.clone(), config.clone()));
            self.register_provider(name, provider).await?;
        }
        
        Ok(())
    }
    
    /// Register a provider
    pub async fn register_provider(&self, name: &str, provider: Arc<dyn UniversalAIProvider>) -> Result<()> {
        let mut providers = self.providers.write().await;
        providers.insert(name.to_string(), provider);
        
        // Update capabilities registry
        self.update_capabilities_registry(name).await?;
        
        info!("Registered provider: {}", name);
        Ok(())
    }
    
    /// Process an AI request
    #[instrument(skip(self, request))]
    pub async fn process_request(&self, request: UniversalAIRequest) -> Result<UniversalAIResponse> {
        let start_time = std::time::Instant::now();
        
        // Route the request to appropriate provider
        let provider_name = self.router.route_request(&request, &self.providers).await?;
        
        // Get the provider
        let provider = {
            let providers = self.providers.read().await;
            providers.get(&provider_name)
                .ok_or_else(|| MCPError::Configuration(
                    format!("Provider '{}' not found", provider_name)
                ))?
                .clone()
        };
        
        // Process the request
        let response = provider.process_request(request.clone()).await?;
        
        // Update metrics
        self.update_metrics(&request, &response, &provider_name).await;
        
        // Update router performance
        let performance = PerformanceMetrics {
            avg_latency: start_time.elapsed(),
            success_rate: 1.0,
            cost_per_request: Some(response.cost),
            quality_score: None,
        };
        self.router.update_performance(&provider_name, performance).await?;
        
        Ok(response)
    }
    
    /// Create a new session
    pub async fn create_session(&self, preferences: UserPreferences) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let session = AISession {
            id: session_id.clone(),
            active_models: Vec::new(),
            history: Vec::new(),
            preferences,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    /// List all available models
    pub async fn list_all_models(&self) -> Result<Vec<ModelInfo>> {
        let providers = self.providers.read().await;
        let mut all_models = Vec::new();
        
        for provider in providers.values() {
            let models = provider.list_models().await?;
            all_models.extend(models);
        }
        
        Ok(all_models)
    }
    
    /// Get system metrics
    pub async fn get_metrics(&self) -> AIMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
    
    /// Update capabilities registry
    async fn update_capabilities_registry(&self, provider_name: &str) -> Result<()> {
        // Implementation would update the registry with provider capabilities
        debug!("Updated capabilities registry for provider: {}", provider_name);
        Ok(())
    }
    
    /// Update metrics
    async fn update_metrics(&self, request: &UniversalAIRequest, response: &UniversalAIResponse, provider: &str) {
        let mut metrics = self.metrics.lock().await;
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        metrics.total_cost += response.cost;
        
        // Update average latency
        let total_latency = metrics.avg_latency.as_millis() as u64 * (metrics.total_requests - 1) + response.duration.as_millis() as u64;
        metrics.avg_latency = Duration::from_millis(total_latency / metrics.total_requests);
        
        debug!("Updated metrics for provider: {}", provider);
    }
    
    /// Get provider by name
    pub async fn get_provider(&self, name: &str) -> Result<Option<Arc<dyn UniversalAIProvider>>> {
        let providers = self.providers.read().await;
        Ok(providers.get(name).cloned())
    }
    
    /// List available providers
    pub async fn list_providers(&self) -> Result<Vec<String>> {
        let providers = self.providers.read().await;
        Ok(providers.keys().cloned().collect())
    }
    
    /// Remove a provider
    pub async fn remove_provider(&self, name: &str) -> Result<bool> {
        let mut providers = self.providers.write().await;
        Ok(providers.remove(name).is_some())
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<AISession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }
    
    /// Update session
    pub async fn update_session(&self, session: AISession) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }
    
    /// Remove session
    pub async fn remove_session(&self, session_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        Ok(sessions.remove(session_id).is_some())
    }
    
    /// Get router
    pub fn get_router(&self) -> &AIRouter {
        &self.router
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &AICoordinatorConfig {
        &self.config
    }
    
    /// Health check
    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let providers = self.providers.read().await;
        let mut results = HashMap::new();
        
        for (name, provider) in providers.iter() {
            let healthy = provider.health_check().await.unwrap_or(false);
            results.insert(name.clone(), healthy);
        }
        
        Ok(results)
    }
    
    /// Get provider capabilities
    pub async fn get_provider_capabilities(&self, provider_name: &str) -> Result<Vec<String>> {
        let providers = self.providers.read().await;
        let provider = providers.get(provider_name)
            .ok_or_else(|| MCPError::Configuration(
                format!("Provider '{}' not found", provider_name)
            ))?;
        
        Ok(provider.get_capabilities())
    }
    
    /// Stream request (placeholder)
    pub async fn stream_request(&self, request: UniversalAIRequest) -> Result<UniversalAIStream> {
        // Route the request to appropriate provider
        let provider_name = self.router.route_request(&request, &self.providers).await?;
        
        // Get the provider
        let provider = {
            let providers = self.providers.read().await;
            providers.get(&provider_name)
                .ok_or_else(|| MCPError::Configuration(
                    format!("Provider '{}' not found", provider_name)
                ))?
                .clone()
        };
        
        // Stream the request
        provider.stream_request(request).await
    }
    
    /// Estimate cost for a request
    pub async fn estimate_cost(&self, request: &UniversalAIRequest) -> Result<CostEstimate> {
        // Route the request to appropriate provider
        let provider_name = self.router.route_request(request, &self.providers).await?;
        
        // Get the provider
        let provider = {
            let providers = self.providers.read().await;
            providers.get(&provider_name)
                .ok_or_else(|| MCPError::Configuration(
                    format!("Provider '{}' not found", provider_name)
                ))?
                .clone()
        };
        
        // Estimate cost
        provider.estimate_cost(request).await
    }
} 