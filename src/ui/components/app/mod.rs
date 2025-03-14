use crossterm::event::{KeyEvent, KeyCode};
use ratatui::{
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    style::{Style, Modifier},
    text::{Line, Span},
    Frame,
};
use crate::ui::{Size, Rect};
use crate::ui::error::UiError;
use crate::ui::theme::{Theme, Themeable, ColorRole, StyleRole};
use crate::ui::components::{Component, Resizable};
use std::sync::{Arc, Mutex};
use crate::ui::components::{ComponentId, ComponentError};
use crate::ui::events::Event;

/// Main application component that manages the UI state and rendering.
pub struct App {
    /// The title displayed at the top of the application.
    pub title: String,
    /// Flag indicating whether the application should quit.
    pub should_quit: bool,
    /// List of status indicators displayed in the main area.
    pub indicators: Vec<String>,
    /// List of key bindings displayed at the bottom (key, description).
    pub bindings: Vec<String>,
    /// Current size of the component
    size: Size,
    /// Current theme
    theme: Theme,
    id: ComponentId,
    children: Vec<Arc<Mutex<dyn Component>>>,
}

impl App {
    /// Creates a new application instance with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display at the top of the application
    pub fn new(id: ComponentId, theme: Theme) -> Self {
        Self {
            title: String::new(),
            should_quit: false,
            indicators: Vec::new(),
            bindings: Vec::new(),
            size: Size::new(0, 0),
            theme,
            id,
            children: Vec::new(),
        }
    }

    /// Checks if the application should quit based on the key event.
    ///
    /// # Arguments
    /// * `key` - The key event to check
    ///
    /// # Returns
    /// `true` if the application should quit, `false` otherwise
    fn should_quit(&self, key: KeyEvent) -> bool {
        matches!(key.code, KeyCode::Char('q'))
    }

    pub fn add_indicator(&mut self, indicator: String) {
        self.indicators.push(indicator);
    }

    pub fn add_binding(&mut self, binding: String) {
        self.bindings.push(binding);
    }

    pub fn clear_indicators(&mut self) {
        self.indicators.clear();
    }

    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
    }

    /// Set whether the app should quit
    pub fn set_should_quit(&mut self, should_quit: bool) {
        self.should_quit = should_quit;
    }

    pub fn add_child(&mut self, component: Box<dyn Component>) {
        self.children.push(Arc::new(Mutex::new(component)));
    }
}

impl Component for App {
    fn draw(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), ComponentError> {
        for child in &self.children {
            let child = child.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
            child.draw(frame, area)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &dyn Event) -> std::result::Result<(), ComponentError> {
        for child in &mut self.children {
            let mut child = child.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
            child.handle_event(event)?;
        }
        Ok(())
    }

    fn required_size(&self) -> Size {
        let mut max_width = 0;
        let mut max_height = 0;
        for child in &self.children {
            if let Ok(child) = child.lock() {
                let size = child.required_size();
                max_width = max_width.max(size.width);
                max_height = max_height.max(size.height);
            }
        }
        Size::new(max_width, max_height)
    }

    fn id(&self) -> ComponentId {
        self.id
    }
}

impl Themeable for App {
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
        for child in &mut self.children {
            let mut child = child.lock().map_err(|e| ComponentError::Lock(e.to_string()))?;
            child.apply_theme(theme)?;
        }
        Ok(())
    }

    fn get_style(&self) -> Style {
        self.theme.get_style(StyleRole::Default)
    }

    fn get_color(&self, role: ColorRole) -> ratatui::style::Color {
        self.theme.get_color(role)
    }
}

impl Resizable for App {
    fn resize(&mut self, size: Size) -> Result<(), UiError> {
        self.size = size;
        Ok(())
    }

    fn resizable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::ui::theme::Theme;

    #[test]
    fn test_should_quit() {
        let app = App::new(ComponentId::new("app"), Theme::default());
        let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        let other_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty());

        assert!(app.should_quit(quit_key));
        assert!(!app.should_quit(other_key));
    }

    #[test]
    fn test_theme_application() {
        let mut app = App::new(ComponentId::new("app"), Theme::default());
        let theme = Theme::default();

        assert!(app.theme().is_none());
        app.apply_theme(&theme).unwrap();
        assert!(app.theme().is_some());
        assert_eq!(app.theme().unwrap().name, "Default");
    }
} 