use std::{error::Error, io, sync::Arc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use squirrel_mcp::{error::MCPError, config::McpConfig, MCPInterface, MessageType};
use squirrel_integration::mcp_ai_tools::{
    McpAiToolsAdapter, McpAiToolsConfig, AiMessageType,
    create_mcp_ai_tools_adapter_with_config, ProviderSettings
};
use squirrel_ai_tools::config::{Config, SecretString};
use secrecy::ExposeSecret;
use serde_json::json;
use anyhow::anyhow;
use async_trait::async_trait;
use squirrel_core::error::SquirrelError;

struct ChatApp {
    input: String,
    messages: Vec<(String, AiMessageType)>,
    conversation_id: String,
    mcp_ai_adapter: Arc<McpAiToolsAdapter>,
    running: bool,
    should_send: bool,
}

impl ChatApp {
    fn new(mcp_ai_adapter: Arc<McpAiToolsAdapter>) -> Self {
        let conversation_id = mcp_ai_adapter.create_conversation();
        
        // Add a system message to set up the assistant's behavior
        let _ = mcp_ai_adapter.add_message(
            &conversation_id,
            "You are a helpful assistant. Provide concise responses.",
            AiMessageType::System,
        );
        
        Self {
            input: String::new(),
            messages: vec![
                ("Welcome to the OpenAI chat interface!".to_string(), AiMessageType::Assistant),
                ("Type your message and press Enter to send.".to_string(), AiMessageType::Assistant),
            ],
            conversation_id,
            mcp_ai_adapter,
            running: true,
            should_send: false,
        }
    }
    
    fn on_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    self.should_send = true;
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.running = false;
            }
            _ => {}
        }
    }
    
    async fn send_message(&mut self) {
        let user_message = self.input.clone();
        self.messages.push((user_message.clone(), AiMessageType::Human));
        self.input.clear();
        
        // Add message to the conversation
        match self.mcp_ai_adapter.add_message(
            &self.conversation_id,
            user_message,
            AiMessageType::Human,
        ) {
            Ok(_) => {},
            Err(e) => {
                self.messages.push((format!("Error adding message: {}", e), AiMessageType::System));
                return;
            }
        }
        
        // Generate response
        self.messages.push(("Thinking...".to_string(), AiMessageType::System));
        
        // Generate response using the adapter
        match self.mcp_ai_adapter.generate_response(
            &self.conversation_id,
            None,
            None,
            None,
        ).await {
            Ok(response) => {
                // Remove the "Thinking..." message
                self.messages.pop();
                
                // Add the response to the UI
                self.messages.push((response, AiMessageType::Assistant));
            }
            Err(e) => {
                // Remove the "Thinking..." message
                self.messages.pop();
                
                // Add the error message
                self.messages.push((format!("Error generating response: {}", e), AiMessageType::System));
            }
        }
    }
}

fn ui(f: &mut Frame, app: &ChatApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ].as_ref())
        .split(f.size());
    
    // Chat history
    let messages: Vec<ListItem> = app.messages
        .iter()
        .map(|(msg, msg_type)| {
            let (prefix, style) = match msg_type {
                AiMessageType::Human => ("You: ", Style::default().fg(Color::Cyan)),
                AiMessageType::Assistant => ("Assistant: ", Style::default().fg(Color::Green)),
                AiMessageType::System => ("System: ", Style::default().fg(Color::Yellow)),
                _ => ("", Style::default()),
            };
            
            // Create a line with prefix and message
            let line = Line::from(vec![
                Span::styled(prefix, style),
                Span::raw(msg),
            ]);
            
            // Create Text from the Line
            let content = Text::from(line);
            
            ListItem::new(content)
        })
        .collect();
    
    let messages = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Chat"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
    
    f.render_widget(messages, chunks[0]);
    
    // Input box
    let input = Paragraph::new(app.input.as_ref() as &str)
        .style(Style::default())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input (Press Esc to quit)"));
    
    f.render_widget(input, chunks[1]);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from disk
    let mut config = Config::load().unwrap_or_else(|_| {
        // If loading fails, create a new config
        Config {
            openai_api_key: secrecy::Secret::new(SecretString::from(String::new())),
        }
    });
    
    // Check if we have an API key in the config
    let api_key = config.openai_api_key.expose_secret().0.clone();
    
    // If API key is empty, try to get it from environment variable as fallback
    let api_key = if api_key.is_empty() {
        match std::env::var("OPENAI_API_KEY") {
            Ok(key) => {
                // Save the key to config for future use
                config.set_openai_api_key(key.clone());
                if let Err(e) = config.save() {
                    eprintln!("Warning: Failed to save config: {}", e);
                } else {
                    println!("API key saved to configuration file");
                }
                key
            }
            Err(_) => {
                return Err(anyhow!("OpenAI API key not found in config or environment. Please set OPENAI_API_KEY or run squirrel-config utility to configure").into());
            }
        }
    } else {
        println!("Using API key from configuration file");
        api_key
    };
    
    // Create a mock MCP interface
    struct MockMCP;
    
    #[async_trait]
    impl MCPInterface for MockMCP {
        fn initialize(&self) -> Result<(), SquirrelError> {
            Ok(())
        }
        
        fn is_initialized(&self) -> bool {
            true
        }
        
        fn get_config(&self) -> Result<McpConfig, SquirrelError> {
            Ok(McpConfig::default())
        }
        
        fn send_message(&self, _message: &str) -> Result<String, SquirrelError> {
            Ok("success".to_string())
        }
    }
    
    let mcp = Arc::new(MockMCP);
    
    // Create MCP-AI tools config
    let mut tools_config = McpAiToolsConfig::default();
    
    // Configure OpenAI provider with the correct API structure
    let openai_settings = ProviderSettings::default_openai()
        .with_parameter("api_key".to_string(), json!(api_key))
        .with_parameter("temperature".to_string(), json!(0.7))
        .with_parameter("max_tokens".to_string(), json!(500));
        
    tools_config = tools_config
        .with_provider("openai".to_string(), openai_settings)
        .with_timeout(30000)
        .with_streaming(true);
    
    // Create MCP-AI tools adapter
    let adapter = create_mcp_ai_tools_adapter_with_config(mcp, tools_config)
        .map_err(|e| anyhow!("Failed to create MCP-AI tools adapter: {}", e))?;
    
    // Setup terminal UI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create chat app
    let mut app = ChatApp::new(adapter);
    
    // Main event loop
    while app.running {
        terminal.draw(|f| ui(f, &app))?;
        
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
        
        // Handle message sending asynchronously
        if app.should_send {
            app.should_send = false;
            app.send_message().await;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    Ok(())
} 