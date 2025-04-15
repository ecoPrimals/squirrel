pub mod state;
pub mod events;
pub mod chat;
pub mod dashboard;
// pub mod alerts; // Removing non-existent module

use std::sync::Arc;
use dashboard_core::service::DashboardService;
use crate::error::Error;
use state::AppState;
use crossterm::event::KeyEvent;
use chrono;

/// Define the available tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Overview = 0,
    System = 1,
    Network = 2,
    Protocol = 3,
    Alerts = 4,
}

impl std::fmt::Display for AppTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTab::Overview => write!(f, "Overview"),
            AppTab::System => write!(f, "System"),
            AppTab::Network => write!(f, "Network"),
            AppTab::Protocol => write!(f, "Protocol"),
            AppTab::Alerts => write!(f, "Alerts"),
        }
    }
}

impl AppTab {
    /// Convert from usize to AppTab
    pub fn from_usize(idx: usize) -> Option<AppTab> {
        match idx {
            0 => Some(AppTab::Overview),
            1 => Some(AppTab::System),
            2 => Some(AppTab::Network),
            3 => Some(AppTab::Protocol),
            4 => Some(AppTab::Alerts),
            _ => None,
        }
    }
}

/// Main application state holder
pub struct App<S: ?Sized> {
    /// Dashboard service
    pub service: Arc<S>,
    /// Application state
    pub state: AppState,
    /// Tab titles
    pub tabs: Vec<String>,
}

impl<S: DashboardService + Send + Sync + 'static + ?Sized> App<S> {
    /// Create a new application instance
    pub fn new(service: Arc<S>) -> Self {
        let tabs = vec![
            AppTab::Overview.to_string(),
            AppTab::System.to_string(),
            AppTab::Network.to_string(),
            AppTab::Protocol.to_string(),
            AppTab::Alerts.to_string(),
        ];
        
        Self {
            service,
            state: AppState::default(),
            tabs,
        }
    }

    /// Handle a keyboard event
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle key event based on current state
        if self.state.show_help {
            // Any key dismisses help
            self.state.show_help = false;
            return;
        }

        // Let the app handle key events
        match key.code {
            crossterm::event::KeyCode::Char('h') => {
                self.state.show_help = !self.state.show_help;
            }
            crossterm::event::KeyCode::Char('r') => {
                // Request data update
                // We'll just update right away next time
            }
            crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Right => {
                self.next_tab();
            }
            crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Left => {
                self.previous_tab();
            }
            crossterm::event::KeyCode::Char('1') => {
                self.state.active_tab = AppTab::Overview;
            }
            crossterm::event::KeyCode::Char('2') => {
                self.state.active_tab = AppTab::System;
            }
            crossterm::event::KeyCode::Char('3') => {
                self.state.active_tab = AppTab::Network;
            }
            crossterm::event::KeyCode::Char('4') => {
                self.state.active_tab = AppTab::Protocol;
            }
            crossterm::event::KeyCode::Char('5') => {
                self.state.active_tab = AppTab::Alerts;
            }
            _ => {}
        }
    }

    /// Move to the next tab
    pub fn next_tab(&mut self) {
        let current_idx = self.state.active_tab as usize;
        let next_idx = (current_idx + 1) % self.tabs.len();
        if let Some(tab) = AppTab::from_usize(next_idx) {
            self.state.active_tab = tab;
        }
    }

    /// Move to the previous tab
    pub fn previous_tab(&mut self) {
        let current_idx = self.state.active_tab as usize;
        let next_idx = if current_idx == 0 {
            self.tabs.len() - 1
        } else {
            current_idx - 1
        };
        if let Some(tab) = AppTab::from_usize(next_idx) {
            self.state.active_tab = tab;
        }
    }

    /// Update connection status based on service health
    pub async fn update_connection_status(&mut self) -> Result<(), Error> {
        // Check if we can get dashboard data as a proxy for connection health
        match self.service.get_dashboard_data().await {
            Ok(_) => {
                // If we can get data, connection is healthy
                self.state.connection_status = "Connected".to_string();
                self.state.connection_health = 1; // Set to 1 for connected
                Ok(())
            }
            Err(e) => {
                self.state.connection_status = "Error".to_string();
                self.state.connection_health = 0; // Set to 0 for disconnected
                Err(Error::DataProvider(format!("Connection error: {}", e)))
            }
        }
    }

    /// Update application data from the service
    pub async fn update_data(&mut self) -> Result<(), Error> {
        // First update connection status
        self.update_connection_status().await?;
        
        // Then fetch all dashboard data
        match self.service.get_dashboard_data().await {
            Ok(data) => {
                // Update state with metrics
                self.state.metrics = Some(data.metrics);
                self.state.protocol_data = Some(data.protocol);
                self.state.alerts = Some(data.alerts);
                
                // Update timestamp
                self.state.last_update = Some(chrono::Utc::now());
                
                Ok(())
            }
            Err(e) => {
                Err(Error::DataProvider(format!("Error fetching data: {}", e)))
            }
        }
    }
}

// Re-export main entry functions
pub use crate::run_ai_chat_ui;
pub use dashboard::run_dashboard_ui; 