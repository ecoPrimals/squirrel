use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, Widget},
};

use dashboard_core::data::NetworkMetricsSnapshot;
use crate::util;

/// Widget for displaying network metrics
pub struct NetworkWidget<'a> {
    /// Network metrics data
    metrics: &'a NetworkMetricsSnapshot,
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed information
    detailed: bool,
}

impl<'a> NetworkWidget<'a> {
    /// Create a new network widget
    pub fn new(metrics: &'a NetworkMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics,
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed network widget
    pub fn new_detailed(metrics: &'a NetworkMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics,
            title,
            detailed: true,
        }
    }
    
    /// Set detailed mode
    pub fn detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }
}

impl<'a> Widget for NetworkWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        block.render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Create layout
        let chunks = if self.detailed {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),  // Summary section
                    Constraint::Min(0),     // Connections table
                ])
                .split(inner_area)
        } else {
            vec![inner_area]
        };
        
        // Render summary
        render_network_summary(self.metrics, chunks[0], buf);
        
        // Render connections table if detailed
        if self.detailed && chunks.len() > 1 {
            render_connections_table(self.metrics, chunks[1], buf);
        }
    }
}

/// Render network summary
fn render_network_summary(metrics: &NetworkMetricsSnapshot, area: Rect, buf: &mut Buffer) {
    // Create content
    let mut content = Vec::new();
    
    // Add bytes in/out
    content.push(Spans::from(vec![
        Span::styled("Bytes In: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_bytes(metrics.bytes_in),
            Style::default().fg(Color::Blue),
        ),
        Span::raw("   "),
        Span::styled("Bytes Out: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_bytes(metrics.bytes_out),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add connections
    content.push(Spans::from(vec![
        Span::styled("Active Connections: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.active_connections.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    
    // Add packets in/out
    content.push(Spans::from(vec![
        Span::styled("Packets In: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.packets_in.to_string(),
            Style::default().fg(Color::Blue),
        ),
        Span::raw("   "),
        Span::styled("Packets Out: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.packets_out.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add errors
    content.push(Spans::from(vec![
        Span::styled("Errors: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.errors.to_string(),
            if metrics.errors > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]));
    
    // Add packet loss rate
    let loss_rate_color = if metrics.packet_loss_rate < 0.01 {
        Color::Green
    } else if metrics.packet_loss_rate < 0.05 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    content.push(Spans::from(vec![
        Span::styled("Packet Loss: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_percentage(metrics.packet_loss_rate * 100.0),
            Style::default().fg(loss_rate_color),
        ),
    ]));
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(area, buf);
}

/// Render connections table
fn render_connections_table(metrics: &NetworkMetricsSnapshot, area: Rect, buf: &mut Buffer) {
    // Create table header block
    let block = Block::default()
        .borders(Borders::TOP)
        .title("Active Connections");
    
    // Render block
    block.render(area, buf);
    
    // Get inner area
    let inner_area = block.inner(area);
    
    // Handle no connections
    if metrics.connections.is_empty() {
        let text = vec![Spans::from(vec![
            Span::styled("No active connections", Style::default().fg(Color::Yellow))
        ])];
        
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White));
        
        paragraph.render(inner_area, buf);
        return;
    }
    
    // Create table rows
    let rows: Vec<Row> = metrics.connections
        .iter()
        .map(|conn| {
            // Determine connection status color
            let status_color = if conn.is_active {
                Color::Green
            } else {
                Color::Red
            };
            
            let cells = vec![
                Cell::from(format!("{}", conn.source)).style(Style::default()),
                Cell::from(format!("{}", conn.destination)).style(Style::default()),
                Cell::from(format!("{}", conn.protocol)).style(Style::default()),
                Cell::from(format!("{}", util::format_duration(conn.duration))).style(Style::default()),
                Cell::from(format!("{}", if conn.is_active { "Active" } else { "Inactive" }))
                    .style(Style::default().fg(status_color)),
            ];
            
            Row::new(cells)
        })
        .collect();
    
    // Create table
    let table = Table::new(rows)
        .header(
            Row::new(vec![
                Cell::from("Source").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Destination").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Protocol").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Duration").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Status").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])
        )
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ])
        .column_spacing(1);
    
    // Render table
    table.render(inner_area, buf);
} 