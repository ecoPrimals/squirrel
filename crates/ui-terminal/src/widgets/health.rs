// crates/ui-terminal/src/widgets/health.rs
// Placeholder for HealthWidget implementation

use ratatui::{
    prelude::{Backend, Rect, Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use ratatui::text::{Line, Span}; // Add missing imports

/// Represents the health status of a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

impl HealthStatus {
    /// Gets the display color for the status.
    pub fn color(&self) -> Color {
        match self {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Warning => Color::Yellow,
            HealthStatus::Critical => Color::Red,
            HealthStatus::Unknown => Color::Gray,
        }
    }
}

/// Represents a single health check item.
#[derive(Debug, Clone)]
pub struct HealthCheck {
    name: String,
    status: HealthStatus,
    message: String,
    // Optional percentage for gauge display
    percentage: Option<f64>,
}

impl HealthCheck {
    pub fn new(name: impl Into<String>, status: HealthStatus) -> Self {
        Self {
            name: name.into(),
            status,
            message: String::new(),
            percentage: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_percentage(mut self, percentage: f64) -> Self {
        self.percentage = Some(percentage.clamp(0.0, 100.0));
        self
    }
}

pub fn render<B: Backend>(
    frame: &mut Frame<'_>,
    area: Rect,
    health_checks: &[HealthCheck]
) {
    let block = Block::default().borders(Borders::ALL).title("System Health");

    if health_checks.is_empty() {
        let placeholder = Paragraph::new("No health checks available.").block(block);
        frame.render_widget(placeholder, area);
        return;
    }

    let items: Vec<ListItem> = health_checks
        .iter()
        .map(|check| {
            let status_style = Style::default().fg(check.status.color());
            let status_indicator = Span::styled("● ", status_style); // Simple status indicator
            let name = Span::styled(format!("{:<10}", check.name), Style::default().fg(Color::White));
            let message = Span::raw(check.message.clone());

            // Optional gauge for percentage
            let content = if let Some(percent) = check.percentage {
                // Use layout within the list item for gauge + message
                // This requires more complex rendering, maybe simplify for now
                // For simplicity, just show text
                 Line::from(vec![status_indicator, name, Span::raw(" - "), message, Span::raw(format!(" ({:.0}%) ", percent))])
            } else {
                 Line::from(vec![status_indicator, name, Span::raw(" - "), message])
            };

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
} 