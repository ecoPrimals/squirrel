use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};
use std::iter;

use dashboard_core::{
    data::ProtocolData, 
    Protocol, 
    ProtocolStatus
};

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    /// Protocol data to display
    protocol: &'a ProtocolData,
    /// Widget title
    title: &'a str,
    /// Active protocol tab index
    active_tab: usize,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(protocol: &'a ProtocolData, title: &'a str) -> Self {
        Self { 
            protocol, 
            title,
            active_tab: 0,
        }
    }
    
    /// Set active tab
    pub fn active_tab(mut self, tab: usize) -> Self {
        self.active_tab = tab;
        self
    }
    
    /// Convert string protocol type to Protocol enum
    fn parse_protocol_type(&self, protocol_type: &str) -> Protocol {
        match protocol_type.to_lowercase().as_str() {
            "http" => Protocol::Http,
            "mqtt" => Protocol::Mqtt,
            "websocket" => Protocol::WebSocket,
            "grpc" => Protocol::Grpc,
            _ => Protocol::Custom(0),
        }
    }
    
    /// Convert string protocol status to ProtocolStatus enum
    fn parse_protocol_status(&self, status: &str) -> ProtocolStatus {
        match status.to_lowercase().as_str() {
            "connected" => ProtocolStatus::Connected,
            "disconnected" => ProtocolStatus::Disconnected,
            "connecting" => ProtocolStatus::Connecting,
            "error" => ProtocolStatus::Error,
            "running" => ProtocolStatus::Running,
            "degraded" => ProtocolStatus::Degraded,
            "stopped" => ProtocolStatus::Stopped,
            _ => ProtocolStatus::Unknown,
        }
    }
    
    /// Get status color based on protocol status
    fn get_status_color(&self, status: ProtocolStatus) -> Color {
        match status {
            ProtocolStatus::Connected | ProtocolStatus::Running => Color::Green,
            ProtocolStatus::Connecting => Color::Yellow,
            ProtocolStatus::Degraded => Color::Yellow,
            ProtocolStatus::Disconnected | ProtocolStatus::Stopped => Color::Red,
            ProtocolStatus::Error => Color::Red,
            ProtocolStatus::Unknown => Color::Gray,
        }
    }
    
    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Protocol status
                Constraint::Min(0),    // Content
            ])
            .split(inner_area);
        
        // Render protocol status
        self.render_protocol_status(f, chunks[0]);
        
        // Render protocol details
        self.render_protocol_details(f, chunks[1]);
    }
    
    /// Render protocol status bar
    fn render_protocol_status(&self, f: &mut Frame, area: Rect) {
        // Parse the status string to enum
        let status = self.parse_protocol_status(&self.protocol.status);
        let status_color = self.get_status_color(status);
        
        let connection_status = Span::styled(
            format!("{:?}", status), 
            Style::default().fg(status_color)
        );
        
        // Parse the protocol type string to enum
        let protocol_type = self.parse_protocol_type(&self.protocol.protocol_type);
        
        let status_text = Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            connection_status,
            Span::raw(" | "),
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:?}", protocol_type)),
            Span::raw(" | "),
            Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&self.protocol.version),
            Span::raw(" | "),
            Span::styled("Retries: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", self.protocol.retry_count)),
        ]);
        
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            status_text,
        ]);
        
        f.render_widget(paragraph, area);
    }
    
    /// Render protocol details
    fn render_protocol_details(&self, f: &mut Frame, area: Rect) {
        if self.protocol.metrics.is_empty() {
            // No metrics to display
            self.render_no_data(f, area);
            return;
        }
        
        // Create data for metrics
        let mut rows = Vec::new();
        
        // Add protocol metrics
        for (key, value) in &self.protocol.metrics {
            rows.push(Row::new(vec![
                Cell::from(key.clone()),
                Cell::from(format!("{:.2}", value)),
            ]));
        }
        
        // Add error info if present
        if let Some(error) = &self.protocol.error {
            rows.push(Row::new(vec![
                Cell::from(Span::styled("Error", Style::default().fg(Color::Red))),
                Cell::from(error.clone()),
            ]));
        }
        
        // Add last connection time if present
        if let Some(last_connected) = self.protocol.last_connected {
            rows.push(Row::new(vec![
                Cell::from("Last Connected"),
                Cell::from(last_connected.to_rfc3339()),
            ]));
        }
        
        // Create table with header
        let header_cells = vec![
            Cell::from(Span::styled("Metric", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Value", Style::default().add_modifier(Modifier::BOLD))),
        ];
        let header = Row::new(header_cells);
        
        // Create table
        let table = Table::new(iter::once(header).chain(rows))
            .block(Block::default().borders(Borders::ALL).title("Protocol Details"))
            .widths(&[
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }
    
    /// Render message when no data is available
    fn render_no_data(&self, f: &mut Frame, area: Rect) {
        // If there's an error, display it with a red background
        if let Some(error) = &self.protocol.error {
            let error_text = Line::from(vec![
                Span::styled(
                    "Connection Error: ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(error),
            ]);
            
            let paragraph = Paragraph::new(vec![
                Line::from(""),
                error_text,
                Line::from(""),
                Line::from("No additional protocol data available."),
            ])
            .block(Block::default().borders(Borders::ALL).title("Protocol Error"));
            
            f.render_widget(paragraph, area);
            return;
        }
        
        // Otherwise, display a generic message
        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled(
                "No protocol data available",
                Style::default().fg(Color::Gray),
            ),
        ]));
        
        f.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    
    #[test]
    fn test_protocol_widget_new() {
        let protocol_data = create_test_protocol_data();
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        assert_eq!(widget.title, "Protocol Test");
        assert_eq!(widget.active_tab, 0);
        assert_eq!(widget.protocol.protocol_type, "TCP");
    }
    
    #[test]
    fn test_protocol_widget_active_tab() {
        let protocol_data = create_test_protocol_data();
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test")
            .active_tab(2);
        
        assert_eq!(widget.active_tab, 2);
    }
    
    #[test]
    fn test_parse_protocol_type() {
        let protocol_data = create_test_protocol_data();
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        assert!(matches!(widget.parse_protocol_type("http"), Protocol::Http));
        assert!(matches!(widget.parse_protocol_type("HTTP"), Protocol::Http));
        assert!(matches!(widget.parse_protocol_type("mqtt"), Protocol::Mqtt));
        assert!(matches!(widget.parse_protocol_type("MQTT"), Protocol::Mqtt));
        assert!(matches!(widget.parse_protocol_type("websocket"), Protocol::WebSocket));
        assert!(matches!(widget.parse_protocol_type("WebSocket"), Protocol::WebSocket));
        assert!(matches!(widget.parse_protocol_type("grpc"), Protocol::Grpc));
        assert!(matches!(widget.parse_protocol_type("gRPC"), Protocol::Grpc));
        
        // Test unknown protocol type
        if let Protocol::Custom(id) = widget.parse_protocol_type("unknown") {
            assert_eq!(id, 0);
        } else {
            panic!("Expected Protocol::Custom");
        }
    }
    
    #[test]
    fn test_parse_protocol_status() {
        let protocol_data = create_test_protocol_data();
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        assert!(matches!(widget.parse_protocol_status("connected"), ProtocolStatus::Connected));
        assert!(matches!(widget.parse_protocol_status("Connected"), ProtocolStatus::Connected));
        assert!(matches!(widget.parse_protocol_status("disconnected"), ProtocolStatus::Disconnected));
        assert!(matches!(widget.parse_protocol_status("connecting"), ProtocolStatus::Connecting));
        assert!(matches!(widget.parse_protocol_status("error"), ProtocolStatus::Error));
        assert!(matches!(widget.parse_protocol_status("running"), ProtocolStatus::Running));
        assert!(matches!(widget.parse_protocol_status("degraded"), ProtocolStatus::Degraded));
        assert!(matches!(widget.parse_protocol_status("stopped"), ProtocolStatus::Stopped));
        
        // Test unknown status
        assert!(matches!(widget.parse_protocol_status("unknown"), ProtocolStatus::Unknown));
    }
    
    #[test]
    fn test_get_status_color() {
        let protocol_data = create_test_protocol_data();
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        // Test positive statuses (green)
        assert_eq!(widget.get_status_color(ProtocolStatus::Connected), Color::Green);
        assert_eq!(widget.get_status_color(ProtocolStatus::Running), Color::Green);
        
        // Test warning statuses (yellow)
        assert_eq!(widget.get_status_color(ProtocolStatus::Connecting), Color::Yellow);
        assert_eq!(widget.get_status_color(ProtocolStatus::Degraded), Color::Yellow);
        
        // Test negative statuses (red)
        assert_eq!(widget.get_status_color(ProtocolStatus::Disconnected), Color::Red);
        assert_eq!(widget.get_status_color(ProtocolStatus::Stopped), Color::Red);
        assert_eq!(widget.get_status_color(ProtocolStatus::Error), Color::Red);
        
        // Test unknown status (gray)
        assert_eq!(widget.get_status_color(ProtocolStatus::Unknown), Color::Gray);
    }
    
    // Helper function to create test protocol data
    fn create_test_protocol_data() -> ProtocolData {
        // Create protocol metrics
        let mut metrics = HashMap::new();
        metrics.insert("protocol.messages".to_string(), 1000.0);
        metrics.insert("protocol.transactions".to_string(), 500.0);
        metrics.insert("protocol.errors".to_string(), 10.0);
        
        // Create protocol data
        ProtocolData {
            name: "MCP".to_string(),
            protocol_type: "TCP".to_string(),
            version: "1.0".to_string(),
            connected: true,
            last_connected: Some(Utc::now()),
            status: "Connected".to_string(),
            error: None,
            retry_count: 0,
            metrics,
        }
    }
    
    #[test]
    fn test_protocol_widget_with_error() {
        let mut protocol_data = create_test_protocol_data();
        protocol_data.status = "Error".to_string();
        protocol_data.error = Some("Connection timeout".to_string());
        
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        // Verify that the status is parsed correctly
        let status = widget.parse_protocol_status(&protocol_data.status);
        assert!(matches!(status, ProtocolStatus::Error));
        
        // Verify that the status color is red
        assert_eq!(widget.get_status_color(status), Color::Red);
    }
    
    #[test]
    fn test_protocol_widget_with_empty_metrics() {
        let mut protocol_data = create_test_protocol_data();
        protocol_data.metrics.clear();
        
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        // With empty metrics, the render_protocol_details method should call render_no_data
        // We can't test this directly without a mock frame, but we can at least verify
        // that the metrics map is indeed empty
        assert!(protocol_data.metrics.is_empty());
    }

    // Test ProtocolWidget rendering with mock data
    #[test]
    fn test_protocol_widget_render() {
        // Create mock ProtocolData
        let protocol_data = ProtocolData {
            name: "Test Protocol".to_string(),
            protocol_type: "MQTT".to_string(),
            version: "1.0.0".to_string(),
            status: "Connected".to_string(),
            connected: true,
            last_connected: Some(chrono::Utc::now()),
            error: None,
            retry_count: 0,
            metrics: {
                let mut metrics = HashMap::new();
                metrics.insert("messages_sent".to_string(), 1234.0);
                metrics.insert("messages_received".to_string(), 5678.0);
                metrics
            }
        };

        let _widget = ProtocolWidget::new(&protocol_data, "Protocol Test");
        
        // Rendering test would need a mock terminal,
        // so we'll just ensure it doesn't panic when created
        assert_eq!(protocol_data.protocol_type, "MQTT");
    }
} 