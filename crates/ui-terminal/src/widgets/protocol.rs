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
use dashboard_core::service::DashboardService;

/// Renders the Protocol tab widget.
///
/// Displays the overall connection status and detailed protocol information
/// fetched from the application state's `protocol_data` field, including name,
/// type, version, connection status, last connection time, retries, errors, and protocol-specific metrics.
pub fn render_protocol_widget<B: Backend, S: DashboardService + Send + Sync + 'static + ?Sized>(
    frame: &mut Frame<'_>,
    app: &App<S>,
    area: Rect,
) {
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
    use crate::app::{App, AppState};
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style, Stylize},
        Terminal,
    };
    use std::collections::VecDeque;
    use chrono::Utc;
    use dashboard_core::service::MockDashboardService;
    use std::sync::Arc;

    // Helper to create a default App
    fn create_test_app() -> App<MockDashboardService> {
        App::new(Arc::new(MockDashboardService::new()))
    }

    // Helper function to create basic ProtocolData
    fn create_basic_protocol_data() -> ProtocolData {
        ProtocolData {
            messages_sent: 10,
            messages_received: 5,
            last_message_time: Some(Utc::now()), // Use current time for simplicity
            error_count: 0,
            // Add other fields as needed based on ProtocolData definition
            name: "MockProto".to_string(),
            protocol_type: "Mock".to_string(),
            version: "0.1".to_string(),
            status: "Connected".to_string(),
            connected: true,
            last_connected: Some(Utc::now()),
            retry_count: 0,
            error: None,
            metrics: Default::default(),
        }
    }

    // Helper function to create more detailed ProtocolData
    fn create_detailed_protocol_data() -> ProtocolData {
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("latency_ms".to_string(), 15.5);
        metrics.insert("throughput_kbps".to_string(), 1024.7);

        ProtocolData {
            name: "ExampleMCP".to_string(),
            protocol_type: "CustomTCP".to_string(),
            version: "1.2.3".to_string(),
            status: "ActivePolling".to_string(),
            connected: true,
            last_connected: Some(chrono::DateTime::parse_from_rfc3339("2024-01-01T10:00:00+00:00").unwrap().with_timezone(&chrono::Utc)),
            retry_count: 2,
            error: Some("Timeout during last poll".to_string()),
            metrics,
            // Populate other fields if they exist in ProtocolData struct
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
            render_protocol_widget::<TestBackend, _>(f, &app, area);
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
        // Increased size to accommodate more details
        let backend = TestBackend::new(60, 16);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.protocol_data = Some(create_detailed_protocol_data());
        // Set overall status (can differ from protocol_data.status)
        app.state.connection_status = ConnectionStatus::Warning; 
        let area = Rect::new(0, 0, 60, 16);

        terminal.draw(|f| {
            render_protocol_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Updated expected buffer for detailed view
        let mut expected = Buffer::with_lines(vec![
            "┌Protocol Status────────────────────────────────────────────┐",
            "│Overall Status: Warning                                   │", // Uses app.state.connection_status
            "│                                                          │",
            "│Protocol Name: ExampleMCP                                 │",
            "│Type:          CustomTCP                                 │",
            "│Version:       1.2.3                                     │",
            "│Status:        ActivePolling                             │", // Uses protocol_data.status
            "│Connected:     Yes                                       │",
            "│Last Conn.:    2024-01-01 10:00:00 UTC                   │",
            "│Retries:       2                                         │",
            "│                                                          │",
            "│Error:         Timeout during last poll                  │", // Error message
            "│                                                          │",
            "│Protocol Metrics:                                         │",
            "│  latency_ms: 15.50                                       │", // Sorted metrics
            "│  throughput_kbps: 1024.70                                │", // Sorted metrics
            "└───────────────────────────────────────────────────────────┘",
        ]);
        // Styles
        expected.set_style(Rect::new(1, 1, 25, 1), Style::default().fg(Color::Yellow).bold()); // Overall Status: Warning
        expected.set_style(Rect::new(1, 11, 58, 1), Style::default().fg(Color::Red)); // Error line
        expected.set_style(Rect::new(1, 11, 15, 1), Style::default().fg(Color::Red).bold()); // Error label


        terminal.backend().assert_buffer(&expected);
    }
} 