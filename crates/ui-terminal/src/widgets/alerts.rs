use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line as Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget},
};

use dashboard_core::data::{Alert, AlertSeverity};
use crate::util;

/// Widget for displaying system alerts
pub struct AlertsWidget<'a> {
    /// Alerts to display
    alerts: &'a [Alert],
    
    /// Widget title
    title: &'a str,
    
    /// Whether to show detailed information
    detailed: bool,
    
    /// Currently selected alert index
    selected_index: Option<usize>,
}

impl<'a> AlertsWidget<'a> {
    /// Create a new alerts widget
    pub fn new(alerts: &'a [Alert], title: &'a str) -> Self {
        Self {
            alerts,
            title,
            detailed: false,
            selected_index: None,
        }
    }
    
    /// Create a new alerts widget with a selected item
    pub fn new_with_selection(alerts: &'a [Alert], title: &'a str, selected: usize) -> Self {
        Self {
            alerts,
            title,
            detailed: true,
            selected_index: Some(selected),
        }
    }
    
    /// Set detailed mode
    pub fn detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }
    
    /// Set selected alert index
    pub fn select(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }
}

impl<'a> Widget for AlertsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        block.clone().render(area, buf);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Handle empty alerts
        if self.alerts.is_empty() {
            let text = vec![Spans::from(vec![
                Span::styled("No active alerts", Style::default().fg(Color::Green))
            ])];
            
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(Color::White));
            
            paragraph.render(inner_area, buf);
            return;
        }
        
        // Create layout for list and details
        let chunks = if self.detailed && self.selected_index.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(inner_area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(inner_area)
        };
        
        // Render alerts list
        render_alerts_list(self.alerts, chunks[0], buf, self.selected_index);
        
        // Render details if selected
        if self.detailed && self.selected_index.is_some() {
            if let Some(index) = self.selected_index {
                if index < self.alerts.len() {
                    render_alert_details(&self.alerts[index], chunks[1], buf);
                }
            }
        }
    }
}

/// Render the list of alerts
fn render_alerts_list(alerts: &[Alert], area: Rect, buf: &mut Buffer, selected: Option<usize>) {
    // Create list items
    let items: Vec<ListItem> = alerts
        .iter()
        .enumerate()
        .map(|(i, alert)| {
            let level_style = get_alert_level_style(alert.severity);
            let level_str = format!("[{}]", get_alert_level_name(alert.severity));
            
            let spans = vec![
                Span::styled(level_str, level_style),
                Span::raw(" "),
                Span::styled(&alert.title, Style::default()),
                Span::raw(" - "),
                Span::styled(
                    util::format_timestamp(alert.triggered_at),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            
            ListItem::new(Spans::from(spans))
                .style(
                    if Some(i) == selected {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    }
                )
        })
        .collect();
    
    // Create and render list
    let list = List::new(items)
        .style(Style::default().fg(Color::White));
    
    list.render(area, buf);
}

/// Render details of a selected alert
fn render_alert_details(alert: &Alert, area: Rect, buf: &mut Buffer) {
    // Create block
    let block = Block::default()
        .borders(Borders::TOP)
        .title("Details");
    
    // Render block
    block.clone().render(area, buf);
    
    // Get inner area
    let inner_area = block.inner(area);
    
    // Create content
    let level_style = get_alert_level_style(alert.severity);
    let level_str = format!("[{}]", get_alert_level_name(alert.severity));
    
    let content = vec![
        Spans::from(vec![
            Span::styled("Level: ", Style::default().fg(Color::White)),
            Span::styled(level_str, level_style),
        ]),
        Spans::from(vec![
            Span::styled("Time: ", Style::default().fg(Color::White)),
            Span::styled(
                util::format_timestamp(alert.triggered_at),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Spans::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::White)),
            Span::styled(&alert.id, Style::default().fg(Color::Cyan)),
        ]),
        Spans::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled(&alert.title, Style::default().fg(Color::White)),
        ]),
        Spans::from(""),
        Spans::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::White)),
        ]),
        Spans::from(vec![
            Span::styled(&alert.description, Style::default().fg(Color::White)),
        ]),
    ];
    
    // Render paragraph
    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    paragraph.render(inner_area, buf);
}

/// Get the style for an alert level
fn get_alert_level_style(level: AlertSeverity) -> Style {
    match level {
        AlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        AlertSeverity::High => Style::default().fg(Color::Red),
        AlertSeverity::Medium => Style::default().fg(Color::Yellow),
        AlertSeverity::Low => Style::default().fg(Color::Blue),
        AlertSeverity::Info => Style::default().fg(Color::Green),
    }
}

/// Get the name of an alert level
fn get_alert_level_name(level: AlertSeverity) -> &'static str {
    match level {
        AlertSeverity::Critical => "CRITICAL",
        AlertSeverity::High => "HIGH",
        AlertSeverity::Medium => "MEDIUM",
        AlertSeverity::Low => "LOW",
        AlertSeverity::Info => "INFO",
    }
} 