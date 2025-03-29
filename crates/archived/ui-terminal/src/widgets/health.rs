use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem, Widget},
    Frame,
    buffer::Buffer,
};

use chrono::Utc;
use dashboard_core::health::{HealthCheck as DashboardHealthCheck, HealthStatus as DashboardHealthStatus};
use dashboard_core::data::HealthStatus;
use crate::widgets::health::connection_health::ConnectionHealthWidget;
use crate::adapter::ConnectionHealth;
use crate::health::HealthStatus as WidgetHealthStatus;
use crate::ui::UiApp;
use std::collections::VecDeque;
use std::time::Instant;
use crate::theme::Theme;
use crate::adapter::{ConnectionEvent, ConnectionEventType};

/// Health status enum
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    /// Healthy status
    Healthy,
    /// Warning status
    Warning,
    /// Critical status
    Critical,
    /// Unknown status
    Unknown,
}

impl HealthStatus {
    /// Get color for the health status
    pub fn color(&self) -> Color {
        match self {
            HealthStatus::Healthy => Color::Green,
            HealthStatus::Warning => Color::Yellow,
            HealthStatus::Critical => Color::Red,
            HealthStatus::Unknown => Color::Gray,
        }
    }
    
    /// Get label for the health status
    pub fn label(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "Healthy",
            HealthStatus::Warning => "Warning",
            HealthStatus::Critical => "Critical",
            HealthStatus::Unknown => "Unknown",
        }
    }
    
    /// Get the display string for a health status
    pub fn as_str(&self) -> &str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Unknown => "Unknown",
        }
    }
    
    /// Get status from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "healthy" | "ok" | "good" => HealthStatus::Healthy,
            "warning" | "warn" => HealthStatus::Warning,
            "critical" | "error" | "bad" => HealthStatus::Critical,
            _ => HealthStatus::Unknown,
        }
    }
    
    /// Convert from ui::TerminalDashboardHealthStatus
    pub fn from_dashboard_status(status: crate::ui::TerminalDashboardHealthStatus) -> Self {
        match status {
            crate::ui::TerminalDashboardHealthStatus::Healthy => Self::Healthy,
            crate::ui::TerminalDashboardHealthStatus::Warning => Self::Warning,
            crate::ui::TerminalDashboardHealthStatus::Critical => Self::Critical,
            crate::ui::TerminalDashboardHealthStatus::Unknown => Self::Unknown,
        }
    }

    fn get_style(&self) -> Style {
        match self {
            HealthStatus::Healthy => Theme::default().success_style,
            HealthStatus::Warning => Theme::default().warning_style,
            HealthStatus::Critical => Theme::default().error_style,
            HealthStatus::Unknown => Theme::default().secondary_style,
        }
    }
}

/// Health check item
#[derive(Debug, Clone)]
pub struct HealthCheck {
    /// Name of the service or component
    pub name: String,
    /// Status of the health check
    pub status: HealthStatus,
    /// Additional details
    pub details: Option<String>,
    /// Last check time (can be used to show stale checks)
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(name: impl Into<String>, status: HealthStatus) -> Self {
        Self {
            name: name.into(),
            status,
            details: None,
            last_check: Some(chrono::Utc::now()),
        }
    }
    
    /// Add details to the health check
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
    
    /// Set percentage value (stored in details)
    pub fn with_percentage(mut self, percentage: f64) -> Self {
        self.details = Some(format!("{}%", percentage.round() as i32));
        self
    }
    
