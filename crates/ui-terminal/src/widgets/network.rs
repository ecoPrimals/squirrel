use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line as Spans, Text},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, Widget},
};

use dashboard_core::data::NetworkSnapshot;
use crate::util;

/// Widget for displaying network metrics
pub struct NetworkWidget<'a> {
    /// Network metrics data
    metrics: &'a NetworkSnapshot,
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed information
    detailed: bool,
}

impl<'a> NetworkWidget<'a> {
    /// Create a new network widget
    pub fn new(metrics: &'a NetworkSnapshot, title: &'a str) -> Self {
        Self {
            metrics,
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed network widget
    pub fn new_detailed(metrics: &'a NetworkSnapshot, title: &'a str) -> Self {
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
        block.clone().render(area, buf);
        
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
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(inner_area)
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
fn render_network_summary(metrics: &NetworkSnapshot, area: Rect, buf: &mut Buffer) {
    // Create content
    let mut content = Vec::new();
    
    // Add bytes in/out
    content.push(Spans::from(vec![
        Span::styled("Bytes In: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_bytes(metrics.rx_bytes),
            Style::default().fg(Color::Blue),
        ),
        Span::raw("   "),
        Span::styled("Bytes Out: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_bytes(metrics.tx_bytes),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add connections
    let active_connections = metrics.interfaces.len();
    content.push(Spans::from(vec![
        Span::styled("Active Interfaces: ", Style::default().fg(Color::White)),
        Span::styled(
            active_connections.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    
    // Add packets in/out
    content.push(Spans::from(vec![
        Span::styled("Packets In: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.rx_packets.to_string(),
            Style::default().fg(Color::Blue),
        ),
        Span::raw("   "),
        Span::styled("Packets Out: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.tx_packets.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Count errors (dummy value for now)
    let errors = 0;
    content.push(Spans::from(vec![
        Span::styled("Errors: ", Style::default().fg(Color::White)),
        Span::styled(
            errors.to_string(),
            if errors > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]));
    
    // Calculate packet loss rate (dummy value for now)
    let packet_loss_rate = 0.0;
    let loss_rate_color = if packet_loss_rate < 0.01 {
        Color::Green
    } else if packet_loss_rate < 0.05 {
        Color::Yellow
    } else {
        Color::Red
    };
    
    content.push(Spans::from(vec![
        Span::styled("Packet Loss: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_percentage(packet_loss_rate * 100.0),
            Style::default().fg(loss_rate_color),
        ),
    ]));
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(area, buf);
}

/// Render connections table
fn render_connections_table(metrics: &NetworkSnapshot, area: Rect, buf: &mut Buffer) {
    // Create table header block
    let block = Block::default()
        .borders(Borders::TOP)
        .title("Network Interfaces");
    
    // Render block
    block.clone().render(area, buf);
    
    // Get inner area
    let inner_area = block.inner(area);
    
    // Handle no connections
    if metrics.interfaces.is_empty() {
        let text = vec![Spans::from(vec![
            Span::styled("No active interfaces", Style::default().fg(Color::Yellow))
        ])];
        
        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::White));
        
        paragraph.render(inner_area, buf);
        return;
    }
    
    // Create table rows
    let rows: Vec<Row> = metrics.interfaces
        .iter()
        .map(|(name, info)| {
            // Determine connection status color
            let status_color = if info.is_up {
                Color::Green
            } else {
                Color::Red
            };
            
            let cells = vec![
                Cell::from(name.clone()).style(Style::default()),
                Cell::from(format!("{}", util::format_bytes(info.rx_bytes))).style(Style::default()),
                Cell::from(format!("{}", util::format_bytes(info.tx_bytes))).style(Style::default()),
                Cell::from(format!("{}", info.rx_packets)).style(Style::default()),
                Cell::from(format!("{}", if info.is_up { "Up" } else { "Down" }))
                    .style(Style::default().fg(status_color)),
            ];
            
            Row::new(cells)
        })
        .collect();
    
    // Create table
    let table = Table::new(rows)
        .header(
            Row::new(vec![
                Cell::from("Interface").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("RX Bytes").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("TX Bytes").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("RX Packets").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Status").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .column_spacing(1);
    
    // Render table
    table.render(inner_area, buf);
} 