//! Chat state module
use std::time::Instant;
use std::path::{Path, PathBuf};
use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
    prelude::Alignment,
};

use super::types::InputMode;
use super::message::ChatMessage;
use super::render::{render_help_overlay, render_message};
use super::persistence::{default_history_file, load_history, save_history};

/// Represents the state of the chat interface
#[derive(Debug, Clone)]
pub struct ChatState {
    /// The messages in the chat
    pub messages: Vec<ChatMessage>,
    /// The current input text
    pub input: String,
    /// The cursor position in the input
    pub cursor_position: usize,
    /// Whether we're showing the help overlay
    pub show_help: bool,
    /// The last time we received a message
    pub last_update: Option<Instant>,
    /// Whether we are currently sending a message
    pub sending: bool,
    /// Current input mode
    pub input_mode: InputMode,
    /// The scroll position for the message history (0 = newest messages at bottom)
    pub scroll_position: usize,
    /// In-memory message history
    cached_messages: Vec<ChatMessage>,
    /// Optional history file path
    history_file: Option<PathBuf>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            show_help: false,
            last_update: None,
            sending: false,
            input_mode: InputMode::Normal,
            scroll_position: 0,
            cached_messages: Vec::new(),
            history_file: None,
        }
    }
}

impl ChatState {
    /// Create a new chat state
    pub fn new() -> Self {
        let mut state = Self::default();
        // Try to load history from the default file
        let _ = state.load_history_from_default_file();
        state
    }

    /// Create a new chat state with history from a specific file
    pub fn with_history_file(file_path: &Path) -> Self {
        let mut state = Self::default();
        state.history_file = Some(file_path.to_path_buf());
        let _ = state.load_history();
        state
    }

    /// Load history from the default file location
    pub fn load_history_from_default_file(&mut self) -> std::io::Result<()> {
        if let Some(file_path) = default_history_file() {
            self.history_file = Some(file_path);
            self.load_history()
        } else {
            log::warn!("Could not determine default history file location");
            Ok(())
        }
    }

    /// Load chat history from the configured file
    pub fn load_history(&mut self) -> std::io::Result<()> {
        if let Some(file_path) = &self.history_file {
            match load_history(file_path) {
                Ok(messages) => {
                    self.messages = messages;
                    Ok(())
                },
                Err(e) => Err(e)
            }
        } else {
            log::debug!("No history file configured");
            Ok(())
        }
    }

    /// Save chat history to file
    pub fn save_history(&self) -> std::io::Result<()> {
        if let Some(file_path) = &self.history_file {
            save_history(file_path, &self.messages)
        } else {
            log::debug!("No history file configured, skipping save");
            Ok(())
        }
    }

