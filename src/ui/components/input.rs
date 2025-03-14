use std::fmt;
use ratatui::prelude::{Frame, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};
use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::components::{Component, ComponentError, ComponentId, Size};
use crate::ui::theme::{Theme, Themeable, ColorRole, StyleRole};
use crate::ui::events::Event;

/// An input component for text entry
pub struct Input {
    id: ComponentId,
    theme: Theme,
    value: String,
    placeholder: String,
}

impl Input {
    /// Create a new input component
    pub fn new(id: ComponentId, theme: Theme, placeholder: String) -> Self {
        Self {
            id,
            theme,
            value: String::new(),
            placeholder,
        }
    }

    /// Get the current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the current value
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    /// Get the placeholder
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Set the placeholder
    pub fn set_placeholder(&mut self, placeholder: String) {
        self.placeholder = placeholder;
    }
}

impl Component for Input {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), ComponentError> {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(self.theme.get_style(StyleRole::Default));

        let text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        let input = Paragraph::new(text)
            .block(block)
            .style(self.theme.get_style(StyleRole::Default));

        frame.render_widget(input, area);
        Ok(())
    }

    fn handle_event(&mut self, event: &dyn Event) -> std::result::Result<(), ComponentError> {
        // Handle input events here
        Ok(())
    }

    fn required_size(&self) -> Size {
        Size::new(0, 3) // Fixed height for input with borders
    }

    fn id(&self) -> ComponentId {
        self.id
    }
}

impl Themeable for Input {
    fn theme(&self) -> Option<&Theme> {
        Some(&self.theme)
    }

    fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }

    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    fn apply_theme(&mut self, theme: &Theme) -> std::result::Result<(), ComponentError> {
        self.theme = theme.clone();
        Ok(())
    }
} 