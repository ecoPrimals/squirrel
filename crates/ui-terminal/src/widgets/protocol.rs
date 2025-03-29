use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use dashboard_core::health::HealthStatus as CoreHealthStatus; // Import the enum

/// Renders the Protocol tab widget.
///
/// Displays the overall connection status and detailed protocol information
/// fetched from the application state's `protocol_data` field, including name,
/// type, version, connection status, last connection time, retries, errors, and protocol-specific metrics.
pub fn render_protocol_widget<B: Backend>(frame: &mut Frame, app: &App, area: Rect) {
    let state = &app.state;
    let mut text = Vec::<Line>::new();

    // Determine overall status color and text first (lives longer)
    let status_color = match state.connection_status {
        CoreHealthStatus::Ok => Color::Green,
        CoreHealthStatus::Warning => Color::Yellow,
        CoreHealthStatus::Critical => Color::Red,
        CoreHealthStatus::Unknown => Color::Gray,
    };
    let status_text = format!("{:?}", state.connection_status);

    // Display Protocol Details if available
    if let Some(protocol_data) = &state.protocol_data {
        // Use overall connection status from app state for the main indicator
        text.push(Line::from(vec![
            Span::styled("Overall Status: ", Style::default().bold()),
            Span::styled(&status_text, Style::default().fg(status_color).bold()),
        ]));
        text.push(Line::from("")); // Spacer

        text.push(Line::from(vec![
            Span::styled("Protocol Name: ", Style::default().bold()),
            Span::raw(&protocol_data.name),
        ]));
        text.push(Line::from(vec![
            Span::styled("Type:          ", Style::default().bold()),
            Span::raw(&protocol_data.protocol_type),
        ]));
        text.push(Line::from(vec![
            Span::styled("Version:       ", Style::default().bold()),
            Span::raw(&protocol_data.version),
        ]));
        text.push(Line::from(vec![
            Span::styled("Status:        ", Style::default().bold()),
            Span::raw(&protocol_data.status), // Specific status from protocol data
        ]));
        text.push(Line::from(vec![
            Span::styled("Connected:     ", Style::default().bold()),
            Span::raw(if protocol_data.connected { "Yes" } else { "No" }),
        ]));
        text.push(Line::from(vec![
            Span::styled("Last Conn.:    ", Style::default().bold()),
            Span::raw(protocol_data.last_connected.map_or_else(|| "N/A".to_string(), |ts| ts.format("%Y-%m-%d %H:%M:%S %Z").to_string())),
        ]));
        text.push(Line::from(vec![
            Span::styled("Retries:       ", Style::default().bold()),
            Span::raw(protocol_data.retry_count.to_string()),
        ]));

        if let Some(error_msg) = &protocol_data.error {
            text.push(Line::from("")); // Spacer
            text.push(Line::from(vec![
                Span::styled("Error:         ", Style::default().bold().fg(Color::Red)),
                Span::styled(error_msg, Style::default().fg(Color::Red)),
            ]));
        }

        if !protocol_data.metrics.is_empty() {
            text.push(Line::from("")); // Spacer
            text.push(Line::from(Span::styled("Protocol Metrics:", Style::default().bold())));
            // Sort metrics by key for consistent order
            let mut sorted_metrics: Vec<_> = protocol_data.metrics.iter().collect();
            sorted_metrics.sort_by_key(|(k, _)| *k);
            for (key, value) in sorted_metrics {
                text.push(Line::from(format!("  {}: {:.2}", key, value)));
            }
        }

    } else {
        // Fallback if no protocol_data is available
        // Still show overall status
        text.push(Line::from(vec![
            Span::styled("Overall Status: ", Style::default().bold()),
            Span::styled(&status_text, Style::default().fg(status_color).bold()), // Use status_text here too
        ]));
        text.push(Line::from("")); // Spacer
        text.push(Line::from("Protocol details unavailable."));
        text.push(Line::from("(Waiting for data from provider...)"));
    }

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Protocol Status"));

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppState, ConnectionStatus};
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style, Stylize},
        Terminal,
    };
    use std::collections::VecDeque;

    // Helper to create a default App
    fn create_test_app() -> App {
        App::default()
    }

    // Helper function to create basic ProtocolData
    fn create_basic_protocol_data() -> ProtocolData {
        ProtocolData {
            messages_sent: 10,
            messages_received: 5,
            last_message_time: Some(Utc::now()), // Use current time for simplicity
            error_count: 0,
            // Add other fields as needed based on ProtocolData definition
        }
    }

    #[test]
    fn test_render_protocol_widget_no_data() {
        let backend = TestBackend::new(50, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.protocol = None; // Ensure no protocol data
        app.state.connection_status = ConnectionStatus::Disconnected; // Assume disconnected if no data
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_protocol_widget::<TestBackend>(f, &app, area);
        }).unwrap();

        let expected = Buffer::with_lines(vec![
            "┌Protocol Monitor─────────────────────────────────┐",
            "│Status: Disconnected          (No Protocol Data)│", // Expect Disconnected status and indication of no data
            "│                                                 │",
            "│                                                 │",
            "└─────────────────────────────────────────────────┘",
        ]);
        // Add style check for the status line if needed, e.g., Red for Disconnected
        expected.set_style(Rect::new(1, 1, 48, 1), Style::default().fg(Color::Red)); 

        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_render_protocol_widget_basic_data() {
        let backend = TestBackend::new(50, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.protocol = Some(create_basic_protocol_data());
        app.state.connection_status = ConnectionStatus::Connected; // Explicitly set Connected
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_protocol_widget::<TestBackend>(f, &app, area);
        }).unwrap();

        // Expected buffer will depend on how the widget formats the data
        // Assuming a format like: "Status: Connected | Sent: 10 | Rcvd: 5"
        let expected = Buffer::with_lines(vec![
            "┌Protocol Monitor─────────────────────────────────┐",
            "│Status: Connected  | Sent: 10 | Rcvd: 5         │", // Example format
            "│                                                 │",
            "│                                                 │",
            "└─────────────────────────────────────────────────┘",
        ]);
        // Style for Connected status (e.g., Green)
        expected.set_style(Rect::new(1, 1, 48, 1), Style::default().fg(Color::Green));

        terminal.backend().assert_buffer(&expected);
    }
} 