    /// Add a message to the chat
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message.clone());
        self.last_update = Some(Instant::now());
        
        // Also add to cached messages for persistence
        self.cached_messages.push(message);
        
        // Auto-scroll to the bottom when a new message is added
        self.scroll_to_bottom();
        
        // Save history after adding a message
        if let Err(e) = self.save_history() {
            log::error!("Failed to save chat history: {}", e);
        }
    }

    /// Add a user message to the chat
    pub fn add_user_message(&mut self, content: String) {
        if content.trim().is_empty() {
            log::debug!("UI: Attempted to add empty user message, ignoring");
            return;
        }
        log::debug!("UI: Adding user message: \"{}\"", content.chars().take(30).collect::<String>());
        self.add_message(ChatMessage::new_user(content));
        log::debug!("UI: Total messages after adding user message: {}", self.messages.len());
    }

    /// Add an AI message to the chat
    pub fn add_ai_message(&mut self, content: String) {
        if content.trim().is_empty() {
            log::debug!("UI: Attempted to add empty AI message, ignoring");
            return;
        }
        log::debug!("UI: Adding AI message: \"{}\"", content.chars().take(30).collect::<String>());
        self.add_message(ChatMessage::new_ai(content));
        self.sending = false;
        log::debug!("UI: Total messages after adding AI message: {}", self.messages.len());
    }

    /// Send the current input as a message
    pub fn send_message(&mut self) {
        if self.input.trim().is_empty() || self.sending {
            log::debug!("UI: Attempted to send empty message or already sending, ignoring");
            return;
        }
        log::debug!("UI: Sending message: \"{}\"", self.input.chars().take(30).collect::<String>());
        let message = self.input.clone();
        self.add_user_message(message);
        self.input.clear();
        self.cursor_position = 0;
        self.sending = true;
        // After sending, return to normal mode
        self.input_mode = InputMode::Normal;
        log::debug!("UI: Message sent, input cleared, mode set to Normal");
    }

    /// Clear the chat history
    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.cached_messages.clear();
        // Also clear persisted history
        let _ = self.save_history();
    }

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Delete a character at the cursor position
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Move the cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move the cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
    
    /// Toggle between normal and editing mode
    pub fn toggle_input_mode(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Normal => InputMode::Editing,
            InputMode::Editing => InputMode::Normal,
        };
    }
    
    /// Enter editing mode
    pub fn enter_edit_mode(&mut self) {
        self.input_mode = InputMode::Editing;
    }
    
    /// Enter normal mode
    pub fn enter_normal_mode(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Scroll up in the message history
    pub fn scroll_up(&mut self) {
        // Increase scroll position to look at older messages
        // Cap at the total number of messages
        let max_scroll = self.messages.len().saturating_sub(1);
        if self.scroll_position < max_scroll {
            self.scroll_position += 1;
            log::debug!("Scrolling up to position {}/{}", self.scroll_position, max_scroll);
        } else {
            log::debug!("Already at top of scroll history: {}/{}", self.scroll_position, max_scroll);
        }
    }

    /// Scroll down in the message history
    pub fn scroll_down(&mut self) {
        // Decrease scroll position to look at newer messages
        if self.scroll_position > 0 {
            self.scroll_position -= 1;
            log::debug!("Scrolling down to position {}", self.scroll_position);
        } else {
            log::debug!("Already at bottom of scroll history");
        }
    }

    /// Scroll to the top of the message history
    pub fn scroll_to_top(&mut self) {
        // Set to max scroll position to view oldest messages
        self.scroll_position = self.messages.len().saturating_sub(1);
        log::debug!("Scrolled to top (position {})", self.scroll_position);
    }

    /// Scroll to the bottom of the message history
    pub fn scroll_to_bottom(&mut self) {
        // Set to 0 to view newest messages
        self.scroll_position = 0;
        log::debug!("Scrolled to bottom (position 0)");
    }

    /// Handle key input based on current mode
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> (bool, bool) {
        // Returns (handled, should_quit)
        match self.input_mode {
            InputMode::Normal => {
                // Handle normal mode key events
                match key.code {
                    crossterm::event::KeyCode::Char('q') => {
                        return (true, true); // Quit
                    }
                    crossterm::event::KeyCode::Char('i') => {
                        self.enter_edit_mode();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Char('?') => {
                        self.toggle_help();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Up => {
                        self.scroll_up();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Down => {
                        self.scroll_down();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Home => {
                        self.scroll_to_top();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::End => {
                        self.scroll_to_bottom();
                        return (true, false);
                    }
                    _ => return (false, false),
                }
            }
            InputMode::Editing => {
                // Handle editing mode key events
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        self.enter_normal_mode();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.send_message();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Backspace => {
                        self.delete_char();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Left => {
                        self.move_cursor_left();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Right => {
                        self.move_cursor_right();
                        return (true, false);
                    }
                    crossterm::event::KeyCode::Char(c) => {
                        // This is crucial - in editing mode, all characters (including 'q') 
                        // are inserted as text, not processed as commands
                        self.insert_char(c);
                        return (true, false);
                    }
                    _ => return (false, false),
                }
            }
        }
    }

    /// Render the chat state to the screen
    pub fn render<B: Backend>(&self, f: &mut Frame<'_>, area: Rect) {
        let messages = &self.messages;
        
        // Create a layout for the messages and input area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),       // Messages area
                Constraint::Length(3),    // Input area with border
            ])
            .split(area);
        
        let messages_area = chunks[0];
        let input_area = chunks[1];
        
        // Create a block for the messages
        let messages_block = Block::default()
            .title("Chat")
            .borders(Borders::ALL);
        
        // Render the messages block
        f.render_widget(messages_block.clone(), messages_area);
        // Get the inner area of the messages block
        let inner_messages_area = messages_block.inner(messages_area);
        
        // Messages to display
        let start_idx = self.scroll_position.min(messages.len().saturating_sub(1));
        let visible_messages = if !messages.is_empty() {
            let filtered_messages = messages.iter()
                .take(messages.len() - start_idx)
                .collect::<Vec<_>>();
            filtered_messages
        } else {
            Vec::new()
        };
        
        // Calculate total number of displayed messages
        let displayed_count = visible_messages.len();
        let available_height = inner_messages_area.height as usize;
        
        log::debug!("Rendering {} messages with scroll_position {}, available height {}", 
            displayed_count, self.scroll_position, available_height);
        
        // Use layout to position messages vertically
        if !visible_messages.is_empty() {
            // Calculate constraints for message areas based on estimated content height
            let mut constraints = Vec::with_capacity(displayed_count);
            
            // First pass: give each message a minimum constraint
            for msg in &visible_messages {
                // Calculate rough height based on content
                let line_count = msg.content.chars().filter(|&c| c == '\n').count() + 1;
                // Rough estimate of wrapped lines (characters / width)
                let wrap_count = (msg.content.len() / inner_messages_area.width as usize).max(1);
                
                // Height = lines + estimated wrapped lines + padding
                let height = (line_count + wrap_count).min(20) as u16; // Cap at 20 lines for any message
                constraints.push(Constraint::Min(height));
            }
            
            // Create layout with calculated constraints
            let message_areas = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner_messages_area);
            
            // Render each message in its area
            for (i, msg) in visible_messages.iter().enumerate() {
                if i < message_areas.len() {
                    // Use the specialized render_message function
                    render_message(f, message_areas[i], &msg.content, msg.is_user, false);
                }
            }
        } else {
            // No messages, display placeholder
            let text = Text::from("No messages yet. Type something below to start chatting!");
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, inner_messages_area);
        }
        
        // Input area at the bottom
        let input_block = match self.input_mode {
            InputMode::Normal => {
                Block::default()
                    .title("Input [i to edit]")
                    .borders(Borders::ALL)
                    .border_style(Style::default())
            }
            InputMode::Editing => {
                Block::default()
                    .title("Input [Esc to cancel, Enter to send]")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            }
        };
        
        let input_text = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(input_block);
        
        f.render_widget(input_text, input_area);
        
        // Show cursor if we're in editing mode
        if let InputMode::Editing = self.input_mode {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                input_area.x + self.cursor_position as u16 + 1,
                // Put cursor on the input line
                input_area.y + 1,
            );
        }
        
        // Render help overlay if needed
        if self.show_help {
            render_help_overlay(f, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_state_default() {
        let state = ChatState::default();
        assert!(state.messages.is_empty());
        assert_eq!(state.input, "");
        assert_eq!(state.cursor_position, 0);
        assert!(!state.show_help);
    }

    #[test]
    fn test_chat_state_add_message() {
        let mut state = ChatState::default();
        let msg = ChatMessage::new_user("Hello".to_string());
        state.add_message(msg.clone());
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].content, msg.content);
        assert_eq!(state.messages[0].is_user, msg.is_user);
    }
} 