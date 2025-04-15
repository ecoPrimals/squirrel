// crates/ui-terminal/tests/app_basic_test.rs
use std::sync::Arc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ui_terminal::app::events::KeyEventHandler;

use ui_terminal::app::{App, ActiveTab};
use crate::mocks::MockDashboardService;

mod mocks;

#[tokio::test]
async fn test_app_startup() {
    // Create App with Mock Service
    let mock_service = Arc::new(MockDashboardService::new());
    let mut app = App::new(mock_service);

    // Initialize the app
    app.update().await;
    
    // Verify initial state
    assert_eq!(app.state.active_tab, ActiveTab::System);
}

#[tokio::test]
async fn test_app_tab_switching() {
    // Create App with Mock Service
    let mock_service = Arc::new(MockDashboardService::new());
    let mut app = App::new(mock_service);

    // --- Initial State: System ---
    assert_eq!(app.state.active_tab, ActiveTab::System);

    // --- Switch to System (Tab 2) ---
    app.on_key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE));
    assert_eq!(app.state.active_tab, ActiveTab::System, "App state should be System tab after pressing '2'");

    // --- Switch to Network (Tab 3) ---
    app.on_key(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
    assert_eq!(app.state.active_tab, ActiveTab::Network, "App state should be Network tab after pressing '3'");

    // --- Switch back to Overview (Tab 1) ---
    app.on_key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
    assert_eq!(app.state.active_tab, ActiveTab::Overview, "App state should be Overview tab after pressing '1'");
} 