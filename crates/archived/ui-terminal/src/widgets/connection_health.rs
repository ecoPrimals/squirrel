use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use chrono::{DateTime, Utc};

use crate::adapter::{ConnectionHealth, ConnectionEvent, ConnectionEventType, ConnectionStatus};
use crate::widgets::Widget;

const MAX_HISTORY_POINTS: usize = 50;

/// Widget for displaying connection health information
pub struct ConnectionHealthWidget<'a> {
    /// Connection health data
    connection_health: Option<&'a ConnectionHealth>,
    /// Connection history data
    connection_history: Option<&'a [ConnectionEvent]>,
    /// Connection history metrics (for sparkline visualization)
    history_metrics: Option<&'a [(DateTime<Utc>, f64)]>,
    /// Widget title
    title: &'a str,
    /// Last update time
    last_update: Option<Instant>,
    /// Health score history for visualization
    health_score_history: VecDeque<u64>,
}

impl<'a> ConnectionHealthWidget<'a> {
    /// Create a new connection health widget
    pub fn new() -> Self {
        Self {
            connection_health: None,
            connection_history: None,
            history_metrics: None,
            title: "Connection Health",
            last_update: None,
            health_score_history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
        }
    }
    
    /// Create a new connection health widget with a custom title
    pub fn with_title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
    
    /// Set connection health data
    pub fn with_health(mut self, health: &'a ConnectionHealth) -> Self {
        self.connection_health = Some(health);
        
        // Update health score history if health data is available
        if let Some(health) = self.connection_health {
            // Convert health score to u64 percentage (0-100)
            let score = (health.health_score * 100.0) as u64;
            
            // Add to history, maintaining max size
            self.health_score_history.push_back(score);
            if self.health_score_history.len() > MAX_HISTORY_POINTS {
                self.health_score_history.pop_front();
            }
        }
        
        self.last_update = Some(Instant::now());
        self
    }
    
    /// Set connection history data
    pub fn with_history(mut self, history: &'a [ConnectionEvent]) -> Self {
        self.connection_history = Some(history);
        self
    }
    
    /// Set connection history metrics
    pub fn with_history_metrics(mut self, metrics: &'a [(DateTime<Utc>, f64)]) -> Self {
        self.history_metrics = Some(metrics);
        self
    }
    
    /// Set connection health data with owned data
    pub fn with_health_owned(mut self, health: ConnectionHealth) -> Self {
        // Convert health score to u64 percentage (0-100)
        let score = (health.health_score * 100.0) as u64;
        
        // Add to history, maintaining max size
        self.health_score_history.push_back(score);
        if self.health_score_history.len() > MAX_HISTORY_POINTS {
            self.health_score_history.pop_front();
        }
        
        self.last_update = Some(Instant::now());
        // Convert to reference for storage
        let static_health = Box::leak(Box::new(health));
        self.connection_health = Some(static_health);
        self
    }
    
    /// Set connection history data with owned data
    pub fn with_history_owned(mut self, history: Vec<ConnectionEvent>) -> Self {
        // Convert to reference for storage
        let static_history = Box::leak(Box::new(history));
        self.connection_history = Some(static_history);
        self
    }
    
    /// Set connection history metrics with owned data
    pub fn with_history_metrics_owned(mut self, metrics: Vec<(DateTime<Utc>, f64)>) -> Self {
        // Convert to reference for storage
        let static_metrics = Box::leak(Box::new(metrics));
        self.history_metrics = Some(static_metrics);
        self
    }
    
    /// Get the color for a health score
    fn health_score_color(score: f64) -> Color {
        if score >= 0.8 {
            Color::Green
        } else if score >= 0.6 {
            Color::Yellow
        } else if score >= 0.4 {
            Color::Rgb(255, 165, 0) // Orange
        } else {
            Color::Red
        }
    }
    
    /// Get color for a connection status
    fn connection_status_color(status: &ConnectionStatus) -> Color {
        match status {
            ConnectionStatus::Connected => Color::Green,
            ConnectionStatus::Connecting => Color::Yellow,
            ConnectionStatus::Disconnected => Color::Red,
            ConnectionStatus::Error(_) => Color::Red,
            ConnectionStatus::Degraded => Color::Yellow,
            ConnectionStatus::Unknown => Color::Gray,
        }
    }
    
