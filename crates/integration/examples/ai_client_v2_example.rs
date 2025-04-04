use async_trait::async_trait;
use anyhow::Result;
use squirrel_ai_tools::common::{
    ChatRequest, ChatResponse, ChatResponseStream, ChatResponseChunk, 
    ModelParameters, MessageRole, ChatMessage, UsageInfo
};
use squirrel_integration::ai_agent::{
    AIClientV2, AIClientCallbacks, AIClientWrapper,
    AgentRequest, AgentResponse, ContentFormat, OperationType
};
use futures::stream::StreamExt;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// A simple example implementation of AIClientV2
#[derive(Debug)]
struct SimpleAIClientV2 {
    /// Provider name
    provider_name: String,
    
    /// Default model name
    default_model_name: String,
    
    /// API key for authentication
    api_key: String,
    
    /// Available models
    available_models: Vec<String>,
    
    /// Storage for request counts and statistics
    request_count: Arc<Mutex<usize>>,
    
    /// Logging callback
    log_event: Option<Box<dyn Fn(&str, &str) -> Result<()> + Send + Sync>>,
    
    /// Track usage callback
    track_usage: Option<Box<dyn Fn(i32, i32, i32) -> Result<()> + Send + Sync>>,
}

impl SimpleAIClientV2 {
    /// Create a new instance with the given provider and API key
    pub fn new(provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        let provider_name = provider.into();
        let default_model = match provider_name.as_str() {
            "openai" => "gpt-4",
            "anthropic" => "claude-3-opus",
            "gemini" => "gemini-1.5-pro",
            _ => "default-model",
        };
        
        let available_models = match provider_name.as_str() {
            "openai" => vec![
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            "anthropic" => vec![
                "claude-3-opus".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-haiku".to_string(),
            ],
            "gemini" => vec![
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
            _ => vec![default_model.to_string()],
        };
        
        Self {
            provider_name,
            default_model_name: default_model.to_string(),
            api_key: api_key.into(),
            available_models,
            request_count: Arc::new(Mutex::new(0)),
            log_event: None,
            track_usage: None,
        }
    }
    
    /// Log an event using the callback if available
    fn log(&self, event_type: &str, message: &str) {
        if let Some(log) = &self.log_event {
            if let Err(e) = log(event_type, message) {
                eprintln!("Error logging event: {}", e);
            }
        }
    }
    
    /// Track usage using the callback if available
    fn track(&self, prompt_tokens: i32, completion_tokens: i32, total_tokens: i32) {
        if let Some(track) = &self.track_usage {
            if let Err(e) = track(prompt_tokens, completion_tokens, total_tokens) {
                eprintln!("Error tracking usage: {}", e);
            }
        }
    }
}

#[async_trait]
impl AIClientV2 for SimpleAIClientV2 {
    fn provider_name(&self) -> &str {
        &self.provider_name
    }
    
    fn default_model(&self) -> &str {
        &self.default_model_name
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        self.log("list_models", "Listing available models");
        
        // Return the available models
        Ok(self.available_models.clone())
    }
    
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Increment request count
        {
            let mut count = self.request_count.lock().unwrap();
            *count += 1;
        }
        
        self.log("chat", &format!("Processing chat request with {} messages", request.messages.len()));
        
        // Create a mock response based on the request
        let mut content = String::new();
        
        // Simplified "processing" - concatenate parts of the input
        for message in &request.messages {
            if let Some(msg_content) = &message.content {
                if message.role == MessageRole::User {
                    if !content.is_empty() {
                        content.push_str("\n\n");
                    }
                    
                    // Generate a simple response based on the input
                    // This is just a mock example
                    let response = format!("Response to: {}", msg_content);
                    content.push_str(&response);
                }
            }
        }
        
        // If content is empty, provide a default response
        if content.is_empty() {
            content = "I'm a simple AI assistant example.".to_string();
        }
        
        let model = request.model.unwrap_or_else(|| self.default_model_name.clone());
        
        // Estimate token usage
        let prompt_tokens = request.messages.iter()
            .filter_map(|m| m.content.as_ref())
            .map(|c| c.len() / 4) // Very rough estimate
            .sum::<usize>();
        
        let completion_tokens = content.len() / 4; // Very rough estimate
        let total_tokens = prompt_tokens + completion_tokens;
        
        // Track usage
        self.track(prompt_tokens as i32, completion_tokens as i32, total_tokens as i32);
        
        // Create usage info
        let usage = UsageInfo {
            prompt_tokens: Some(prompt_tokens as u32),
            completion_tokens: Some(completion_tokens as u32),
            total_tokens: Some(total_tokens as u32),
        };
        
        // Create the response
        let response = ChatResponse {
            choices: vec![
                ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(content),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
            ],
            usage: Some(usage),
        };
        
        self.log("chat", "Completed chat request");
        
        Ok(response)
    }
    
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        // Increment request count
        {
            let mut count = self.request_count.lock().unwrap();
            *count += 1;
        }
        
        self.log("chat_stream", &format!("Processing streaming chat request with {} messages", request.messages.len()));
        
        // Create a mock response based on the request
        let mut content = String::new();
        
        // Simplified "processing" - concatenate parts of the input
        for message in &request.messages {
            if let Some(msg_content) = &message.content {
                if message.role == MessageRole::User {
                    if !content.is_empty() {
                        content.push_str("\n\n");
                    }
                    
                    // Generate a simple response based on the input
                    let response = format!("Response to: {}", msg_content);
                    content.push_str(&response);
                }
            }
        }
        
        // If content is empty, provide a default response
        if content.is_empty() {
            content = "I'm a simple AI assistant example.".to_string();
        }
        
        // Split the content into chunks for streaming
        let chunks: Vec<&str> = content.split(' ').collect();
        
        // Create a stream of chunks
        let stream = futures::stream::iter(chunks.into_iter().enumerate().map(|(i, chunk)| {
            // Artificial delay for realism
            tokio::time::sleep(Duration::from_millis(50)).await;
            
            let chunk_content = if i == 0 {
                // First chunk includes the role
                chunk.to_string()
            } else {
                // Other chunks only have content
                format!(" {}", chunk)
            };
            
            Ok(ChatResponseChunk {
                role: if i == 0 { Some("assistant".to_string()) } else { None },
                content: Some(chunk_content),
                tool_calls: None,
            })
        }));
        
        // Boxed stream
        let boxed_stream = Box::pin(stream);
        
        // Estimate token usage (for tracking)
        let prompt_tokens = request.messages.iter()
            .filter_map(|m| m.content.as_ref())
            .map(|c| c.len() / 4) // Very rough estimate
            .sum::<usize>();
        
        let completion_tokens = content.len() / 4; // Very rough estimate
        let total_tokens = prompt_tokens + completion_tokens;
        
        // Track usage
        self.track(prompt_tokens as i32, completion_tokens as i32, total_tokens as i32);
        
        Ok(ChatResponseStream {
            inner: boxed_stream,
        })
    }
    
