//! AI Agent Adapter implementation
//!
//! This module provides the integration between the AI Agent system and MCP.

use std::sync::{Arc, RwLock};
use tokio::sync::Mutex;
use tracing::{debug, info};
use uuid;
use std::collections::HashMap;

use squirrel_ai_tools::prelude::*;
use squirrel_ai_tools::openai::OpenAIClient;
use squirrel_ai_tools::anthropic::AnthropicClient;
use squirrel_ai_tools::gemini::GeminiClient;

use squirrel_mcp::adapter::MCPInterface;
use squirrel_mcp::Context;
use squirrel_mcp::protocol::types::{MCPMessage, MessageType};

use super::config::AIAgentConfig;
use super::types::{
    AgentCapabilities, AgentContext, AgentRequest, AgentResponse,
    Content, GenerationOptions, AnalysisOptions, CircuitBreakerConfig,
    CircuitBreakerState, UsageStatistics, ContentFormat, Usage, OperationType
};
use super::error::AIAgentError;
use crate::AIClientV2;

/// Circuit breaker for resilience
#[derive(Debug)]
struct CircuitBreaker {
    /// Current state
    state: CircuitBreakerState,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Failure count
    failure_count: u32,
    /// Last failure time
    last_failure_time: std::time::Instant,
    /// Half-open call count
    half_open_calls: u32,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            config,
            failure_count: 0,
            last_failure_time: std::time::Instant::now(),
            half_open_calls: 0,
        }
    }
    
    /// Get the current state of the circuit breaker
    pub fn state(&self) -> CircuitBreakerState {
        self.state
    }
    
    /// Reset the circuit breaker to closed state
    pub fn reset(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.half_open_calls = 0;
    }
    
    /// Record a failure and potentially transition to open state
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = std::time::Instant::now();
        
        // Check if we should open the circuit
        if self.state == CircuitBreakerState::Closed && self.failure_count as f64 >= self.config.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
    
    /// Record a success and potentially transition back to closed state
    pub fn record_success(&mut self) {
        if self.state == CircuitBreakerState::HalfOpen {
            self.half_open_calls += 1;
            
            if self.half_open_calls >= self.config.half_open_max_calls {
                self.reset();
            }
        }
    }
    
    /// Call a function through the circuit breaker
    pub async fn call<F, Fut, T>(&mut self, f: F) -> std::result::Result<T, AIAgentError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<T, AIAgentError>>,
    {
        match self.state {
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                let elapsed = self.last_failure_time.elapsed().as_millis() as u64;
                if elapsed > self.config.reset_timeout {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.half_open_calls = 0;
                } else {
                    return Err(AIAgentError::CircuitBreakerOpen("Circuit breaker open".to_string()));
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Only allow a limited number of calls in half-open state
                if self.half_open_calls >= self.config.half_open_max_calls {
                    return Err(AIAgentError::CircuitBreakerOpen("Circuit breaker half-open, max calls reached".to_string()));
                }
            }
            CircuitBreakerState::Closed => {}
        }
        
        // Make the call
        match f().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(err) => {
                self.record_failure();
                Err(err)
            }
        }
    }
}

/// AI Agent adapter status information
#[derive(Debug, Clone)]
pub struct AdapterStatus {
    /// Whether the adapter is initialized
    pub initialized: bool,
    
    /// Whether the adapter is operational
    pub operational: bool,
    
    /// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,
    
    /// Provider status
    pub provider_status: String,
    
    /// Resource usage as a percentage of limits
    pub resource_usage: f64,
}

/// Core adapter for AI Agent integration
pub struct AIAgentAdapter {
    /// Configuration
    config: AIAgentConfig,
    
    /// AI client
    client: Option<Arc<dyn AIClient>>,
    
    /// MCP adapter
    mcp: Option<Arc<dyn MCPInterface>>,
    
    /// Circuit breaker for resilience - use our own CircuitBreaker
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    
    /// Initialization state
    initialized: bool,
    
    /// Resource usage tracking
    resource_usage: Arc<Mutex<ResourceUsage>>,
    
    /// Cache for recent responses
    response_cache: Arc<Mutex<lru::LruCache<String, AgentResponse>>>,
}

