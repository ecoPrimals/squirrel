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
};

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
}

impl TuiDashboard {
    /// Create a new TUI dashboard
    pub fn new(dashboard_service: Arc<dyn DashboardService>) -> Self {
        Self {
            dashboard_service,
            app: app::App::new(),
            update_rx: None,
            tick_rate: Duration::from_millis(250),
        }
    }
    
    /// Create a new TUI dashboard from a DefaultDashboardService
    pub fn new_with_default_service(dashboard_service: Arc<DefaultDashboardService>) -> Self {
        Self::new(dashboard_service as Arc<dyn DashboardService>)
    }
    
    /// Create a new TUI dashboard from a DefaultDashboardService tuple with receiver
    pub fn new_from_default_service(dashboard_service_tuple: (Arc<DefaultDashboardService>, mpsc::Receiver<DashboardUpdate>)) -> Self {
        let (dashboard_service, _rx) = dashboard_service_tuple;
        Self::new(dashboard_service as Arc<dyn DashboardService>)
    }
    
    /// Run the dashboard UI
    pub async fn run(&mut self) -> io::Result<()> {
        // Initialize terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Subscribe to dashboard updates
        self.update_rx = Some(self.dashboard_service.subscribe().await);
        
        // Load initial dashboard data
        match self.dashboard_service.get_dashboard_data().await {
            Ok(data) => self.app.update_dashboard_data(data),
            Err(e) => eprintln!("Failed to load initial dashboard data: {}", e),
        }
        
        // Start events handling
        let mut events = events::Events::new(Duration::from_millis(100));
        
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