    fn register_callbacks(&mut self, callbacks: AIClientCallbacks) {
        self.log_event = callbacks.log_event;
        self.track_usage = callbacks.track_usage;
    }
}

/// Example function to create a request
fn create_sample_request() -> ChatRequest {
    ChatRequest::new()
        .add_system("You are a helpful assistant.")
        .add_user("Tell me a short story about a squirrel programmer.")
        .with_model("gpt-4")
        .with_parameters(ModelParameters {
            temperature: Some(0.7),
            max_tokens: Some(500),
            ..Default::default()
        })
}

/// Example agent adapter to demonstrate AIClientV2 usage
struct ExampleAgentAdapter {
    client: Arc<dyn AIClientV2>,
}

impl ExampleAgentAdapter {
    fn new(client: impl AIClientV2 + 'static) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
    
    /// Process a request using the AI client
    async fn process_request(&self, request: AgentRequest) -> Result<AgentResponse> {
        // Convert AgentRequest to ChatRequest
        let mut chat_request = ChatRequest::new();
        
        // Add system message if present
        if let Some(system) = &request.system_message {
            chat_request = chat_request.add_system(system);
        } else {
            chat_request = chat_request.add_system("You are a helpful assistant.");
        }
        
        // Add user prompt
        chat_request = chat_request.add_user(&request.prompt);
        
        // Add model if specified
        if let Some(model) = &request.options.model {
            chat_request = chat_request.with_model(model);
        }
        
        // Convert other options
        let params = ModelParameters {
            temperature: request.options.temperature,
            max_tokens: request.options.max_tokens,
            top_p: request.options.top_p,
            frequency_penalty: request.options.frequency_penalty,
            presence_penalty: request.options.presence_penalty,
            ..Default::default()
        };
        
        chat_request = chat_request.with_parameters(params);
        
        // Get start time for performance tracking
        let start = std::time::Instant::now();
        
        // Process the request
        let response = self.client.chat(chat_request).await?;
        
        // Calculate completion time
        let completion_time = start.elapsed().as_millis() as u64;
        
        // Extract response text
        let text = response.choices.first()
            .and_then(|msg| msg.content.clone())
            .unwrap_or_else(|| "No response generated.".to_string());
        
        // Convert usage information
        let usage_info = response.usage.unwrap_or_default();
        let usage = squirrel_integration::ai_agent::Usage {
            tokens: squirrel_integration::ai_agent::UsageStatistics {
                prompt_tokens: usage_info.prompt_tokens,
                completion_tokens: usage_info.completion_tokens,
                total_tokens: usage_info.total_tokens,
            },
            requests: 1,
            billable_duration_ms: Some(completion_time),
        };
        
        // Create agent response
        let agent_response = AgentResponse {
            id: Uuid::new_v4(),
            text,
            request_id: request.id,
            completion_time,
            format: ContentFormat::PlainText, // Default format
            usage,
            metadata: std::collections::HashMap::new(),
        };
        
        Ok(agent_response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an instance of our AIClientV2 implementation
    let mut client = SimpleAIClientV2::new("openai", "fake-api-key");
    
    // Set up callbacks
    let callbacks = AIClientCallbacks {
        mcp_service: Some(Box::new(|msg| {
            println!("MCP Service called with: {}", msg);
            Ok("MCP response".to_string())
        })),
        log_event: Some(Box::new(|event_type, msg| {
            println!("LOG [{}]: {}", event_type, msg);
            Ok(())
        })),
        track_usage: Some(Box::new(|prompt, completion, total| {
            println!("USAGE: {} prompt + {} completion = {} total tokens", 
                     prompt, completion, total);
            Ok(())
        })),
        check_rate_limit: Some(Box::new(|| {
            // Always allow in this example
            Ok(true)
        })),
    };
    
    // Register callbacks
    client.register_callbacks(callbacks);
    
    // Test listing models
    let models = client.list_models().await?;
    println!("Available models: {:?}", models);
    
    // Create a sample request
    let request = create_sample_request();
    
    // Process the request
    let response = client.chat(request.clone()).await?;
    println!("\nChat Response:");
    if let Some(message) = response.choices.first() {
        if let Some(content) = &message.content {
            println!("{}", content);
        }
    }
    
    println!("\nUsage: {:?}", response.usage);
    
    // Test streaming
    println!("\nStreaming Response:");
    let stream_response = client.chat_stream(request).await?;
    
    // Process the stream
    let mut stream = stream_response.inner;
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                if let Some(content) = &chunk.content {
                    print!("{}", content);
                    std::io::Write::flush(&mut std::io::stdout())?;
                }
            }
            Err(e) => {
                eprintln!("Error in stream: {}", e);
                break;
            }
        }
    }
    println!("\n");
    
