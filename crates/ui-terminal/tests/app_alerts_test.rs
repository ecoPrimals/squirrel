// crates/ui-terminal/tests/app_alerts_test.rs
use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ui_terminal::app::events::KeyEventHandler;
use chrono::Utc;

use ui_terminal::app::{App, ActiveTab};
use dashboard_core::data::{Alert, AlertSeverity};
use crate::mocks::MockDashboardService;

mod mocks;

#[tokio::test]
async fn test_alert_handling() {
    // Create App with Mock Service
    let mock_service = Arc::new(MockDashboardService::new());
    let mut app = App::new(mock_service);
    
    // Check initial state before adding any alerts
    assert_eq!(app.state.alerts.len(), 0);
    
    // Create a test alert
    let alert = Alert {
        id: "test-alert-1".to_string(),
        title: "Test Alert".to_string(),
        message: "This is a test alert".to_string(),
        severity: AlertSeverity::Warning,
        source: "test".to_string(),
        timestamp: Utc::now(),
        acknowledged: false,
        acknowledged_by: None,
        acknowledged_at: None,
    };
    
    // Add the alert directly to the app state
    app.state.alerts.push(alert);
    
    // Navigate to Alerts tab
    app.on_key(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE));
    assert_eq!(app.state.active_tab, ActiveTab::Alerts);
    
    // Verify alert is in the state - should now be 1 alert
    assert_eq!(app.state.alerts.len(), 1, "Should have exactly 1 alert");
} 