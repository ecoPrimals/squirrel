// crates/ui-terminal/src/widgets/alerts.rs

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use dashboard_core::data::{Alert, AlertSeverity};

// Function to determine the color based on AlertSeverity
// ... (determine_severity_color remains internal, no doc needed unless exported)

/// Render the alerts widget
pub fn render(f: &mut Frame, area: Rect, alerts: Option<&Vec<Alert>>) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(match alerts {
            Some(a) => format!("Alerts ({})", a.len()),
            None => "Alerts (0)".to_string()
        });
    
    // If no alerts, display empty message
    if alerts.is_none() || alerts.as_ref().unwrap().is_empty() {
        let empty_widget = List::new(vec![
            ListItem::new("No alerts to display")
        ])
        .block(block);
        f.render_widget(empty_widget, area);
        return;
    }
    
    let alerts = alerts.unwrap();
    
    // Create list items for each alert
    let items: Vec<ListItem> = alerts.iter().map(|alert| {
        let severity_style = match alert.severity {
            AlertSeverity::Info => Style::default().fg(Color::Blue),
            AlertSeverity::Warning => Style::default().fg(Color::Yellow),
            AlertSeverity::Error => Style::default().fg(Color::Red),
            AlertSeverity::Critical => Style::default().fg(Color::Red).bg(Color::Black),
        };
        
        let timestamp = alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
        let source = &alert.source;
        
        // Format the alert with timestamp, severity, source, and message
        let line = Line::from(vec![
            Span::raw(format!("[{}] ", timestamp)),
            Span::styled(format!("{:?} ", alert.severity), severity_style),
            Span::raw(format!("{}: ", source)),
            Span::raw(alert.message.clone()),
        ]);
        
        ListItem::new(line)
    }).collect();
    
    let list = List::new(items)
        .block(block);
    
    f.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppState};
    use dashboard_core::data::Alert;
    use dashboard_core::service::MockDashboardService;
    use ratatui::{
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style, Stylize},
        Terminal,
    };
    use std::collections::VecDeque;
    use chrono::Utc;
    use std::sync::Arc;

    // Helper to create a default App
    fn create_test_app() -> App<MockDashboardService> {
        App::new(Arc::new(MockDashboardService::new()))
    }

    // Helper to create sample alerts
    fn create_sample_alerts() -> Vec<Alert> {
        let mut alerts = Vec::new();
        alerts.push(Alert {
            id: "alert1".to_string(),
            title: "High CPU Alert".to_string(),
            message: "High CPU usage detected".to_string(),
            severity: AlertSeverity::Critical,
            timestamp: Utc::now(),
            source: "system_monitor".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        });
        alerts.push(Alert {
            id: "alert2".to_string(),
            title: "Disk Space Warning".to_string(),
            message: "Low disk space warning".to_string(),
            severity: AlertSeverity::Warning,
            timestamp: Utc::now(),
            source: "disk_monitor".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        });
         alerts.push(Alert {
            id: "alert3".to_string(),
            title: "Information Notice".to_string(),
            message: "Informational message".to_string(),
            severity: AlertSeverity::Info,
            timestamp: Utc::now(),
            source: "general".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        });
        alerts
    }

    #[test]
    fn test_render_alerts_empty() {
        // Test rendering when no alerts are present
    }
    
    #[test]
    fn test_render_alerts_with_data() {
        // Test rendering with alerts data
    }
} 