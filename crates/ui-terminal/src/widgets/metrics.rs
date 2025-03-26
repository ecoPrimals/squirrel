use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use dashboard_core::data::{SystemMetricsSnapshot, ProtocolMetricsSnapshot};
use crate::util;

/// Widget for displaying system metrics
pub struct MetricsWidget<'a> {
    /// Metrics data
    metrics: Option<&'a SystemMetricsSnapshot>,
    
    /// Protocol metrics
    protocol_metrics: Option<&'a ProtocolMetricsSnapshot>,
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed metrics
    detailed: bool,
}

impl<'a> MetricsWidget<'a> {
    /// Create a new metrics widget for system metrics
    pub fn new(metrics: &'a SystemMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: Some(metrics),
            protocol_metrics: None,
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed metrics widget for system metrics
    pub fn new_detailed(metrics: &'a SystemMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: Some(metrics),
            protocol_metrics: None,
            title,
            detailed: true,
        }
    }
    
    /// Create a new metrics widget for protocol metrics
    pub fn new_protocol(metrics: &'a ProtocolMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: None,
            protocol_metrics: Some(metrics),
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed metrics widget for protocol metrics
    pub fn new_protocol_detailed(metrics: &'a ProtocolMetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: None,
            protocol_metrics: Some(metrics),
            title,
            detailed: true,
        }
    }
}

impl<'a> Widget for MetricsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        block.render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Render metrics
        if let Some(metrics) = self.metrics {
            render_system_metrics(metrics, inner_area, buf, self.detailed);
        } else if let Some(metrics) = self.protocol_metrics {
            render_protocol_metrics(metrics, inner_area, buf, self.detailed);
        }
    }
}

/// Render system metrics
fn render_system_metrics(metrics: &SystemMetricsSnapshot, area: Rect, buf: &mut Buffer, detailed: bool) {
    // Format metrics
    let cpu_text = format!("CPU: {}", util::format_percentage(metrics.cpu_usage));
    let memory_text = format!("Memory: {}", util::format_bytes(metrics.memory_usage));
    let uptime_text = format!("Uptime: {}", util::format_duration(metrics.uptime));
    let threads_text = format!("Threads: {}", metrics.thread_count);
    let errors_text = format!("Errors: {}", metrics.error_count);
    
    // Create text content
    let mut content = Vec::new();
    
    // Add CPU usage with bar
    if detailed {
        content.push(Spans::from(vec![
            Span::styled("CPU Usage: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_percentage(metrics.cpu_usage),
                Style::default().fg(get_usage_color(metrics.cpu_usage)),
            ),
        ]));
        content.push(Spans::from(vec![
            Span::styled(
                util::calculate_bar(metrics.cpu_usage, area.width as usize - 4),
                Style::default().fg(get_usage_color(metrics.cpu_usage)),
            ),
        ]));
        content.push(Spans::from(""));
    } else {
        content.push(Spans::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_percentage(metrics.cpu_usage),
                Style::default().fg(get_usage_color(metrics.cpu_usage)),
            ),
        ]));
    }
    
    // Add memory usage
    if detailed {
        content.push(Spans::from(vec![
            Span::styled("Memory Usage: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_bytes(metrics.memory_usage),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    } else {
        content.push(Spans::from(vec![
            Span::styled("Memory: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_bytes(metrics.memory_usage),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }
    
    // Add uptime
    content.push(Spans::from(vec![
        Span::styled("Uptime: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_duration(metrics.uptime),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add thread count
    content.push(Spans::from(vec![
        Span::styled("Threads: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.thread_count.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    
    // Add error count
    content.push(Spans::from(vec![
        Span::styled("Errors: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.error_count.to_string(),
            if metrics.error_count > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]));
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(area, buf);
}

/// Render protocol metrics
fn render_protocol_metrics(metrics: &ProtocolMetricsSnapshot, area: Rect, buf: &mut Buffer, detailed: bool) {
    // Create text content
    let mut content = Vec::new();
    
    // Add messages processed
    content.push(Spans::from(vec![
        Span::styled("Messages: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.messages_processed.to_string(),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add average latency
    content.push(Spans::from(vec![
        Span::styled("Latency: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{}ms", metrics.avg_latency.as_millis()),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    
    // Add error rate
    content.push(Spans::from(vec![
        Span::styled("Error Rate: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_percentage(metrics.error_rate),
            if metrics.error_rate > 5.0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]));
    
    // Add active connections
    content.push(Spans::from(vec![
        Span::styled("Connections: ", Style::default().fg(Color::White)),
        Span::styled(
            metrics.active_connections.to_string(),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    
    // Add visualization if detailed
    if detailed {
        content.push(Spans::from(""));
        content.push(Spans::from(Span::styled("Error Rate", Style::default().fg(Color::White))));
        content.push(Spans::from(vec![
            Span::styled(
                util::calculate_bar(metrics.error_rate * 10.0, area.width as usize - 4),
                if metrics.error_rate > 5.0 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]));
    }
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(area, buf);
}

/// Get the color for usage values
fn get_usage_color(usage: f64) -> Color {
    if usage > 90.0 {
        Color::Red
    } else if usage > 70.0 {
        Color::Yellow
    } else {
        Color::Green
    }
} 