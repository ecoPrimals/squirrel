use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell},
    Frame,
};
use crate::app::App;
 // Import the enum
use dashboard_core::service::DashboardService;
use dashboard_core::data::ProtocolData;
use std::collections::HashMap;

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
    let status_color = if state.connection_status.contains("Connected") {
        Color::Green
    } else if state.connection_status.contains("Degraded") || state.connection_status.contains("Warning") {
        Color::Yellow
    } else if state.connection_status.contains("Failed") || state.connection_status.contains("Error") {
        Color::Red
    } else {
        Color::Gray
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

/// Render protocol information
pub fn render(f: &mut Frame, area: Rect, protocol: Option<&ProtocolData>) {
    let block = Block::default()
        .title("Protocol Status")
        .borders(Borders::ALL);
    
    // If no protocol data, display empty message
    if protocol.is_none() {
        let empty_widget = Paragraph::new("No protocol data available")
            .block(block);
        f.render_widget(empty_widget, area);
        return;
    }
    
    let protocol = protocol.unwrap();
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    // Create layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Status
            Constraint::Length(2),  // Version
            Constraint::Min(3),     // Details
        ])
        .split(inner_area);
    
    // Render status
    let status = protocol.status.clone();
    let status_style = match status.as_str() {
        "healthy" | "Healthy" => Style::default().fg(Color::Green),
        "warning" | "Warning" => Style::default().fg(Color::Yellow),
        "critical" | "Critical" => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::Gray),
    };
    
    let status_text = Paragraph::new(format!("Status: {}", status))
        .style(status_style);
    f.render_widget(status_text, chunks[0]);
    
    // Render version
    let version_text = Paragraph::new(format!("Version: {}", protocol.version));
    f.render_widget(version_text, chunks[1]);
    
    // Gather protocol details in a HashMap for display
    let mut details: HashMap<String, String> = HashMap::new();
    
    details.insert("Name".to_string(), protocol.name.clone());
    details.insert("Type".to_string(), protocol.protocol_type.clone());
    details.insert("Status".to_string(), protocol.status.clone());
    details.insert("Version".to_string(), protocol.version.clone());
    details.insert("Connected".to_string(), if protocol.connected { "Yes".to_string() } else { "No".to_string() });
    
    if let Some(last_connected) = &protocol.last_connected {
        details.insert("Last Connected".to_string(), last_connected.format("%Y-%m-%d %H:%M:%S").to_string());
    }
    
    // Additional details - metadata could be stored differently in your actual implementation
    // This is just a placeholder for any additional fields you might want to display
    
    // Create detail rows
    let rows = details.iter().map(|(key, value)| {
        Row::new(vec![
            Cell::from(key.clone()),
            Cell::from(value.clone()),
        ])
    });
    
    // Create and render the detail table
    let header = Row::new(vec!["Property", "Value"])
        .style(Style::default().fg(Color::Yellow));
    
    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ];
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Details"));
    
    f.render_widget(table, chunks[2]);
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

    // Helper function to create sample protocol data for tests
    fn create_sample_protocol_data() -> ProtocolData {
        ProtocolData {
            name: "TEST_PROTOCOL".to_string(),
            protocol_type: "TEST".to_string(),
            version: "1.0".to_string(),
            status: "Connected".to_string(),
            connected: true,
            last_connected: Some(Utc::now()),
            retry_count: 0,
            error: None,
            metrics: {
                let mut m = std::collections::HashMap::new();
                m.insert("latency".to_string(), 3.0);
                m
            },
        }
    }

    #[test]
    fn test_render_protocol_widget_no_data() {
        let backend = TestBackend::new(50, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        app.state.protocol_data = None; // Ensure no protocol data
        app.state.connection_status = ConnectionStatus::Unknown; // Set to Unknown status
        let area = Rect::new(0, 0, 50, 5);

        terminal.draw(|f| {
            render_protocol_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();

        // Verify that key content is rendered correctly
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for expected content
        assert!(rendered_content.contains("Protocol Status"));
        assert!(rendered_content.contains("Overall Status: Unknown"));
        assert!(rendered_content.contains("Protocol details unavailable"));
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

        // Verify that key content is rendered correctly
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for expected content
        assert!(rendered_content.contains("Protocol Status"));
        assert!(rendered_content.contains("Overall Status: Warning"));
        assert!(rendered_content.contains("ExampleMCP")); // Protocol name
        assert!(rendered_content.contains("CustomTCP")); // Protocol type
        assert!(rendered_content.contains("1.2.3")); // Version
        assert!(rendered_content.contains("ActivePolling")); // Status
        assert!(rendered_content.contains("Yes")); // Connected
        assert!(rendered_content.contains("2024-01-01")); // Last connected date
        assert!(rendered_content.contains("Error:")); // Error section
        assert!(rendered_content.contains("Protocol Metrics:")); // Metrics section
        assert!(rendered_content.contains("latency_ms: 15.50")); // Specific metric
    }

    #[test]
    fn test_draw_protocol_data() {
        // Setup
        let backend = TestBackend::new(50, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app();
        
        // Add sample data to app
        app.state.protocol_data = Some(create_sample_protocol_data());
        let area = Rect::new(0, 0, 50, 10);
        
        terminal.draw(|f| {
            render_protocol_widget::<TestBackend, _>(f, &app, area);
        }).unwrap();
        
        // Verify core functionality
        let buffer = terminal.backend().buffer();
        let rendered_content = buffer.content.iter().map(|cell| cell.symbol()).collect::<String>();
        
        // Check for key content
        assert!(rendered_content.contains("Protocol Status"));
        assert!(rendered_content.contains("TEST_PROTOCOL")); // Protocol name
        assert!(rendered_content.contains("TEST")); // Protocol type
        assert!(rendered_content.contains("1.0")); // Version
        assert!(rendered_content.contains("Connected")); // Status
        assert!(rendered_content.contains("Yes")); // Connected status
    }

    #[test]
    fn test_render_empty_protocol_widget() {
        // This test would check rendering when no protocol data is provided
    }
} 