    /// Get color for a connection event type
    fn connection_event_color(event_type: &ConnectionEventType) -> Color {
        match event_type {
            ConnectionEventType::Connected => Color::Green,
            ConnectionEventType::ReconnectSuccess => Color::Green,
            ConnectionEventType::Reconnecting => Color::Yellow,
            ConnectionEventType::Disconnected => Color::Red,
            ConnectionEventType::ReconnectFailure => Color::Red,
            ConnectionEventType::Error => Color::Red,
        }
    }
    
    /// Format duration as human-readable string
    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        
        if total_secs < 60 {
            format!("{}s", total_secs)
        } else if total_secs < 3600 {
            format!("{}m {}s", total_secs / 60, total_secs % 60)
        } else {
            format!("{}h {}m", total_secs / 3600, (total_secs % 3600) / 60)
        }
    }
    
    /// Render the connection status section
    fn render_status(&self, f: &mut Frame, area: Rect) {
        if let Some(health) = self.connection_health {
            // Create block
            let block = Block::default()
                .title("Connection Status")
                .borders(Borders::ALL);
            
            // Render block
            f.render_widget(block.clone(), area);
            
            // Calculate inner area
            let inner_area = block.inner(area);
            
            // Create vertical layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1), // Status
                    Constraint::Length(1), // Since
                    Constraint::Length(1), // Health score
                    Constraint::Length(1), // Sparkline
                    Constraint::Length(1), // Latency
                    Constraint::Length(1), // Stability
                ])
                .split(inner_area);
            
            // Status text
            let status_text = match &health.status {
                ConnectionStatus::Connected => "Connected",
                ConnectionStatus::Connecting => "Connecting...",
                ConnectionStatus::Disconnected => "Disconnected",
                ConnectionStatus::Error(msg) => {
                    if msg.len() > 30 {
                        &msg[..30]
                    } else {
                        msg
                    }
                },
                ConnectionStatus::Degraded => "Degraded",
                ConnectionStatus::Unknown => "Unknown",
            };
            
            let status_line = Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    status_text, 
                    Style::default()
                        .fg(Self::connection_status_color(&health.status))
                        .add_modifier(Modifier::BOLD),
                ),
            ]);
            
            // Since/uptime text
            let since_text = if let Some(since) = health.connected_since {
                let now = Utc::now();
                let duration = now.signed_duration_since(since);
                
                // Format duration
                let hours = duration.num_hours();
                let mins = duration.num_minutes() % 60;
                let secs = duration.num_seconds() % 60;
                
                if hours > 0 {
                    format!("{}h {}m {}s", hours, mins, secs)
                } else if mins > 0 {
                    format!("{}m {}s", mins, secs)
                } else {
                    format!("{}s", secs)
                }
            } else {
                match health.status {
                    ConnectionStatus::Connected => "Just connected".to_string(),
                    ConnectionStatus::Connecting => "Connecting...".to_string(),
                    ConnectionStatus::Disconnected => {
                        if let Some(last) = health.last_status_change {
                            let now = Utc::now();
                            let duration = now.signed_duration_since(last);
                            
                            // Format duration for how long disconnected
                            let hours = duration.num_hours();
                            let mins = duration.num_minutes() % 60;
                            let secs = duration.num_seconds() % 60;
                            
                            if hours > 0 {
                                format!("Down for {}h {}m {}s", hours, mins, secs)
                            } else if mins > 0 {
                                format!("Down for {}m {}s", mins, secs)
                            } else {
                                format!("Down for {}s", secs)
                            }
                        } else {
                            "Disconnected".to_string()
                        }
                    }
                    ConnectionStatus::Error(_) => "Error".to_string(),
                    ConnectionStatus::Degraded => "Degraded".to_string(),
                    ConnectionStatus::Unknown => "Unknown Status".to_string(),
                }
            };
            
            let since_line = Line::from(vec![
                Span::raw("Uptime: "),
                Span::styled(
                    since_text, 
                    Style::default().fg(Color::White),
                ),
            ]);
            
            // Health score
            let health_score = health.health_score * 100.0;
            let health_color = Self::health_score_color(health.health_score);
            
            let health_gauge = Gauge::default()
                .block(Block::default().title("Health Score"))
                .gauge_style(Style::default().fg(health_color))
                .use_unicode(true)
                .ratio(health.health_score)
                .label(format!("{:.1}%", health_score));
            
            // Convert health score history to vector for sparkline
            let history_data: Vec<u64> = self.health_score_history.iter().copied().collect();
            
            // Health score history sparkline
            let sparkline = Sparkline::default()
                .block(Block::default().title("Health Score History"))
                .style(Style::default().fg(Color::Blue))
                .data(&history_data);
            
            // Latency
            let latency_text = format!("{:.2} ms", health.latency_ms);
            let latency_line = Line::from(vec![
                Span::raw("Latency: "),
                Span::styled(
                    latency_text, 
                    Style::default().fg(Color::White),
                ),
            ]);
            
            // Stability
            let stability_text = format!("{:.1}%", health.stability);
            let stability_color = if health.stability >= 90.0 {
                Color::Green
            } else if health.stability >= 70.0 {
                Color::Yellow
            } else {
                Color::Red
            };
            
            let stability_line = Line::from(vec![
                Span::raw("Stability: "),
                Span::styled(
                    stability_text, 
                    Style::default().fg(stability_color),
                ),
            ]);
            
            // Render everything
            f.render_widget(Paragraph::new(status_line), chunks[0]);
            f.render_widget(Paragraph::new(since_line), chunks[1]);
            f.render_widget(health_gauge, chunks[2]);
            f.render_widget(sparkline, chunks[3]);
            f.render_widget(Paragraph::new(latency_line), chunks[4]);
            f.render_widget(Paragraph::new(stability_line), chunks[5]);
        } else {
            // No health data available
            let block = Block::default()
                .title("Connection Status")
                .borders(Borders::ALL);
            
            f.render_widget(block.clone(), area);
            
            let inner_area = block.inner(area);
            
            let text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "No connection health data available", 
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .alignment(ratatui::layout::Alignment::Center);
            
            f.render_widget(text, inner_area);
        }
    }
    
    /// Render connection history
    fn render_history(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Connection History")
            .borders(Borders::ALL);
        
        f.render_widget(block.clone(), area);
        
        let inner_area = block.inner(area);
        
        if let Some(history) = self.connection_history {
            if history.is_empty() {
                // No history data
                let text = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "No connection history data available", 
                        Style::default().fg(Color::Gray),
                    ),
                ]))
                .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(text, inner_area);
                return;
            }
            
            // Get the most recent events first (reversed)
            let recent_events: Vec<_> = history.iter().rev().take(10).collect();
            
            // Create lines for each event
            let mut event_lines = Vec::new();
            
            for event in recent_events {
                let event_type_text = match event.event_type {
                    ConnectionEventType::Connected => "Connected",
                    ConnectionEventType::Disconnected => "Disconnected",
                    ConnectionEventType::Reconnecting => "Reconnecting",
                    ConnectionEventType::ReconnectSuccess => "Reconnect Success",
                    ConnectionEventType::ReconnectFailure => "Reconnect Failed",
                    ConnectionEventType::Error => "Error",
                };
                
                let time_text = event.timestamp.format("%H:%M:%S").to_string();
                
                let mut spans = vec![
                    Span::styled(
                        format!("[{}] ", time_text), 
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        event_type_text, 
                        Style::default()
                            .fg(Self::connection_event_color(&event.event_type))
                            .add_modifier(Modifier::BOLD),
                    ),
                ];
                
                // Add details if available
                if !event.details.is_empty() {
                    spans.push(Span::raw(" - "));
                    spans.push(Span::raw(event.details.clone()));
                }
                
                event_lines.push(Line::from(spans));
            }
            
            // Render as paragraph
            let events_paragraph = Paragraph::new(event_lines);
            
            f.render_widget(events_paragraph, inner_area);
        } else {
            // No history data
            let text = Paragraph::new(Line::from(vec![
                Span::styled(
                    "No connection history data available", 
                    Style::default().fg(Color::Gray),
                ),
            ]))
            .alignment(ratatui::layout::Alignment::Center);
            
            f.render_widget(text, inner_area);
        }
    }
}

impl<'a> Widget for ConnectionHealthWidget<'a> {
    fn render(&self, f: &mut Frame, area: Rect) {
        // Create block
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Create horizontal layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(inner_area);
        
        // Render the two sections
        self.render_status(f, chunks[0]);
        self.render_history(f, chunks[1]);
    }
} 