use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use chrono::DateTime;
use chrono::Utc;
use dashboard_core::data::ProtocolData;
use crate::widgets::ChartWidget;

/// Widget for displaying protocol metrics
pub struct ProtocolWidget<'a> {
    protocol: &'a ProtocolData,
    title: &'a str,
}

impl<'a> ProtocolWidget<'a> {
    /// Create a new protocol widget
    pub fn new(protocol: &'a ProtocolData, title: &'a str) -> Self {
        Self { protocol, title }
    }

    /// Render the widget
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Create a layout with sections for different protocol metrics
        let _chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25), // Message stats
                Constraint::Percentage(25), // Transaction stats
                Constraint::Percentage(50), // Latency & Error stats
            ])
            .split(area);

        // Draw title block around the whole widget with data quality indicator
        let title = format!("{}{}", self.title, self.get_data_quality_indicator());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);
        f.render_widget(block, area);

        // Apply inner margins for content
        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(25), // Connection status
                Constraint::Percentage(25), // Protocol info
                Constraint::Percentage(50), // Protocol metrics
            ])
            .split(area);

        // Render connection status
        self.render_connection_status::<B>(f, inner_area[0]);
        
        // Render protocol info
        self.render_protocol_info::<B>(f, inner_area[1]);
        
        // Render protocol metrics
        self.render_protocol_metrics::<B>(f, inner_area[2]);
    }

    /// Get data quality indicator for the title
    fn get_data_quality_indicator(&self) -> &str {
        if self.is_simulated_data() {
            " [Simulated]"
        } else if self.is_stale_data() {
            " [Stale]"
        } else {
            ""
        }
    }

    /// Check if the data is simulated
    fn is_simulated_data(&self) -> bool {
        self.protocol.metrics.get("simulated")
            .map_or(false, |v| v == "true")
    }

    /// Check if the data is stale (cached)
    fn is_stale_data(&self) -> bool {
        match self.protocol.metrics.get("last_real_data") {
            Some(timestamp_str) => {
                // Try to parse the timestamp
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                    let now = Utc::now();
                    let time_diff = now.signed_duration_since(timestamp.with_timezone(&Utc));
                    // Consider data stale if it's more than 5 minutes old
                    time_diff.num_minutes() > 5
                } else {
                    false
                }
            },
            None => false,
        }
    }

    /// Render connection status
    fn render_connection_status<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let status_color = if self.protocol.connected {
            Color::Green
        } else {
            Color::Red
        };
        
        let status_text = if self.protocol.connected {
            "Connected"
        } else {
            "Disconnected"
        };
        
        let status_since = if let Some(last_connected) = self.protocol.last_connected {
            let now = Utc::now();
            let duration = now.signed_duration_since(last_connected);
            
            if duration.num_seconds() < 60 {
                format!("since {} seconds ago", duration.num_seconds())
            } else if duration.num_minutes() < 60 {
                format!("since {} minutes ago", duration.num_minutes())
            } else {
                format!("since {}", last_connected.format("%Y-%m-%d %H:%M:%S"))
            }
        } else {
            "".to_string()
        };
        
        let error_text = if let Some(error) = &self.protocol.error {
            format!("Error: {}", error)
        } else {
            "No errors".to_string()
        };
        
        let rows = vec![
            Row::new(vec![
                Cell::from("Status:"),
                Cell::from(status_text).style(Style::default().fg(status_color)),
            ]),
            Row::new(vec![
                Cell::from("Connection:"),
                Cell::from(status_since),
            ]),
            Row::new(vec![
                Cell::from("Retries:"),
                Cell::from(format!("{}", self.protocol.retry_count)),
            ]),
            Row::new(vec![
                Cell::from("Error:"),
                Cell::from(error_text).style(Style::default().fg(if self.protocol.error.is_some() { Color::Red } else { Color::Green })),
            ]),
        ];
        
        let table = Table::new(rows)
            .block(Block::default().borders(Borders::ALL).title("Connection Status"))
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }

    /// Render protocol info
    fn render_protocol_info<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from("Version:"),
                Cell::from(&self.protocol.version),
            ]),
        ];
        
        // Add protocol-specific data
        let mut all_rows = rows;
        for (key, value) in &self.protocol.data {
            all_rows.push(Row::new(vec![
                Cell::from(format!("{}:", key)),
                Cell::from(value),
            ]));
        }
        
        let table = Table::new(all_rows)
            .block(Block::default().borders(Borders::ALL).title("Protocol Info"))
            .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
            .column_spacing(1);
        
        f.render_widget(table, area);
    }

    /// Render protocol metrics
    fn render_protocol_metrics<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Split the area for different metrics categories
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        let metrics_keys: Vec<&String> = self.protocol.metrics.keys().collect();
        
        // Message and transaction metrics
        let message_keys: Vec<&String> = metrics_keys.iter()
            .filter(|k| k.contains("packet") || k.contains("message") || k.contains("latency"))
            .cloned()
            .collect();
            
        // Error metrics
        let error_keys: Vec<&String> = metrics_keys.iter()
            .filter(|k| k.contains("error") || k.contains("fail"))
            .cloned()
            .collect();
        
        // Message metrics table
        let mut message_rows = Vec::new();
        for key in message_keys {
            if let Some(value) = self.protocol.metrics.get(key) {
                message_rows.push(Row::new(vec![
                    Cell::from(key.to_string()),
                    Cell::from(value.to_string()),
                ]));
            }
        }
        
        let message_table = Table::new(message_rows)
            .block(Block::default().borders(Borders::ALL).title("Message Metrics"))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(1);
        
        // Error metrics table
        let mut error_rows = Vec::new();
        for key in error_keys {
            if let Some(value) = self.protocol.metrics.get(key) {
                error_rows.push(Row::new(vec![
                    Cell::from(key.to_string()),
                    Cell::from(value.to_string()).style(Style::default().fg(Color::Red)),
                ]));
            }
        }
        
        let error_table = Table::new(error_rows)
            .block(Block::default().borders(Borders::ALL).title("Error Metrics"))
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .column_spacing(1);
        
        // Render tables
        f.render_widget(message_table, chunks[0]);
        f.render_widget(error_table, chunks[1]);
    }
} 