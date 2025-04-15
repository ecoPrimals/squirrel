use std::sync::Arc;
use ratatui::widgets::ListState;
use ratatui::{Frame, layout::Rect};

use squirrel_integration::mcp_ai_tools::McpAiToolsAdapter;

use super::models::AiModel;
use super::messages::ChatMessage;
use super::render;

/// State for the AI Chat widget
#[derive(Debug)]
pub struct AiChatWidgetState {
    /// The input text to send to the AI
    pub input: String,
    
    /// Whether the input is focused
    pub input_focused: bool,
    
    /// The messages in the chat
    pub messages: Vec<ChatMessage>,
    
    /// Adapter for AI interactions
    pub adapter: Arc<McpAiToolsAdapter>,
    
    /// Whether a message is currently being sent
    pub is_sending: bool,
    
    /// Whether a response is being generated
    pub generating_response: bool,
    
    /// Available AI models
    pub models: Vec<AiModel>,
    
    /// The currently selected model
    pub selected_model: usize,
    
    /// List state for message navigation
    pub list_state: ListState,
    
    /// Whether the model selection popup is visible
    pub show_model_selection: bool,
    
    /// Current model selection index (may differ from selected_model until confirmed)
    pub model_selection_index: usize,
    
    /// Current scroll position in the message list
    pub scroll_position: usize,
}

impl AiChatWidgetState {
    /// Create a new AI chat widget state
    pub fn new(adapter: Arc<McpAiToolsAdapter>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            input: String::new(),
            input_focused: true,
            messages: vec![
                ChatMessage::system("Welcome to AI Chat. Type your message below.".to_string()),
            ],
            adapter,
            is_sending: false,
            generating_response: false,
            models: AiModel::all(),
            selected_model: 0,
            list_state,
            show_model_selection: false,
            model_selection_index: 0,
            scroll_position: 0,
        }
    }
    
    /// Add a user message to the chat
    pub fn add_user_message(&mut self, content: String) {
        if content.trim().is_empty() {
            return;
        }
        
        self.messages.push(ChatMessage::user(content));
        self.scroll_to_bottom();
    }
    
    /// Add an assistant message to the chat
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage::assistant(content));
        self.scroll_to_bottom();
    }
    
    /// Add a system message to the chat
    pub fn add_system_message(&mut self, content: String) {
        self.messages.push(ChatMessage::system(content));
        self.scroll_to_bottom();
    }
    
    /// Get the selected model
    pub fn get_selected_model(&self) -> Result<&AiModel, String> {
        if self.models.is_empty() {
            return Err("No models available".to_string());
        }
        
        self.models.get(self.selected_model)
            .ok_or_else(|| "Invalid model selected".to_string())
    }
    
    /// Select the next model
    pub fn next_model(&mut self) {
        if !self.models.is_empty() {
            self.model_selection_index = (self.model_selection_index + 1) % self.models.len();
        }
    }
    
    /// Select the previous model
    pub fn previous_model(&mut self) {
        if !self.models.is_empty() {
            self.model_selection_index = if self.model_selection_index == 0 {
                self.models.len() - 1
            } else {
                self.model_selection_index - 1
            };
        }
    }
    
    /// Also aliased as prev_model for compatibility
    pub fn prev_model(&mut self) {
        self.previous_model();
    }
    
    /// Scroll the message list to the bottom
    pub fn scroll_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.list_state.select(Some(self.messages.len() - 1));
            self.scroll_position = self.messages.len().saturating_sub(1);
        }
    }
    
    /// Find the latest message of a specific role
    pub fn latest_message_by_role(&self, role: &str) -> Option<&ChatMessage> {
        self.messages.iter().filter(|m| m.role == role).last()
    }
    
    /// Find the latest message of a specific role (mutable)
    pub fn latest_message_by_role_mut(&mut self, role: &str) -> Option<&mut ChatMessage> {
        self.messages.iter_mut().filter(|m| m.role == role).last()
    }
    
    /// Clear the chat history
    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.add_system_message("Chat history cleared.".to_string());
    }
    
    /// Toggle the model selection popup
    pub fn toggle_model_selection(&mut self) {
        self.show_model_selection = !self.show_model_selection;
        if self.show_model_selection {
            self.model_selection_index = self.selected_model;
        }
    }
    
    /// Confirm the model selection
    pub fn confirm_model_selection(&mut self) {
        self.selected_model = self.model_selection_index;
        self.show_model_selection = false;
    }
    
    /// Focus the input field
    pub fn focus_input(&mut self) {
        self.input_focused = true;
        self.show_model_selection = false;
    }
    
    /// Render the AI chat widget
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        render::render(f, area, &mut self.clone())
    }
}

// Need to implement Clone for AiChatWidgetState to make draw work
impl Clone for AiChatWidgetState {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            input_focused: self.input_focused,
            messages: self.messages.clone(),
            adapter: self.adapter.clone(),
            is_sending: self.is_sending,
            generating_response: self.generating_response,
            models: self.models.clone(),
            selected_model: self.selected_model,
            list_state: {
                let mut state = ListState::default();
                if let Some(selected) = self.list_state.selected() {
                    state.select(Some(selected));
                }
                state
            },
            show_model_selection: self.show_model_selection,
            model_selection_index: self.model_selection_index,
            scroll_position: self.scroll_position,
        }
    }
} 