use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use dashboard_core::data::{Alert as DashboardAlert, AlertSeverity as DashboardAlertSeverity};
use crate::alert::{Alert, AlertSeverity, AlertManager};
use std::sync::Arc;
use chrono::Utc;

/// Widget for displaying system alerts
pub struct AlertsWidget<'a> {
    /// Alert manager containing the alerts to display
    alert_manager: Option<Arc<AlertManager>>,
    /// Fallback alerts when alert_manager is not available
    fallback_alerts: Option<&'a Vec<Alert>>,
    /// Dashboard alerts for direct rendering
    dashboard_alerts: Option<&'a Vec<DashboardAlert>>,
    /// Widget title
    title: &'a str,
    /// Selected alert index
    selected: Option<usize>,
    /// Whether to show acknowledged alerts
    show_acknowledged: bool,
}

impl<'a> AlertsWidget<'a> {
    /// Create a new alerts widget with an alert manager
    pub fn new(alert_manager: Option<Arc<AlertManager>>, title: &'a str) -> Self {
        Self {
            alert_manager,
            fallback_alerts: None,
            dashboard_alerts: None,
            title,
            selected: None,
            show_acknowledged: false,
        }
    }
    
    /// Create a new alerts widget with fallback alerts
    pub fn with_fallback(fallback_alerts: &'a Vec<Alert>, title: &'a str) -> Self {
        Self {
            alert_manager: None,
            fallback_alerts: Some(fallback_alerts),
            dashboard_alerts: None,
            title,
            selected: None,
            show_acknowledged: false,
        }
    }
    
    /// Create a new alerts widget with dashboard alerts
    pub fn from_dashboard(dashboard_alerts: &'a Vec<DashboardAlert>, title: &'a str) -> Self {
        Self {
            alert_manager: None,
            fallback_alerts: None,
            dashboard_alerts: Some(dashboard_alerts),
            title,
            selected: None,
            show_acknowledged: false,
        }
    }
    
