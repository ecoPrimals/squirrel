//! MCP-AI Tools adapter implementation
//!
//! Adapter for integrating AI tools with the MCP system.

use squirrel_mcp::MCPInterface;
use squirrel_mcp::MCPError;
use crate::mcp_ai_tools::config::{McpAiToolsConfig, ProviderSettings};
use crate::mcp_ai_tools::types::{AiMessageType, AiToolInvocation, AiToolResponse, AiToolResponseStatus};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error, info};
use uuid::Uuid;

/// MCP-AI Tools adapter errors
#[derive(Debug, Error)]
pub enum McpAiToolsAdapterError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Provider error
    #[error("Provider error for {provider}: {message}")]
    Provider {
        /// Provider name
        provider: String,
        /// Error message
        message: String,
    },
    
    /// Tool invocation error
    #[error("Tool invocation error for {tool}: {message}")]
    ToolInvocation {
        /// Tool name
        tool: String,
        /// Error message
        message: String,
    },
    
    /// Timeout error
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),
    
    /// MCP adapter error
    #[error("MCP adapter error: {0}")]
    McpAdapter(#[from] MCPError),
    
    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Provider not found
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    
    /// Model not found
    #[error("Model not found for provider {provider}: {model}")]
    ModelNotFound {
        /// Provider name
        provider: String,
        /// Model name
        model: String,
    },
    
    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

/// MCP-AI Tools adapter
pub struct McpAiToolsAdapter {
    /// MCP adapter
    mcp_adapter: Arc<dyn MCPInterface>,
    
    /// AI tools configuration
    config: McpAiToolsConfig,
    
    /// Conversation history
    history: Arc<Mutex<HashMap<String, Vec<ConversationMessage>>>>,
    
    /// Registered tools
    tools: Arc<Mutex<HashMap<String, ToolDefinition>>>,
}

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message ID
    pub id: String,
    
    /// Message content
    pub content: String,
    
    /// Message type
    pub message_type: AiMessageType,
    
    /// Message timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Tool invocation if applicable
    pub tool_invocation: Option<AiToolInvocation>,
    
    /// Tool response if applicable
    pub tool_response: Option<AiToolResponse>,
}

/// Tool definition
#[derive(Debug, Clone)]
struct ToolDefinition {
    /// Tool name
    name: String,
    
    /// Tool description
    description: String,
    
    /// Tool parameters schema
    parameters_schema: serde_json::Value,
    
    /// Tool handler
    handler: Arc<dyn ToolHandler>,
}

/// Tool handler trait
#[async_trait]
pub trait ToolHandler: Send + Sync + std::fmt::Debug {
    /// Handle a tool invocation
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError>;
}

/// Callbacks available to tools for adapter interaction
#[derive(Clone)]
pub struct ToolCallbacks {
    /// Add a message to a conversation
    pub add_message: Box<dyn Fn(&str, &str, AiMessageType) -> Result<String, McpAiToolsAdapterError> + Send + Sync>,
    
    /// Get conversation history
    pub get_conversation: Box<dyn Fn(&str) -> Result<Vec<ConversationMessage>, McpAiToolsAdapterError> + Send + Sync>,
    
    /// Send a message to MCP
    pub send_mcp_message: Box<dyn Fn(&str) -> Result<String, McpAiToolsAdapterError> + Send + Sync>,
}

/// Tool handler trait version 2 with improved thread safety
///
/// This version of the trait doesn't require passing an adapter reference in the handle method,
/// which avoids potential Send/Sync issues. Instead, tools can register callbacks for adapter
/// functionality they need.
#[async_trait]
pub trait ToolHandlerV2: Send + Sync + std::fmt::Debug {
    /// Handle a tool invocation without adapter dependency
    async fn handle(
        &self,
        invocation: AiToolInvocation,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError>;
    
    /// Register callbacks for adapter interaction
    fn register_callbacks(&mut self, callbacks: ToolCallbacks) {
        // Default empty implementation
        let _ = callbacks; // Suppress unused variable warning
    }
}

impl McpAiToolsAdapter {
    /// Create a new MCP-AI Tools adapter with default configuration
    pub fn new(mcp_adapter: Arc<dyn MCPInterface>) -> Self {
        Self::with_config(mcp_adapter, McpAiToolsConfig::default())
    }
    
