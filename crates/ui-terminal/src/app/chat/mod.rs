pub mod openai;

use std::sync::Arc;
use crossterm::event::{Event, KeyCode, KeyEvent};
use dashboard_core::service::DashboardService;
use tokio::sync::{Mutex, mpsc, RwLock};
use crate::widgets::chat::{ChatState, InputMode};
use crate::error::Error;
pub use openai::{OpenAIService, OpenAIError};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use rand::Rng;
use serde_json;
use chrono;
use log::{debug, error, info, warn};
use squirrel_mcp::chat_history::ChatHistoryClient;
use std::path::PathBuf;

/// Chat message handler used for processing MCP updates
struct ChatMessageHandler<S: DashboardService + ?Sized + 'static> {
    app: Arc<RwLock<ChatApp<S>>>,
}

impl<S: DashboardService + ?Sized + 'static> ChatMessageHandler<S> {
    fn new(app: Arc<RwLock<ChatApp<S>>>) -> Self {
        Self { app }
    }
    
    async fn handle_messages(&self, history: Vec<(String, bool, u64)>) {
        debug!("Processing {} messages from MCP update", history.len());
        
        match self.app.write().await {
            mut app => {
                app.import_conversation_history(history);
            }
        }
    }
}

/// Response type for chat messages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChatResponse {
    /// Response content
    pub content: String,
    /// Whether this is an error message
    pub is_error: bool,
    /// Unique ID for deduplication
    pub id: u64,
}

impl ChatResponse {
    /// Create a new final response
    pub fn final_response(content: String) -> Self {
        Self {
            content,
            is_error: false,
            id: generate_unique_id(),
        }
    }

    /// Create a new error response
    pub fn error(content: String) -> Self {
        Self {
            content: format!("Error: {}", content),
            is_error: true,
            id: generate_unique_id(),
        }
    }
}

/// Generate a unique ID based on timestamp and random component
fn generate_unique_id() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let random_component = rand::thread_rng().gen::<u16>() as u64;
    now ^ (random_component << 48)
}

/// Chat application
pub struct ChatApp<S: ?Sized> {
    /// Dashboard service
    pub service: Arc<S>,
    /// Chat state
    pub state: ChatState,
    /// Current input text
    pub input: String,
    /// Whether to show help
    pub show_help: bool,
    /// Should quit
    pub should_quit: bool,
    /// Message transmitter
    pub tx: mpsc::Sender<ChatResponse>,
    /// Message receiver - excluded from clone
    pub rx: mpsc::Receiver<ChatResponse>,
    /// Has temporary message (thinking)
    pub has_temp_message: bool,
    /// OpenAI service (if configured)
    pub openai: Option<Arc<Mutex<OpenAIService>>>,
    /// Set of processed message IDs for deduplication
    pub processed_messages: HashSet<u64>,
}

impl<S: ?Sized> Clone for ChatApp<S> {
    fn clone(&self) -> Self {
        // Create a new channel for the clone
        let (tx, rx) = mpsc::channel(100);
        
        Self {
            service: self.service.clone(),
            state: self.state.clone(),
            input: self.input.clone(),
            show_help: self.show_help,
            should_quit: self.should_quit,
            tx,
            rx,
            has_temp_message: self.has_temp_message,
            openai: self.openai.clone(),
            processed_messages: self.processed_messages.clone(),
        }
    }
}

