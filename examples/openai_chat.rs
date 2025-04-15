use std::{error::Error, io, sync::Arc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use ratatui::widgets::ListDirection;
use squirrel_mcp::{MCPInterface, config::McpConfig};
use squirrel_core::error::SquirrelError;
use squirrel_integration::mcp_ai_tools::{
    McpAiToolsAdapter, McpAiToolsConfig, AiMessageType,
    create_mcp_ai_tools_adapter_with_config, ProviderSettings
};
use squirrel_ai_tools::config::{Config, SecretString};
use secrecy::ExposeSecret;
use serde_json::json;
use anyhow::anyhow;
use async_trait::async_trait;
use std::cmp::min;

struct ChatApp {
    input: String,
    messages: Vec<(String, AiMessageType)>,
    conversation_id: String,
    mcp_ai_adapter: Arc<McpAiToolsAdapter>,
    running: bool,
    should_send: bool,
    scroll_position: usize,
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
            scroll_position: 0,
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
            KeyCode::Up => {
                // Scroll up
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            KeyCode::Down => {
                // Scroll down
                self.scroll_position += 1;
            }
            KeyCode::PageUp => {
                // Scroll up by 10
                self.scroll_position = self.scroll_position.saturating_sub(10);
            }
            KeyCode::PageDown => {
                // Scroll down by 10
                self.scroll_position += 10;
            }
            KeyCode::Home => {
                // Scroll to top
                self.scroll_position = 0;
            }
            KeyCode::End => {
                // Scroll to bottom (handle this in the render method)
                self.scroll_position = usize::MAX;
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

fn ui(f: &mut Frame, app: &mut ChatApp) {
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
            
            // Create a wrapped, multi-line message
            let prefix_span = Span::styled(prefix, style);
            
            // Split the message text to allow wrapping
            let mut lines = Vec::new();
            let available_width = chunks[0].width.saturating_sub(2) as usize; // Account for borders
            let prefix_len = prefix.chars().count();
            let wrap_width = available_width.saturating_sub(prefix_len);
            
            // Split the message into words
            let mut current_line = String::new();
            let mut first_line = true;
            
            for word in msg.split_whitespace() {
                if first_line {
                    // First line includes the prefix
                    if current_line.len() + word.len() + (if !current_line.is_empty() { 1 } else { 0 }) <= wrap_width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        // Add the first line with prefix
                        let mut spans = vec![prefix_span.clone()];
                        spans.push(Span::raw(current_line));
                        lines.push(Line::from(spans));
                        
                        // Start a new line with the current word
                        current_line = word.to_string();
                        first_line = false;
                    }
                } else {
                    // Subsequent lines are indented to align with the text after the prefix
                    if current_line.len() + word.len() + (if !current_line.is_empty() { 1 } else { 0 }) <= available_width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        // Add the continuation line (with proper indentation)
                        let indent = " ".repeat(prefix_len);
                        let mut spans = vec![Span::raw(indent)];
                        spans.push(Span::raw(current_line));
                        lines.push(Line::from(spans));
                        
                        // Start a new line with the current word
                        current_line = word.to_string();
                    }
                }
            }
            
            // Add the last line
            if first_line {
                let mut spans = vec![prefix_span];
                spans.push(Span::raw(current_line));
                lines.push(Line::from(spans));
            } else {
                let indent = " ".repeat(prefix_len);
                let mut spans = vec![Span::raw(indent)];
                spans.push(Span::raw(current_line));
                lines.push(Line::from(spans));
            }
            
            ListItem::new(lines)
        })
        .collect();
    
    // Define the styles
    let user_style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);
    let assistant_style = Style::default().fg(Color::Blue);

    let mut list_items = Vec::new();
    let mut total_height: u16 = 0;

    // Use app.messages directly which should be a Vec<Message> with role and content fields
    for message in &app.messages {
        let is_user = message.role == "user";
        let style = if is_user { user_style } else { assistant_style };
        let prefix = if is_user { "You: " } else { "Assistant: " };
        
        // Create span directly from the content
        let spans = vec![Span::styled(prefix, style), Span::raw(&message.content)];
        let item = ListItem::new(Line::from(spans));
        
        // Calculate height by counting newlines plus 1
        let line_count = message.content.lines().count() as u16 + 1;
        total_height += line_count;
        
        list_items.push(item);
    }

    let visible_height = chunks[0].height.saturating_sub(2); // Account for borders
    let max_scroll = total_height.saturating_sub(visible_height);
    let offset = min(app.scroll_position as u16, max_scroll);

    // Create the list widget
    let list = List::new(list_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Chat"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .direction(ListDirection::TopToBottom);

    // Render list with scroll state
    f.render_stateful_widget(
        list, 
        chunks[0],
        &mut ListState::default().with_selected(Some(offset as usize))
    );
    
    // Input box
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input (Press Esc to quit)"))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
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
    let mut config = McpAiToolsConfig::default();
    
    // Configure OpenAI provider with the correct API structure
    let openai_settings = ProviderSettings::default_openai()
        .with_parameter("api_key".to_string(), json!(api_key))
        .with_parameter("temperature".to_string(), json!(0.7))
        .with_parameter("max_tokens".to_string(), json!(500));
        
    config = config
        .with_provider("openai".to_string(), openai_settings)
        .with_timeout(30000)
        .with_streaming(true);
    
    // Create MCP-AI tools adapter
    let adapter = create_mcp_ai_tools_adapter_with_config(mcp, config)
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
        terminal.draw(|f| ui(f, &mut app))?;
        
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