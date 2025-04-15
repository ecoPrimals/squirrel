use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, 
        ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage,
        ChatCompletionRequestAssistantMessage,
        ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest,
        Role
    },
};
use secrecy::{ExposeSecret, Secret};
use std::env;
use thiserror::Error;
use log;
use rand;
use std::time::Duration;

/// Error type for OpenAI operations
#[derive(Debug, Error)]
pub enum OpenAIError {
    /// No API key provided
    #[error("No OpenAI API key provided")]
    NoApiKey,
    
    /// API error from OpenAI
    #[error("OpenAI API error: {0}")]
    ApiError(String),
    
    /// Error setting up the client
    #[error("Failed to set up OpenAI client: {0}")]
    SetupError(String),
    
    /// Error loading config
    #[error("Failed to load configuration: {0}")]
    ConfigError(String),

    /// API key not found in environment variables
    #[error("API key not found in environment variables")]
    ApiKeyNotFound,

    /// No response from OpenAI API
    #[error("No response from OpenAI API")]
    NoResponse,
}

/// Type alias for OpenAI result
pub type Result<T> = std::result::Result<T, OpenAIError>;

/// Service for interacting with OpenAI API
pub struct OpenAIService {
    client: Client<OpenAIConfig>,
    model: String,
    use_real_api: bool,
}

impl OpenAIService {
    /// Create a new OpenAI service with the default model
    pub fn new() -> Result<Self> {
        Self::new_with_model("gpt-3.5-turbo")
    }
    
    /// Create a new OpenAI service with a specific model
    pub fn new_with_model(model: &str) -> Result<Self> {
        // First check for API key in environment variables
        let api_key = match env::var("OPENAI_API_KEY") {
            Ok(key) => Secret::new(key),
            Err(_) => {
                // Try to load from ai-tools configuration
                match Self::load_api_key_from_config() {
                    Some(key) => key,
                    None => {
                        // For now, just implement a mock service
                        // This will allow the application to function without API credentials
                        eprintln!("No OpenAI API key found, using mock implementation");
                        return Ok(Self::mock_service(model));
                    }
                }
            }
        };
        
        // Check if we should use the real API
        let use_real_api = env::var("USE_REAL_OPENAI_API").is_ok();
        
        // Create service with the real API key
        let mut service = Self::new_with_key(api_key.expose_secret(), model);
        
        // Enable real API if requested via environment variable
        if use_real_api {
            println!("OpenAI service initialized with real API mode");
            service.use_real_api = true;
        } else {
            println!("OpenAI service initialized with mock API mode");
        }
        
        Ok(service)
    }
    
    /// Create a new service with the provided API key
    fn new_with_key(api_key: &str, model: &str) -> Self {
        // Create client with API key
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        
        Self {
            client,
            model: model.to_string(),
            use_real_api: false, // Default to mock for safety
        }
    }
    
    /// Create a mock service for development without an API key
    fn mock_service(model: &str) -> Self {
        let config = OpenAIConfig::new().with_api_key("dummy-key-for-testing");
        let client = Client::with_config(config);
        
        Self {
            client,
            model: model.to_string(),
            use_real_api: false, // Always use mock
        }
    }
    
    /// Attempt to load API key from squirrel-ai-tools config
    fn load_api_key_from_config() -> Option<Secret<String>> {
        // Use the ai-tools crate directly to get the proper API key
        match squirrel_ai_tools::config::Config::load() {
            Ok(config) => {
                // Access the inner String from SecretString properly
                let api_key = config.openai_api_key.expose_secret().0.clone();
                Some(Secret::new(api_key))
            },
            Err(e) => {
                eprintln!("Failed to load API key from config: {}", e);
                None
            }
        }
    }
    
    /// Enable the real API implementation
    pub fn enable_real_api(&mut self, enable: bool) {
        self.use_real_api = enable;
    }
    
