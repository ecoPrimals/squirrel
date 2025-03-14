use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::ui::{
    components::{Component, ComponentId, ComponentError},
    layout::{Size},
    theme::{Theme, Themeable, ColorRole, StyleRole},
    events::Event,
};
use crate::core::error::Result;

/// A header component that displays a title
pub struct Header {
    id: ComponentId,
    theme: Theme,
    title: String,
}

impl Header {
    /// Create a new header component
    pub fn new(id: ComponentId, theme: Theme, title: String) -> Self {
        Self {
            id,
            theme,
            title,
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

impl Component for Header {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), ComponentError> {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(self.theme.get_style(StyleRole::Default));

        let title = Paragraph::new(self.title.as_str())
            .block(block)
            .style(self.theme.get_style(StyleRole::Default));

        frame.render_widget(title, area);
        Ok(())
    }

    fn handle_event(&mut self, _event: &dyn Event) -> std::result::Result<(), ComponentError> {
        // Headers don't handle events
        Ok(())
    }

    fn required_size(&self) -> Size {
        Size::new(0, 3) // Fixed height for header with borders
    }

    fn id(&self) -> ComponentId {
        self.id
    }
}

impl Themeable for Header {
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

    fn get_style(&self) -> Style {
        self.theme.get_style(StyleRole::Default)
    }

    fn get_color(&self, role: ColorRole) -> ratatui::style::Color {
        self.theme.get_color(role)
    }
} 