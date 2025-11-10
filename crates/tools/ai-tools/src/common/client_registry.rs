//! AI client registry and routing
//!
//! This module provides the AIRouterClient which manages multiple AI providers
//! and routes requests to the appropriate provider based on various strategies.

use crate::router::RoutingStrategy;
use crate::AIToolsConfig;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_error::tools::AIToolsError;

// Import types from capability module
use super::capability::{AICapabilities, AITask, RoutingPreferences};
use super::providers::{create_provider, AICapability, AIProvider};
use super::{AIClient, ChatRequest, ChatResponse, ChatResponseStream};
use crate::common::capability::CostTier;

/// AI router client that manages multiple providers
#[derive(Debug)]
pub struct AIRouterClient {
    /// Configuration for the AI tools
    config: Arc<AIToolsConfig>,
    /// Registered AI providers
    providers: Arc<RwLock<HashMap<String, Arc<dyn AIProvider>>>>,
    /// Request counter for load balancing
    request_counter: Arc<std::sync::atomic::AtomicU64>,
    /// Client registry for fallback
    client_registry: Arc<RwLock<HashMap<String, Arc<dyn AIClient>>>>,
}

impl AIRouterClient {
    /// Create a new AI router client
    pub fn new(config: AIToolsConfig) -> Self {
        Self {
            config: Arc::new(config),
            providers: Arc::new(RwLock::new(HashMap::new())),
            request_counter: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            client_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the router with configured providers
    pub async fn initialize(&self) -> crate::Result<()> {
        let providers = self.providers.write().await;

        // Initialize OpenAI provider if configured
        if let Some(openai_config) = self.config.providers.get("openai") {
            match create_provider("openai", openai_config.clone()) {
                Ok(provider) => {
                    self.providers
                        .write()
                        .await
                        .insert("openai".to_string(), Arc::from(provider));
                    info!("OpenAI provider initialized successfully");
                }
                Err(e) => {
                    error!("Failed to initialize OpenAI provider: {}", e);
                }
            }
        }

        // Initialize Anthropic provider if configured
        if let Some(anthropic_config) = self.config.providers.get("anthropic") {
            match create_provider("anthropic", anthropic_config.clone()) {
                Ok(provider) => {
                    self.providers
                        .write()
                        .await
                        .insert("anthropic".to_string(), Arc::from(provider));
                    info!("Anthropic provider initialized successfully");
                }
                Err(e) => {
                    error!("Failed to initialize Anthropic provider: {}", e);
                }
            }
        }

        // Initialize Ollama provider if configured
        if let Some(ollama_config) = self.config.providers.get("ollama") {
            match create_provider("ollama", ollama_config.clone()) {
                Ok(provider) => {
                    self.providers
                        .write()
                        .await
                        .insert("ollama".to_string(), Arc::from(provider));
                    info!("Ollama provider initialized successfully");
                }
                Err(e) => {
                    error!("Failed to initialize Ollama provider: {}", e);
                }
            }
        }

        if providers.is_empty() {
            return Err(AIToolsError::Configuration(
                "No providers configured in ai-tools config. Add at least one provider (openai, anthropic, ollama).".to_string(),
            ).into());
        }

        info!("AI router initialized with {} providers", providers.len());
        Ok(())
    }

    /// Select the best provider for a request
    async fn select_provider(&self, request: &ChatRequest) -> crate::Result<Arc<dyn AIProvider>> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(AIToolsError::Configuration(
                "No providers available for routing. Initialize providers first.".to_string(),
            ).into());
        }

        // If a specific model is requested, try to find the provider that supports it
        if let Some(model) = &request.model {
            // Check if the model indicates a specific provider
            if model.starts_with("gpt-") {
                if let Some(provider) = providers.get("openai") {
                    return Ok(Arc::clone(provider));
                }
            } else if model.starts_with("claude-") {
                if let Some(provider) = providers.get("anthropic") {
                    return Ok(Arc::clone(provider));
                }
            } else if model.starts_with("llama") || model.starts_with("mistral") {
                if let Some(provider) = providers.get("ollama") {
                    return Ok(Arc::clone(provider));
                }
            }
        }

        // Use intelligent routing strategy from config - Universal Primal Architecture
        let strategy = match self.config.routing_strategy.as_str() {
            "round_robin" => RoutingStrategy::RoundRobin,
            "random" => RoutingStrategy::Random,
            "health_based" => RoutingStrategy::LowestLatency, // Map to available variant
            "cost_optimized" => RoutingStrategy::LowestCost,
            "performance_based" => RoutingStrategy::BestFit,
            _ => RoutingStrategy::FirstMatch, // Default fallback
        };

        // Implement intelligent AI provider selection based on strategy
        debug!(
            "🐿️ Using AI routing strategy: {:?} for Universal Primal Architecture",
            strategy
        );

        match strategy {
            RoutingStrategy::RoundRobin => {
                info!("🔄 Using round-robin AI provider selection");
                self.select_round_robin(&providers).await
            }
            RoutingStrategy::Random => {
                info!("🎲 Using random AI provider selection");
                self.select_random(&providers).await
            }
            RoutingStrategy::LowestLatency => {
                info!("🏥 Using health-based AI provider selection");
                self.select_health_based(&providers).await
            }
            RoutingStrategy::LowestCost => {
                info!("💰 Using cost-optimized AI provider selection");
                self.select_cost_optimized(&providers).await
            }
            RoutingStrategy::BestFit => {
                info!("⚡ Using performance-based AI provider selection");
                self.select_performance_based(&providers).await
            }
            RoutingStrategy::HighestPriority => {
                info!("🏆 Using priority-based AI provider selection");
                // Priority-based selection: prefer providers in order of capability/reliability
                // OpenAI (most reliable) -> Anthropic (high quality) -> Ollama (cost effective)
                let priority_order = ["openai", "anthropic", "ollama"];
                for provider_name in &priority_order {
                    if let Some(provider) = providers.get(*provider_name) {
                        debug!("Selected priority provider: {}", provider_name);
                        return Ok(Arc::clone(provider));
                    }
                }
                // If none of the priority providers are available, fall back to first available
                if let Some(provider) = providers.values().next() {
                    warn!("No priority providers available, using fallback");
                    Ok(provider.clone())
                } else {
                    Err(AIToolsError::Provider(
                        "No providers available".to_string(),
                    ).into())
                }
            }
            RoutingStrategy::FirstMatch => {
                info!("📍 Using first available AI provider (fallback)");
                // Simple fallback: return first available provider
                if let Some(provider) = providers.values().next() {
                    Ok(provider.clone())
                } else {
                    Err(AIToolsError::Provider(
                        "No providers available".to_string(),
                    ).into())
                }
            }
        }
    }

