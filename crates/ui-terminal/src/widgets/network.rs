use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph},
    Frame,
};

use dashboard_core::data::NetworkMetrics;
use crate::util::{format_bytes, format_bytes_rate};

/// Widget for displaying network metrics
pub struct NetworkWidget<'a> {
    /// Network metrics to display
    metrics: &'a NetworkMetrics,
    /// Widget title
    title: &'a str,
}

impl<'a> NetworkWidget<'a> {
    /// Create a new network widget
    pub fn new(metrics: &'a NetworkMetrics, title: &'a str) -> Self {
        Self { metrics, title }
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
        
        // Format network stats
        let total_rx = format_bytes_rate(self.metrics.total_rx_bytes, "/s");
        let total_tx = format_bytes_rate(self.metrics.total_tx_bytes, "/s");
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Stats header
                Constraint::Min(0),    // Interface list
            ])
            .split(inner_area);
        
        // Create stats header with RX/TX info
        let rx_line = Line::from(vec![
            Span::styled("Received: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(total_rx),
            Span::raw(format!(" ({} packets/s)", self.metrics.total_rx_packets)),
        ]);
        
        let tx_line = Line::from(vec![
            Span::styled("Transmitted: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(total_tx),
            Span::raw(format!(" ({} packets/s)", self.metrics.total_tx_packets)),
        ]);
        
        let stats_paragraph = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("Network Interfaces ({})", self.metrics.interfaces.len()),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
                ),
            ]),
            rx_line,
            tx_line,
        ]);
        
        f.render_widget(stats_paragraph, chunks[0]);
        
        // Create interface list rows
        let mut rows = Vec::new();
        
        // Add header row
        let header = Row::new(vec![
            Cell::from(Span::styled("Interface", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Status", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("RX", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("TX", Style::default().add_modifier(Modifier::BOLD))),
            Cell::from(Span::styled("Errors", Style::default().add_modifier(Modifier::BOLD))),
        ]);
        
        // Add interface rows
        for interface in &self.metrics.interfaces {
            let status_style = if interface.is_up {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            
            rows.push(
                Row::new(vec![
                    Cell::from(interface.name.clone()),
                    Cell::from(Span::styled(
                        if interface.is_up { "UP" } else { "DOWN" },
                        status_style,
                    )),
                    Cell::from(format_bytes(interface.rx_bytes)),
                    Cell::from(format_bytes(interface.tx_bytes)),
                    Cell::from(format!("Rx: {}, Tx: {}", interface.rx_errors, interface.tx_errors)),
                ])
            );
        }
        
        // Create interface table with all rows
        let rows_with_header = std::iter::once(header).chain(rows).collect::<Vec<_>>();
        let table = Table::new(
            rows_with_header,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ]
        )
            .block(Block::default().borders(Borders::ALL).title("Interface Details"))
            .widths([
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ])
            .column_spacing(1);
        
        f.render_widget(table, chunks[1]);
    }
} 