/*!
 * Terminal UI implementation of the Squirrel monitoring dashboard.
 * 
 * This crate provides a terminal-based user interface for the dashboard
 * using the Ratatui library.
 */

pub mod adapter;
pub mod app;
pub mod events;
pub mod ui;
pub mod util;
pub mod config;
pub mod widgets;
pub mod alert;
pub mod help;
pub mod widget_manager;
pub mod monitoring;

#[cfg(test)]
mod tests;

use std::io;
use std::sync::Arc;
use std::time::Duration;
use std::error::Error;
use std::collections::HashMap;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    Frame,
};

use dashboard_core::{
    DashboardConfig,
    data::{AlertSeverity as DashboardAlertSeverity, DashboardData, Metrics, 
           ProtocolData, Protocol, ProtocolStatus, Alert as DashboardAlert},
    health::HealthCheck as DashboardHealthCheck,
    service::{DashboardService, DefaultDashboardService},
};

use chrono::Utc;
use tokio::sync::mpsc;
use tokio::time;

use adapter::MonitoringToDashboardAdapter;
use alert::AlertManager;
use help::HelpSystem;
use crate::config::Config;
use crate::widget_manager::WidgetManager;
use crate::widgets::Widget;
use crate::monitoring::MonitoringAdapter;
use crate::events::Event as DashboardEvent;

/// Helper function to create a dashboard service
fn create_dashboard_service() -> Arc<dyn DashboardService> {
    // DefaultDashboardService::default() already returns Arc<DefaultDashboardService>
    // No need to wrap it in another Arc
    DefaultDashboardService::default() as Arc<dyn DashboardService>
}

/// Terminal UI Dashboard
pub struct TuiDashboard {
    /// Core dashboard service
    dashboard_service: Arc<dyn DashboardService>,
    /// App state
    app: app::App,
    /// Update channel receiver
    update_rx: Option<mpsc::Receiver<DashboardEvent>>,
    /// Tick rate for the event loop
    tick_rate: Duration,
    /// Monitoring adapter
    monitoring_adapter: Option<MonitoringToDashboardAdapter>,
    /// Alert manager
    alert_manager: AlertManager,
    /// Help system
    help: HelpSystem,
}