    /// Select provider using round-robin strategy
    async fn select_round_robin(
        &self,
        providers: &HashMap<String, Arc<dyn AIProvider>>,
    ) -> crate::Result<Arc<dyn AIProvider>> {
        let counter = self
            .request_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let provider_names: Vec<&String> = providers.keys().collect();
        let selected_name = provider_names[counter as usize % provider_names.len()];

        Ok(Arc::clone(providers.get(selected_name).unwrap()))
    }

    /// Select provider using random strategy
    async fn select_random(
        &self,
        providers: &HashMap<String, Arc<dyn AIProvider>>,
    ) -> crate::Result<Arc<dyn AIProvider>> {
        use rand::seq::SliceRandom;
        let provider_names: Vec<&String> = providers.keys().collect();
        let selected_name = provider_names
            .choose(&mut rand::thread_rng())
            .ok_or_else(|| {
                AIToolsError::Configuration("No providers available for routing. Add providers to config.".to_string())
            })?;

        Ok(Arc::clone(providers.get(*selected_name).unwrap()))
    }

    /// Select provider based on health checks
    async fn select_health_based(
        &self,
        providers: &HashMap<String, Arc<dyn AIProvider>>,
    ) -> crate::Result<Arc<dyn AIProvider>> {
        // Check health of all providers
        for (name, provider) in providers {
            if provider.health_check().await {
                debug!("Selected healthy provider: {}", name);
                return Ok(Arc::clone(provider));
            }
        }

        // If no healthy providers, fall back to first available
        warn!("No healthy providers found, falling back to first available");
        if let Some(provider) = providers.values().next() {
            Ok(Arc::clone(provider))
        } else {
            Err(AIToolsError::Configuration(
                "No providers available".to_string(),
            ).into())
        }
    }

