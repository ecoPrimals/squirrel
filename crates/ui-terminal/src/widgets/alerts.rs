// crates/ui-terminal/src/widgets/alerts.rs

use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::app::App;
use dashboard_core::data::AlertSeverity;

// Function to determine the color based on AlertSeverity
// ... (determine_severity_color remains internal, no doc needed unless exported)

/// Renders the Alerts tab widget.
///
/// Displays a scrollable list of alerts fetched from the application state.
/// Alerts are colored based on their severity.
pub fn render_alerts_widget<B: Backend>(frame: &mut Frame, app: &App, area: Rect) {
    let alerts = &app.state.alerts;

    let list_items: Vec<ListItem> = alerts
        .iter()
        .rev() // Show newest alerts first
        .map(|alert| {
            let severity_style = match alert.severity {
                AlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                AlertSeverity::Warning | AlertSeverity::Error => Style::default().fg(Color::Yellow),
                AlertSeverity::Info => Style::default().fg(Color::Cyan),
            };

            let timestamp = alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let source: &str = alert.source.as_str();

            let content = Line::from(vec![
                Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:?} ", alert.severity), severity_style),
                Span::styled(format!("({}) ", source), Style::default().fg(Color::Blue)),
                Span::raw(alert.message.clone()),
            ]);
            ListItem::new(content)
        })
        .collect();

    let list_widget = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title("Active Alerts"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)) // Style for potential selection
        .highlight_symbol("> "); // Symbol for potential selection

    frame.render_widget(list_widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppState};
    use dashboard_core::alerts::{Alert, AlertSeverity, AlertState};
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style, Stylize},
        Terminal,
    };
    use std::collections::VecDeque;
    use chrono::Utc;

    // Helper to create a default App
    fn create_test_app() -> App {
        App::default()
    }

    // Helper to create sample alerts
    fn create_sample_alerts() -> VecDeque<Alert> {
        let mut alerts = VecDeque::new();
        alerts.push_back(Alert {
            id: "alert1".to_string(),
            message: "High CPU usage detected".to_string(),
            severity: AlertSeverity::Critical,
            state: AlertState::Active,
            timestamp: Utc::now(),
            source: "system_monitor".to_string(),
            details: Default::default(), // Empty details for simplicity
        });
        alerts.push_back(Alert {
            id: "alert2".to_string(),
            message: "Low disk space warning".to_string(),
            severity: AlertSeverity::Warning,
            state: AlertState::Active,
            timestamp: Utc::now(),
            source: "disk_monitor".to_string(),
            details: Default::default(),
        });
         alerts.push_back(Alert {
            id: "alert3".to_string(),
            message: "Informational message".to_string(),
            severity: AlertSeverity::Info,
            state: AlertState::Active,
            timestamp: Utc::now(),
            source: "general".to_string(),
            details: Default::default(),
        });
        alerts
    }

    #[test]
    fn test_render_alerts_widget_no_alerts() {
        let backend = TestBackend::new(50, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        // Ensure alerts are empty
        app.state.alerts = VecDeque::new();
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_alerts_widget::<TestBackend>(f, &app, area);
        }).unwrap();

        let expected = Buffer::with_lines(vec![
            "┌Alerts───────────────────────────────────────────┐",
            "│No active alerts.                                │",
            "│                                                 │",
            "│                                                 │",
            "└─────────────────────────────────────────────────┘",
        ]);

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_render_alerts_widget_with_alerts() {
        let backend = TestBackend::new(60, 7); // Increased size for multiple alerts
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.alerts = create_sample_alerts();
        let area = Rect::new(0, 0, 60, 7);

        terminal.draw(|f| {
            render_alerts_widget::<TestBackend>(f, &app, area);
        }).unwrap();

        let mut expected = Buffer::with_lines(vec![
            "┌Alerts─────────────────────────────────────────────────────┐",
            "│[CRITICAL] High CPU usage detected                       │", // Alert 1
            "│[ WARNING] Low disk space warning                        │", // Alert 2
            "│[  INFO  ] Informational message                        │", // Alert 3
            "│                                                         │",
            "│                                                         │",
            "└─────────────────────────────────────────────────────┘",
        ]);

        // Set styles based on severity
        expected.set_style(Rect::new(1, 1, 58, 1), Style::default().fg(Color::Red).bold());    // Critical
        expected.set_style(Rect::new(1, 2, 58, 1), Style::default().fg(Color::Yellow)); // Warning
        expected.set_style(Rect::new(1, 3, 58, 1), Style::default().fg(Color::Blue));   // Info

        terminal.backend().assert_buffer(&expected);
    }
} 