    /// Set the health status
    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set message in details field
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.details = Some(message.into());
        self
    }
    
    /// Set last check time
    pub fn with_last_check(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
        self.last_check = Some(time);
        self
    }
    
    /// Create from dashboard health check
    pub fn from_dashboard(check: &DashboardHealthCheck) -> Self {
        Self {
            name: check.name.clone(),
            status: HealthStatus::from_dashboard_status(check.status),
            details: Some(check.details.clone()),
            last_check: Some(Utc::now()),
        }
    }
    
    /// Format as styled lines
    pub fn as_lines(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        
        // Name and status
        lines.push(Line::from(vec![
            Span::raw(&self.name),
            Span::raw(": "),
            Span::styled(
                self.status.label(),
                Style::default()
                    .fg(self.status.color())
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        
        // Details if present
        if let Some(details) = &self.details {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(details),
            ]));
        }
        
        // Last check time if present
        if let Some(time) = self.last_check {
            let now = chrono::Utc::now();
            let diff = now.signed_duration_since(time);
            
            let time_str = if diff.num_seconds() < 60 {
                format!("{} seconds ago", diff.num_seconds())
            } else if diff.num_minutes() < 60 {
                format!("{} minutes ago", diff.num_minutes())
            } else {
                format!("{} hours ago", diff.num_hours())
            };
            
            lines.push(Line::from(vec![
                Span::raw("  Last check: "),
                Span::raw(time_str),
            ]));
        }
        
        lines
    }
}

/// Widget for displaying health checks
pub struct HealthWidget<'a> {
    /// Health checks to display
    health_checks: &'a [HealthCheck],
    /// Widget title
    title: &'a str,
}

impl<'a> HealthWidget<'a> {
    /// Create a new health widget
    pub fn new(health_checks: &'a [HealthCheck], title: &'a str) -> Self {
        Self {
            health_checks,
            title,
        }
    }
    
    /// Create health checks from dashboard health checks
    pub fn from_dashboard(health_checks: &'a [DashboardHealthCheck], _title: &'a str) -> Vec<HealthCheck> {
        health_checks
            .iter()
            .map(HealthCheck::from_dashboard)
            .collect()
    }

    /// Calculate overall health status
    pub fn overall_health(&self) -> HealthStatus {
        let mut worst_status = HealthStatus::Healthy;
        for check in self.health_checks {
            match check.status {
                HealthStatus::Critical => return HealthStatus::Critical, // Critical is immediate overall failure
                HealthStatus::Warning => worst_status = HealthStatus::Warning,
                HealthStatus::Unknown if worst_status == HealthStatus::Healthy => worst_status = HealthStatus::Unknown,
                _ => {}
            }
        }
        worst_status
    }
}

impl<'a> Widget for HealthWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        
        let inner_area = block.inner(area);
        block.render(area, buf);

        // Create layout for health checks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                self.health_checks
                    .iter()
                    .map(|_| Constraint::Ratio(1, self.health_checks.len() as u32))
                    .collect::<Vec<_>>()
            )
            .split(inner_area);

        // Render each health check
        for (i, check) in self.health_checks.iter().enumerate() {
            let style = match check.status {
                HealthStatus::Healthy => Style::default().fg(Color::Green),
                HealthStatus::Warning => Style::default().fg(Color::Yellow),
                HealthStatus::Critical => Style::default().fg(Color::Red),
                HealthStatus::Unknown => Style::default().fg(Color::Gray),
            };

            let percentage = format!("{}%", check.percentage.unwrap_or(0.0));
            let message = &check.details.clone().unwrap_or_else(|| "No details".to_string());

            let check_block = Block::default()
                .title(check.name)
                .borders(Borders::ALL)
                .style(style);

            let inner = check_block.inner(chunks[i]);
            check_block.render(chunks[i], buf);

            let text = vec![
                Line::from(percentage),
                Line::from(message),
            ];

            Paragraph::new(text)
                .alignment(Alignment::Center)
                .render(inner, buf);
        }
    }
}

/// Connection Health Widget
pub struct ConnectionHealthWidget<'a> {
    connection_health: Option<&'a ConnectionHealth>,
    connection_history: Option<&'a [ConnectionEvent]>,
    history_metrics: Option<&'a [(chrono::DateTime<Utc>, f64)]>,
    title: &'a str,
    last_update: Option<Instant>,
    health_score_history: VecDeque<f64>,
}