    /// Select provider based on cost optimization
    async fn select_cost_optimized(
        &self,
        providers: &HashMap<String, Arc<dyn AIProvider>>,
    ) -> crate::Result<Arc<dyn AIProvider>> {
        // Simple cost-based selection (Ollama < OpenAI < Anthropic)
        let cost_preference = ["ollama", "openai", "anthropic"];

        for provider_name in &cost_preference {
            if let Some(provider) = providers.get(*provider_name) {
                debug!("Selected cost-optimized provider: {}", provider_name);
                return Ok(Arc::clone(provider));
            }
        }

        // Fall back to any available provider
        if let Some(provider) = providers.values().next() {
            Ok(Arc::clone(provider))
        } else {
            Err(AIToolsError::Configuration(
                "No providers available".to_string(),
            ).into())
        }
    }

    /// Select provider based on performance
    async fn select_performance_based(
        &self,
        providers: &HashMap<String, Arc<dyn AIProvider>>,
    ) -> crate::Result<Arc<dyn AIProvider>> {
        // Simple performance-based selection (OpenAI > Anthropic > Ollama)
        let performance_preference = ["openai", "anthropic", "ollama"];

        for provider_name in &performance_preference {
            if let Some(provider) = providers.get(*provider_name) {
                debug!("Selected performance-optimized provider: {}", provider_name);
                return Ok(Arc::clone(provider));
            }
        }

        // Fall back to any available provider
        if let Some(provider) = providers.values().next() {
            Ok(Arc::clone(provider))
        } else {
            Err(AIToolsError::Configuration(
                "No providers available".to_string(),
            ).into())
        }
    }

