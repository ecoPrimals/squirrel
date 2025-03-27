use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line as Spans},
    widgets::{Block, Borders, Paragraph, Table, Row, Cell, Widget},
};

use chrono::{DateTime, Utc};
use crate::util;

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has warnings
    Warning, 
    /// System is unhealthy
    Unhealthy,
}

/// Health check information
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Health check name
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Last check time
    pub last_checked: DateTime<Utc>,
    /// Message with details
    pub message: String,
}

/// Widget for displaying system health checks
pub struct HealthWidget<'a> {
    /// Health checks to display
    health_checks: &'a [HealthCheck],
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed information
    detailed: bool,
}

impl<'a> HealthWidget<'a> {
    /// Create a new health widget
    pub fn new(health_checks: &'a [HealthCheck], title: &'a str) -> Self {
        Self {
            health_checks,
            title,
            detailed: false,
        }
    }
    
    /// Create a new detailed health widget
    pub fn new_detailed(health_checks: &'a [HealthCheck], title: &'a str) -> Self {
        Self {
            health_checks,
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

impl<'a> Widget for HealthWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        block.clone().render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Handle empty health checks
        if self.health_checks.is_empty() {
            let text = vec![Spans::from(vec![
                Span::styled("No health checks available", Style::default().fg(Color::Yellow))
            ])];
            
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(Color::White));
            
            paragraph.render(inner_area, buf);
            return;
        }
        
        // Calculate overall health
        let (healthy, unhealthy) = count_statuses(self.health_checks);
        let overall_status = if unhealthy == 0 {
            HealthStatus::Healthy
        } else if unhealthy > healthy {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Warning
        };
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(inner_area);
        
        // Render overall status
        let _overall_text = format!("Overall: {}", get_status_name(overall_status));
        let overall_spans = Spans::from(vec![
            Span::raw("Overall: "),
            Span::styled(
                get_status_name(overall_status),
                get_status_style(overall_status),
            ),
        ]);
        
        let overall_paragraph = Paragraph::new(overall_spans)
            .style(Style::default().fg(Color::White));
        
        overall_paragraph.render(chunks[0], buf);
        
        // Render summary
        let summary_spans = Spans::from(vec![
            Span::styled(
                format!("{} healthy", healthy),
                Style::default().fg(Color::Green),
            ),
            Span::raw(", "),
            Span::styled(
                format!("{} unhealthy", unhealthy),
                if unhealthy > 0 {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]);
        
        let summary_paragraph = Paragraph::new(summary_spans)
            .style(Style::default().fg(Color::White));
        
        summary_paragraph.render(chunks[1], buf);
        
        // Render health checks as table or simple list
        if self.detailed {
            render_detailed_health_checks(self.health_checks, chunks[2], buf);
        } else {
            render_simple_health_checks(self.health_checks, chunks[2], buf);
        }
    }
}

/// Render health checks as a simple list
fn render_simple_health_checks(health_checks: &[HealthCheck], area: Rect, buf: &mut Buffer) {
    // Create content
    let mut content = Vec::new();
    
    for check in health_checks {
        let status_style = get_status_style(check.status);
        let status_str = get_status_symbol(check.status);
        
        content.push(Spans::from(vec![
            Span::styled(status_str, status_style),
            Span::raw(" "),
            Span::styled(&check.name, Style::default()),
        ]));
    }
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(area, buf);
}

/// Render health checks as a detailed table
fn render_detailed_health_checks(health_checks: &[HealthCheck], area: Rect, buf: &mut Buffer) {
    // Create table rows
    let rows: Vec<Row> = health_checks
        .iter()
        .map(|check| {
            let status_style = get_status_style(check.status);
            let status_str = get_status_name(check.status);
            
            let cells = vec![
                Cell::from(format!("{}", check.name)).style(Style::default()),
                Cell::from(format!("{}", status_str)).style(status_style),
                Cell::from(format!("{}", util::format_timestamp(check.last_checked))).style(Style::default().fg(Color::DarkGray)),
                Cell::from(format!("{}", check.message)).style(Style::default()),
            ];
            
            Row::new(cells)
        })
        .collect();
    
    // Create table
    let table = Table::new(rows)
        .header(
            Row::new(vec![
                Cell::from("Name").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Status").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Last Checked").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Cell::from("Message").style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])
        )
        .widths(&[
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .column_spacing(1);
    
    // Render table
    table.render(area, buf);
}

/// Get the style for a health status
fn get_status_style(status: HealthStatus) -> Style {
    match status {
        HealthStatus::Healthy => Style::default().fg(Color::Green),
        HealthStatus::Warning => Style::default().fg(Color::Yellow),
        HealthStatus::Unhealthy => Style::default().fg(Color::Red),
    }
}

/// Get the name of a health status
fn get_status_name(status: HealthStatus) -> &'static str {
    match status {
        HealthStatus::Healthy => "Healthy",
        HealthStatus::Warning => "Warning",
        HealthStatus::Unhealthy => "Unhealthy",
    }
}

/// Get the symbol for a health status
fn get_status_symbol(status: HealthStatus) -> &'static str {
    match status {
        HealthStatus::Healthy => "✓",
        HealthStatus::Warning => "!",
        HealthStatus::Unhealthy => "✗",
    }
}

/// Count the number of healthy and unhealthy checks
fn count_statuses(health_checks: &[HealthCheck]) -> (usize, usize) {
    let mut healthy = 0;
    let mut unhealthy = 0;
    
    for check in health_checks {
        match check.status {
            HealthStatus::Healthy => healthy += 1,
            HealthStatus::Warning => {
                // Count warnings as half healthy, half unhealthy
                healthy += 1;
                unhealthy += 1;
            },
            HealthStatus::Unhealthy => unhealthy += 1,
        }
    }
    
    (healthy, unhealthy)
} 