impl<'a> ConnectionHealthWidget<'a> {
    pub fn new(
        connection_health: Option<&'a ConnectionHealth>,
        connection_history: Option<&'a [ConnectionEvent]>,
        history_metrics: Option<&'a [(chrono::DateTime<Utc>, f64)]>,
        title: &'a str,
    ) -> Self {
        Self {
            connection_health,
            connection_history,
            history_metrics,
            title,
            last_update: Some(Instant::now()),
            health_score_history: VecDeque::with_capacity(100),
        }
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().title("Connection Status").borders(Borders::ALL);
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        if let Some(health) = self.connection_health {
            let health_score = calculate_health_score(health);
            let status_text = get_status_text(health_score);
            let status_style = get_status_style(health_score);

            let latency = format!("{:.2} ms", health.latency_ms);
            let packet_loss = format!("{:.1}%", health.packet_loss);
            let stability = format!("{:.1}%", health.stability);

            let last_checked_str = health.last_checked.format("%H:%M:%S UTC").to_string();

            let details_lines = vec![
                Line::from(vec![Span::styled("Status: ", Theme::default().primary_style), Span::styled(status_text, status_style)]),                
                Line::from(format!("Latency: {}", latency)),
                Line::from(format!("Packet Loss: {}", packet_loss)),
                Line::from(format!("Stability: {}", stability)),
                Line::from(format!("Last Checked: {}", last_checked_str)),
            ];

            let paragraph = Paragraph::new(details_lines).block(Block::default());
            f.render_widget(paragraph, inner_area);

            // TODO: Add Gauge and Sparkline rendering here if needed

        } else {
            let paragraph = Paragraph::new("Connection health data unavailable.")
                .style(Theme::default().secondary_style);
            f.render_widget(paragraph, inner_area);
        }
    }

    fn render_history(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().title("Connection History").borders(Borders::ALL);
        let inner_area = block.inner(area);
        f.render_widget(block, area);

        if let Some(history) = self.connection_history {
            let items: Vec<ListItem> = history
                .iter()
                .rev()
                .take(50)
                .map(|event| {
                    let timestamp = event.timestamp.format("%H:%M:%S").to_string();
                    let event_type_str = format!("{:?}", event.event_type);
                    let style = match event.event_type {
                        ConnectionEventType::Connected | ConnectionEventType::Reconnected => Theme::default().success_style,
                        ConnectionEventType::Disconnected | ConnectionEventType::ReconnectFailed => Theme::default().error_style,
                        ConnectionEventType::Connecting | ConnectionEventType::ReconnectAttempt => Theme::default().warning_style,
                        _ => Theme::default().secondary_style,
                    };
                    ListItem::new(Line::from(format!("[{}] {}: {}", timestamp, event_type_str, event.details)))
                        .style(style)
                })
                .collect();

            let list = List::new(items).block(Block::default());
            f.render_widget(list, inner_area);
        } else {
            let paragraph = Paragraph::new("Connection history unavailable.")
                .style(Theme::default().secondary_style);
            f.render_widget(paragraph, inner_area);
        }
    }
}

fn calculate_health_score(health: &ConnectionHealth) -> f64 {
    let latency_score = (100.0 - health.latency_ms.min(100.0)).max(0.0);
    let loss_score = 100.0 - health.packet_loss;
    let stability_score = health.stability;
    
    (latency_score * 0.3 + loss_score * 0.4 + stability_score * 0.3).max(0.0).min(100.0)
}

fn get_status_text(score: f64) -> &'static str {
    if score >= 80.0 {
        "Healthy"
    } else if score >= 50.0 {
        "Warning"
    } else {
        "Critical"
    }
}

fn get_status_style(score: f64) -> Style {
    if score >= 80.0 {
        Theme::default().success_style
    } else if score >= 50.0 {
        Theme::default().warning_style
    } else {
        Theme::default().error_style
    }
}

impl<'a> Widget for ConnectionHealthWidget<'a> {
    fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        let inner_area = block.inner(area);
        f.render_widget(block.clone(), area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(inner_area);

        self.render_status(f, chunks[0]);
        self.render_history(f, chunks[1]);
    }
}

/// Helper function to convert error log Vec<String> to Lines for Paragraph
fn errors_to_lines(errors: Vec<String>) -> Vec<Line<'static>> {
    errors.into_iter().map(|message| Line::from(message)).collect()
} 