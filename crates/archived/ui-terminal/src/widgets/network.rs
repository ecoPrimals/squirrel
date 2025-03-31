use ratatui::{
    layout::{Rect, Alignment},
    widgets::{Widget, Block, Borders, List, ListItem},
    buffer::Buffer,
    style::{Style, Modifier},
    text::{Line, Span},
    prelude::{Alignment, Constraint, Direction, Layout, Style, Text},
};

use dashboard_core::data::NetworkMetrics;
use crate::adapter::ConnectionStatus;
use crate::theme::Theme;
use dashboard_core::{NetworkInterface, NetworkMetrics};
use ratatui::{
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Paragraph},
};

/// Widget for displaying network metrics
pub struct NetworkWidget<'a> {
    /// Network metrics to display
    metrics: Option<&'a NetworkMetrics>,
    /// Title of the widget
    title: &'a str,
}

impl<'a> NetworkWidget<'a> {
    /// Create a new network widget
    pub fn new(metrics: Option<&'a NetworkMetrics>, title: &'a str) -> Self {
        Self {
            metrics,
            title,
        }
    }
}

/// Format bytes into human readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

impl<'a> Widget for NetworkWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title(self.title).borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = match self.metrics {
            Some(m) if !m.interfaces.is_empty() => m
                .interfaces
                .iter()
                .map(|iface| {
                    let status_color = if iface.is_up {
                        Theme::default().success_style.fg.unwrap_or(Color::Green)
                    } else {
                        Theme::default().error_style.fg.unwrap_or(Color::Red)
                    };
                    let status_text = if iface.is_up { "UP" } else { "DOWN" };
                    // TODO: Calculate rate properly, using totals for now
                    let network_info = format!(
                        "RX: {} bytes | TX: {} bytes",
                        iface.rx_bytes, iface.tx_bytes // Use existing fields
                    );

                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<15}", iface.name),
                            Style::default().fg(Theme::default().primary_color),
                        ),
                        Span::styled(format!(" [{}]", status_text), Style::default().fg(status_color)),
                        Span::raw(" | "),
                        Span::styled(network_info, Style::default().fg(Theme::default().secondary_color)),
                    ]))
                })
                .collect(),
            Some(_) => {
                // Handles the case where metrics exist but interfaces list is empty
                let text = Paragraph::new("No network interfaces detected.") // Use Paragraph
                    .style(Style::default().fg(Color::DarkGray)); // Use Color
                // Need to render Paragraph, handle differently or return empty Vec<ListItem>
                // For now, return Vec with a single placeholder item
                vec![ListItem::new(Line::from(Span::styled(
                    "No network interfaces detected.",
                     Style::default().fg(Color::DarkGray) // Use Color
                )))]

            }
            None => {
                // Handles the case where metrics is None
                let text = Paragraph::new("Network metrics unavailable.") // Use Paragraph
                    .style(Style::default().fg(Color::DarkGray)); // Use Color
                // Need to render Paragraph, handle differently or return empty Vec<ListItem>
                 // For now, return Vec with a single placeholder item
                vec![ListItem::new(Line::from(Span::styled(
                    "Network metrics unavailable.",
                     Style::default().fg(Color::DarkGray) // Use Color
                )))]
            }
        };

        let list = List::new(items)
            .block(Block::default()) // Optional: remove inner block if redundant
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        list.render(inner_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_widget_creation() {
        let metrics = NetworkMetrics {
            interfaces: vec![
                dashboard_core::data::NetworkInterface {
                    name: "eth0".to_string(),
                    rx_bytes: 1024,
                    tx_bytes: 2048,
                    rx_packets: 100,
                    tx_packets: 200,
                },
            ],
        };
        
        let widget = NetworkWidget::new(Some(&metrics), "Test Network");
        assert_eq!(widget.title, "Test Network");
    }
} 