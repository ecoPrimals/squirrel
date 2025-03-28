use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Tabs, List, ListItem, Chart, Dataset, Axis, GraphType},
    symbols,
    Frame,
};
use std::{iter, collections::HashMap};
use chrono::{DateTime, Utc, Duration};

use dashboard_core::{
    data::ProtocolData, 
    Protocol, 
    ProtocolStatus
};

use crate::adapter::{ConnectionHealth, ConnectionEvent, ConnectionEventType, ConnectionStatus};

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    /// Protocol data to display
    protocol: &'a ProtocolData,
    /// Widget title
    title: &'a str,
    /// Active protocol tab index
    active_tab: usize,
    /// Connection health data
    connection_health: Option<&'a ConnectionHealth>,
    /// Connection history
    connection_history: Option<&'a Vec<ConnectionEvent>>,
    /// Metrics history
    metrics_history: Option<&'a HashMap<String, Vec<(DateTime<Utc>, f64)>>>,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(protocol: &'a ProtocolData, title: &'a str) -> Self {
        Self { 
            protocol, 
            title,
            active_tab: 0,
            connection_health: None,
            connection_history: None,
            metrics_history: None,
        }
    }
    
    /// Set active tab
    pub fn active_tab(mut self, tab: usize) -> Self {
        self.active_tab = tab;
        self
    }
    
    /// Set connection health
    pub fn with_connection_health(mut self, health: &'a ConnectionHealth) -> Self {
        self.connection_health = Some(health);
        self
    }
    
    /// Set connection history
    pub fn with_connection_history(mut self, history: &'a Vec<ConnectionEvent>) -> Self {
        self.connection_history = Some(history);
        self
    }
    
    /// Set metrics history
    pub fn with_metrics_history(mut self, history: &'a HashMap<String, Vec<(DateTime<Utc>, f64)>>) -> Self {
        self.metrics_history = Some(history);
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
    
    /// Get connection status color
    fn get_connection_status_color(&self, status: &ConnectionStatus) -> Color {
        match status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Connecting => Color::Yellow,
            ConnectionStatus::Disconnected => Color::Red,
            ConnectionStatus::Error(_) => Color::Red,
        }
    }

    /// Get connection event color
    fn get_connection_event_color(&self, event_type: &ConnectionEventType) -> Color {
        match event_type {
            ConnectionEventType::Connected => Color::Green,
            ConnectionEventType::ReconnectSuccess => Color::Green,
            ConnectionEventType::Reconnecting => Color::Yellow,
            ConnectionEventType::Disconnected => Color::Red,
            ConnectionEventType::ReconnectFailure => Color::Red,
            ConnectionEventType::Error => Color::Red,
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
                Constraint::Length(3),  // Protocol status
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Content
            ])
            .split(inner_area);
        
        // Render protocol status
        self.render_protocol_status(f, chunks[0]);
        
        // Create tab titles
        let tab_titles = vec!["Overview", "Metrics", "Connection", "History"];
        
        // Render tabs
        self.render_tabs(f, chunks[1], &tab_titles);
        
        // Render tab content based on active tab
        match self.active_tab {
            0 => self.render_overview_tab(f, chunks[2]),
            1 => self.render_metrics_tab(f, chunks[2]),
            2 => self.render_connection_tab(f, chunks[2]),
            3 => self.render_history_tab(f, chunks[2]),
            _ => self.render_overview_tab(f, chunks[2]),
        }
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
        
        // Get connection health information if available
        let health_info = if let Some(health) = self.connection_health {
            let status_color = self.get_connection_status_color(&health.status);
            let latency = health.latency_ms.map_or_else(
                || "N/A".to_string(),
                |ms| format!("{}ms", ms)
            );
            
            vec![
                Span::raw(" | "),
                Span::styled("Health: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:?}", health.status),
                    Style::default().fg(status_color)
                ),
                Span::raw(" | "),
                Span::styled("Latency: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(latency),
            ]
        } else {
            vec![]
        };
        
        let status_text = Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            connection_status,
            Span::raw(" | "),
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:?}", protocol_type)),
            Span::raw(" | "),
            Span::styled("Version: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&self.protocol.version)
        ].into_iter().chain(health_info).collect::<Vec<_>>());
        
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            status_text,
        ]);
        
        f.render_widget(paragraph, area);
    }
    
    /// Render tabs
    fn render_tabs(&self, f: &mut Frame, area: Rect, titles: &[&str]) {
        let tab_titles: Vec<Line> = titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Line::from(vec![
                    Span::styled(first, Style::default().add_modifier(Modifier::UNDERLINED)),
                    Span::raw(rest),
                ])
            })
            .collect();
        
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .select(self.active_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        
        f.render_widget(tabs, area);
    }
    
    /// Render overview tab
    fn render_overview_tab(&self, f: &mut Frame, area: Rect) {
        if self.protocol.metrics.is_empty() {
            // No metrics to display
            self.render_no_data(f, area);
            return;
        }
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Render metrics in the top section
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[0]);
        
        // Render general metrics
        self.render_protocol_metrics(f, inner_chunks[0]);
        
        // Render connection status and info
        self.render_connection_info(f, inner_chunks[1]);
        
        // Render a simple time series chart in the bottom if history is available
        if let Some(history) = self.metrics_history {
            if let Some(data) = history.get("protocol.messages") {
                self.render_metrics_chart(f, chunks[1], "Message Metrics", data);
            } else {
                self.render_no_history(f, chunks[1]);
            }
        } else {
            self.render_no_history(f, chunks[1]);
        }
    }
    
    /// Render metrics tab
    fn render_metrics_tab(&self, f: &mut Frame, area: Rect) {
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
        
        // Create table with header
        let header_cells = vec![
            Cell::from(Span::styled("Metric", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Value", Style::default().add_modifier(Modifier::BOLD))),
        ];
        let header = Row::new(header_cells);
        
        // Create table
        let table = Table::new(iter::once(header).chain(rows))
            .block(Block::default().borders(Borders::ALL).title("Protocol Metrics"))
            .widths(&[
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }
    
    /// Render connection tab
    fn render_connection_tab(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area);
        
        // Render connection details
        let connection_block = Block::default()
            .borders(Borders::ALL)
            .title("Connection Details");
        
        f.render_widget(connection_block.clone(), chunks[0]);
        
        let connection_area = connection_block.inner(chunks[0]);
        
        // Build connection info text
        let mut lines = vec![];
        
        // Add connection status
        lines.push(Line::from(vec![
            Span::styled("Connected: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", self.protocol.connected),
                Style::default().fg(if self.protocol.connected { Color::Green } else { Color::Red })
            ),
        ]));
        
        // Add last connected time
        if let Some(last_connected) = self.protocol.last_connected {
            lines.push(Line::from(vec![
                Span::styled("Last Connected: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(last_connected.to_rfc3339()),
            ]));
        }
        
        // Add retry count
        lines.push(Line::from(vec![
            Span::styled("Retry Count: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", self.protocol.retry_count)),
        ]));
        
        // Add error if present
        if let Some(error) = &self.protocol.error {
            lines.push(Line::from(vec![
                Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                Span::raw(error),
            ]));
        }
        
        // Add connection health information if available
        if let Some(health) = self.connection_health {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Health Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:?}", health.status),
                    Style::default().fg(self.get_connection_status_color(&health.status))
                ),
            ]));
            
            if let Some(last_successful) = health.last_successful {
                lines.push(Line::from(vec![
                    Span::styled("Last Successful: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(last_successful.to_rfc3339()),
                ]));
            }
            
            lines.push(Line::from(vec![
                Span::styled("Failure Count: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", health.failure_count)),
            ]));
            
            if let Some(latency) = health.latency_ms {
                lines.push(Line::from(vec![
                    Span::styled("Latency: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}ms", latency)),
                ]));
            }
            
            if let Some(error_details) = &health.error_details {
                lines.push(Line::from(vec![
                    Span::styled("Error Details: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                    Span::raw(error_details),
                ]));
            }
        }
        
        let connection_paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Left);
        
        f.render_widget(connection_paragraph, connection_area);
        
        // Render connection history if available
        if let Some(history) = self.connection_history {
            let history_block = Block::default()
                .borders(Borders::ALL)
                .title("Connection History");
            
            f.render_widget(history_block.clone(), chunks[1]);
            
            let history_area = history_block.inner(chunks[1]);
            
            if history.is_empty() {
                let paragraph = Paragraph::new("No connection history available")
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(paragraph, history_area);
            } else {
                let events: Vec<ListItem> = history.iter()
                    .rev() // Most recent first
                    .take(8) // Show only the most recent events
                    .map(|event| {
                        let event_color = self.get_connection_event_color(&event.event_type);
                        let time_str = event.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                        
                        let content = Line::from(vec![
                            Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::Gray)),
                            Span::styled(format!("{:?}", event.event_type), Style::default().fg(event_color)),
                            Span::raw(if let Some(details) = &event.details {
                                format!(": {}", details)
                            } else {
                                String::new()
                            }),
                        ]);
                        
                        ListItem::new(content)
                    })
                    .collect();
                
                let events_list = List::new(events)
                    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
                
                f.render_widget(events_list, history_area);
            }
        } else {
            let paragraph = Paragraph::new("No connection history available")
                .block(Block::default().borders(Borders::ALL).title("Connection History"))
                .alignment(ratatui::layout::Alignment::Center);
            
            f.render_widget(paragraph, chunks[1]);
        }
    }
    
    /// Render history tab
    fn render_history_tab(&self, f: &mut Frame, area: Rect) {
        if let Some(history) = self.metrics_history {
            if history.is_empty() {
                let paragraph = Paragraph::new("No metrics history available")
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(paragraph, area);
                return;
            }
            
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(area);
            
            // Render message rate history in top chart if available
            if let Some(message_data) = history.get("protocol.messages") {
                self.render_metrics_chart(f, chunks[0], "Message Rate", message_data);
            } else {
                self.render_no_history(f, chunks[0]);
            }
            
            // Render latency history in bottom chart if available
            if let Some(latency_data) = history.get("protocol.latency") {
                self.render_metrics_chart(f, chunks[1], "Latency (ms)", latency_data);
            } else {
                self.render_no_history(f, chunks[1]);
            }
        } else {
            let paragraph = Paragraph::new("No metrics history available")
                .alignment(ratatui::layout::Alignment::Center);
            
            f.render_widget(paragraph, area);
        }
    }
    
    /// Render connection information
    fn render_connection_info(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Connection Status");
        
        f.render_widget(block.clone(), area);
        
        let inner_area = block.inner(area);
        
        // Create status text
        let mut lines = vec![];
        
        // Add connected status
        lines.push(Line::from(vec![
            Span::styled("Connected: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{}", self.protocol.connected),
                Style::default().fg(if self.protocol.connected { Color::Green } else { Color::Red }),
            ),
        ]));
        
        // Add connection status
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(
                self.protocol.status.clone(),
                Style::default().fg(self.get_status_color(self.parse_protocol_status(&self.protocol.status))),
            ),
        ]));
        
        // Add retry count
        lines.push(Line::from(vec![
            Span::styled("Retry Count: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", self.protocol.retry_count)),
        ]));
        
        // Add error if present
        if let Some(error) = &self.protocol.error {
            lines.push(Line::from(vec![
                Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                Span::raw(error),
            ]));
        }
        
        // Add last connected time
        if let Some(last_connected) = self.protocol.last_connected {
            lines.push(Line::from(vec![
                Span::styled("Last Connected: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(last_connected.to_rfc3339()),
            ]));
        }
        
        if let Some(health) = self.connection_health {
            // Add connection health
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Health Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:?}", health.status),
                    Style::default().fg(self.get_connection_status_color(&health.status)),
                ),
            ]));
            
            // Add failure count if > 0
            if health.failure_count > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Failures: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}", health.failure_count)),
                ]));
            }
        }
        
        let paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Left);
        
        f.render_widget(paragraph, inner_area);
    }
    
    /// Render protocol metrics
    fn render_protocol_metrics(&self, f: &mut Frame, area: Rect) {
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Protocol Metrics");
        
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Filter out important metrics to show
        let important_metrics = [
            "messages",
            "transactions",
            "errors",
            "requests",
            "responses",
            "latency",
            "success_rate",
        ];
        
        // Get the important metrics from the protocol metrics
        let mut filtered_metrics = Vec::new();
        for key in important_metrics.iter() {
            for (metric_key, metric_value) in &self.protocol.metrics {
                if metric_key.contains(key) {
                    filtered_metrics.push((metric_key, *metric_value));
                    break;
                }
            }
        }
        
        // If we don't have any important metrics, show all metrics up to a limit
        if filtered_metrics.is_empty() {
            filtered_metrics = self.protocol.metrics.iter()
                .take(8)
                .map(|(k, v)| (k, *v))
                .collect();
        }
        
        // Sort by key
        filtered_metrics.sort_by(|a, b| a.0.cmp(b.0));
        
        // Create text for metrics
        let mut lines = Vec::new();
        
        for (key, value) in filtered_metrics {
            lines.push(Line::from(vec![
                Span::styled(format!("{}: ", key), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:.2}", value)),
            ]));
        }
        
        let paragraph = Paragraph::new(lines);
        
        f.render_widget(paragraph, inner_area);
    }
    
    /// Render a metrics chart
    fn render_metrics_chart(&self, f: &mut Frame, area: Rect, title: &str, data: &[(DateTime<Utc>, f64)]) {
        // Skip if there's not enough data
        if data.len() < 2 {
            self.render_no_history(f, area);
            return;
        }
        
        // Create block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);
        
        // Convert data to chart format
        let cutoff = Utc::now() - Duration::minutes(10);
        let filtered_data: Vec<(f64, f64)> = data.iter()
            .filter(|(time, _)| *time > cutoff)
            .map(|(time, value)| {
                let timestamp = time.timestamp() as f64;
                (timestamp, *value)
            })
            .collect();
        
        // If not enough data after filtering, render no history
        if filtered_data.len() < 2 {
            self.render_no_history(f, area);
            return;
        }
        
        // Find min/max for proper scaling
        let min_x = filtered_data.iter().map(|(x, _)| *x).fold(f64::INFINITY, f64::min);
        let max_x = filtered_data.iter().map(|(x, _)| *x).fold(f64::NEG_INFINITY, f64::max);
        let min_y = filtered_data.iter().map(|(_, y)| *y).fold(f64::INFINITY, f64::min);
        let max_y = filtered_data.iter().map(|(_, y)| *y).fold(f64::NEG_INFINITY, f64::max);
        let y_margin = (max_y - min_y) * 0.1;
        
        // Create dataset
        let dataset = Dataset::default()
            .name(title)
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&filtered_data);
        
        // Create chart
        let chart = Chart::new(vec![dataset])
            .block(block)
            .x_axis(
                Axis::default()
                    .title("Time")
                    .bounds([min_x, max_x])
                    .labels(vec![])
            )
            .y_axis(
                Axis::default()
                    .title("Value")
                    .bounds([min_y.max(0.0) - y_margin, max_y + y_margin])
                    .labels(vec![
                        Span::raw(format!("{:.1}", min_y.max(0.0))),
                        Span::raw(format!("{:.1}", (min_y.max(0.0) + max_y) / 2.0)),
                        Span::raw(format!("{:.1}", max_y)),
                    ])
            );
        
        f.render_widget(chart, area);
    }
    
    /// Render message when no data is available
    fn render_no_data(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new("No protocol metrics data available.")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        
        f.render_widget(paragraph, area);
    }
    
    /// Render message when no history is available
    fn render_no_history(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new("No metrics history available.")
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        
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
        assert!(widget.connection_health.is_none());
        assert!(widget.connection_history.is_none());
        assert!(widget.metrics_history.is_none());
    }
    
    #[test]
    fn test_protocol_widget_with_connection_health() {
        let protocol_data = create_test_protocol_data();
        let health = ConnectionHealth {
            status: ConnectionStatus::Connected,
            last_successful: Some(Utc::now()),
            failure_count: 0,
            latency_ms: Some(50),
            error_details: None,
        };
        
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test")
            .with_connection_health(&health);
        
        assert!(widget.connection_health.is_some());
        assert_eq!(widget.connection_health.unwrap().status, ConnectionStatus::Connected);
    }
    
    #[test]
    fn test_protocol_widget_with_connection_history() {
        let protocol_data = create_test_protocol_data();
        let history = vec![
            ConnectionEvent {
                timestamp: Utc::now(),
                event_type: ConnectionEventType::Connected,
                details: Some("Initial connection".to_string()),
            }
        ];
        
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test")
            .with_connection_history(&history);
        
        assert!(widget.connection_history.is_some());
        assert_eq!(widget.connection_history.unwrap().len(), 1);
    }
    
    #[test]
    fn test_protocol_widget_with_metrics_history() {
        let protocol_data = create_test_protocol_data();
        let mut history = HashMap::new();
        history.insert("protocol.messages".to_string(), vec![
            (Utc::now(), 100.0),
            (Utc::now(), 150.0),
        ]);
        
        let widget = ProtocolWidget::new(&protocol_data, "Protocol Test")
            .with_metrics_history(&history);
        
        assert!(widget.metrics_history.is_some());
        assert_eq!(widget.metrics_history.unwrap().len(), 1);
    }
    
    // Other tests remain unchanged...
    
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
} 