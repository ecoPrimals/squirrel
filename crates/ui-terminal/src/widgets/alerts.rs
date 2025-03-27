use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};

use dashboard_core::data::Alert;
use chrono::DateTime;
use chrono::Utc;

/// Widget for displaying alerts
pub struct AlertsWidget<'a> {
    alerts: &'a [Alert],
    title: &'a str,
    selected_index: usize,
    show_details: bool,
    filter: AlertFilter,
}

/// Alert filter type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertFilter {
    All,
    Active,
    Acknowledged,
    Critical,
    Warning,
    Info,
}

impl AlertFilter {
    /// Get filter display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Acknowledged => "Acknowledged",
            Self::Critical => "Critical",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }
    
    /// Get all available filters
    pub fn all() -> Vec<Self> {
        vec![
            Self::All,
            Self::Active,
            Self::Acknowledged,
            Self::Critical,
            Self::Warning,
            Self::Info,
        ]
    }
    
    /// Check if an alert matches this filter
    pub fn matches(&self, alert: &Alert) -> bool {
        match self {
            Self::All => true,
            Self::Active => !alert.acknowledged,
            Self::Acknowledged => alert.acknowledged,
            Self::Critical => alert.severity == "critical",
            Self::Warning => alert.severity == "warning",
            Self::Info => alert.severity == "info",
        }
    }
}

impl<'a> AlertsWidget<'a> {
    /// Create a new alerts widget
    pub fn new(alerts: &'a [Alert], title: &'a str) -> Self {
        Self {
            alerts,
            title,
            selected_index: 0,
            show_details: false,
            filter: AlertFilter::All,
        }
    }
    
