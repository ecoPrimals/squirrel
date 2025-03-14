use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};

use crate::ui::{
    components::{Component, Themeable, UiError},
    layout::Size,
    theme::{Theme, ColorRole},
};

pub struct Progress {
    value: u16,
    max: u16,
    label: String,
    theme: Theme,
}

impl Progress {
    pub fn new(max: u16, theme: Theme) -> Self {
        Self {
            value: 0,
            max,
            label: String::new(),
            theme,
        }
    }

    pub fn set_value(&mut self, value: u16) {
        self.value = value.min(self.max);
    }

    pub fn increment(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn set_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn progress(&self) -> f64 {
        self.value as f64 / self.max as f64
    }
}

impl Component for Progress {
    fn render(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), UiError> {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(self.theme.get_color(ColorRole::Primary)));

        let gauge = Gauge::default()
            .block(block)
            .gauge_style(Style::default().fg(self.theme.get_color(ColorRole::Secondary)))
            .label(self.label.as_str())
            .ratio(self.progress());

        frame.render_widget(gauge, area);
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn on_key(&mut self, _key: KeyEvent) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn update(&mut self) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn required_size(&self) -> Size {
        Size::new(0, 3) // Minimum height for progress bar with borders
    }
}

impl Themeable for Progress {
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