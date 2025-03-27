use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use chrono::DateTime;
use chrono::Utc;
use dashboard_core::data::MetricsSnapshot;
use crate::widgets::ChartWidget;

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    metrics: &'a MetricsSnapshot,
    title: &'a str,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(metrics: &'a MetricsSnapshot, title: &'a str) -> Self {
        Self { metrics, title }
    }

    /// Render the widget
    pub fn render<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // Create a layout with sections for different protocol metrics
        let _chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Message stats
                Constraint::Percentage(25), // Transaction stats
                Constraint::Percentage(50), // Latency & Error stats
            ])
            .split(area);

        // Draw title block around the whole widget if a title is provided
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        f.render_widget(block, area);

        // Apply inner margins for content
        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(25), // Message stats
                Constraint::Percentage(25), // Transaction stats
                Constraint::Percentage(50), // Latency & Error stats
            ])
            .split(area);

        // Render message statistics
        self.render_message_stats::<B>(f, inner_area[0]);
        
        // Render transaction statistics
        self.render_transaction_stats::<B>(f, inner_area[1]);
        
        // Render latency and error statistics
        self.render_latency_error_stats::<B>(f, inner_area[2]);
    }

    /// Render message statistics
    fn render_message_stats<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // Get message count and rate from metrics
        let message_count = self.metrics.counters.get("protocol.messages").unwrap_or(&0);
        let message_rate = self.metrics.gauges.get("protocol.message_rate").unwrap_or(&0.0);
        
        // Get MCP-specific message metrics (if available)
        let mcp_requests = self.metrics.counters.get("mcp.requests").unwrap_or(&0);
        let mcp_responses = self.metrics.counters.get("mcp.responses").unwrap_or(&0);
        
        // Format message statistics
        let mut message_stats = vec![
            Row::new(vec![
                Cell::from("Total Messages:"),
                Cell::from(format!("{}", message_count)).style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Message Rate:"),
                Cell::from(format!("{:.2} msg/s", message_rate)).style(Style::default().fg(Color::Cyan)),
            ]),
        ];
        
        // Add MCP-specific metrics if they exist
        if *mcp_requests > 0 || *mcp_responses > 0 {
            message_stats.push(Row::new(vec![
                Cell::from("MCP Requests:"),
                Cell::from(format!("{}", mcp_requests)).style(Style::default().fg(Color::Green)),
            ]));
            message_stats.push(Row::new(vec![
                Cell::from("MCP Responses:"),
                Cell::from(format!("{}", mcp_responses)).style(Style::default().fg(Color::Green)),
            ]));
        }
        
        // Create table widget for message statistics
        let message_table = Table::new(message_stats)
            .block(Block::default().borders(Borders::ALL).title("Message Statistics"))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(1);
        
        f.render_widget(message_table, area);
    }

    /// Render transaction statistics
    fn render_transaction_stats<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // Get transaction count and rate from metrics
        let transaction_count = self.metrics.counters.get("protocol.transactions").unwrap_or(&0);
        let transaction_rate = self.metrics.gauges.get("protocol.transaction_rate").unwrap_or(&0.0);
        
        // Get MCP-specific transaction metrics (if available)
        let mcp_transactions = self.metrics.counters.get("mcp.transactions").unwrap_or(&0);
        let mcp_success_rate = self.metrics.gauges.get("mcp.success_rate").unwrap_or(&100.0);
        
        // Format transaction statistics
        let mut transaction_stats = vec![
            Row::new(vec![
                Cell::from("Total Transactions:"),
                Cell::from(format!("{}", transaction_count)).style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Transaction Rate:"),
                Cell::from(format!("{:.2} tx/s", transaction_rate)).style(Style::default().fg(Color::Green)),
            ]),
        ];
        
        // Add MCP-specific metrics if they exist
        if *mcp_transactions > 0 {
            transaction_stats.push(Row::new(vec![
                Cell::from("MCP Transactions:"),
                Cell::from(format!("{}", mcp_transactions)).style(Style::default().fg(Color::Green)),
            ]));
            transaction_stats.push(Row::new(vec![
                Cell::from("Success Rate:"),
                Cell::from(format!("{:.2}%", mcp_success_rate)).style(Style::default().fg(Color::Green)),
            ]));
        }
        
        // Create table widget for transaction statistics
        let transaction_table = Table::new(transaction_stats)
            .block(Block::default().borders(Borders::ALL).title("Transaction Statistics"))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(1);
        
        f.render_widget(transaction_table, area);
    }

    /// Render latency and error statistics
    fn render_latency_error_stats<B: Backend>(&self, f: &mut Frame, area: Rect) {
        // Split the area for latency and error stats
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Latency chart
                Constraint::Percentage(50), // Error stats
            ])
            .split(area);
        
        // Get latency data from metrics
        if let Some(latency_data) = self.metrics.histograms.get("protocol.latency") {
            // Create dummy data points for the chart with timestamps
            let now = Utc::now();
            let chart_data: Vec<(DateTime<Utc>, f64)> = latency_data
                .iter()
                .enumerate()
                .map(|(i, &value)| {
                    // Create timestamps at 1-second intervals
                    let timestamp = now - chrono::Duration::seconds(latency_data.len() as i64 - i as i64);
                    (timestamp, value)
                })
                .collect();
            
            // Create latency chart
            let latency_chart = ChartWidget::new(&chart_data, "Latency Distribution")
                .y_label("ms")
                .chart_type(crate::widgets::ChartType::Line);
            
            f.render_widget(latency_chart, chunks[0]);
        } else {
            // Show message if no latency data available
            let no_data = Paragraph::new("No latency data available")
                .block(Block::default().borders(Borders::ALL).title("Latency Distribution"))
                .wrap(Wrap { trim: true });
            
            f.render_widget(no_data, chunks[0]);
        }
        
        // Get error metrics
        let error_count = self.metrics.counters.get("protocol.errors").unwrap_or(&0);
        let error_rate = self.metrics.gauges.get("protocol.error_rate").unwrap_or(&0.0);
        
        // Get MCP-specific error metrics (if available)
        let mcp_connection_errors = self.metrics.counters.get("mcp.connection_errors").unwrap_or(&0);
        let mcp_protocol_errors = self.metrics.counters.get("mcp.protocol_errors").unwrap_or(&0);
        
        // Calculate error color based on rate
        let error_color = if *error_rate > 5.0 {
            Color::Red
        } else if *error_rate > 1.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        
        // Format error statistics
        let mut error_stats = vec![
            Row::new(vec![
                Cell::from("Total Errors:"),
                Cell::from(format!("{}", error_count)).style(Style::default().fg(error_color)),
            ]),
            Row::new(vec![
                Cell::from("Error Rate:"),
                Cell::from(format!("{:.2}%", error_rate)).style(Style::default().fg(error_color)),
            ]),
            Row::new(vec![
                Cell::from("Status:"),
                Cell::from(if *error_rate > 5.0 {
                    "Critical"
                } else if *error_rate > 1.0 {
                    "Warning"
                } else {
                    "Healthy"
                }).style(Style::default().fg(error_color)),
            ]),
        ];
        
        // Add MCP-specific errors if they exist
        if *mcp_connection_errors > 0 || *mcp_protocol_errors > 0 {
            error_stats.push(Row::new(vec![
                Cell::from("Connection Errors:"),
                Cell::from(format!("{}", mcp_connection_errors)).style(Style::default().fg(Color::Red)),
            ]));
            error_stats.push(Row::new(vec![
                Cell::from("Protocol Errors:"),
                Cell::from(format!("{}", mcp_protocol_errors)).style(Style::default().fg(Color::Red)),
            ]));
        }
        
        // Create table widget for error statistics
        let error_table = Table::new(error_stats)
            .block(Block::default().borders(Borders::ALL).title("Error Statistics"))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(1);
        
        f.render_widget(error_table, chunks[1]);
    }
} 