impl TuiDashboard {
    /// Create a new TUI dashboard
    pub fn new(dashboard_service: Arc<dyn DashboardService>) -> Self {
        // Create app with default state
        let app = app::App::default();
        
        // Create help system
        let help = HelpSystem::default();
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
            alert_manager,
            help,
        }
    }
    
    /// Create a new TUI dashboard with monitoring adapter
    pub fn new_with_monitoring() -> Self {
        // Create a dashboard service with default configuration
        let dashboard_service = create_dashboard_service();
        
        // Create help system
        let help_system = HelpSystem::default();
        
        // Create monitoring adapter
        let monitoring_adapter = MonitoringToDashboardAdapter::new(DashboardConfig::default()
            .with_update_interval(5) // 5 seconds
            .with_max_history_points(1000));
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        // Create app with default state
        let mut app = app::App::default();
        app.show_help = false; // Ensure help is initially hidden
        
        // Initialize with widget managers
        // (Left empty for now, would be populated with specific widget instances)
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: Some(monitoring_adapter),
            alert_manager,
            help: help_system,
        }
    }
    
    /// Create a new TUI dashboard with MCP integration
    pub fn new_with_mcp() -> Self {
        // Create a dashboard service with default configuration
        let dashboard_service = create_dashboard_service();
        
        // Create mock MCP client for testing
        let mcp_client = Arc::new(crate::adapter::MockMcpClient::new());
        
        // Create monitoring adapter with MCP client
        let monitoring_adapter = MonitoringToDashboardAdapter::new(DashboardConfig::default()
            .with_update_interval(5) // 5 seconds
            .with_max_history_points(1000));
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        // Create help system
        let help_system = HelpSystem::default();
        
        // Create app with default state
        let app = app::App::default();
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: Some(monitoring_adapter),
            alert_manager,
            help: help_system,
        }
    }
    
    /// Create a new TUI dashboard with monitoring adapter
    pub fn new_with_monitoring_adapter(monitoring_adapter: Box<dyn MonitoringAdapter>) -> Self {
        // Create a dashboard service with default configuration
        let dashboard_service = create_dashboard_service();
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        // Create help system
        let help = HelpSystem::default();
        
        // Create app with default state
        let app = app::App::default();
        
        // Create monitoring adapter wrapper with a reasonable poll interval
        let adapter_wrapper = MonitoringToDashboardAdapter::new(DashboardConfig::default()
            .with_update_interval(5) // 5 seconds
            .with_max_history_points(1000))
            .with_max_history_points(1000)
            .with_poll_interval(Duration::from_secs(5))
            .with_monitoring_adapter(monitoring_adapter);
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: Some(adapter_wrapper),
            alert_manager,
            help,
        }
    }
    
    /// Create a new TUI dashboard with custom app
    pub fn new_with_custom_app(
        dashboard_service: Arc<dyn DashboardService>,
        title: String,
        config: Config,
        help_system: HelpSystem,
        widget_managers: Vec<Box<dyn WidgetManager>>,
    ) -> Self {
        // Create app with custom configuration
        let app = app::App::with_config(
            title,
            config,
            Arc::new(help_system.clone()),
            widget_managers,
        );
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
            alert_manager,
            help: help_system,
        }
    }
    
    /// Get the alert manager
    pub fn alert_manager(&self) -> AlertManager {
        self.alert_manager.clone()
    }
    
    /// Get the help system
    pub fn help_system(&self) -> HelpSystem {
        self.help.clone()
    }
    
    /// Set the show help flag
    pub fn set_show_help(&mut self, show_help: bool) {
        self.app.show_help = show_help;
    }
    
    /// Run the dashboard UI
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        // Define Event enum for event handling
        enum AppEvent {
            Input(event::Event),
            Tick,
        }

        // Set up event handling
        let (tx, mut rx) = mpsc::channel(100); // Buffer size of 100
        let tick_rate = Duration::from_millis(250); // Default tick rate

        // Spawn input handling thread
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut last_tick = std::time::Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap() {
                    if let Ok(event) = event::read() {
                        if tx_clone.send(AppEvent::Input(event)).await.is_err() {
                            break;
                        }
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if tx_clone.send(AppEvent::Tick).await.is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });
        
        // Main event loop
        let mut last_refresh = std::time::Instant::now();
        let refresh_interval = Duration::from_secs(5); // 5 seconds default

        // Main loop
        loop {
            // Check if we need to refresh dashboard data
            if last_refresh.elapsed() >= refresh_interval {
                // Update dashboard data
                let dashboard_data = self.load_dashboard_data();
                // Move the app update outside the borrow
                let app_update_data = dashboard_data.clone();
                self.app.update_dashboard_data(app_update_data);
                last_refresh = std::time::Instant::now();

                // Check for metrics alerts with a separate borrow
                self.check_metrics_alerts(&Some(dashboard_data));
            }

            // Draw UI
            terminal.draw(|f| {
                self.app.render_to_frame(f);
            })?;

            // Handle events
            let timeout = Duration::from_millis(100);
            match tokio::time::timeout(timeout, rx.recv()).await {
                Ok(Some(event)) => {
                    match event {
                        AppEvent::Input(event) => match event {
                            event::Event::Key(key) => {
                                if key.code == KeyCode::Char('q') {
                                    break;
                                }
                                self.app.on_key(key.code);
                            }
                            event::Event::Mouse(mouse) => {
                                self.app.on_mouse(mouse);
                            }
                            event::Event::Resize(width, height) => {
                                self.app.on_resize(width, height);
                            }
                            _ => {}
                        },
                        AppEvent::Tick => {
                            self.app.on_tick();
                        }
                    }
                }
                Ok(None) => break, // Channel closed
                Err(_) => {
                    // Timeout, just continue the loop
                },
            }

            // Check if app is still running
            if !self.app.running {
                break;
            }
        }

        // Clean up terminal before returning
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }
    
    /// Load dashboard data
    fn load_dashboard_data(&mut self) -> DashboardData {
        // If we have a monitoring adapter configured, use it to collect data
        if let Some(monitoring_adapter) = &mut self.monitoring_adapter {
            // Use monitoring adapter to collect data
            monitoring_adapter.collect_dashboard_data_with_monitoring(self.dashboard_service.as_ref())
        } else {
            // Use default empty dashboard data if there's no adapter
            DashboardData {
                metrics: dashboard_core::data::Metrics::default(),
                protocol: dashboard_core::data::ProtocolData::default(),
                alerts: Vec::new(),
                timestamp: Utc::now(),
            }
        }
    }

    /// Check for metrics alerts based on dashboard data
    fn check_metrics_alerts(&self, dashboard_data: &Option<DashboardData>) {
        // Skip if no data is available
        let data = match dashboard_data {
            Some(data) => data,
            None => return,
        };

        // Get alert manager reference
        let alert_manager = &self.alert_manager;

        // Check CPU usage
        // Check CPU utilization threshold
        let cpu_usage = data.metrics.cpu.usage;
        if cpu_usage > 90.0 {
            alert_manager.add_alert(
                alert::AlertSeverity::Critical,
                format!("CPU usage is critically high at {:.1}%", cpu_usage),
                "System".to_string(),
                "CPU".to_string(),
            );
        } else if cpu_usage > 80.0 {
            alert_manager.add_alert(
                alert::AlertSeverity::Warning,
                format!("CPU usage is high at {:.1}%", cpu_usage),
                "System".to_string(),
                "CPU".to_string(),
            );
        }
        
        // Check memory usage
        let memory_used_percent = (data.metrics.memory.used as f64 / data.metrics.memory.total as f64) * 100.0;
        if memory_used_percent > 90.0 {
            alert_manager.add_alert(
                alert::AlertSeverity::Critical,
                format!("Memory usage is critically high at {:.1}%", memory_used_percent),
                "System".to_string(),
                "Memory".to_string(),
            );
        } else if memory_used_percent > 80.0 {
            alert_manager.add_alert(
                alert::AlertSeverity::Warning,
                format!("Memory usage is high at {:.1}%", memory_used_percent),
                "System".to_string(),
                "Memory".to_string(),
            );
        }
        
        // Check disk usage - iterate over each volume if available
        for (mount_point, volume) in &data.metrics.disk.usage {
            let used_percent = volume.used_percentage;
            if used_percent > 90.0 {
                alert_manager.add_alert(
                    alert::AlertSeverity::Critical,
                    format!("Disk usage is critically high on volume '{}': {:.1}%", 
                            mount_point, used_percent),
                    "System".to_string(),
                    "Disk".to_string(),
                );
            } else if used_percent > 80.0 {
                alert_manager.add_alert(
                    alert::AlertSeverity::Warning,
                    format!("Disk usage is high on volume '{}': {:.1}%", 
                            mount_point, used_percent),
                    "System".to_string(),
                    "Disk".to_string(),
                );
            }
        }
    }

    /// Check for updates from the update channel
    fn check_update_channel(&mut self) -> bool {
        if let Some(rx) = &mut self.update_rx {
            // Try to receive an update without blocking
            match rx.try_recv() {
                Ok(update) => {
                    // Convert DashboardEvent to DashboardData and apply it
                    let data = self.convert_event_to_data(update);
                    self.app.update_dashboard_data(data);
                    return true;
                }
                _ => {}
            }
        }
        false
    }
    
    /// Convert a dashboard event to dashboard data
    fn convert_event_to_data(&self, event: DashboardEvent) -> DashboardData {
        // Start with the current data or create a new one
        let mut data = if let Some(dashboard_data) = self.app.dashboard_data() {
            dashboard_data.clone()
        } else {
            DashboardData::default()
        };
        
        // Update the timestamp
        data.timestamp = Utc::now();
        
        data
    }

    /// Reset terminal to initial state
    fn reset_terminal() -> Result<(), Box<dyn Error>> {
        // Disable raw mode
        crossterm::terminal::disable_raw_mode()?;
        
        // Clean up terminal
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
        
        Ok(())
    }
    
    /// Handle terminal lock
    fn handle_terminal_lock() -> Result<(), Box<dyn Error>> {
        // Enable raw mode
        crossterm::terminal::enable_raw_mode()?;
        
        // Set up stdout
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        
        Ok(())
    }

    /// Create a dashboard UI with the provided app configuration
    pub fn with_app(
        dashboard_service: Arc<dyn DashboardService>,
        app: app::App,
    ) -> Self {
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        // Get help system from app (we need HelpSystem, not Arc<HelpSystem>)
        let help_system = app.help_system.clone();
        // Unwrap the Arc to get the inner HelpSystem (not actually unwrapping, just need to match type)
        let help = (*help_system).clone();
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
            alert_manager,
            help,
        }
    }

    /// Create a new dashboard UI with custom configuration
    pub fn with_custom_config(
        dashboard_service: Arc<dyn DashboardService>,
        title: String,
        config: Config,
        help_system: HelpSystem,
        widget_managers: Vec<Box<dyn WidgetManager>>,
    ) -> Self {
        // Create app with custom configuration
        let app = App::with_config(
            title,
            config,
            Arc::new(help_system.clone()),
            widget_managers,
        );
        
        Self::with_app(dashboard_service, app)
    }

    /// Build a dashboard for the terminal UI
    pub fn buildDashboard() -> Self {
        // Create a dashboard service with default configuration
        let dashboard_service = DefaultDashboardService::default() as Arc<dyn DashboardService>;
        
        // Create help system
        let help_system = HelpSystem::default();
        
        // Create app with default config
        let widget_managers: Vec<Box<dyn WidgetManager>> = Vec::new();
        
        let app = App::with_config(
            "Squirrel Dashboard".to_string(),
            Config::default(),
            Arc::new(help_system.clone()),
            widget_managers,
        );
        
        Self {
            dashboard_service,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
            alert_manager: AlertManager::new(),
            help: help_system,
        }
    }

    /// Create a dashboard UI from a default service with receiver
    pub fn new_from_default_service(service_with_rx: (Arc<DefaultDashboardService>, mpsc::Receiver<dashboard_core::update::DashboardUpdate>)) -> Self {
        let (service, rx) = service_with_rx;
        
        // Create app with default state
        let app = app::App::default();
        
        // Create help system
        let help = HelpSystem::default();
        
        // Create alert manager
        let alert_manager = AlertManager::new();
        
        // Create a dashboard without the update channel
        let mut dashboard = Self {
            dashboard_service: service as Arc<dyn DashboardService>,
            app,
            update_rx: None,
            tick_rate: Duration::from_millis(250),
            monitoring_adapter: None,
            alert_manager,
            help,
        };
        
        // Handle updates in the background
        tokio::spawn(async move {
            let mut rx = rx;
            while let Some(update) = rx.recv().await {
                // Process updates directly within the task
                match update {
                    dashboard_core::update::DashboardUpdate::FullUpdate(data) => {
                        // Do something with data
                        println!("Received full update");
                    },
                    dashboard_core::update::DashboardUpdate::MetricsUpdate { metrics, timestamp } => {
                        // Do something with metrics
                        println!("Received metrics update at {}", timestamp);
                    },
                    dashboard_core::update::DashboardUpdate::AlertUpdate { alert, timestamp } => {
                        // Do something with alert
                        println!("Received alert update at {}", timestamp);
                    },
                    dashboard_core::update::DashboardUpdate::ConfigUpdate { config } => {
                        // Do something with config
                        println!("Received config update");
                    },
                    // Handle all other variants with a wildcard pattern
                    _ => {
                        // Generic handler for other update types
                        println!("Received other update type");
                    }
                }
            }
        });
        
        dashboard
    }
}

