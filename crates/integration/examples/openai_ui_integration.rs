use squirrel_integration::mcp_ai_tools::{
    create_mcp_ai_tools_adapter_with_config,
    AiMessageType, McpAiToolsAdapter, McpAiToolsConfig, ProviderSettings
};
use squirrel_ai_tools::config::Config;
use squirrel_mcp::MCPInterface;
use squirrel_mcp::config::McpConfig;
use squirrel_core::error::SquirrelError;
use async_trait::async_trait;
use secrecy::ExposeSecret;
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio::task;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

// Create a simple mock MCP for testing
struct MockMCP;

impl MockMCP {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MCPInterface for MockMCP {
    /// Initialize the MCP system.
    fn initialize(&self) -> Result<(), SquirrelError> {
        Ok(())
    }

    /// Check if the MCP system is initialized.
    fn is_initialized(&self) -> bool {
        true
    }

    /// Get the MCP configuration.
    fn get_config(&self) -> Result<McpConfig, SquirrelError> {
        Ok(McpConfig::default())
    }

    /// Send a message through the MCP system.
    fn send_message(&self, message: &str) -> Result<String, SquirrelError> {
        info!("Mock MCP received message: {}", message);
        Ok("success".to_string())
    }
}

// Dashboard message interface
struct DashboardMessage {
    id: String,
    content: String,
    role: String,
    timestamp: String,
    context: Option<serde_json::Value>,
}

// Dashboard context interface
struct DashboardContext {
    id: String,
    data: serde_json::Value,
    timestamp: String,
}

// Dashboard client - mocked for this example
struct DashboardClient {
    message_tx: mpsc::Sender<DashboardMessage>,
    context_tx: mpsc::Sender<DashboardContext>,
}

impl DashboardClient {
    fn new() -> (Self, mpsc::Receiver<DashboardMessage>, mpsc::Receiver<DashboardContext>) {
        let (message_tx, message_rx) = mpsc::channel(100);
        let (context_tx, context_rx) = mpsc::channel(100);
        
        (
            Self { message_tx, context_tx },
            message_rx,
            context_rx,
        )
    }
    
    async fn send_message(&self, message: DashboardMessage) {
        if let Err(e) = self.message_tx.send(message).await {
            warn!("Failed to send message to dashboard: {}", e);
        }
    }
    
    async fn update_context(&self, context: DashboardContext) {
        if let Err(e) = self.context_tx.send(context).await {
            warn!("Failed to send context to dashboard: {}", e);
        }
    }
}

// Mock terminal UI for demonstration
struct TerminalUI {
    ai_tools_adapter: Arc<McpAiToolsAdapter>,
    conversation_id: String,
    dashboard_client: DashboardClient,
}

impl TerminalUI {
    fn new(ai_tools_adapter: Arc<McpAiToolsAdapter>, dashboard_client: DashboardClient) -> Self {
        // Create a new conversation
        let conversation_id = ai_tools_adapter.create_conversation();
        info!("Created conversation with ID: {}", conversation_id);
        
        Self {
            ai_tools_adapter,
            conversation_id,
            dashboard_client,
        }
    }
    
    async fn setup_system_prompt(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Add system message
        self.ai_tools_adapter.add_message(
            &self.conversation_id,
            "You are a helpful AI assistant. You provide clear, concise answers to user questions.",
            AiMessageType::System,
        )?;
        
        // Send to dashboard
        self.dashboard_client.send_message(DashboardMessage {
            id: format!("msg_{}", Uuid::new_v4()),
            content: "You are a helpful AI assistant. You provide clear, concise answers to user questions.".to_string(),
            role: "system".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            context: None,
        }).await;
        
        Ok(())
    }
    
