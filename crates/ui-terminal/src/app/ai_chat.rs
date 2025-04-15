use crate::app::state::AppState;
use crate::error::Error as UiError;
use crate::widgets::ai_chat::AiChatWidgetState;
use secrecy::ExposeSecret;
use serde_json::json;
use squirrel_ai_tools::config::Config;
use squirrel_integration::mcp_ai_tools::{
    create_mcp_ai_tools_adapter_with_config, McpAiToolsConfig, ProviderSettings,
};
use squirrel_mcp::MCPInterface;
use std::sync::Arc;
use log::{error, warn};
use uuid::Uuid;

/// AI chat related functions
pub trait AiChatHandler {
    /// Initializes the AI adapter and state for the chat feature.
    async fn init_ai_chat(&mut self) -> Result<(), String>;
    
    /// Processes an AI chat message when the send button is clicked.
    async fn process_ai_chat(&mut self) -> Result<(), String>;

    /// Sends an AI chat message to the adapter.
    async fn send_ai_chat_message(&self, state: &mut AiChatWidgetState) -> Result<(), String>;
}

impl<S: dashboard_core::service::DashboardService + Send + Sync + 'static + ?Sized> AiChatHandler for crate::app::App<S> {
    async fn init_ai_chat(&mut self) -> Result<(), String> {
        // Log the initialization attempt
        log::info!("Attempting to initialize AI chat");
        
        // Configure AI adapter if not already done
        if self.state.ai_adapter.is_none() {
            // Load configuration from disk
            let config = match Config::load() {
                Ok(config) => {
                    log::info!("Successfully loaded OpenAI configuration");
                    config
                },
                Err(e) => {
                    let err_msg = format!("Failed to load OpenAI configuration: {}", e);
                    log::error!("{}", err_msg);
                    return Err(err_msg);
                }
            };
            
            // Check if we have an API key in the config
            let api_key = config.openai_api_key.expose_secret().0.clone();
            
            if api_key.is_empty() {
                let err_msg = "OpenAI API key not found in configuration";
                log::error!("{}", err_msg);
                return Err(err_msg.to_string());
            }
            
            log::info!("API key is present in configuration (not shown for security)");
            
            // Create a mock MCP interface for the AI tools adapter
            struct MockMCP;
            
            #[async_trait::async_trait]
            impl squirrel_mcp::MCPInterface for MockMCP {
                fn initialize(&self) -> Result<(), squirrel_core::error::SquirrelError> {
                    Ok(())
                }
                
                fn is_initialized(&self) -> bool {
                    true
                }
                
                fn get_config(&self) -> Result<squirrel_mcp::config::McpConfig, squirrel_core::error::SquirrelError> {
                    Ok(squirrel_mcp::config::McpConfig::default())
                }
                
                fn send_message(&self, _message: &str) -> Result<String, squirrel_core::error::SquirrelError> {
                    Ok("success".to_string())
                }
            }
            
            let mcp = Arc::new(MockMCP);
            log::info!("Created mock MCP interface");
            
            // Create MCP-AI tools config
            let mut tools_config = McpAiToolsConfig::default();
            
            // Configure OpenAI provider with the API key and additional parameters
            // Add all available models to the provider settings
            let openai_settings = ProviderSettings::default_openai()
                .with_parameter("api_key".to_string(), json!(api_key))
                .with_parameter("temperature".to_string(), json!(0.7))
                .with_parameter("max_tokens".to_string(), json!(2000))
                .with_models(vec![
                    "gpt-3.5-turbo".to_string(),
                    "gpt-4".to_string(),
                    "gpt-4-turbo-preview".to_string()
                ]);
            
            log::info!("Configured OpenAI provider settings with models: gpt-3.5-turbo, gpt-4, gpt-4-turbo-preview");
                
            tools_config = tools_config
                .with_provider("openai".to_string(), openai_settings)
                .with_timeout(60000) // Increase timeout to 60 seconds
                .with_streaming(true);
            
            log::info!("Created MCP-AI tools config with 60s timeout and streaming enabled");
            
            // Create MCP-AI tools adapter
            match create_mcp_ai_tools_adapter_with_config(mcp, tools_config) {
                Ok(adapter) => {
                    log::info!("Successfully created MCP-AI tools adapter");
                    self.state.ai_adapter = Some(adapter);
                }
                Err(e) => {
                    let err_msg = format!("Failed to create MCP-AI tools adapter: {}", e);
                    log::error!("{}", err_msg);
                    return Err(err_msg);
                }
            }
        } else {
            log::info!("AI adapter already exists, skipping adapter creation");
        }
        
        // Initialize AI chat state if adapter is available
        if let Some(adapter) = &self.state.ai_adapter {
            // Create the AI chat state
            log::info!("Creating AI chat widget state with the adapter");
            let ai_chat_state = AiChatWidgetState::new(adapter.clone());
            
            // Check if the models list is empty and add a debug message
            if ai_chat_state.models.is_empty() {
                let error_msg = "No AI models available. Check your API key and provider settings.";
                log::error!("{}", error_msg);
                self.state.add_error(UiError::AiChatError(error_msg.to_string()));
                // We'll still continue initialization even with empty models list
            } else {
                log::info!("Found {} available AI models", ai_chat_state.models.len());
            }
            
            self.state.ai_chat_state = Some(ai_chat_state);
            
            // Reset both error flags since we successfully initialized
            self.state.ai_chat_error_reported = false;
            log::info!("AI chat successfully initialized");
        } else {
            let err_msg = "Failed to initialize AI adapter - adapter is missing after creation";
            log::error!("{}", err_msg);
            return Err(err_msg.to_string());
        }
        
        Ok(())
    }

    async fn process_ai_chat(&mut self) -> Result<(), String> {
        if let Some(ai_chat_state) = &mut self.state.ai_chat_state {
            // Check if we're already sending a message
            if ai_chat_state.is_sending {
                // Get the current message input text
                let message_text = ai_chat_state.input.clone();
                
                // Validate message
                if message_text.trim().is_empty() {
                    ai_chat_state.is_sending = false; // Reset sending flag
                    let error_msg = "Message cannot be empty".to_string();
                    self.state.add_error(UiError::AiChatError(error_msg.clone()));
                    return Err(error_msg);
                }
                
                // Check if we have a selected model
                if ai_chat_state.models.is_empty() {
                    ai_chat_state.is_sending = false; // Reset sending flag
                    let error_msg = "No AI models available".to_string();
                    self.state.add_error(UiError::AiChatError(error_msg.clone()));
                    return Err(error_msg);
                }
                
                // Ensure we have a valid selected model index
                if ai_chat_state.selected_model >= ai_chat_state.models.len() {
                    ai_chat_state.selected_model = 0;
                }
                
                // Add the message to the chat widget state
                ai_chat_state.add_user_message(message_text);
                
                // Get selected model
                let selected_model = match ai_chat_state.models.get(ai_chat_state.selected_model) {
                    Some(model) => model.clone(),
                    None => {
                        ai_chat_state.is_sending = false; // Reset sending flag
                        let error_msg = "No model selected".to_string();
                        self.state.add_error(UiError::AiChatError(error_msg.clone()));
                        return Err(error_msg);
                    }
                };
                
                // Create a conversation ID
                let conversation_id = Uuid::new_v4().to_string();
                
                // Reference to the adapter
                let adapter = ai_chat_state.adapter.clone();
                
                // Process the message (using the cloned values to avoid borrowing issues)
                let result = adapter.generate_response(
                    &conversation_id,
                    Some("openai".to_string()),
                    Some(selected_model.to_api_name().to_string()),
                    None
                ).await;
                
                match result {
                    Ok(response) => {
                        // Add the assistant's response to the chat history
                        ai_chat_state.add_assistant_message(response);
                        ai_chat_state.is_sending = false;
                        // Successfully processed, reset error reported flag
                        self.state.ai_chat_error_reported = false;
                    }
                    Err(e) => {
                        ai_chat_state.is_sending = false; // Reset sending flag
                        let error_msg = format!("Failed to process message: {}", e);
                        // Only add the error if we haven't already reported an error
                        if !self.state.ai_chat_error_reported {
                            self.state.add_error(UiError::AiChatError(error_msg.clone()));
                        }
                        return Err(error_msg);
                    }
                }
            }
        } else {
            // Return error but don't add to the error list - this is handled in on_tick
            return Err("AI chat not initialized".to_string());
        }
        Ok(())
    }

    async fn send_ai_chat_message(&self, _state: &mut AiChatWidgetState) -> Result<(), String> {
        // This method is no longer used - all processing is done in process_ai_chat
        Err("Method obsolete - use process_ai_chat instead".to_string())
    }
} 