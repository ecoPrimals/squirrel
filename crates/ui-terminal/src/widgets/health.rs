use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use chrono::Utc;
use dashboard_core::health::{HealthCheck as DashboardHealthCheck, HealthStatus as DashboardHealthStatus};

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    
    /// Get status from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "healthy" => HealthStatus::Healthy,
            "warning" => HealthStatus::Warning,
            "critical" => HealthStatus::Critical,
            _ => HealthStatus::Unknown,
        }
    }
    
    /// Convert from dashboard health status
    pub fn from_dashboard_status(status: DashboardHealthStatus) -> Self {
        match status {
            DashboardHealthStatus::Ok => HealthStatus::Healthy,
            DashboardHealthStatus::Warning => HealthStatus::Warning,
            DashboardHealthStatus::Critical => HealthStatus::Critical,
            DashboardHealthStatus::Unknown => HealthStatus::Unknown,
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
    
    /// Create a new health widget from dashboard health checks
    pub fn from_dashboard(health_checks: &'a [DashboardHealthCheck], _title: &'a str) -> Vec<HealthCheck> {
        health_checks.iter().map(HealthCheck::from_dashboard).collect()
    }
    
    /// Get overall health status based on individual checks
    pub fn overall_health(&self) -> HealthStatus {
        if self.health_checks.is_empty() {
            return HealthStatus::Unknown;
        }
        
        let has_critical = self.health_checks.iter().any(|check| check.status == HealthStatus::Critical);
        if has_critical {
            return HealthStatus::Critical;
        }
        
        let has_warning = self.health_checks.iter().any(|check| check.status == HealthStatus::Warning);
        if has_warning {
            return HealthStatus::Warning;
        }
        
        if self.health_checks.iter().all(|check| check.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
    
    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create base block
        let overall_status = self.overall_health();
        
        // Create block with colored title based on overall status
        let title = format!("{} - {}", self.title, overall_status.label());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                title,
                Style::default()
                    .fg(overall_status.color())
                    .add_modifier(Modifier::BOLD)
            ));
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // If there are no health checks, show a message
        if self.health_checks.is_empty() {
            let message = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        "No health checks available",
                        Style::default().fg(Color::Gray),
                    ),
                ]),
            ]);
            f.render_widget(message, inner_area);
            return;
        }
        
        // Create layout with two columns for health checks
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(inner_area);
        
        // Split health checks between columns
        let mid = (self.health_checks.len() + 1) / 2;
        let (left_checks, right_checks) = self.health_checks.split_at(mid);
        
        // Render left column
        let left_lines: Vec<Line> = left_checks
            .iter()
            .flat_map(|check| {
                let mut lines = check.as_lines();
                lines.push(Line::from(""));
                lines
            })
            .collect();
        
        let left_paragraph = Paragraph::new(left_lines);
        f.render_widget(left_paragraph, chunks[0]);
        
        // Render right column
        let right_lines: Vec<Line> = right_checks
            .iter()
            .flat_map(|check| {
                let mut lines = check.as_lines();
                lines.push(Line::from(""));
                lines
            })
            .collect();
        
        let right_paragraph = Paragraph::new(right_lines);
        f.render_widget(right_paragraph, chunks[1]);
    }
} 