    /// Send a message to the OpenAI API and get a response
    pub async fn send_message(&self, message: &str, conversation_history: &[(String, bool)]) -> Result<String> {
        // Use mock by default, use real API only when explicitly enabled
        if self.use_real_api {
            log::info!("Using real OpenAI API");
            self.send_message_real(message, conversation_history).await
        } else {
            log::info!("Using mock API response");
            self.get_mock_response(message, conversation_history).await
        }
    }
    
    /// Real implementation of send_message - to be implemented later
    #[allow(dead_code)]
    async fn send_message_real(&self, message: &str, conversation_history: &[(String, bool)]) -> Result<String> {
        log::debug!("Sending message to OpenAI API: {}", message);
        log::debug!("History contains {} messages", conversation_history.len());
        
        // Format the conversation history for OpenAI API
        let mut messages = Vec::new();
        
        // Add system message
        let system_message = ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                role: Role::System,
                content: Some("You are a helpful assistant in a terminal-based chat application.".to_string()),
                name: None,
            }
        );
        messages.push(system_message);
        
        // Add conversation history
        for (content, is_user) in conversation_history {
            let message = if *is_user {
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        role: Role::User,
                        content: Some(ChatCompletionRequestUserMessageContent::Text(content.clone())),
                        name: None,
                    }
                )
            } else {
                ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        role: Role::Assistant,
                        content: Some(content.clone()),
                        name: None,
                        function_call: None,
                        tool_calls: None,
                    }
                )
            };
            messages.push(message);
        }
        
        // Add the current message
        let user_message = ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                role: Role::User,
                content: Some(ChatCompletionRequestUserMessageContent::Text(message.to_string())),
                name: None,
            }
        );
        messages.push(user_message);
        
        log::debug!("Prepared {} messages for OpenAI API request", messages.len());
        
        // Create the request - only setting the essential fields
        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            frequency_penalty: None,
            logit_bias: None,
            max_tokens: Some(4000), // Set a higher token limit for longer responses
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            stop: None,
            stream: Some(false),
            temperature: Some(0.7),
            top_p: None,
            tools: None,
            tool_choice: None,
            user: None,
            // These are deprecated but still required by the struct
            function_call: None,
            functions: None,
        };
        
        log::debug!("Sending request to OpenAI API with model: {}", self.model);
        
        // Send the request to OpenAI API
        let response = self.client.chat().create(request).await
            .map_err(|e| {
                log::error!("OpenAI API error: {}", e);
                OpenAIError::ApiError(e.to_string())
            })?;
        
        log::debug!("Response received from OpenAI API");
        
        // Extract the response text
        match response.choices.first() {
            Some(choice) => {
                match &choice.message.content {
                    Some(content) => {
                        log::debug!("Got content from OpenAI: {}", content);
                        Ok(content.clone())
                    },
                    None => {
                        log::error!("No content in OpenAI response");
                        Err(OpenAIError::NoResponse)
                    },
                }
            },
            None => {
                log::error!("No choices in OpenAI response");
                Err(OpenAIError::NoResponse)
            },
        }
    }
    
    /// Get a mock response for testing
    async fn get_mock_response(&self, message: &str, conversation_history: &[(String, bool)]) -> Result<String> {
        // Extract previous context from conversation history
        let context = if conversation_history.len() > 1 {
            let mut context_str = String::new();
            // Get the last 2-3 messages for context
            let start_idx = if conversation_history.len() > 3 {
                conversation_history.len() - 3
            } else {
                0
            };
            
            for (idx, (content, is_user)) in conversation_history.iter().enumerate().skip(start_idx) {
                if idx == conversation_history.len() - 1 {
                    // Skip the most recent message (it's the current one)
                    continue;
                }
                let speaker = if *is_user { "User" } else { "AI" };
                context_str.push_str(&format!("{}: {}\n", speaker, content));
            }
            context_str
        } else {
            String::new()
        };
        
        // Create a more realistic AI-like response based on the message content
        let response = if !context.is_empty() {
            // Use context to generate a more coherent response
            if message.to_lowercase().contains("hello") || message.to_lowercase().contains("hi") {
                "Hello again! I see we've been talking. How can I continue to assist you today?".to_string()
            } else if message.to_lowercase().contains("help") {
                "I'd be happy to help! Based on our conversation, it seems you're working on a chat interface. What specific assistance do you need?".to_string()
            } else if message.to_lowercase().contains("thank") {
                "You're welcome! Is there anything else I can help you with based on what we've discussed?".to_string()
            } else {
                // More generic contextual response
                format!("I understand you're asking about '{}'. Based on our previous conversation, let me provide a helpful response...\n\nWhat specific details would you like me to address about this topic?", message)
            }
        } else {
            // First-time responses with no context
            if message.to_lowercase().contains("hello") || message.to_lowercase().contains("hi") {
                "Hello! I'm your AI assistant. How can I help you today?".to_string()
            } else if message.to_lowercase().contains("help") {
                "I'd be happy to help! I can answer questions, provide information, or assist with various tasks. What do you need help with specifically?".to_string()
            } else if message.to_lowercase().contains("feature") || message.to_lowercase().contains("can you") {
                "I have several capabilities! I can answer questions, provide explanations, offer suggestions, and engage in conversation on a wide range of topics. What would you like to explore?".to_string()
            } else if message.to_lowercase().contains("openai") || message.to_lowercase().contains("api") {
                "OpenAI provides powerful AI capabilities through their API. To use it, you'll need an API key and the appropriate client library. Would you like more specific information about integrating with the OpenAI API?".to_string()
            } else if message.to_lowercase().contains("scroll") || message.to_lowercase().contains("scrolling") {
                "Scrolling functionality in terminal UIs can be implemented using libraries like ratatui. You typically need to track the scroll position and adjust what content is displayed based on that position. Would you like more specific details about implementing scrolling?".to_string()
            } else {
                // Generic response
                format!("I'm a mock AI assistant. You said: \"{}\"", message)
            }
        };
        
        // Add a small delay to simulate network latency
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_openai_service_new() {
        // This test just verifies the constructor doesn't panic
        // It should return a mock service since we're in testing
        let service = OpenAIService::new();
        
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_mock_response() {
        // Test that the mock response functionality works
        let service = OpenAIService::new().unwrap();
        let conversation = vec![];
        let response = service.get_mock_response("hello", &conversation).await;
        
        assert!(response.is_ok());
        let content = response.unwrap();
        assert!(content.contains("Hello"));
    }

    #[tokio::test]
    async fn test_conversation_context() {
        // Test that the mock handles conversation context
        let service = OpenAIService::new().unwrap();
        
        // Create a conversation history
        let conversation = vec![
            ("Hello".to_string(), true),  // User
            ("Hi there! How can I help?".to_string(), false), // AI
        ];
        
        let response = service.get_mock_response("help me", &conversation).await;
        
        assert!(response.is_ok());
        let content = response.unwrap();
        // Should contain contextual response
        assert!(content.contains("based on") || content.contains("conversation"));
    }

    #[tokio::test]
    #[ignore = "Requires real OpenAI API key and makes external API calls"]
    async fn test_real_api_connection() {
        // This test requires a real API key to be set via environment variable or config
        // It will be skipped by default to avoid external API calls during regular testing
        
        // Check if we have a real API key
        let api_key = std::env::var("OPENAI_API_KEY").ok()
            .or_else(|| {
                match squirrel_ai_tools::config::Config::load() {
                    Ok(config) => Some(config.openai_api_key.expose_secret().0.clone()),
                    Err(_) => None,
                }
            });
        
        if api_key.is_none() {
            println!("Skipping real API test - no API key available");
            return;
        }
        
        // Create a service with the real API
        let mut service = OpenAIService::new_with_model("gpt-3.5-turbo").unwrap();
        service.enable_real_api(true);
        
        // Make a simple request
        let conversation: Vec<(String, bool)> = vec![];
        let response = service.send_message_real("Hello, are you connected?", &conversation).await;
        
        assert!(response.is_ok(), "Failed to get response from OpenAI API: {:?}", response.err());
        let content = response.unwrap();
        
        // The response should be non-empty
        assert!(!content.is_empty());
        println!("OpenAI API response: {}", content);
    }
} 