    /// Set the filter
    pub fn filter(mut self, filter: AlertFilter) -> Self {
        self.filter = filter;
        self
    }
    
    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }
    
    /// Select the next alert
    pub fn next(&mut self) {
        let filtered_alerts: Vec<&Alert> = self.alerts.iter()
            .filter(|a| self.filter.matches(a))
            .collect();
            
        if !filtered_alerts.is_empty() {
            self.selected_index = (self.selected_index + 1) % filtered_alerts.len();
        }
    }
    
    /// Select the previous alert
    pub fn previous(&mut self) {
        let filtered_alerts: Vec<&Alert> = self.alerts.iter()
            .filter(|a| self.filter.matches(a))
            .collect();
            
        if !filtered_alerts.is_empty() {
            self.selected_index = if self.selected_index > 0 {
                self.selected_index - 1
            } else {
                filtered_alerts.len() - 1
            };
        }
    }
    
    /// Set the filter
    pub fn set_filter(&mut self, filter: AlertFilter) {
        self.filter = filter;
        // Reset selection when filter changes
        self.selected_index = 0;
    }
    
    /// Get the currently selected alert
    pub fn selected_alert(&self) -> Option<&Alert> {
        let filtered_alerts: Vec<&Alert> = self.alerts.iter()
            .filter(|a| self.filter.matches(a))
            .collect();
            
        if filtered_alerts.is_empty() {
            None
        } else {
            Some(filtered_alerts[self.selected_index.min(filtered_alerts.len() - 1)])
        }
    }
    
    /// Cycle to the next filter
    pub fn next_filter(&mut self) {
        let all_filters = AlertFilter::all();
        let current_idx = all_filters.iter().position(|f| *f == self.filter).unwrap_or(0);
        let next_idx = (current_idx + 1) % all_filters.len();
        self.filter = all_filters[next_idx];
        // Reset selection when filter changes
        self.selected_index = 0;
    }
    
    /// Render the widget
    pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Draw block around the whole widget
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        f.render_widget(block, area);
        
        // Filter tabs
        let filter_names: Vec<String> = AlertFilter::all().iter()
            .map(|f| f.display_name().to_string())
            .collect();
        
        let current_idx = AlertFilter::all().iter()
            .position(|f| *f == self.filter)
            .unwrap_or(0);
        
        let tabs = Tabs::new(filter_names.iter().map(|name| {
                Spans::from(vec![Span::styled(name, Style::default())])
            }).collect())
            .select(current_idx)
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        
        // Inner area for content
        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Filter tabs
                Constraint::Min(0),    // Alert content
            ])
            .split(area);
        
        f.render_widget(tabs, inner_area[0]);
        
        // Filter alerts
        let filtered_alerts: Vec<&Alert> = self.alerts.iter()
            .filter(|a| self.filter.matches(a))
            .collect();
        
        if filtered_alerts.is_empty() {
            // No alerts to display
            let message = Paragraph::new("No alerts match the current filter")
                .style(Style::default().fg(Color::Gray));
            f.render_widget(message, inner_area[1]);
            return;
        }
        
        // Render alert list or detail view
        if self.show_details && !filtered_alerts.is_empty() {
            // Show details for selected alert
            self.render_alert_details(f, inner_area[1], filtered_alerts);
        } else {
            // Show alert list
            self.render_alert_list(f, inner_area[1], filtered_alerts);
        }
    }
    
    // Render alert list
    fn render_alert_list<B: Backend>(&self, f: &mut Frame<B>, area: Rect, alerts: Vec<&Alert>) {
        let header = Row::new(vec![
            Cell::from("Severity").style(Style::default().fg(Color::Yellow)),
            Cell::from("Time").style(Style::default().fg(Color::Yellow)),
            Cell::from("Message").style(Style::default().fg(Color::Yellow)),
            Cell::from("Status").style(Style::default().fg(Color::Yellow)),
        ]);
        
        let rows: Vec<Row> = alerts.iter().enumerate().map(|(i, alert)| {
            let severity_color = match alert.severity.as_str() {
                "critical" => Color::Red,
                "warning" => Color::Yellow,
                "info" => Color::Blue,
                _ => Color::White,
            };
            
            let status_text = if alert.acknowledged {
                "Acknowledged"
            } else {
                "Active"
            };
            
            let status_color = if alert.acknowledged {
                Color::Green
            } else {
                Color::Red
            };
            
            let style = if i == self.selected_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };
            
            Row::new(vec![
                Cell::from(alert.severity.as_str()).style(Style::default().fg(severity_color)),
                Cell::from(format_time(&alert.timestamp)),
                Cell::from(alert.message.as_str()),
                Cell::from(status_text).style(Style::default().fg(status_color)),
            ]).style(style)
        }).collect();
        
        let table = Table::new(rows)
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Alerts"))
            .widths(&[
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(15),
            ])
            .column_spacing(1)
            .highlight_style(Style::default().bg(Color::DarkGray));
        
        f.render_widget(table, area);
    }
    
    // Render alert details
    fn render_alert_details<B: Backend>(&self, f: &mut Frame<B>, area: Rect, alerts: Vec<&Alert>) {
        let alert = alerts[self.selected_index.min(alerts.len() - 1)];
        
        // Split area for different detail sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Summary
                Constraint::Length(3),  // Time info
                Constraint::Length(3),  // Status
                Constraint::Min(6),     // Details
                Constraint::Length(3),  // Actions
            ])
            .split(area);
        
        // Summary
        let severity_color = match alert.severity.as_str() {
            "critical" => Color::Red,
            "warning" => Color::Yellow,
            "info" => Color::Blue,
            _ => Color::White,
        };
        
        let summary = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("Alert: ", Style::default().fg(Color::Yellow)),
                Span::styled(&alert.message, Style::default().fg(Color::White)),
            ]),
            Spans::from(vec![
                Span::styled("Severity: ", Style::default().fg(Color::Yellow)),
                Span::styled(&alert.severity, Style::default().fg(severity_color)),
            ]),
        ]).block(Block::default().borders(Borders::ALL).title("Summary"));
        
        // Time info
        let time_info = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("Created: ", Style::default().fg(Color::Yellow)),
                Span::styled(format_time(&alert.timestamp), Style::default().fg(Color::White)),
            ])
        ]).block(Block::default().borders(Borders::ALL).title("Time"));
        
        // Status
        let status_color = if alert.acknowledged {
            Color::Green
        } else {
            Color::Red
        };
        
        let status_text = if alert.acknowledged {
            format!("Acknowledged by {} at {}", 
                alert.acknowledged_by.as_deref().unwrap_or("Unknown"),
                format_time(&alert.acknowledged_at.unwrap_or_else(Utc::now)))
        } else {
            "Active - Not Acknowledged".to_string()
        };
        
        let status = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::styled(if alert.acknowledged { "Acknowledged" } else { "Active" }, 
                             Style::default().fg(status_color)),
            ]),
            Spans::from(status_text),
        ]).block(Block::default().borders(Borders::ALL).title("Status"));
        
        // Details - could be long text
        let details_text = alert.details.as_deref().unwrap_or("No additional details available.");
        let details = Paragraph::new(details_text)
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        // Actions
        let actions = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("A", Style::default().fg(Color::Yellow)),
                Span::styled(" to acknowledge, ", Style::default().fg(Color::Gray)),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::styled(" to return to list", Style::default().fg(Color::Gray)),
            ]),
        ]).block(Block::default().borders(Borders::ALL).title("Actions"));
        
        // Render all sections
        f.render_widget(summary, chunks[0]);
        f.render_widget(time_info, chunks[1]);
        f.render_widget(status, chunks[2]);
        f.render_widget(details, chunks[3]);
        f.render_widget(actions, chunks[4]);
    }
}

// Format timestamp to readable string
fn format_time(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
} 