/// Resource usage tracking
#[derive(Debug, Default)]
struct ResourceUsage {
    /// Total API calls made
    api_calls: usize,
    
    /// Total tokens used
    tokens_used: usize,
    
    /// Total processing time in milliseconds
    processing_time_ms: u64,
    
    /// Current rate of API calls per minute
    rate_per_minute: f64,
}

impl AIAgentAdapter {
    /// Create a new AI Agent adapter with the given configuration
    pub fn new(config: AIAgentConfig) -> Self {
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: config.circuit_breaker.failure_threshold as f64,
            reset_timeout: config.circuit_breaker.reset_timeout,
            half_open_max_calls: config.circuit_breaker.half_open_max_calls,
        });
        
        let cache_size = config.cache_size.unwrap_or(100);
        let cache_size = std::num::NonZeroUsize::new(cache_size).unwrap_or(std::num::NonZeroUsize::new(1).unwrap());
        
        Self {
            config,
            client: None,
            mcp: None,
            circuit_breaker: Arc::new(RwLock::new(circuit_breaker)),
            initialized: false,
            resource_usage: Arc::new(Mutex::new(ResourceUsage::default())),
            response_cache: Arc::new(Mutex::new(lru::LruCache::new(cache_size))),
        }
    }
    
    /// Set the MCP adapter
    pub fn with_mcp(mut self, mcp: Arc<dyn MCPInterface>) -> Self {
        self.mcp = Some(mcp);
        self
    }
    
    /// Initialize the adapter
    pub async fn initialize(&mut self) -> std::result::Result<(), AIAgentError> {
        info!("Initializing AI Agent adapter");
        
        if self.initialized {
            debug!("AI Agent adapter already initialized");
            return Ok(());
        }
        
        // Create AI client based on configured provider
        let client = create_ai_client(
            &self.config.provider,
            self.config.api_key.clone(),
        )?;
        
        self.client = Some(client);
        
        // Initialize MCP connection if provided
        if let Some(mcp) = &self.mcp {
            debug!("Initializing MCP connection");
            if !mcp.is_initialized() {
                return Err(AIAgentError::ConfigurationError(
                    "MCP adapter not initialized".to_string()
                ));
            }
        }
        
        self.initialized = true;
        info!("AI Agent adapter initialized successfully");
        
        Ok(())
    }
    
    /// Get the current status of the adapter
    pub async fn get_status(&self) -> AdapterStatus {
        let cb = self.circuit_breaker.read().unwrap();
        let usage = self.resource_usage.lock().await;
        
        AdapterStatus {
            initialized: self.initialized,
            operational: self.initialized && cb.state() != CircuitBreakerState::Open,
            circuit_breaker_state: cb.state(),
            provider_status: self.client.as_ref()
                .map(|c| c.provider_name().to_string())
                .unwrap_or_else(|| "Not initialized".to_string()),
            resource_usage: usage.rate_per_minute / self.config.resource_limits.max_requests_per_minute as f64,
        }
    }
    
    /// Process a request using the AI agent
    pub async fn process_request(&self, request: AgentRequest) -> std::result::Result<AgentResponse, AIAgentError> {
        if !self.initialized {
            return Err(AIAgentError::NotInitialized);
        }
        
        if let Some(client) = &self.client {
            // Generate a cache key
            let cache_key = format!("{}:{}:{}", 
                request.operation_type, 
                request.prompt,
                request.system_message.clone().unwrap_or_default());
                
            // Check cache first
            {
                let mut cache = self.response_cache.lock().await;
                if let Some(cached) = cache.get(&cache_key) {
                    return Ok(cached.clone());
                }
            }
            
            // Use circuit breaker for resilience
            let cb_result = {
                let mut cb = self.circuit_breaker.write().unwrap();
                
                // Wrap the async block in a closure that returns a Future
                cb.call(|| async {
                    let start_time = std::time::Instant::now();
                    
                    // Create chat request
                    let mut chat_request = ChatRequest::new();
                    
                    // Add system message if provided
                    if let Some(system) = &request.system_message {
                        chat_request = chat_request.add_system(system);
                        } else {
                        chat_request = chat_request.add_system("You are a helpful assistant.");
                    }
                    
                    // Add user message
                    chat_request = chat_request.add_user(&request.prompt);
                    
                    // Add parameters from options
                    let model_params = ModelParameters {
                        temperature: Some(request.options.temperature.unwrap_or(0.7)),
                        max_tokens: request.options.max_tokens,
                        top_p: Some(request.options.top_p.unwrap_or(1.0)),
                        ..Default::default()
                    };
                    chat_request = chat_request.with_parameters(model_params);
                    
                    // Make the API call
                    let response = client.chat(chat_request).await
                        .map_err(|e| AIAgentError::AIToolsError(e))?;
                    
                    // Update resource usage
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let tokens = response.usage.as_ref()
                        .map(|u| u.total_tokens)
                        .unwrap_or(0) as usize;
                        
                    self.update_resource_usage(tokens, elapsed).await;
                    
                    // Process the response
                    let agent_response = if let Some(message) = response.choices.first() {
                        AgentResponse {
                            id: uuid::Uuid::new_v4(),
                            request_id: request.id,
                            text: message.content.clone().unwrap_or_default(),
                            completion_time: elapsed,
                            format: ContentFormat::PlainText,
                            usage: Usage {
                                tokens: UsageStatistics {
                                    prompt_tokens: Some(response.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0)),
                                    completion_tokens: Some(response.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0)),
                                    total_tokens: Some(response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0)),
                                },
                                requests: 1,
                                billable_duration_ms: Some(elapsed),
                            },
                            metadata: HashMap::new(),
                        }
                    } else {
                        return Err(AIAgentError::ServiceError("No response from AI service".to_string()));
                    };
                    
                    // Cache the response
                    {
                        let mut cache = self.response_cache.lock().await;
                        cache.put(cache_key, agent_response.clone());
                    }
                    
                    Ok(agent_response)
                }).await
            };
            
            match cb_result {
                Ok(result) => Ok(result),
                Err(e) => Err(e),
            }
        } else {
            Err(AIAgentError::NotInitialized)
        }
    }
    
    /// Generate content using the AI agent
    pub async fn generate_content(&self, prompt: impl Into<String>) -> std::result::Result<String, AIAgentError> {
        let prompt_str = prompt.into();
        let request = AgentRequest {
            id: uuid::Uuid::new_v4(),
            prompt: prompt_str,
            system_message: None,
            options: GenerationOptions::default(),
            context: None,
            operation_type: OperationType::Generate,
            content: None,
        };
        
        let response = self.process_request(request).await?;
        Ok(response.text)
    }
    
    /// Analyze content using the AI agent
    pub async fn analyze_content(&self, content: Content, options: Option<AnalysisOptions>) -> std::result::Result<AgentResponse, AIAgentError> {
        let system_message = "You are an analysis assistant. Analyze the following content:".to_string();
            
        let mut gen_options = GenerationOptions::default();
        if let Some(opts) = &options {
            if let Some(model) = &opts.model {
                gen_options.model = Some(model.clone());
            }
            gen_options.temperature = opts.temperature;
            // AnalysisOptions doesn't have max_tokens field, use default
        }
        
        let request = AgentRequest {
            id: uuid::Uuid::new_v4(),
            prompt: content.data.clone(),
            system_message: Some(system_message),
            options: gen_options,
            context: None,
            operation_type: OperationType::Analyze,
            content: Some(content),
        };
        
        self.process_request(request).await
    }
    
    /// Process an MCP message using the AI agent
    pub async fn process_mcp_message(&self, message: &MCPMessage, context: &Context) -> std::result::Result<MCPMessage, AIAgentError> {
        if !self.initialized {
            return Err(AIAgentError::NotInitialized);
        }
        
        if let Some(_client) = &self.client {
            // Extract prompt from message payload
            let prompt = match message.payload.get("prompt") {
                Some(p) => p.as_str().unwrap_or_default().to_string(),
                None => return Err(AIAgentError::ValidationError("No prompt in message".to_string())),
            };
            
            // Extract optional system message
            let system_message = message.payload.get("system_message")
                .and_then(|s| s.as_str())
                .map(String::from);
                
            // Create request
            let request = AgentRequest {
                id: uuid::Uuid::new_v4(),
                prompt: prompt.clone(),
                system_message,
                options: GenerationOptions::default(),
                context: Some(AgentContext {
                    id: context.id,
                    working_directory: None,
                    user: None,
                    environment: HashMap::new(),
                    files: Vec::new(),
                    metadata: HashMap::new(),
                    capabilities: AgentCapabilities::default(),
                    account_id: context.id.to_string(),
                    session_id: context.id.to_string(),
                }),
                operation_type: OperationType::Execute,
                content: None,
            };
            
            // Process the request
            let response = self.process_request(request).await?;
            
            // Create response message
            let mut response_msg = MCPMessage::new(MessageType::Response, serde_json::json!({}));
            response_msg.payload = serde_json::json!({
                "content": response.text,
                "usage": response.usage,
            });
            
            // Set trace ID if present in original message
            if let Some(trace_id) = &message.trace_id {
                response_msg.trace_id = Some(trace_id.clone());
            }
            
            Ok(response_msg)
        } else {
            Err(AIAgentError::NotInitialized)
        }
    }
    
    /// Update resource usage tracking
    async fn update_resource_usage(&self, tokens: usize, elapsed_ms: u64) {
        let mut usage = self.resource_usage.lock().await;
        usage.api_calls += 1;
        usage.tokens_used += tokens;
        usage.processing_time_ms += elapsed_ms;
        
        // Simple rate calculation - could be improved with a moving window
        usage.rate_per_minute = usage.api_calls as f64 / 10.0 * 60.0; // Assuming 10 second window for simplicity
    }
    
    /// Create a new AI Agent adapter with an AIClientV2 implementation
    pub fn with_client_v2<T: AIClientV2 + 'static>(mut self, mut client: T) -> Self {
        // Setup callbacks for the client
        let mut callbacks = super::types::AIClientCallbacks {
            mcp_service: None,
            log_event: Some(Box::new(|event_type, message| {
                tracing::debug!("[{}] {}", event_type, message);
                Ok(())
            })),
            track_usage: Some(Box::new(|prompt_tokens, completion_tokens, total_tokens| {
                tracing::debug!("AI usage: {} prompt + {} completion = {} total tokens", 
                    prompt_tokens, completion_tokens, total_tokens);
                Ok(())
            })),
            check_rate_limit: None,
        };
        
        // Add MCP callback if we have one
        if let Some(mcp) = &self.mcp {
            let mcp_clone = mcp.clone();
            callbacks.mcp_service = Some(Box::new(move |msg| {
                let result = mcp_clone.send_message(msg);
                match result {
                    Ok(response) => Ok(response),
                    Err(e) => Err(anyhow::anyhow!("MCP error: {}", e)),
                }
            }));
        }
        
        // Register callbacks with the client
        client.register_callbacks(callbacks);
        
        // Create a wrapper to adapt the V2 client to the original AIClient trait
        let wrapped_client = super::types::AIClientWrapper::new(client);
        
        // Set the client
        self.client = Some(Arc::new(wrapped_client));
        
        self
    }
}

