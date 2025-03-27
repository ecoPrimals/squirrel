/*!
 * Terminal UI implementation of the Squirrel monitoring dashboard.
 * 
 * This crate provides a terminal-based user interface for the dashboard
 * using the Ratatui library.
 */

pub mod app;
pub mod ui;
pub mod widgets;
pub mod events;
pub mod util;
pub mod adapter;

#[cfg(test)]
mod tests;

use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use dashboard_core::{
    DashboardService,
    DashboardUpdate,
    service::DefaultDashboardService,
    config::DashboardConfig,
};

use adapter::MonitoringToDashboardAdapter;

/// Terminal UI Dashboard
pub struct TuiDashboard {
    /// Dashboard service for data access
    dashboard_service: Arc<dyn DashboardService>,
    
    /// Application state
    app: app::App,
    
    /// Update receiver channel
    update_rx: Option<mpsc::Receiver<DashboardUpdate>>,
    
    /// UI tick rate (for animations and non-input updates)
    tick_rate: Duration,
    
    /// Monitoring to Dashboard adapter
    monitoring_adapter: Option<MonitoringToDashboardAdapter>,
}

impl TuiDashboard {
    /// Create a new TUI dashboard
    pub fn new(dashboard_service: Arc<dyn DashboardService>) -> Self {
        Self {
            dashboard_service,
            app: app::App::new(),
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
        }
    }
    
    /// Create a new TUI dashboard with monitoring adapter
    pub fn new_with_monitoring() -> Self {
        // Create default dashboard config
        let config = DashboardConfig::default()
            .with_update_interval(5) // 5 seconds
            .with_max_history_points(1000);
        
        // Create dashboard service
        let (dashboard_service, rx) = DefaultDashboardService::new(config);
        
        // Create monitoring adapter
        let monitoring_adapter = MonitoringToDashboardAdapter::new();
        
        Self {
            dashboard_service: dashboard_service.clone(),
            app: app::App::new(),
            update_rx: Some(rx),
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: Some(monitoring_adapter),
        }
    }
    
    /// Create a new TUI dashboard from a DefaultDashboardService
    pub fn new_with_default_service(dashboard_service: Arc<DefaultDashboardService>) -> Self {
        Self::new(dashboard_service as Arc<dyn DashboardService>)
    }
    
    /// Create a new TUI dashboard from a DefaultDashboardService tuple with receiver
    pub fn new_from_default_service(dashboard_service_tuple: (Arc<DefaultDashboardService>, mpsc::Receiver<DashboardUpdate>)) -> Self {
        let (dashboard_service, rx) = dashboard_service_tuple;
        Self {
            dashboard_service: dashboard_service.clone(),
            app: app::App::new(),
            update_rx: Some(rx),
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
        }
    }
    
    /// Run the dashboard UI
    pub async fn run(&mut self) -> io::Result<()> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Subscribe to dashboard updates if not already subscribed
        if self.update_rx.is_none() {
            self.update_rx = Some(self.dashboard_service.subscribe().await);
        }
        
        // Load initial dashboard data
        match self.dashboard_service.get_dashboard_data().await {
            Ok(data) => self.app.update_dashboard_data(data),
            Err(e) => eprintln!("Failed to load initial dashboard data: {}", e),
        }
        
        // Start events handling
        let mut events = events::Events::new(self.tick_rate);
        
        // Start monitoring adapter if available
        if let Some(mut adapter) = self.monitoring_adapter.take() {
            let dashboard_service_clone = self.dashboard_service.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                
                loop {
                    interval.tick().await;
                    
                    // Collect dashboard data from monitoring
                    let data = adapter.collect_dashboard_data();
                    
                    // Update dashboard
                    let _ = dashboard_service_clone.update_dashboard_data(data).await;
                }
            });
        }
        
        // Main loop
        loop {
            // Draw UI
            terminal.draw(|f| ui::draw(f, &mut self.app))?;
            
            // Handle events
            if let Some(event) = events.next()? {
                // Pass appropriate events to the app
                match event {
                    events::Event::Key(key_event) => {
                        if !self.app.handle_event(key_event) {
                            break;
                        }
                    }
                    events::Event::Mouse(mouse_event) => {
                        self.app.handle_mouse(mouse_event);
                    }
                    events::Event::Resize(width, height) => {
                        self.app.handle_resize(width, height);
                    }
                    events::Event::Tick => {
                        // Just continue, handled by tick logic below
                    }
                }
            }
            
            // Check for dashboard updates
            if let Some(rx) = &mut self.update_rx {
                while let Ok(Some(update)) = rx.try_recv().map_or(Ok::<Option<DashboardUpdate>, tokio::sync::mpsc::error::TryRecvError>(None), |u| Ok(Some(u))) {
                    self.app.handle_update(update);
                }
            }
            
            // Tick for animations
            if events.tick() {
                self.app.tick();
            }
        }
        
        // Restore terminal
        terminal::disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(())
    }
} 