use ratatui::{
    layout::{Rect, Alignment},
    widgets::{Widget, Block, Borders, List, ListItem},
    buffer::Buffer,
    style::{Style, Modifier, Color},
    text::{Line, Span},
};

use dashboard_core::Alert;
use dashboard_core::data::{Alert as DashboardAlert, AlertSeverity};
use chrono::{DateTime, Utc};

/// Widget for displaying system alerts
pub struct AlertsWidget<'a> {
    /// Alerts to display
    alerts: Option<&'a [DashboardAlert]>,
    /// Title of the widget
    title: &'a str,
}

impl<'a> AlertsWidget<'a> {
    /// Create a new alerts widget
    pub fn new(alerts: Option<&'a [DashboardAlert]>, title: &'a str) -> Self {
        Self {
            alerts,
            title,
        }
    }
}

impl<'a> Widget for AlertsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(ratatui::widgets::Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = self.alerts.unwrap_or_default()
            .iter()
            .map(|alert| {
                let severity_style = match alert.severity {
                    AlertSeverity::Info => Style::default().fg(Color::Blue),
                    AlertSeverity::Warning => Style::default().fg(Color::Yellow),
                    AlertSeverity::Error => Style::default().fg(Color::Red),
                    AlertSeverity::Critical => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                };

                let title = Span::styled(&alert.title, severity_style);
                let message = Span::raw(&alert.message);
                let timestamp = Span::raw(alert.timestamp.format("%H:%M:%S").to_string());

                ListItem::new(vec![
                    Line::from(vec![title]),
                    Line::from(vec![message]),
                    Line::from(vec![timestamp]),
                ])
            })
            .collect();

        List::new(items)
            .render(inner_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alerts_widget_creation() {
        let alerts = vec![
            DashboardAlert {
                title: "Test Alert 1".to_string(),
                message: "This is a test alert".to_string(),
                severity: AlertSeverity::Info,
                timestamp: chrono::Utc::now(),
            },
        ];
        
        let widget = AlertsWidget::new(Some(&alerts), "Test Alerts");
        assert_eq!(widget.title, "Test Alerts");
        assert_eq!(widget.alerts.unwrap().len(), 1);
    }
} 