    /// Set the selected alert index
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }
    
    /// Set whether to show acknowledged alerts
    pub fn show_acknowledged(mut self, show: bool) -> Self {
        self.show_acknowledged = show;
        self
    }
    
    /// Render the widget
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Create base block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title);
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Split area into list and details
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(inner_area);
        
        // Render alert list
        let alerts = self.get_alerts();
        self.render_alert_list(f, chunks[0], &alerts);
        
        // Render selected alert details
        if let Some(selected) = self.selected {
            if selected < alerts.len() {
                self.render_alert_details(f, chunks[1], &alerts[selected]);
            } else {
                self.render_no_selection(f, chunks[1]);
            }
        } else {
            self.render_no_selection(f, chunks[1]);
        }
    }
    
    /// Get the alerts to display
    fn get_alerts(&self) -> Vec<Alert> {
        if let Some(manager) = &self.alert_manager {
            // Use alerts from manager
            if self.show_acknowledged {
                // Get both active and recent alerts if we want to show acknowledged ones
                let mut alerts = manager.get_active_alerts();
                alerts.extend(manager.get_recent_alerts());
                alerts
            } else {
                manager.get_active_alerts()
            }
        } else if let Some(alerts) = &self.fallback_alerts {
            // Use fallback alerts
            alerts.to_vec()
        } else if let Some(dashboard_alerts) = &self.dashboard_alerts {
            // Convert dashboard alerts to internal format
            dashboard_alerts.iter()
                .map(|alert| Alert::from_dashboard_alert(alert))
                .collect()
        } else {
            // No alerts available
            Vec::new()
        }
    }
    
    /// Get severity style based on AlertSeverity
    fn get_severity_style(&self, severity: AlertSeverity) -> Style {
        match severity {
            AlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            AlertSeverity::Warning => Style::default().fg(Color::Yellow),
            AlertSeverity::Info => Style::default().fg(Color::Blue),
            AlertSeverity::Error => Style::default().fg(Color::Magenta),
        }
    }
    
    /// Get severity style based on DashboardAlertSeverity
    fn get_dashboard_severity_style(&self, severity: DashboardAlertSeverity) -> Style {
        match severity {
            DashboardAlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            DashboardAlertSeverity::Warning => Style::default().fg(Color::Yellow),
            DashboardAlertSeverity::Info => Style::default().fg(Color::Blue),
            DashboardAlertSeverity::Error => Style::default().fg(Color::Magenta),
        }
    }
    
    /// Render the alert list
    fn render_alert_list(&self, f: &mut Frame, area: Rect, alerts: &[Alert]) {
        if alerts.is_empty() {
            // Render message when no alerts exist
            let paragraph = Paragraph::new(Line::from(vec![
                Span::styled(
                    "No alerts",
                    Style::default().fg(Color::Green),
                ),
            ]));
            
            f.render_widget(paragraph, area);
            return;
        }
        
        // Create list items for each alert
        let items: Vec<ListItem> = alerts
            .iter()
            .map(|alert| {
                // Create styled alert entry
                let severity_style = self.get_severity_style(alert.severity);
                
                let acknowledged_style = if alert.acknowledged {
                    Style::default().add_modifier(Modifier::DIM)
                } else {
                    Style::default()
                };
                
                // Format timestamp
                let timestamp = alert.timestamp.format("%H:%M:%S").to_string();
                
                // Build acknowledgment info
                let ack_info = if alert.acknowledged {
                    format!("[ACK: {}]", alert.acknowledged_by.clone().unwrap_or_else(|| "unknown".to_string()))
                } else {
                    "".to_string()
                };
                
                // Create alert line
                let content = Line::from(vec![
                    Span::styled(
                        format!("[{:?}] ", alert.severity),
                        severity_style,
                    ),
                    Span::styled(
                        format!("{}: ", timestamp),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!("[{}] ", alert.category),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        alert.message.clone(),
                        acknowledged_style,
                    ),
                    if !ack_info.is_empty() {
                        Span::styled(
                            format!(" {}", ack_info),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else {
                        Span::raw("")
                    },
                ]);
                
                ListItem::new(content)
            })
            .collect();
        
        // Create list with items
        let alerts_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Alerts"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
        // Create list state if there's a selection
        if let Some(selected) = self.selected {
            let mut state = ListState::default();
            state.select(Some(selected.min(alerts.len().saturating_sub(1))));
            
            // Render stateful list
            f.render_stateful_widget(alerts_list, area, &mut state.clone());
        } else {
            // Render regular list without selection
            f.render_widget(alerts_list, area);
        }
    }
    
    /// Render selected alert details
    fn render_alert_details(&self, f: &mut Frame, area: Rect, alert: &Alert) {
        // Create block for alert details
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Alert Details");
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Format timestamp
        let timestamp = alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        
        // Create severity span with appropriate color
        let severity_style = self.get_severity_style(alert.severity);
        let severity_span = Span::styled(
            format!("{:?}", alert.severity),
            severity_style,
        );
        
        // Create acknowledged span
        let acknowledged_span = if alert.acknowledged {
            Span::styled(
                "Yes",
                Style::default().fg(Color::Gray),
            )
        } else {
            Span::styled(
                "No",
                Style::default().fg(Color::White),
            )
        };
        
        // Create alert details
        let mut details = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", alert.id)),
            ]),
            Line::from(vec![
                Span::styled("Time: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(timestamp),
            ]),
            Line::from(vec![
                Span::styled("Severity: ", Style::default().add_modifier(Modifier::BOLD)),
                severity_span,
            ]),
            Line::from(vec![
                Span::styled("Acknowledged: ", Style::default().add_modifier(Modifier::BOLD)),
                acknowledged_span,
            ]),
        ];
        
        // Add acknowledged details if available
        if alert.acknowledged {
            if let Some(acknowledged_by) = &alert.acknowledged_by {
                details.push(Line::from(vec![
                    Span::styled("Acknowledged by: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(acknowledged_by.clone()),
                ]));
            }
            
            if let Some(acknowledged_at) = alert.acknowledged_at {
                let ack_time = acknowledged_at.format("%Y-%m-%d %H:%M:%S").to_string();
                details.push(Line::from(vec![
                    Span::styled("Acknowledged at: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(ack_time),
                ]));
            }
        }
        
        // Add category and source
        details.push(Line::from(vec![
            Span::styled("Category: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(alert.category.clone()),
        ]));
        
        details.push(Line::from(vec![
            Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(alert.source.clone()),
        ]));
        
        details.push(Line::from(""));
        details.push(Line::from(vec![
            Span::styled("Message:", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        details.push(Line::from(alert.message.clone()));
        details.push(Line::from(""));
        
        // Add details if present
        let mut detail_lines = vec![];
        if let Some(details_str) = &alert.details {
            if !details_str.is_empty() {
                detail_lines.push(
                    Line::from(vec![
                        Span::styled("Details:", Style::default().add_modifier(Modifier::BOLD)),
                    ])
                );
                
                // Add each detail line
                for line in details_str.split('\n') {
                    detail_lines.push(Line::from(line.to_string()));
                }
            }
        }
        
        // Create paragraph with details
        let paragraph = if !detail_lines.is_empty() {
            // Extend details with detail lines
            let mut full_details = details;
            full_details.extend(detail_lines);
            Paragraph::new(full_details)
        } else {
            // Create paragraph with details (no additional details)
            Paragraph::new(details)
        };
        
        f.render_widget(paragraph, inner_area);
    }
    
    /// Render message when no alert is selected
    fn render_no_selection(&self, f: &mut Frame, area: Rect) {
        // Create block for alert details
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Alert Details");
        
        // Render block
        f.render_widget(block.clone(), area);
        
        // Get inner area
        let inner_area = block.inner(area);
        
        // Create paragraph with message
        let paragraph = Paragraph::new(Line::from(vec![
            Span::styled(
                "Select an alert to view details",
                Style::default().fg(Color::Gray),
            ),
        ]));
        
        f.render_widget(paragraph, inner_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alerts_widget_creation() {
        // Create a new AlertsWidget
        let widget = AlertsWidget::new(None, "Test Widget");
        
        // Check default values
        assert_eq!(widget.title, "Test Widget");
        assert_eq!(widget.selected, None);
        assert_eq!(widget.show_acknowledged, false);
        
        // Test with alert manager
        let manager = Arc::new(AlertManager::new());
        let widget = AlertsWidget::new(Some(manager), "With Manager");
        
        // Test with fallback alerts
        let alerts = vec![
            Alert {
                id: "test1".to_string(),
                message: "This is a test alert".to_string(),
                source: "Test".to_string(),
                details: Some("This is a test alert".to_string()),
                category: "Test".to_string(),
                severity: AlertSeverity::Warning,
                timestamp: Utc::now(),
                acknowledged: false,
                acknowledged_at: None,
                acknowledged_by: None,
            }
        ];
        
        let widget = AlertsWidget::with_fallback(&alerts, "With Fallback");
        assert_eq!(widget.title, "With Fallback");
        
        // Test with dashboard alerts
        let dashboard_alerts = vec![
            DashboardAlert {
                id: "dash1".to_string(),
                title: "Dashboard Alert".to_string(),
                message: "This is a dashboard alert".to_string(),
                severity: DashboardAlertSeverity::Critical,
                source: "Dashboard".to_string(),
                timestamp: Utc::now(),
                acknowledged: false,
                acknowledged_at: None,
                acknowledged_by: None,
            }
        ];
        
        let widget = AlertsWidget::from_dashboard(&dashboard_alerts, "Dashboard Alerts");
        assert_eq!(widget.title, "Dashboard Alerts");
        
        // Test with selection
        let widget = widget.selected(Some(0));
        assert_eq!(widget.selected, Some(0));
        
        // Test with show_acknowledged
        let widget = widget.show_acknowledged(true);
        assert_eq!(widget.show_acknowledged, true);
    }
    
    #[test]
    fn test_alerts_widget_get_alerts() {
        // Create mock data
        let alerts = vec![
            Alert {
                id: "alert1".to_string(),
                message: "First test alert".to_string(),
                source: "Test Source".to_string(),
                details: Some("First test alert".to_string()),
                category: "Test Source".to_string(),
                severity: AlertSeverity::Warning,
                timestamp: Utc::now(),
                acknowledged: false,
                acknowledged_at: None,
                acknowledged_by: None,
            },
            Alert {
                id: "alert2".to_string(),
                message: "Second test alert".to_string(),
                source: "Test Source".to_string(),
                details: Some("Second test alert".to_string()),
                category: "Test Source".to_string(),
                severity: AlertSeverity::Critical,
                timestamp: Utc::now(),
                acknowledged: true,
                acknowledged_at: Some(Utc::now()),
                acknowledged_by: Some("Tester".to_string()),
            }
        ];

        // Test with fallback alerts
        let widget = AlertsWidget::with_fallback(&alerts, "Test Alerts");
        
        // Should return all alerts by default
        let widget_alerts = widget.get_alerts();
        assert_eq!(widget_alerts.len(), 2);
    }
    
    #[test]
    fn test_alerts_widget_severity_styles() {
        let widget = AlertsWidget::new(None, "Test");
        
        // Test local severity styles
        let critical_style = widget.get_severity_style(AlertSeverity::Critical);
        let warning_style = widget.get_severity_style(AlertSeverity::Warning);
        let info_style = widget.get_severity_style(AlertSeverity::Info);
        let error_style = widget.get_severity_style(AlertSeverity::Error);
        
        // Verify colors are appropriate for each severity
        assert_eq!(critical_style.fg, Some(Color::Red));
        assert_eq!(warning_style.fg, Some(Color::Yellow));
        assert_eq!(info_style.fg, Some(Color::Blue));
        assert_eq!(error_style.fg, Some(Color::Magenta));
        
        // Test dashboard severity styles
        let dashboard_critical_style = widget.get_dashboard_severity_style(DashboardAlertSeverity::Critical);
        let dashboard_warning_style = widget.get_dashboard_severity_style(DashboardAlertSeverity::Warning);
        let dashboard_info_style = widget.get_dashboard_severity_style(DashboardAlertSeverity::Info);
        let dashboard_error_style = widget.get_dashboard_severity_style(DashboardAlertSeverity::Error);
        
        // Verify dashboard styles match corresponding local styles
        assert_eq!(critical_style.fg, dashboard_critical_style.fg);
        assert_eq!(warning_style.fg, dashboard_warning_style.fg);
        assert_eq!(info_style.fg, dashboard_info_style.fg);
        assert_eq!(error_style.fg, dashboard_error_style.fg);
    }
    
    #[test]
    fn test_alerts_widget_selection() {
        // Create mock alerts
        let alerts = vec![
            Alert {
                id: "alert1".to_string(),
                message: "First test alert".to_string(),
                source: "Test Source".to_string(),
                details: Some("First test alert".to_string()),
                category: "Test Source".to_string(),
                severity: AlertSeverity::Warning,
                timestamp: Utc::now(),
                acknowledged: false,
                acknowledged_at: None,
                acknowledged_by: None,
            },
            Alert {
                id: "alert2".to_string(),
                message: "Second test alert".to_string(),
                source: "Test Source".to_string(),
                details: Some("Second test alert".to_string()),
                category: "Test Source".to_string(),
                severity: AlertSeverity::Critical,
                timestamp: Utc::now(),
                acknowledged: true,
                acknowledged_at: Some(Utc::now()),
                acknowledged_by: Some("Tester".to_string()),
            }
        ];

        // Test selection functionality
        let mut widget = AlertsWidget::with_fallback(&alerts, "Test Alerts");
        
        // Default selection
        assert_eq!(widget.selected, None);
        
        // Set selection
        widget = widget.selected(Some(0));
        assert_eq!(widget.selected, Some(0));
        
        // Out of bounds selection should be clamped
        widget = widget.selected(Some(10));
        assert_eq!(widget.selected, Some(10));
    }
} 