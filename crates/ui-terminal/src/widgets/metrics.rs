use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line as Spans, Text},
    widgets::{Block, Borders, Paragraph, Widget},
};

use dashboard_core::data::{SystemSnapshot, MetricsSnapshot};
use crate::util;
use std::time::Duration;

/// Widget for displaying system metrics
pub struct MetricsWidget<'a> {
    /// Metrics data
    metrics: Option<&'a MetricsSnapshot>,
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed metrics
    detailed: bool,
}

impl<'a> MetricsWidget<'a> {
    /// Create a new metrics widget for system metrics
    pub fn new(metrics: &'a MetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: Some(metrics),
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed metrics widget for system metrics
    pub fn new_detailed(metrics: &'a MetricsSnapshot, title: &'a str) -> Self {
        Self {
            metrics: Some(metrics),
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
        block.clone().render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Render metrics
        if let Some(metrics) = self.metrics {
            render_system_metrics(metrics, inner_area, buf, self.detailed);
        }
    }
}

/// Render system metrics
fn render_system_metrics(metrics: &MetricsSnapshot, area: Rect, buf: &mut Buffer, detailed: bool) {
    // Get metrics from values
    let cpu_usage = *metrics.values.get("cpu_usage").unwrap_or(&0.0);
    let memory_usage = *metrics.values.get("memory_usage").unwrap_or(&0.0);
    let uptime = *metrics.values.get("uptime").unwrap_or(&0.0);
    let thread_count = *metrics.counters.get("thread_count").unwrap_or(&0);
    let error_count = *metrics.counters.get("error_count").unwrap_or(&0);
    
    // Create text content
    let mut content = Vec::new();
    
    // Add CPU usage with bar
    if detailed {
        content.push(Spans::from(vec![
            Span::styled("CPU Usage: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_percentage(cpu_usage),
                Style::default().fg(get_usage_color(cpu_usage)),
            ),
        ]));
        content.push(Spans::from(vec![
            Span::styled(
                util::calculate_bar(cpu_usage, area.width as usize - 4),
                Style::default().fg(get_usage_color(cpu_usage)),
            ),
        ]));
        content.push(Spans::from(""));
    } else {
        content.push(Spans::from(vec![
            Span::styled("CPU: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_percentage(cpu_usage),
                Style::default().fg(get_usage_color(cpu_usage)),
            ),
        ]));
    }
    
    // Add memory usage
    if detailed {
        content.push(Spans::from(vec![
            Span::styled("Memory Usage: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_bytes(memory_usage as u64),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    } else {
        content.push(Spans::from(vec![
            Span::styled("Memory: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_bytes(memory_usage as u64),
                Style::default().fg(Color::Cyan),
            ),
        ]));
    }
    
    // Add uptime
    content.push(Spans::from(vec![
        Span::styled("Uptime: ", Style::default().fg(Color::White)),
        Span::styled(
            util::format_duration(Duration::from_secs(uptime as u64)),
            Style::default().fg(Color::Green),
        ),
    ]));
    
    // Add thread count
    content.push(Spans::from(vec![
        Span::styled("Threads: ", Style::default().fg(Color::White)),
        Span::styled(
            thread_count.to_string(),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    
    // Add error count
    content.push(Spans::from(vec![
        Span::styled("Errors: ", Style::default().fg(Color::White)),
        Span::styled(
            error_count.to_string(),
            if error_count > 0 {
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

/// Get color based on usage percentage
fn get_usage_color(usage: f64) -> Color {
    if usage < 60.0 {
        Color::Green
    } else if usage < 80.0 {
        Color::Yellow
    } else {
        Color::Red
    }
} 