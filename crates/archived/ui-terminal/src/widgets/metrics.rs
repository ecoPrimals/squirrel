use ratatui::{
    layout::{Rect, Alignment},
    widgets::{Widget, Block, Borders, List, ListItem},
    buffer::Buffer,
    style::{Style, Modifier},
    text::{Line, Span},
    prelude::{Alignment, Constraint, Direction, Layout, Style, Text},
};

use dashboard_core::Metrics;
use crate::util::format_bytes;

/// Widget for displaying system metrics
pub struct MetricsWidget<'a> {
    /// System metrics to display
    metrics: Option<&'a Metrics>,
    /// Title of the widget
    title: &'a str,
}

impl<'a> MetricsWidget<'a> {
    /// Create a new metrics widget
    pub fn new(metrics: Option<&'a Metrics>, title: &'a str) -> Self {
        Self {
            metrics,
            title,
        }
    }
}

impl<'a> Widget for MetricsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(metrics) = self.metrics {
            let items = vec![
                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw("CPU Usage: "),
                        Span::styled(
                            format!("{:.1}%", metrics.cpu.usage),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Load Avg: "),
                        Span::styled(
                            format!("{:.2} {:.2} {:.2}", 
                                metrics.cpu.load[0],
                                metrics.cpu.load[1],
                                metrics.cpu.load[2],
                            ),
                            Style::default(),
                        ),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw("Memory Used: "),
                        Span::styled(
                            format!("{} / {}", 
                                format_bytes(metrics.memory.used),
                                format_bytes(metrics.memory.total),
                            ),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Swap Used: "),
                        Span::styled(
                            format!("{} / {}", 
                                format_bytes(metrics.memory.swap_used),
                                format_bytes(metrics.memory.swap_total),
                            ),
                            Style::default(),
                        ),
                    ]),
                ]),
                ListItem::new(vec![
                    Line::from(vec![
                        Span::raw("Disk I/O: "),
                        Span::styled(
                            format!("R: {} W: {}", 
                                format_bytes(metrics.disk.read_bytes),
                                format_bytes(metrics.disk.written_bytes),
                            ),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::raw("Disk Ops: "),
                        Span::styled(
                            format!("R: {} W: {}", 
                                metrics.disk.total_reads,
                                metrics.disk.total_writes,
                            ),
                            Style::default(),
                        ),
                    ]),
                ]),
            ];
    
            let list = List::new(items)
                .block(block)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            list.render(inner_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_widget_creation() {
        let metrics = Metrics {
            cpu: dashboard_core::CpuMetrics {
                usage: 50.0,
                cores: vec![],
                temperature: None,
                load: [1.0, 1.5, 2.0],
            },
            memory: dashboard_core::MemoryMetrics {
                total: 16 * 1024 * 1024 * 1024,
                used: 8 * 1024 * 1024 * 1024,
                available: 8 * 1024 * 1024 * 1024,
                free: 8 * 1024 * 1024 * 1024,
                swap_used: 1 * 1024 * 1024 * 1024,
                swap_total: 8 * 1024 * 1024 * 1024,
            },
            network: dashboard_core::NetworkMetrics {
                interfaces: vec![],
                total_rx_bytes: 0,
                total_tx_bytes: 0,
                total_rx_packets: 0,
                total_tx_packets: 0,
            },
            disk: dashboard_core::DiskMetrics {
                usage: Default::default(),
                total_reads: 1000,
                total_writes: 500,
                read_bytes: 1024 * 1024,
                written_bytes: 512 * 1024,
            },
            history: Default::default(),
        };
        
        let widget = MetricsWidget::new(Some(&metrics), "Test Metrics");
        assert_eq!(widget.title, "Test Metrics");
    }
} 