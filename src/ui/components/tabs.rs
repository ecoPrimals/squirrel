use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Tabs as RatatuiTabs},
    Frame,
};

use crate::ui::{
    components::{Component, Themeable, UiError},
    layout::Size,
    theme::{Theme, ColorRole},
};

pub struct Tabs {
    titles: Vec<String>,
    active: usize,
    theme: Theme,
}

impl Tabs {
    pub fn new(titles: Vec<String>, theme: Theme) -> Self {
        Self {
            titles,
            active: 0,
            theme,
        }
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn next(&mut self) {
        self.active = (self.active + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.active > 0 {
            self.active -= 1;
        } else {
            self.active = self.titles.len() - 1;
        }
    }
}

impl Component for Tabs {
    fn render(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), UiError> {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(self.theme.get_color(ColorRole::Primary)));

        let tabs = RatatuiTabs::new(self.titles.iter().map(|t| t.as_str()).collect())
            .block(block)
            .select(self.active)
            .style(Style::default().fg(self.theme.get_color(ColorRole::Secondary)))
            .highlight_style(Style::default().fg(self.theme.get_color(ColorRole::Accent)));

        frame.render_widget(tabs, area);
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn on_key(&mut self, key: KeyEvent) -> std::result::Result<(), UiError> {
        match key.code {
            KeyCode::Right | KeyCode::Tab => self.next(),
            KeyCode::Left => self.previous(),
            _ => (),
        }
        Ok(())
    }

    fn update(&mut self) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn required_size(&self) -> Size {
        Size::new(0, 3) // Minimum height for tabs with borders
    }
}

impl Themeable for Tabs {
    fn theme(&self) -> Option<&Theme> {
        Some(&self.theme)
    }

    fn theme_mut(&mut self) -> &mut Theme {
        &mut self.theme
    }

    fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    fn apply_theme(&mut self, theme: &Theme) -> std::result::Result<(), UiError> {
        self.theme = theme.clone();
        Ok(())
    }
} 