    /// Process a request with retry logic
    async fn process_with_retry(&self, request: &ChatRequest) -> crate::Result<ChatResponse> {
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 0..max_retries {
            match self.select_provider(request).await {
                Ok(provider) => {
                    match provider.process_chat(request).await {
                        Ok(response) => {
                            if attempt > 0 {
                                info!("Request succeeded on attempt {}", attempt + 1);
                            }
                            return Ok(response);
                        }
                        Err(e) => {
                            error!(
                                "Provider {} failed on attempt {}: {}",
                                provider.name(),
                                attempt + 1,
                                e
                            );
                            last_error = Some(e);

                            // Add exponential backoff
                            if attempt < max_retries - 1 {
                                let delay = std::time::Duration::from_millis(
                                    100 * (2_u64.pow(attempt as u32)),
                                );
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to select provider on attempt {}: {}",
                        attempt + 1,
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AIToolsError::Provider("All provider retries exhausted. Check provider availability and configuration.".to_string()).into()
        }))
    }

    /// Add a client to the registry
    pub async fn register_client(&self, name: String, client: Arc<dyn AIClient>) {
        let mut registry = self.client_registry.write().await;
        registry.insert(name.clone(), client);
        info!("Registered AI client: {}", name);
    }

    /// Remove a client from the registry
    pub async fn unregister_client(&self, name: &str) {
        let mut registry = self.client_registry.write().await;
        if registry.remove(name).is_some() {
            info!("Unregistered AI client: {}", name);
        }
    }

    /// Get a client from the registry
    pub async fn get_client(&self, name: &str) -> Option<Arc<dyn AIClient>> {
        let registry = self.client_registry.read().await;
        registry.get(name).cloned()
    }

    /// List all registered clients
    pub async fn list_clients(&self) -> Vec<String> {
        let registry = self.client_registry.read().await;
        registry.keys().cloned().collect()
    }

    /// Get provider statistics
    pub async fn get_provider_stats(&self) -> HashMap<String, ProviderStats> {
        let providers = self.providers.read().await;
        let mut stats = HashMap::new();

        for (name, provider) in providers.iter() {
            let is_healthy = provider.health_check().await;
            stats.insert(
                name.clone(),
                ProviderStats {
                    name: name.clone(),
                    healthy: is_healthy,
                    capabilities: provider.capabilities().to_vec(),
                },
            );
        }

        stats
    }

    /// Check if a specific provider is available
    pub async fn is_provider_available(&self, name: &str) -> bool {
        let providers = self.providers.read().await;
        providers.contains_key(name)
    }

    /// Get the list of available providers
    pub async fn available_providers(&self) -> Vec<String> {
        let providers = self.providers.read().await;
        providers.keys().cloned().collect()
    }

    /// Force a specific provider for testing
    pub async fn force_provider(&self, name: &str) -> crate::Result<Arc<dyn AIProvider>> {
        let providers = self.providers.read().await;
        providers.get(name).cloned().ok_or_else(|| {
            AIToolsError::Configuration(format!("Provider '{}' not found in registry. Check available providers with list_providers().", name)).into()
        })
    }
}

/// Statistics for a provider
#[derive(Debug, Clone)]
pub struct ProviderStats {
    /// Provider name
    pub name: String,
    /// Whether the provider is healthy
    pub healthy: bool,
    /// Provider capabilities
    pub capabilities: Vec<AICapability>,
}

#[async_trait]
impl AIClient for AIRouterClient {
    fn provider_name(&self) -> &str {
        "router"
    }

    fn default_model(&self) -> &str {
        "auto"
    }

    async fn get_capabilities(&self, _model: &str) -> crate::Result<AICapabilities> {
        // Return combined capabilities of all providers
        Ok(AICapabilities::default())
    }

    async fn list_models(&self) -> crate::Result<Vec<String>> {
        let providers = self.providers.read().await;
        let mut models = Vec::new();

        // Add common models from each provider
        if providers.contains_key("openai") {
            models.extend(vec!["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"]);
        }
        if providers.contains_key("anthropic") {
            models.extend(vec!["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"]);
        }
        if providers.contains_key("ollama") {
            models.extend(vec!["llama2", "mistral", "codellama"]);
        }

        Ok(models.into_iter().map(|s| s.to_string()).collect())
    }

    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        debug!(
            "Router processing chat request with {} messages",
            request.messages.len()
        );

        // Validate request
        if request.messages.is_empty() {
            return Err(AIToolsError::InvalidRequest(
                "No messages provided".to_string(),
            ).into());
        }

        // Process with retry logic
        match self.process_with_retry(&request).await {
            Ok(response) => {
                debug!(
                    "Router successfully processed request, got {} choices",
                    response.choices.len()
                );
                Ok(response)
            }
            Err(e) => {
                error!("Router failed to process request: {}", e);
                Err(e)
            }
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream> {
        // For now, convert streaming to non-streaming
        // In a real implementation, this would handle streaming properly
        let response = self.chat(request).await?;

        // Convert to stream (simplified implementation)
        let stream = futures::stream::once(async move {
            Ok(super::ChatResponseChunk {
                id: response.id,
                model: response.model,
                choices: response
                    .choices
                    .into_iter()
                    .map(|choice| super::ChatChoiceChunk {
                        index: choice.index,
                        delta: super::ChatMessage {
                            role: choice.role,
                            content: choice.content,
                            name: None,
                            tool_calls: choice.tool_calls,
                            tool_call_id: None,
                        },
                        finish_reason: choice.finish_reason,
                    })
                    .collect(),
            })
        });

        Ok(Box::pin(stream))
    }

    async fn is_available(&self) -> bool {
        let providers = self.providers.read().await;

        // Check if any provider is healthy
        for provider in providers.values() {
            if provider.health_check().await {
                return true;
            }
        }

        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn capabilities(&self) -> AICapabilities {
        // Return combined capabilities of all providers
        AICapabilities::default()
    }

    fn can_handle_task(&self, _task: &AITask) -> bool {
        // The router can handle any task if at least one provider can handle it
        // This is a simplified implementation
        true
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        RoutingPreferences {
            priority: 50,
            allows_forwarding: true,
            handles_sensitive_data: true,
            geo_constraints: None,
            cost_tier: CostTier::Medium,
            prefers_local: false,
            cost_sensitivity: 0.5,
            performance_priority: 0.7,
        }
    }
}

/// Client registry for managing AI clients
#[derive(Debug, Default)]
pub struct ClientRegistry {
    /// Registered clients
    clients: RwLock<HashMap<String, Arc<dyn AIClient>>>,
}

impl ClientRegistry {
    /// Create a new client registry
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Register a client
    pub async fn register(&self, name: String, client: Arc<dyn AIClient>) {
        let mut clients = self.clients.write().await;
        clients.insert(name.clone(), client);
        info!("Registered client: {}", name);
    }

    /// Unregister a client
    pub async fn unregister(&self, name: &str) {
        let mut clients = self.clients.write().await;
        if clients.remove(name).is_some() {
            info!("Unregistered client: {}", name);
        }
    }

    /// Get a client
    pub async fn get(&self, name: &str) -> Option<Arc<dyn AIClient>> {
        let clients = self.clients.read().await;
        clients.get(name).cloned()
    }

    /// List all client names
    pub async fn list(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Get client count
    pub async fn count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }

    /// Check if a client is registered
    pub async fn contains(&self, name: &str) -> bool {
        let clients = self.clients.read().await;
        clients.contains_key(name)
    }

    /// Clear all clients
    pub async fn clear(&self) {
        let mut clients = self.clients.write().await;
        clients.clear();
        info!("Cleared all registered clients");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_registry() {
        let registry = ClientRegistry::new();

        // Test empty registry
        assert_eq!(registry.count().await, 0);
        assert!(registry.list().await.is_empty());

        // Register a mock client
        let mock_client = Arc::new(crate::common::MockAIClient::new());
        registry
            .register("test".to_string(), mock_client.clone())
            .await;

        // Test registry with one client
        assert_eq!(registry.count().await, 1);
        assert!(registry.contains("test").await);
        assert_eq!(registry.list().await, vec!["test"]);

        // Get the client
        let retrieved = registry.get("test").await;
        assert!(retrieved.is_some());

        // Unregister the client
        registry.unregister("test").await;
        assert_eq!(registry.count().await, 0);
        assert!(!registry.contains("test").await);
    }

    #[tokio::test]
    async fn test_ai_router_client_creation() {
        let config = AIToolsConfig {
            default_provider: "openai".to_string(),
            providers: HashMap::new(),
            request_timeout: 30,
            max_retries: 3,
            enable_logging: true,
            routing_strategy: "round_robin".to_string(),
        };

        let router = AIRouterClient::new(config);
        assert_eq!(router.provider_name(), "router");
        assert_eq!(router.default_model(), "auto");
    }

    #[tokio::test]
    async fn test_provider_stats() {
        let config = AIToolsConfig {
            default_provider: "openai".to_string(),
            providers: HashMap::new(),
            request_timeout: 30,
            max_retries: 3,
            enable_logging: true,
            routing_strategy: "round_robin".to_string(),
        };

        let router = AIRouterClient::new(config);
        let stats = router.get_provider_stats().await;
        assert!(stats.is_empty()); // No providers configured
    }
}