    async fn send_user_message(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Add user message
        self.ai_tools_adapter.add_message(
            &self.conversation_id,
            message,
            AiMessageType::Human,
        )?;
        
        // Send to dashboard
        self.dashboard_client.send_message(DashboardMessage {
            id: format!("msg_{}", Uuid::new_v4()),
            content: message.to_string(),
            role: "user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            context: None,
        }).await;
        
        // Update context in dashboard
        let conversation = self.ai_tools_adapter.get_conversation(&self.conversation_id)?;
        self.dashboard_client.update_context(DashboardContext {
            id: self.conversation_id.clone(),
            data: json!({
                "conversation_id": self.conversation_id,
                "message_count": conversation.len(),
                "last_message": message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }).await;
        
        Ok(())
    }
    
    async fn get_ai_response(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Generate AI response
        let ai_response = self.ai_tools_adapter
            .generate_response(&self.conversation_id, None, None, None)
            .await?;
        
        // Send to dashboard
        self.dashboard_client.send_message(DashboardMessage {
            id: format!("msg_{}", Uuid::new_v4()),
            content: ai_response.clone(),
            role: "assistant".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            context: Some(json!({
                "model": "gpt-4",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })),
        }).await;
        
        Ok(ai_response)
    }
}

// Mock dashboard UI for demonstration
struct DashboardUI {
    message_rx: mpsc::Receiver<DashboardMessage>,
    context_rx: mpsc::Receiver<DashboardContext>,
}

impl DashboardUI {
    fn new(message_rx: mpsc::Receiver<DashboardMessage>, context_rx: mpsc::Receiver<DashboardContext>) -> Self {
        Self {
            message_rx,
            context_rx,
        }
    }
    
    async fn run(&mut self) {
        info!("Starting dashboard UI");
        
        // Process messages and context updates
        loop {
            tokio::select! {
                Some(message) = self.message_rx.recv() => {
                    info!("DASHBOARD - New message: [{}] {}: {}", 
                        message.timestamp, 
                        message.role, 
                        message.content);
                },
                Some(context) = self.context_rx.recv() => {
                    info!("DASHBOARD - Context update for [{}]: {}", 
                        context.id,
                        context.data);
                },
                else => break,
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting OpenAI UI integration example");
    
    // Get OpenAI API key from saved configuration or environment variable
    let openai_api_key = match Config::load() {
        Ok(config) => {
            // Use the saved API key from configuration
            let api_key = config.openai_api_key.expose_secret().0.clone();
            if api_key.is_empty() {
                // Fall back to environment variable if config is empty
                env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                    warn!("OpenAI API key not found in config or environment - this will likely fail!");
                    String::new()
                })
            } else {
                info!("Using OpenAI API key from saved configuration");
                api_key
            }
        },
        Err(e) => {
            // If config loading fails, fall back to environment variable
            warn!("Failed to load config: {}", e);
            env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
                warn!("OPENAI_API_KEY not set in environment - this will likely fail!");
                String::new()
            })
        }
    };
    
    // Create a mock MCP adapter for testing
    let mcp_adapter = Arc::new(MockMCP::new());
    
    // Create a custom configuration
    let mut config = McpAiToolsConfig::default();
    
    // Update OpenAI provider with API key and parameters
    let openai_settings = ProviderSettings::default_openai()
        .with_parameter("api_key".to_string(), json!(openai_api_key))
        .with_parameter("temperature".to_string(), json!(0.7));
    
    config = config
        .with_provider("openai".to_string(), openai_settings)
        .with_timeout(30000)
        .with_streaming(true);
    
    // Create MCP-AI Tools adapter with custom configuration
    let ai_tools_adapter = create_mcp_ai_tools_adapter_with_config(mcp_adapter, config)?;
    
    // Create dashboard client and receivers
    let (dashboard_client, message_rx, context_rx) = DashboardClient::new();
    
    // Create and setup terminal UI
    let terminal_ui = TerminalUI::new(ai_tools_adapter, dashboard_client);
    terminal_ui.setup_system_prompt().await?;
    
    // Create and run dashboard UI in a separate task
    let mut dashboard_ui = DashboardUI::new(message_rx, context_rx);
    let dashboard_task = task::spawn(async move {
        dashboard_ui.run().await;
    });
    
    // Simulate user interaction
    terminal_ui.send_user_message("Hello, can you tell me about integration testing?").await?;
    let response = terminal_ui.get_ai_response().await?;
    info!("AI response: {}", response);
    
    // Wait a moment for dashboard to process
    sleep(Duration::from_millis(500)).await;
    
    // Send another message
    terminal_ui.send_user_message("Can you provide a simple example of integration testing in Rust?").await?;
    let response = terminal_ui.get_ai_response().await?;
    info!("AI response: {}", response);
    
    // Wait a moment for dashboard to process
    sleep(Duration::from_millis(500)).await;
    
    // Send a third message
    terminal_ui.send_user_message("Thank you for the information!").await?;
    let response = terminal_ui.get_ai_response().await?;
    info!("AI response: {}", response);
    
    // Wait for dashboard to process all messages
    sleep(Duration::from_secs(1)).await;
    
    info!("OpenAI UI integration example completed");
    
    Ok(())
} 