impl<S: DashboardService + ?Sized + 'static> ChatApp<S> {
    /// Create a new chat application
    pub fn new(service: Arc<S>) -> Self {
        // Try to initialize OpenAI service
        let openai = match OpenAIService::new() {
            Ok(service) => Some(Arc::new(Mutex::new(service))),
            Err(e) => {
                eprintln!("Failed to initialize OpenAI service: {}", e);
                None
            }
        };
        
        // Create channel for receiving responses
        let (tx, rx) = mpsc::channel(10);
        
        Self {
            service,
            state: ChatState::new(),
            show_help: false,
            input: String::new(),
            should_quit: false,
            tx,
            rx,
            has_temp_message: false,
            openai,
            processed_messages: HashSet::new(),
        }
    }

    /// Handle key event
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // If help is shown, any key dismisses it
        if self.state.show_help {
            self.state.show_help = false;
            return;
        }

        // First check the current input mode to determine how to handle input
        match self.state.input_mode {
            InputMode::Normal => {
                // In Normal mode, handle common commands
                match key.code {
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                        return;
                    },
                    KeyCode::Char('?') => {
                        self.state.toggle_help();
                        return;
                    },
                    KeyCode::Char('i') => {
                        // Enter editing mode
                        self.state.enter_edit_mode();
                        return;
                    },
                    KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        // Clear history with Ctrl+K
                        self.clear_history();
                        return;
                    },
                    KeyCode::Up => {
                        self.state.scroll_up();
                        return;
                    },
                    KeyCode::Down => {
                        self.state.scroll_down();
                        return;
                    },
                    KeyCode::PageUp => {
                        // Scroll up by 10 messages
                        for _ in 0..10 {
                            self.state.scroll_up();
                        }
                        return;
                    },
                    KeyCode::PageDown => {
                        // Scroll down by 10 messages
                        for _ in 0..10 {
                            self.state.scroll_down();
                        }
                        return;
                    },
                    KeyCode::Home => {
                        self.state.scroll_to_top();
                        return;
                    },
                    KeyCode::End => {
                        self.state.scroll_to_bottom();
                        return;
                    },
                    KeyCode::Char('g') => {
                        // Go to top of chat history
                        self.state.scroll_to_top();
                        return;
                    },
                    KeyCode::Char('G') => {
                        // Go to bottom of chat history
                        self.state.scroll_to_bottom();
                        return;
                    },
                    _ => {},
                }
            },
            InputMode::Editing => {
                // In Editing mode, handle text input
                match key.code {
                    KeyCode::Esc => {
                        // Exit editing mode without clearing input
                        self.state.enter_normal_mode();
                        return;
                    },
                    KeyCode::Enter => {
                        // Send message
                        self.send_message();
                        return;
                    },
                    KeyCode::Char(c) => {
                        // Insert the character (including 'q' which should just be a character in this mode)
                        self.state.insert_char(c);
                        return;
                    },
                    KeyCode::Backspace => {
                        // Delete character
                        self.state.delete_char();
                        return;
                    },
                    KeyCode::Left => {
                        // Move cursor left
                        self.state.move_cursor_left();
                        return;
                    },
                    KeyCode::Right => {
                        // Move cursor right
                        self.state.move_cursor_right();
                        return;
                    },
                    // In editing mode, we still want to support scrolling
                    KeyCode::Up if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        self.state.scroll_up();
                        return;
                    },
                    KeyCode::Down if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                        self.state.scroll_down();
                        return;
                    },
                    _ => {},
                }
            },
        }
    }

    /// Handle event
    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            self.handle_key_event(key);
        }
    }
    
    /// Check for and process any received messages from the channel
    pub fn process_received_messages(&mut self) {
        // Process all new messages in the channel
        let mut new_messages = Vec::new();
        
        // Try to receive messages without blocking
        while let Ok(response) = self.rx.try_recv() {
            // Only process messages we haven't seen before
            if !self.processed_messages.contains(&response.id) {
                log::debug!("Found new message with ID: {}", response.id);
                new_messages.push(response);
            } else {
                log::debug!("Skipping already processed message ID: {}", response.id);
            }
        }
        
        log::debug!("Collected {} new messages during this processing cycle", new_messages.len());
        
        // Process only the most recent message if any
        if let Some(response) = new_messages.last() {
            log::debug!("Processing most recent message ID: {}", response.id);
            self.process_single_message(response.clone());
        }
    }
    
    /// Process a single message and update UI accordingly
    fn process_single_message(&mut self, response: ChatResponse) {
        // Mark this message as processed
        self.processed_messages.insert(response.id);
        
        log::debug!("Processing message ID {}: {} (is_error={})", response.id, 
            response.content.chars().take(30).collect::<String>(), 
            response.is_error);
        
        // Check if the user is at the bottom of the scroll view (viewing most recent messages)
        // If scroll_position is 0, the user is at the bottom
        let was_at_bottom = self.state.scroll_position == 0;
        
        if self.has_temp_message {
            // Find the temporary "thinking" message which should be the last non-user message
            // Start from the end and find the first non-user message
            let last_non_user_idx = self.state.messages.iter()
                .enumerate()
                .rev()
                .find(|(_, msg)| !msg.is_user)
                .map(|(idx, _)| idx);
            
            if let Some(last_idx) = last_non_user_idx {
                // Check if this is actually a thinking message
                let is_thinking = self.state.messages[last_idx].content.starts_with("Thinking about your message");
                
                if is_thinking {
                    // Update the content of the temporary message
                    log::debug!("Replacing temp thinking message at position {} with response", last_idx);
                    self.state.messages[last_idx].content = response.content;
                    
                    // Mark as no longer temporary
                    self.has_temp_message = false;
                    
                    // Update the state
                    self.state.sending = false;
                    self.state.last_update = Some(std::time::Instant::now());
                    
                    log::debug!("Updated UI state with message");
                } else {
                    // The last non-user message is not a thinking message
                    // Just add a new message
                    log::debug!("Last non-user message is not a thinking message, adding as new");
                    self.state.add_ai_message(response.content);
                    self.has_temp_message = false;
                    self.state.sending = false;
                }
            } else {
                // No AI messages found, add a new one
                log::debug!("No AI messages found, adding as new");
                self.state.add_ai_message(response.content);
                self.has_temp_message = false;
                self.state.sending = false;
            }
        } else {
            // No temporary message, just add the new one
            log::debug!("Adding response as new message");
            self.state.add_ai_message(response.content);
            self.state.sending = false;
        }
        
        // Only auto-scroll if the user was already at the bottom
        // This prevents interrupting manual scrolling through history
        if was_at_bottom {
            log::debug!("User was at bottom, auto-scrolling to show new message");
            self.state.scroll_to_bottom();
        } else {
            log::debug!("User was scrolling history, not auto-scrolling to bottom");
        }
    }

    /// Send a message to the AI
    fn send_message(&mut self) {
        // Only proceed if we're not already sending
        if self.state.sending {
            log::debug!("Message already being sent, ignoring this send_message call");
            return;
        }

        // Get the message text
        let message = self.state.input.clone();
        if message.trim().is_empty() {
            log::debug!("Empty message, ignoring");
            return;
        }
        
        // Add user message to chat
        self.state.send_message();
        log::debug!("Added user message to chat: {}", message);
        
        // Always scroll to bottom when sending a new message
        // This ensures the user can see their message
        self.state.scroll_to_bottom();
        log::debug!("Auto-scrolled to bottom to show new user message");
        
        // Do NOT clear processed messages, as this would lose message history
        // Instead, we'll deduplicate based on the message IDs
        log::debug!("Currently tracking {} processed message IDs", self.processed_messages.len());
        
        // If we have an OpenAI service, use it to get a response
        if let Some(openai) = &self.openai {
            // Clone what we need to move into the async block
            let openai = openai.clone();
            let message_clone = message.clone();
            
            // Convert chat history to format needed by OpenAI
            let history = self.get_conversation_history();
            log::debug!("Prepared conversation history with {} messages for API call", history.len());
            
            // Set sending state
            self.state.sending = true;
            
            // Add a thinking message
            let thinking_msg = format!("Thinking about your message: \"{}\"...", message);
            self.state.add_ai_message(thinking_msg);
            self.has_temp_message = true;
            log::debug!("Added temporary 'thinking' message");
            
            // Auto-scroll to bottom to show the thinking message
            self.state.scroll_to_bottom();
            log::debug!("Auto-scrolled to bottom to show thinking message");
            
            // Clone the sender for this task
            let tx = self.tx.clone();
            
            // Spawn async task to handle the API call
            tokio::spawn(async move {
                log::debug!("Starting async task to send message");
                let service = openai.lock().await;
                
                // Only send one message in the async task
                let result = service.send_message(&message_clone, &history).await;
                
                match result {
                    Ok(response) => {
                        let response_preview = response.chars().take(30).collect::<String>();
                        log::debug!("Received response from service: {}", response_preview);
                        // Create a final response with unique ID
                        let response_msg = ChatResponse::final_response(response);
                        log::debug!("Created final response with ID: {}", response_msg.id);
                        // Send through channel
                        if let Err(e) = tx.send(response_msg).await {
                            log::error!("Failed to send response through channel: {}", e);
                        } else {
                            log::debug!("Sent AI response through channel");
                        }
                    }
                    Err(e) => {
                        log::error!("Error from service: {}", e);
                        // Create an error response with unique ID
                        let error_msg = ChatResponse::error(e.to_string());
                        log::debug!("Created error response with ID: {}", error_msg.id);
                        // Send through channel
                        if let Err(e) = tx.send(error_msg).await {
                            log::error!("Failed to send error through channel: {}", e);
                        }
                    }
                }
            });
        } else {
            // If no OpenAI service, use a mock response
            log::debug!("No OpenAI service, using mock response");
            self.state.add_ai_message(format!("(Mock) I received your message: \"{}\"", message));
            self.has_temp_message = false;
            // Auto-scroll to bottom to show the mock response
            self.state.scroll_to_bottom();
            log::debug!("Auto-scrolled to bottom to show mock response");
        }
    }
    
    /// Get conversation history for OpenAI
    fn get_conversation_history(&self) -> Vec<(String, bool)> {
        // Filter out temporary thinking messages but keep all other messages
        if self.has_temp_message && !self.state.messages.is_empty() {
            // Convert all messages to a vector, but exclude the temporary thinking message
            self.state.messages.iter()
                .filter(|msg| {
                    msg.is_user || 
                    !msg.content.starts_with("Thinking about your message")
                })
                .map(|msg| (msg.content.clone(), msg.is_user))
                .collect()
        } else {
            // Use all messages
            self.state.messages.iter()
                .map(|msg| (msg.content.clone(), msg.is_user))
                .collect()
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    /// Update connection status based on service health
    pub async fn update_connection_status(&mut self) -> Result<(), Error> {
        // We're not really using this for the chat UI, but it's required for the API
        // In a real implementation, we would check the service health and update the UI
        Ok(())
    }

    /// Clear chat history
    pub fn clear_history(&mut self) {
        // Clear the chat history from the widget state
        self.state.clear_history(); // This returns void, so no need to check for errors
        log::debug!("Chat history cleared");
    }

    /// Export the conversation history in a format usable by other systems
    pub fn export_conversation_history(&self) -> Vec<(String, bool, u64)> {
        // Export messages with content, user flag, and timestamp
        self.state.messages.iter()
            .map(|msg| {
                let timestamp = msg.timestamp.timestamp() as u64;
                (msg.content.clone(), msg.is_user, timestamp)
            })
            .collect()
    }

    /// Get a serializable representation of the conversation history
    pub fn get_serializable_history(&self) -> Vec<serde_json::Value> {
        self.state.messages.iter()
            .map(|msg| {
                let mut map = serde_json::Map::new();
                map.insert("content".to_string(), serde_json::Value::String(msg.content.clone()));
                map.insert("is_user".to_string(), serde_json::Value::Bool(msg.is_user));
                map.insert("timestamp".to_string(), 
                           serde_json::Value::Number(serde_json::Number::from(
                               msg.timestamp.timestamp())));
                serde_json::Value::Object(map)
            })
            .collect()
    }

    /// Sync conversation history with MCP
    pub async fn sync_with_mcp(&self, mcp: &squirrel_mcp::client::MCPClient) -> crate::error::Result<()> {
        debug!("Syncing chat history with MCP");
        
        // Export conversation history as tuples
        let history = self.export_conversation_history();
        
        // Update MCP chat history
        mcp.update_chat_history_from_tuples(history)
            .await
            .map_err(|e| Error::External(format!("Failed to sync with MCP: {}", e)))?;
        
        debug!("Successfully synced chat history with MCP");
        Ok(())
    }

    /// Import conversation history from external source
    pub fn import_conversation_history(&mut self, history: Vec<(String, bool, u64)>) {
        // Clear existing history
        self.state.messages.clear();
        
        // Import the new messages
        for (content, is_user, timestamp) in history {
            let timestamp_dt = chrono::DateTime::from_timestamp(timestamp as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
                
            let message = if is_user {
                crate::widgets::chat::ChatMessage {
                    content,
                    is_user: true,
                    timestamp: timestamp_dt,
                }
            } else {
                crate::widgets::chat::ChatMessage {
                    content,
                    is_user: false,
                    timestamp: timestamp_dt,
                }
            };
            
            self.state.messages.push(message);
        }
        
        log::debug!("Imported {} messages from external source", self.state.messages.len());
    }

    /// Import chat history from MCP
    pub async fn import_from_mcp(&mut self, mcp: &squirrel_mcp::client::MCPClient) -> crate::error::Result<()> {
        debug!("Importing chat history from MCP");
        
        // Get chat history from MCP
        let messages = mcp.get_chat_history()
            .await
            .map_err(|e| Error::External(format!("Failed to get chat history from MCP: {}", e)))?;
        
        if messages.is_empty() {
            debug!("No chat history found in MCP");
            return Ok(());
        }
        
        debug!("Found {} messages in MCP chat history", messages.len());
        
        // Convert to tuple format expected by import_conversation_history
        let history: Vec<(String, bool, u64)> = messages
            .into_iter()
            .map(|msg| msg.to_tuple())
            .collect();
        
        // Import the history
        self.import_conversation_history(history);
        
        debug!("Successfully imported chat history from MCP");
        Ok(())
    }

    /// Subscribe to chat history updates from MCP
    /// 
    /// This method sets up a subscription to receive updates from the MCP client
    /// when chat history changes occur.
    pub async fn subscribe_to_mcp_updates(&self, mcp: squirrel_mcp::client::MCPClient) -> crate::error::Result<()> {
        debug!("Setting up subscription for MCP chat history updates");
        
        // Create a shared reference to the ChatApp that can be moved into the async task
        let app = Arc::new(RwLock::new(self.clone()));
        let handler = ChatMessageHandler::new(app);
        
        // Spawn a task to receive updates from MCP
        tokio::spawn(async move {
            match mcp.subscribe_to_chat_updates().await {
                Ok(mut subscription) => {
                    debug!("Successfully subscribed to MCP chat updates");
                    
                    // Process updates in a loop
                    while let Some(messages) = subscription.next_update().await {
                        debug!("Received {} messages from MCP", messages.len());
                        
                        // Convert messages to the format expected by the handler
                        let history: Vec<(String, bool, u64)> = messages
                            .into_iter()
                            .map(|msg| msg.to_tuple())
                            .collect();
                        
                        // Process the messages
                        handler.handle_messages(history).await;
                    }
                },
                Err(err) => {
                    error!("Failed to subscribe to MCP updates: {}", err);
                }
            }
        });
        
        debug!("Successfully set up MCP chat history subscription");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[test]
    fn test_chat_response_creation() {
        // Test that creating responses generates unique IDs
        let response1 = ChatResponse::final_response("test message".to_string());
        let response2 = ChatResponse::final_response("test message".to_string());
        
        // Even with the same content, IDs should be different
        assert_ne!(response1.id, response2.id);
        assert_eq!(response1.content, response2.content);
        assert!(!response1.is_error);
    }
    
    #[test]
    fn test_error_response_creation() {
        let response = ChatResponse::error("error message".to_string());
        assert!(response.is_error);
        assert!(response.content.contains("Error: error message"));
    }
    
    #[tokio::test]
    async fn test_message_deduplication() {
        // Create channels for testing
        let (tx, mut rx) = mpsc::channel(10);
        
        // Create some test responses
        let response1 = ChatResponse::final_response("message 1".to_string());
        let response1_id = response1.id;
        let response2 = ChatResponse::final_response("message 2".to_string());
        let response2_id = response2.id;
        
        // Send the messages
        tx.send(response1.clone()).await.unwrap();
        tx.send(response1.clone()).await.unwrap(); // Duplicate message
        tx.send(response2.clone()).await.unwrap();
        
        // Create a processor with deduplication
        let mut processed = HashSet::new();
        let mut new_messages = Vec::new();
        
        // Process the first message
        let msg1 = rx.recv().await.unwrap();
        assert_eq!(msg1.id, response1_id);
        if !processed.contains(&msg1.id) {
            new_messages.push(msg1);
            processed.insert(response1_id);
        }
        
        // Process the duplicate message
        let msg2 = rx.recv().await.unwrap();
        assert_eq!(msg2.id, response1_id); // Same ID as first message
        if !processed.contains(&msg2.id) {
            new_messages.push(msg2);
        }
        
        // Process the second unique message
        let msg3 = rx.recv().await.unwrap();
        assert_eq!(msg3.id, response2_id);
        if !processed.contains(&msg3.id) {
            new_messages.push(msg3);
            processed.insert(response2_id);
        }
        
        // We should have only 2 messages after deduplication
        assert_eq!(new_messages.len(), 2);
        assert_eq!(new_messages[0].content, "message 1");
        assert_eq!(new_messages[1].content, "message 2");
    }
    
    #[tokio::test]
    async fn test_thinking_message_replacement() {
        // Create a mock chat app with the dashboard service
        let service = Arc::new(MockDashboardService::new());
        let mut app = ChatApp::<MockDashboardService>::new(service);
        
        // Add a thinking message
        app.has_temp_message = true;
        app.state.add_ai_message("Thinking...".to_string());
        
        // Verify the thinking message is there
        assert_eq!(app.state.messages.len(), 1);
        assert_eq!(app.state.messages[0].content, "Thinking...");
        
        // Process a final response
        let response = ChatResponse::final_response("Final answer".to_string());
        app.process_single_message(response);
        
        // Verify the thinking message was replaced
        assert_eq!(app.state.messages.len(), 1); // Still just one message
        assert_eq!(app.state.messages[0].content, "Final answer"); // But content updated
        assert!(!app.has_temp_message); // No longer marked as temporary
    }
}

// Add a mock dashboard service for testing
#[cfg(test)]
use dashboard_core::service::MockDashboardService; 