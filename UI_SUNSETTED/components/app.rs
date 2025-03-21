use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    style::{Style, Modifier, Color},
    text::{Line, Span},
    Frame,
};

/// Main application component that manages the UI state and rendering.
pub struct App {
    /// The title displayed at the top of the application.
    pub title: String,
    /// Flag indicating whether the application should quit.
    pub should_quit: bool,
    /// List of status indicators displayed in the main area.
    pub indicators: Vec<String>,
    /// List of key bindings displayed at the bottom (key, description).
    pub bindings: Vec<(String, String)>,
}

impl App {
    /// Creates a new application instance with the given title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display at the top of the application
    pub fn new(title: String) -> App {
        App {
            title,
            should_quit: false,
            indicators: Vec::new(),
            bindings: vec![
                ("q".to_string(), "Quit".to_string()),
            ],
        }
    }

    /// Handles key events and updates application state.
    ///
    /// # Arguments
    ///
    /// * `key` - The key event to process
    pub fn on_key(&mut self, key: KeyEvent) {
        if self.should_quit(key) {
            self.should_quit = true;
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
        matches!(key.code, crossterm::event::KeyCode::Char('q'))
    }

    /// Renders the application UI to the given frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to
    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(frame.size());

        // Render title
        let title = Paragraph::new(Span::styled(
            &self.title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        // Render indicators
        let indicators: Vec<Line> = self.indicators
            .iter()
            .map(|s| Line::from(vec![Span::raw(s)]))
            .collect();
        let indicators = Paragraph::new(indicators)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(indicators, chunks[1]);

        // Render bindings
        let bindings: Vec<ListItem> = self.bindings
            .iter()
            .map(|(key, desc)| {
                ListItem::new(Line::from(vec![
                    Span::styled(key.clone(), Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" - "),
                    Span::raw(desc.clone()),
                ]))
            })
            .collect();
        let bindings = List::new(bindings)
            .block(Block::default().borders(Borders::ALL).title("Bindings"));
        frame.render_widget(bindings, chunks[2]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_should_quit() {
        let app = App::new("Test App".to_string());
        assert!(app.should_quit(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())));
        assert!(!app.should_quit(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())));
    }
} 