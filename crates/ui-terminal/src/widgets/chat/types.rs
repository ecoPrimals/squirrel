//! Common types for the chat widget

/// Input mode for the chat interface
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InputMode {
    /// Normal mode - used for navigation and commands
    Normal,
    /// Editing mode - used for typing messages
    Editing,
}

impl InputMode {
    /// Determines if a key event should be processed as a global command or as text input
    /// Returns true if the key should be processed as a command, false if it should be treated as text input
    pub fn should_process_globally(&self, key: crossterm::event::KeyCode) -> bool {
        match self {
            InputMode::Normal => true, // In normal mode, all keys are processed as commands
            InputMode::Editing => {
                // In editing mode, only specific keys should be processed as commands
                // Everything else (including 'q') should be treated as text input
                matches!(key, 
                    crossterm::event::KeyCode::Esc |
                    crossterm::event::KeyCode::Enter |
                    crossterm::event::KeyCode::Tab |
                    crossterm::event::KeyCode::BackTab
                )
            }
        }
    }
} 