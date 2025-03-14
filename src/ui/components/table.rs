use std::fmt;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table as RatatuiTable, TableState},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::ui::components::{Component, Themeable, UiError};
use crate::ui::events::Event;
use crate::ui::theme::{Theme, ColorRole, StyleRole};
use crate::core::error::Result;
use crate::ui::layout::Size;
use std::sync::Arc;

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    state: TableState,
    theme: Theme,
}

impl Table {
    pub fn new(headers: Vec<String>, theme: Theme) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            state: TableState::default(),
            theme,
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn clear_rows(&mut self) {
        self.rows.clear();
        self.state.select(None);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.rows.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.rows.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected(&self) -> Option<&Vec<String>> {
        self.state.selected().map(|i| &self.rows[i])
    }
}

impl Component for Table {
    fn render(&self, frame: &mut Frame<'_>, area: Rect) -> std::result::Result<(), UiError> {
        let header_cells = self.headers.iter().map(|h| h.as_str());
        let header = Row::new(header_cells)
            .style(Style::default().fg(self.theme.get_color(ColorRole::Primary)));

        let rows = self.rows.iter().enumerate().map(|(i, row)| {
            let style = if Some(i) == self.state.selected() {
                Style::default().fg(self.theme.get_color(ColorRole::Accent))
            } else {
                Style::default().fg(self.theme.get_color(ColorRole::Secondary))
            };
            Row::new(row.iter().map(|cell| cell.as_str())).style(style)
        });

        let table = RatatuiTable::new(rows)
            .header(header)
            .block(Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(self.theme.get_color(ColorRole::Border))))
            .widths(&vec![]);

        frame.render_stateful_widget(table, area, &mut self.state.clone());
        Ok(())
    }

    fn handle_event(&mut self, event: &Box<dyn Event>) -> std::result::Result<(), UiError> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key(&mut self, key: KeyEvent) -> std::result::Result<(), UiError> {
        match key.code {
            KeyCode::Up => self.previous(),
            KeyCode::Down => self.next(),
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self) -> std::result::Result<(), UiError> {
        Ok(())
    }

    fn required_size(&self) -> Size {
        Size::new(0, self.rows.len() as u16 + 3) // Header + borders + rows
    }
}

impl Themeable for Table {
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