/// Helper function to create a new AI Agent adapter with default configuration
pub fn create_ai_agent_adapter() -> AIAgentAdapter {
    AIAgentAdapter::new(AIAgentConfig::default())
}

/// Helper function to create a new AI Agent adapter with custom configuration
pub fn create_ai_agent_adapter_with_config(config: AIAgentConfig) -> AIAgentAdapter {
    AIAgentAdapter::new(config)
}

/// Helper function to create a new AI Agent adapter with a V2 AI client
pub fn create_ai_agent_adapter_with_client_v2<T: AIClientV2 + 'static>(
    config: AIAgentConfig,
    client: T,
) -> AIAgentAdapter {
    AIAgentAdapter::new(config).with_client_v2(client)
}

/// Helper function to create an AI client
fn create_ai_client(provider: &str, api_key: String) -> std::result::Result<Arc<dyn AIClient>, AIAgentError> {
    match provider.to_lowercase().as_str() {
        "openai" => Ok(Arc::new(OpenAIClient::new(api_key))),
        "anthropic" => Ok(Arc::new(AnthropicClient::new(api_key))),
        "gemini" => Ok(Arc::new(GeminiClient::new(api_key))),
        _ => Err(AIAgentError::ConfigurationError(format!("Unsupported provider: {}", provider)))
    }
} 