// Re-export commonly used types
pub use app::App;
pub use events::Event;
pub use config::ConfigError;
pub use adapter::{McpAdapter, McpMetricsConfig};

// Export the compatibility layer for data structure conversion
pub mod compatibility {
    use dashboard_core::data::{ProtocolData};
    
    /// Convert between new and old data formats
    pub fn protocol_to_metrics(protocol: &ProtocolData) -> ProtocolData {
        // Replace with simple return for now until we resolve the MetricsSnapshot issue
        protocol.clone()
    }
    
    /// Convert between old and new data formats
    pub fn metrics_to_protocol(metrics: &ProtocolData) -> ProtocolData {
        // Replace with simple return for now until we resolve the MetricsSnapshot issue
        metrics.clone()
    }
}

/// Run the terminal UI application
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app state
    let mut app = App::new();
    
    // Run app
    let res = run_app(&mut terminal, &mut app);
    
    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    
    // Handle result
    if let Err(err) = res {
        println!("Error: {:?}", err);
    }
    
    Ok(())
}

/// Run the main application loop
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    while app.running() {
        // Render UI
        app.render(terminal)?;
        
        // Handle input
        if !app.handle_input(terminal)? {
            app.quit();
            break;
        }
    }
    
    Ok(())
}

/// Dashboard application builder
pub struct DashboardBuilder {
    /// Application title
    title: String,
    /// Application config
    config: Option<Config>,
    /// Help system
    help_system: Option<HelpSystem>,
    /// Additional widgets
    widgets: Vec<Box<dyn WidgetManager>>,
}