    // Test with agent adapter
    let agent_adapter = ExampleAgentAdapter::new(client);
    
    // Create an agent request
    let agent_request = AgentRequest {
        id: Uuid::new_v4(),
        prompt: "Write a short poem about rust programming.".to_string(),
        system_message: Some("You are a poetic programmer.".to_string()),
        options: squirrel_integration::ai_agent::GenerationOptions::default(),
        context: None,
        operation_type: OperationType::Generate,
        content: None,
    };
    
    // Process agent request
    let agent_response = agent_adapter.process_request(agent_request).await?;
    
    // Display agent response
    println!("\nAgent Response:");
    println!("{}", agent_response.text);
    println!("\nCompletion time: {}ms", agent_response.completion_time);
    println!("Total tokens: {:?}", agent_response.usage.tokens.total_tokens);
    
    // Demonstrate AIClientWrapper for backward compatibility
    println!("\nTesting AIClientWrapper for backward compatibility:");
    
    let v2_client = SimpleAIClientV2::new("anthropic", "fake-api-key");
    let wrapped_client = AIClientWrapper::new(v2_client);
    
    println!("Provider: {}", wrapped_client.provider_name());
    println!("Default model: {}", wrapped_client.default_model());
    
    let models = wrapped_client.list_models().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Available models: {:?}", models);
    
    Ok(())
} 