    /// Create a new MCP-AI Tools adapter with specific configuration
    pub fn with_config(mcp_adapter: Arc<dyn MCPInterface>, config: McpAiToolsConfig) -> Self {
        Self {
            mcp_adapter,
            config,
            history: Arc::new(Mutex::new(HashMap::new())),
            tools: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register a tool with the adapter
    pub fn register_tool<H>(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
        handler: H,
    ) -> Result<(), McpAiToolsAdapterError>
    where
        H: ToolHandler + 'static,
    {
        let name = name.into();
        let tool_def = ToolDefinition {
            name: name.clone(),
            description: description.into(),
            parameters_schema,
            handler: Arc::new(handler),
        };
        
        let mut tools = self.tools.lock().unwrap();
        tools.insert(name, tool_def);
        Ok(())
    }
    
    /// Register a v2 tool with the adapter
    ///
    /// This method registers a tool that implements the ToolHandlerV2 trait,
    /// which provides improved thread safety through callback registration
    /// rather than direct adapter dependency.
    pub fn register_tool_v2<H>(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters_schema: serde_json::Value,
        mut handler: H,
    ) -> Result<(), McpAiToolsAdapterError>
    where
        H: ToolHandlerV2 + 'static,
    {
        // Create callbacks
        let self_clone = self.clone();
        let callbacks = ToolCallbacks {
            add_message: Box::new(move |conversation_id, content, message_type| {
                self_clone.add_message(conversation_id, content, message_type)
            }),
            
            get_conversation: Box::new(move |conversation_id| {
                self_clone.get_conversation(conversation_id)
            }),
            
            send_mcp_message: Box::new(move |message| {
                match self_clone.mcp_adapter.send_message(message) {
                    Ok(response) => Ok(response),
                    Err(err) => Err(McpAiToolsAdapterError::MCPError(format!("{:?}", err))),
                }
            }),
        };
        
        // Register callbacks with handler
        handler.register_callbacks(callbacks);
        
        // Create tool definition
        let name = name.into();
        let tool_def = ToolDefinition {
            name: name.clone(),
            description: description.into(),
            parameters_schema,
            handler: Arc::new(ToolHandlerWrapper::new(handler)),
        };
        
        // Register tool
        let mut tools = self.tools.lock().unwrap();
        tools.insert(name, tool_def);
        Ok(())
    }
    
    /// Create a new conversation
    pub fn create_conversation(&self) -> String {
        let conversation_id = Uuid::new_v4().to_string();
        let mut history = self.history.lock().unwrap();
        history.insert(conversation_id.clone(), Vec::new());
        conversation_id
    }
    
    /// Add a message to a conversation
    pub fn add_message(
        &self,
        conversation_id: &str,
        content: impl Into<String>,
        message_type: AiMessageType,
    ) -> Result<String, McpAiToolsAdapterError> {
        let message = ConversationMessage {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            message_type,
            timestamp: chrono::Utc::now(),
            tool_invocation: None,
            tool_response: None,
        };
        
        let mut history = self.history.lock().unwrap();
        if let Some(conversation) = history.get_mut(conversation_id) {
            conversation.push(message.clone());
            Ok(message.id)
        } else {
            Err(McpAiToolsAdapterError::Config(format!(
                "Conversation not found: {conversation_id}"
            )))
        }
    }
    
    /// Get conversation history
    pub fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<ConversationMessage>, McpAiToolsAdapterError> {
        let history = self.history.lock().unwrap();
        if let Some(conversation) = history.get(conversation_id) {
            Ok(conversation.clone())
        } else {
            Err(McpAiToolsAdapterError::Config(format!(
                "Conversation not found: {conversation_id}"
            )))
        }
    }
    
    /// Invoke an AI tool
    pub async fn invoke_tool(
        &self,
        conversation_id: &str,
        tool_name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        let tool_name = tool_name.into();
        let invocation = AiToolInvocation::new(tool_name.clone(), arguments);
        
        // Find the tool
        let tools = self.tools.lock().unwrap();
        let tool = tools.get(&tool_name).ok_or_else(|| {
            McpAiToolsAdapterError::ToolNotFound(tool_name.clone())
        })?;
        
        // Add the invocation to the conversation
        {
            let mut history = self.history.lock().unwrap();
            if let Some(conversation) = history.get_mut(conversation_id) {
                let message = ConversationMessage {
                    id: Uuid::new_v4().to_string(),
                    content: format!("Invoking tool: {}", tool_name),
                    message_type: AiMessageType::FunctionCall,
                    timestamp: chrono::Utc::now(),
                    tool_invocation: Some(invocation.clone()),
                    tool_response: None,
                };
                conversation.push(message);
            } else {
                return Err(McpAiToolsAdapterError::Config(format!(
                    "Conversation not found: {conversation_id}"
                )));
            }
        }
        
        // Invoke the tool
        let response = tool.handler.handle(invocation, Arc::new(self.clone())).await?;
        
        // Add the response to the conversation
        {
            let mut history = self.history.lock().unwrap();
            if let Some(conversation) = history.get_mut(conversation_id) {
                let message = ConversationMessage {
                    id: Uuid::new_v4().to_string(),
                    content: format!(
                        "Tool response: {}",
                        if response.status == AiToolResponseStatus::Success {
                            "Success"
                        } else {
                            response.error.as_deref().unwrap_or("Unknown error")
                        }
                    ),
                    message_type: AiMessageType::ToolResult,
                    timestamp: chrono::Utc::now(),
                    tool_invocation: None,
                    tool_response: Some(response.clone()),
                };
                conversation.push(message);
            }
        }
        
        Ok(response)
    }
    
    /// Generate a response from the AI
    pub async fn generate_response(
        &self,
        conversation_id: &str,
        provider: Option<String>,
        model: Option<String>,
        options: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String, McpAiToolsAdapterError> {
        let provider = provider.unwrap_or_else(|| self.config.default_provider.clone());
        
        // Get provider settings
        let provider_settings = self
            .config
            .providers
            .get(&provider)
            .ok_or_else(|| McpAiToolsAdapterError::ProviderNotFound(provider.clone()))?;
        
        let model = model.unwrap_or_else(|| provider_settings.default_model.clone());
        
        // Check if model is available
        if !provider_settings.available_models.contains(&model) {
            return Err(McpAiToolsAdapterError::ModelNotFound {
                provider: provider.clone(),
                model,
            });
        }
        
        // Get conversation history
        let conversation = self.get_conversation(conversation_id)?;
        
        // Prepare message format for the provider
        let formatted_messages = self.format_messages_for_provider(&provider, &conversation)?;
        
        // Combine options
        let mut combined_options = provider_settings.default_parameters.clone();
        if let Some(options) = options {
            for (key, value) in options {
                combined_options.insert(key, value);
            }
        }
        
        // Send request to provider API
        // This is a placeholder implementation that would need to be replaced
        // with actual API calls to the provider
        let response = self.call_provider_api(
            &provider,
            &model,
            formatted_messages,
            combined_options,
        ).await?;
        
        // Add response to conversation
        self.add_message(conversation_id, response.clone(), AiMessageType::Assistant)?;
        
        Ok(response)
    }
    
    /// Format messages for a specific provider
    fn format_messages_for_provider(
        &self,
        provider: &str,
        conversation: &[ConversationMessage],
    ) -> Result<serde_json::Value, McpAiToolsAdapterError> {
        // This is a simplified implementation that would need to be expanded
        // for each supported provider
        match provider {
            "openai" => {
                let messages: Vec<serde_json::Value> = conversation
                    .iter()
                    .map(|msg| {
                        let role = match msg.message_type {
                            AiMessageType::Human => "user",
                            AiMessageType::Assistant => "assistant",
                            AiMessageType::System => "system",
                            AiMessageType::ToolResult => "tool",
                            AiMessageType::FunctionCall => "function",
                        };
                        
                        json!({
                            "role": role,
                            "content": msg.content,
                        })
                    })
                    .collect();
                
                Ok(json!({ "messages": messages }))
            }
            "anthropic" => {
                // Anthropic uses a different format
                let messages: Vec<serde_json::Value> = conversation
                    .iter()
                    .map(|msg| {
                        let role = match msg.message_type {
                            AiMessageType::Human => "human",
                            AiMessageType::Assistant => "assistant",
                            AiMessageType::System => "system",
                            _ => "human", // Default for tool messages
                        };
                        
                        json!({
                            "role": role,
                            "content": msg.content,
                        })
                    })
                    .collect();
                
                Ok(json!({ "messages": messages }))
            }
            _ => Err(McpAiToolsAdapterError::ProviderNotFound(
                provider.to_string(),
            )),
        }
    }
    
    /// Call provider API
    async fn call_provider_api(
        &self,
        provider: &str,
        model: &str,
        messages: serde_json::Value,
        options: HashMap<String, serde_json::Value>,
    ) -> Result<String, McpAiToolsAdapterError> {
        // This is a placeholder implementation
        // In a real implementation, this would make HTTP requests to the provider's API
        info!("Calling {provider} API with model {model}");
        debug!("Messages: {messages}");
        debug!("Options: {options:?}");
        
        // This would be replaced with actual API calls
        // For now, we'll just return a mock response
        Ok(format!("This is a response from {provider} using {model}"))
    }
}

impl Clone for McpAiToolsAdapter {
    fn clone(&self) -> Self {
        Self {
            mcp_adapter: self.mcp_adapter.clone(),
            config: self.config.clone(),
            history: self.history.clone(),
            tools: self.tools.clone(),
        }
    }
}

/// Create an MCP-AI Tools adapter with the given MCP adapter
pub fn create_mcp_ai_tools_adapter(
    mcp_adapter: Arc<dyn MCPInterface>,
) -> Result<Arc<McpAiToolsAdapter>, McpAiToolsAdapterError> {
    Ok(Arc::new(McpAiToolsAdapter::new(mcp_adapter)))
}

/// Create an MCP-AI Tools adapter with the given MCP adapter and configuration
pub fn create_mcp_ai_tools_adapter_with_config(
    mcp_adapter: Arc<dyn MCPInterface>,
    config: McpAiToolsConfig,
) -> Result<Arc<McpAiToolsAdapter>, McpAiToolsAdapterError> {
    Ok(Arc::new(McpAiToolsAdapter::with_config(mcp_adapter, config)))
}

/// Wrapper to adapt ToolHandlerV2 to ToolHandler for backward compatibility
#[derive(Debug)]
struct ToolHandlerWrapper {
    inner: Arc<dyn ToolHandlerV2>,
}

impl ToolHandlerWrapper {
    fn new<H: ToolHandlerV2 + 'static>(handler: H) -> Self {
        Self {
            inner: Arc::new(handler),
        }
    }
}

#[async_trait]
impl ToolHandler for ToolHandlerWrapper {
    async fn handle(
        &self,
        invocation: AiToolInvocation,
        _adapter: Arc<McpAiToolsAdapter>,
    ) -> Result<AiToolResponse, McpAiToolsAdapterError> {
        // Delegate to inner handler without passing adapter
        self.inner.handle(invocation).await
    }
} 