impl DashboardBuilder {
    /// Create a new dashboard builder
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            config: None,
            help_system: None,
            widgets: Vec::new(),
        }
    }
    
    /// Set application config
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    
    /// Set help system
    pub fn with_help_system(mut self, help_system: HelpSystem) -> Self {
        self.help_system = Some(help_system);
        self
    }
    
    /// Add a widget manager
    pub fn with_widget(mut self, widget: Box<dyn WidgetManager>) -> Self {
        self.widgets.push(widget);
        self
    }
    
    /// Build the dashboard application
    pub fn build(self) -> App {
        let config = self.config.unwrap_or_default();
        let help_system = self.help_system.unwrap_or_else(|| HelpSystem::default());
        
        App::with_config(
            self.title,
            config,
            Arc::new(help_system),
            self.widgets,
        )
    }
}

/// Create a dashboard application with default settings
pub fn dashboard() -> App {
    DashboardBuilder::new("Dashboard")
        .build()
}

/// Create a dashboard application with custom title
pub fn dashboard_with_title(title: &str) -> App {
    DashboardBuilder::new(title)
        .build()
}

/// Create a dashboard application with custom configuration
pub fn dashboard_with_config(config: Config) -> App {
    DashboardBuilder::new("Dashboard")
        .with_config(config)
        .build()
}

/// Create a dashboard application with custom help system
pub fn dashboard_with_help(help_system: HelpSystem) -> App {
    DashboardBuilder::new("Dashboard")
        .with_help_system(help_system)
        .build()
}

/// Create app with the given configuration
fn create_app_with_config(
    title: String,
    config: Option<Config>,
    help_system: HelpSystem,
    widget_managers: Vec<Box<dyn WidgetManager>>,
) -> app::App {
    let config = config.unwrap_or_default();
    
    App::with_config(
        title,
        config,
        Arc::new(help_system),
        widget_managers,
    )
} 