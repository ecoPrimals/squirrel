use std::collections::HashMap;
use std::sync::Arc;
use std::io;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::io::Write;

use dashboard_core::{
    DashboardData, MetricType,
    update::DashboardUpdate,
};
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{backend::Backend, Terminal};

use crate::help::HelpSystem;
use crate::ui::{self, UiState, ActiveTab};
use crate::config::Config;
use crate::widget_manager::WidgetManager;
use crate::widgets::health::HealthCheck;

/// App state
pub struct App {
    /// Application title
    pub title: String,
    /// Application config
    pub config: Config,
    /// Help system
    pub help_system: Arc<HelpSystem>,
    /// Dashboard data
    pub dashboard_data: Option<DashboardData>,
    /// Active tab
    pub active_tab: ActiveTab,
    /// Show help
    pub show_help: bool,
    /// Health checks
    pub health_checks: Vec<HealthCheck>,
    /// Time series data
    pub time_series: HashMap<MetricType, Vec<(DateTime<Utc>, f64)>>,
    /// Last update timestamp
    pub last_update: Option<DateTime<Utc>>,
    /// UI state
    pub ui_state: UiState,
    /// Widget managers
    pub widget_managers: Vec<Box<dyn WidgetManager>>,
    /// Whether the application is running
    pub running: bool,
    /// Last widget update times
    pub widget_update_times: Vec<Instant>,
    /// Tracks when the last full UI refresh occurred
    pub last_full_refresh: Instant,
    /// Minimum duration between full UI refreshes
    pub full_refresh_interval: Duration,
}

impl App {
    /// Create a new app
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            dashboard_data: None,
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            last_update: None,
            running: true,
            ui_state: UiState::default(),
            widget_managers: Vec::new(),
            title: "Squirrel UI".to_string(),
            config: Config::default(),
            help_system: Arc::new(HelpSystem::new()),
            widget_update_times: vec![now; 6], // One for each tab
            last_full_refresh: now,
            full_refresh_interval: Duration::from_secs(10), // Full refresh every 10 seconds
        }
    }
    
    /// Create a new app with custom config
    pub fn with_config(
        title: String,
        config: Config,
        help_system: Arc<HelpSystem>,
        widget_managers: Vec<Box<dyn WidgetManager>>,
    ) -> Self {
        let now = Instant::now();
        Self {
            title,
            config,
            help_system,
            dashboard_data: None,
            ui_state: UiState::default(),
            widget_managers,
            running: true,
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            last_update: None,
            widget_update_times: vec![now; 6], // One for each tab
            last_full_refresh: now,
            full_refresh_interval: Duration::from_secs(10), // Full refresh every 10 seconds
        }
    }
    
    /// Update dashboard data
    pub fn update_data(&mut self, data: DashboardData) {
        // Update each widget with new data
        for widget in &mut self.widget_managers {
            widget.update(&data);
        }
        
        // Store dashboard data
        self.dashboard_data = Some(data);
    }
    
    /// Handle keyboard input
    pub fn handle_input<B: Backend>(&mut self, _terminal: &mut Terminal<B>) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                return self.handle_key_event(key);
            }
        }
        
        Ok(true)
    }
    
    /// Handle key event
    pub fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<bool> {
        // First try to handle the key in the active widget
        if let Some(index) = self.widget_managers.iter().position(|w| w.enabled()) {
            if self.widget_managers[index].handle_input(key) {
                return Ok(true);
            }
        }
        
        // If not handled by the widget, handle it here
        match key.code {
            KeyCode::Char('q') => return Ok(false),
            KeyCode::Char('h') => self.ui_state.show_help = !self.ui_state.show_help,
            KeyCode::Tab => self.cycle_selected_tab(),
            KeyCode::Char('1') => self.select_tab(0),
            KeyCode::Char('2') => self.select_tab(1),
            KeyCode::Char('3') => self.select_tab(2),
            KeyCode::Char('4') => self.select_tab(3),
            KeyCode::Char('g') => self.ui_state.layout = ui::WidgetLayout::Grid,
            KeyCode::Char('v') => self.ui_state.layout = ui::WidgetLayout::Vertical,
            KeyCode::Char('H') => self.ui_state.layout = ui::WidgetLayout::Horizontal,
            KeyCode::Char('f') => self.ui_state.layout = ui::WidgetLayout::Focused(self.ui_state.selected_tab),
            _ => {}
        }
        
        Ok(true)
    }
    
    /// Render app to terminal
    pub fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let ui_app: &mut ui::UiApp = ui::convert_app_mut(self);
        ui::draw(terminal, ui_app)?;
        Ok(())
    }
    
    /// Cycle to the next tab
    pub fn cycle_selected_tab(&mut self) {
        let tab_count = 4; // Number of tabs: Overview, Network, Protocol, Alerts
        
        // Cycle to the next tab
        self.ui_state.selected_tab = (self.ui_state.selected_tab + 1) % tab_count;
        
        // Update active_tab to match selected_tab
        self.active_tab = match self.ui_state.selected_tab {
            0 => ActiveTab::Overview,
            1 => ActiveTab::Network,
            2 => ActiveTab::Protocol,
            3 => ActiveTab::Alerts,
            _ => ActiveTab::Overview,
        };
    }

    /// Cycle to the previous tab
    pub fn prev_tab(&mut self) {
        let tab_count = 4; // Number of tabs: Overview, Network, Protocol, Alerts
        
        // Cycle to the previous tab
        self.ui_state.selected_tab = if self.ui_state.selected_tab == 0 {
            tab_count - 1
        } else {
            self.ui_state.selected_tab - 1
        };
        
        // Update active_tab to match selected_tab
        self.active_tab = match self.ui_state.selected_tab {
            0 => ActiveTab::Overview,
            1 => ActiveTab::Network,
            2 => ActiveTab::Protocol,
            3 => ActiveTab::Alerts,
            _ => ActiveTab::Overview,
        };
    }

    /// Select a tab by index
    fn select_tab(&mut self, index: usize) {
        if index < self.widget_managers.len() {
            self.ui_state.selected_tab = index;
        }
    }
    
    /// Check if the app is still running
    pub fn running(&self) -> bool {
        self.running
    }
    
    /// Quit the app
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Handle key event
    pub fn handle_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            KeyCode::Char('q') => return false,
            KeyCode::Char('h') => self.ui_state.show_help = !self.ui_state.show_help,
            KeyCode::Tab => self.cycle_selected_tab(),
            KeyCode::BackTab => self.prev_tab(),
            KeyCode::Left => self.prev_tab(),
            KeyCode::Right => self.cycle_selected_tab(),
            _ => {}
        }
        true
    }

    /// Handle mouse event
    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => self.cycle_selected_tab(),
            MouseEventKind::ScrollUp => self.cycle_selected_tab(),
            _ => {}
        }
    }

    /// Handle resize event
    pub fn handle_resize(&mut self, _width: u16, _height: u16) {
        // Store the new dimensions if needed
        // This is a placeholder for future window size-dependent features
    }

    /// Handle dashboard update
    pub fn handle_update(&mut self, update: DashboardUpdate) {
        match update {
            DashboardUpdate::FullUpdate(data) => {
                self.update_dashboard_data(data);
            },
            DashboardUpdate::MetricsUpdate { metrics: _, timestamp } => {
                // Only update specific metrics
                if let Some(data) = &mut self.dashboard_data {
                    // TODO: Update individual metrics based on the metrics map
                    data.timestamp = timestamp;
                }
            },
            DashboardUpdate::AlertUpdate { alert, timestamp } => {
                // Update specific alert
                if let Some(data) = &mut self.dashboard_data {
                    // Find and update existing alert or add new one
                    let alert_index = data.alerts.iter().position(|a| a.id == alert.id);
                    if let Some(index) = alert_index {
                        data.alerts[index] = alert;
                    } else {
                        data.alerts.push(alert);
                    }
                    data.timestamp = timestamp;
                }
            },
            DashboardUpdate::SystemUpdate { cpu, memory, timestamp } => {
                // Update system metrics
                if let Some(data) = &mut self.dashboard_data {
                    data.metrics.cpu = cpu;
                    data.metrics.memory = memory;
                    data.timestamp = timestamp;
                }
            },
            DashboardUpdate::NetworkUpdate { network, timestamp } => {
                // Update network metrics
                if let Some(data) = &mut self.dashboard_data {
                    data.metrics.network = network;
                    data.timestamp = timestamp;
                }
            },
            DashboardUpdate::DiskUpdate { disk, timestamp } => {
                // Update disk metrics
                if let Some(data) = &mut self.dashboard_data {
                    data.metrics.disk = disk;
                    data.timestamp = timestamp;
                }
            },
            DashboardUpdate::AcknowledgeAlert { alert_id, acknowledged_by, timestamp } => {
                // Mark alert as acknowledged
                if let Some(data) = &mut self.dashboard_data {
                    if let Some(alert) = data.alerts.iter_mut().find(|a| a.id == alert_id) {
                        alert.acknowledged_by = Some(acknowledged_by);
                        alert.acknowledged_at = Some(timestamp);
                    }
                }
            },
            DashboardUpdate::ConfigUpdate { config: _ } => {
                // Update dashboard configuration
                // This would update the config of our app if needed
            },
        }
    }

    /// Update dashboard data completely
    pub fn update_dashboard_data(&mut self, data: DashboardData) {
        // Reset the MetricsWidget and HealthWidget update times to force a refresh
        if let Some(idx) = self.tab_index_by_name("System") {
            self.mark_widget_updated(idx);
        }
        if let Some(idx) = self.tab_index_by_name("Health") {
            self.mark_widget_updated(idx);
        }
        
        // Log to file for debugging
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("dashboard_debug.log") {
                
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(file, "[{}] App updating dashboard data: CPU {:.1}%, Memory {:.1}/{:.1} GB, {} alerts",
                           timestamp,
                           data.metrics.cpu.usage,
                           data.metrics.memory.used as f64 / (1024.0 * 1024.0 * 1024.0),
                           data.metrics.memory.total as f64 / (1024.0 * 1024.0 * 1024.0),
                           data.alerts.len());
        }
        
        // Debug logging to console (may not be visible in TUI mode)
        println!("App updating dashboard data: CPU {}%, Memory {}/{} bytes",
                data.metrics.cpu.usage,
                data.metrics.memory.used,
                data.metrics.memory.total);
        
        self.dashboard_data = Some(data);
        self.last_update = Some(Utc::now());
    }

    /// Update the app's health checks from dashboard data
    /// 
    /// This is currently a placeholder for future implementation that will
    /// process health check data more extensively.
    #[allow(dead_code)]
    fn update_health_checks(&mut self, _data: &DashboardData) {
        // Implementation will be added in a future update
    }

    /// Update time series data from dashboard metrics
    /// 
    /// This method populates historical data from the latest dashboard update.
    /// It's currently used as a reference implementation for future updates.
    #[allow(dead_code)]
    fn update_time_series(&mut self, data: &DashboardData) {
        // Add CPU usage to time series
        let now = Utc::now();
        
        // CPU usage
        let cpu_series = self.time_series.entry(MetricType::CpuUsage).or_default();
        cpu_series.push((now, data.metrics.cpu.usage));
        
        // Memory usage
        let memory_used_percent = data.metrics.memory.used as f64 / data.metrics.memory.total as f64 * 100.0;
        let memory_series = self.time_series.entry(MetricType::MemoryUsage).or_default();
        memory_series.push((now, memory_used_percent));
        
        // If we have too many points, remove oldest ones
        const MAX_POINTS: usize = 100;
        
        for series in self.time_series.values_mut() {
            if series.len() > MAX_POINTS {
                *series = series.iter().skip(series.len() - MAX_POINTS).cloned().collect();
            }
        }
    }
    
    /// Get a reference to the dashboard data
    pub fn dashboard_data(&self) -> Option<&DashboardData> {
        self.dashboard_data.as_ref()
    }

    /// Handle UI tick for animations
    pub fn on_tick(&mut self) {
        // Update widget managers
        for widget in &mut self.widget_managers {
            widget.tick();
        }
        
        // Currently we don't need to track ticks for animations
        // This would be implemented if we added animations
    }

    /// Render the app to the terminal frame
    pub fn render_to_frame(&self, f: &mut ratatui::Frame) {
        // Add debug information to log file
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open("dashboard_debug.log") {
                
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            
            if let Some(data) = &self.dashboard_data {
                let _ = writeln!(file, "[{}] Rendering frame WITH dashboard data: CPU {:.1}%, Memory {:.1}/{:.1} GB, {} alerts",
                         timestamp,
                         data.metrics.cpu.usage,
                         data.metrics.memory.used as f64 / (1024.0 * 1024.0 * 1024.0),
                         data.metrics.memory.total as f64 / (1024.0 * 1024.0 * 1024.0),
                         data.alerts.len());
            } else {
                let _ = writeln!(file, "[{}] Rendering frame WITHOUT dashboard data", timestamp);
            }
        }
        
        // If help is being shown, render the help screen
        if self.show_help {
            let ui_app: &ui::UiApp = ui::convert_app_ref(self);
            ui::draw_help(f, ui_app);
            return;
        }
        
        // Create a basic UI layout
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Min(3),  // Title
                ratatui::layout::Constraint::Min(1),  // Tabs
                ratatui::layout::Constraint::Min(10), // Content
                ratatui::layout::Constraint::Min(1),  // Status
            ])
            .split(f.size());
        
        // Render title bar
        let title = ratatui::widgets::Paragraph::new(format!("{} - Dashboard", self.title))
            .alignment(ratatui::layout::Alignment::Center)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
        f.render_widget(title, chunks[0]);
        
        // If we have dashboard data, show it
        if let Some(data) = &self.dashboard_data {
            // Create main content display
            let content = ratatui::widgets::Paragraph::new(vec![
                ratatui::text::Line::from(format!("CPU: {:.1}%", data.metrics.cpu.usage)),
                ratatui::text::Line::from(format!("Memory: {:.1} GB / {:.1} GB", 
                    data.metrics.memory.used as f64 / (1024.0 * 1024.0 * 1024.0),
                    data.metrics.memory.total as f64 / (1024.0 * 1024.0 * 1024.0))),
                ratatui::text::Line::from(format!("Disk: {:.1}% used", 
                    data.metrics.disk.usage.values().next().map_or(0.0, |v| v.used_percentage))),
                ratatui::text::Line::from(format!("Protocol: {}", data.protocol.status)),
                ratatui::text::Line::from(format!("Alerts: {}", data.alerts.len())),
                ratatui::text::Line::from(format!("Last Update: {}", data.timestamp))
            ])
            .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title("System Overview"));
            
            f.render_widget(content, chunks[2]);
        } else {
            // Show a message if no data is available
            let content = ratatui::widgets::Paragraph::new("No dashboard data available")
                .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL).title("System Overview"));
            f.render_widget(content, chunks[2]);
        }
        
        // Status bar with help text
        let status = ratatui::widgets::Paragraph::new("[q] Quit  [h] Help")
            .alignment(ratatui::layout::Alignment::Right);
        f.render_widget(status, chunks[3]);
    }

    /// Handle dashboard data update
    pub fn on_dashboard_update(&mut self, data: DashboardData) {
        self.dashboard_data = Some(data);
    }

    /// Toggle alerts panel visibility
    pub fn toggle_alerts(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Handle keyboard input
    pub fn on_key(&mut self, key: KeyCode) {
        // First, check for global shortcuts
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char('h') => self.show_help = !self.show_help,
            KeyCode::Tab => self.cycle_selected_tab(),
            KeyCode::BackTab => self.prev_tab(),
            KeyCode::Left => self.prev_tab(),
            KeyCode::Right => self.cycle_selected_tab(),
            KeyCode::Char('1') => self.select_tab(0),
            KeyCode::Char('2') => self.select_tab(1),
            KeyCode::Char('3') => self.select_tab(2),
            KeyCode::Char('4') => self.select_tab(3),
            KeyCode::Char('g') => self.ui_state.layout = ui::WidgetLayout::Grid,
            KeyCode::Char('v') => self.ui_state.layout = ui::WidgetLayout::Vertical,
            KeyCode::Char('H') => self.ui_state.layout = ui::WidgetLayout::Horizontal,
            KeyCode::Char('f') => self.ui_state.layout = ui::WidgetLayout::Focused(self.ui_state.selected_tab),
            _ => {
                // If not a global shortcut, pass to the active widget manager
                if let Some(manager) = self.widget_managers.get_mut(self.ui_state.selected_tab) {
                    manager.handle_key(key);
                }
            }
        }
    }

    /// Handle mouse events
    pub fn on_mouse(&mut self, event: MouseEvent) {
        // Pass mouse events to the active widget manager
        if let Some(manager) = self.widget_managers.get_mut(self.ui_state.selected_tab) {
            manager.handle_mouse(event);
        }
    }

    /// Handle window resize
    pub fn on_resize(&mut self, _width: u16, _height: u16) {
        // Store the new dimensions if needed
        // This is a placeholder for future window size-dependent features
    }

    /// Get the index of a tab by name
    fn tab_index_by_name(&self, name: &str) -> Option<usize> {
        match self.active_tab {
            ActiveTab::Overview if name == "Overview" => Some(0),
            ActiveTab::System if name == "System" => Some(1),
            ActiveTab::Network if name == "Network" => Some(2),
            ActiveTab::Protocol if name == "Protocol" => Some(3),
            ActiveTab::Alerts if name == "Alerts" => Some(4),
            ActiveTab::Tools if name == "Tools" => Some(5),
            _ => None,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            dashboard_data: None,
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            last_update: None,
            running: true,
            ui_state: UiState::default(),
            widget_managers: Vec::new(),
            title: "Squirrel UI".to_string(),
            config: Config::default(),
            help_system: Arc::new(HelpSystem::new()),
            widget_update_times: Vec::new(),
            last_full_refresh: Instant::now(),
            full_refresh_interval: Duration::from_secs(10),
        }
    }
}

impl App {
    pub fn tick_timestamp(&mut self) -> Instant {
        Instant::now()
    }

    /// Render the app to the terminal
    /// 
    /// This method is kept for compatibility with future rendering implementations
    /// that might need direct access to the terminal.
    #[allow(dead_code)]
    fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), io::Error> {
        let ui_app: &mut ui::UiApp = ui::convert_app_mut(self);
        ui::draw(terminal, ui_app)
    }

    /// Determine which widgets need to be updated in the current frame
    /// 
    /// This is an optimization method that identifies which widgets have changed
    /// and need to be redrawn, to avoid unnecessary rendering operations.
    #[allow(dead_code)]
    fn widgets_needing_update(&self) -> HashSet<usize> {
        let mut needs_update = HashSet::new();
        
        // Always update the active tab
        match self.active_tab {
            ActiveTab::Overview => { needs_update.insert(0); }
            ActiveTab::System => { needs_update.insert(1); }
            ActiveTab::Network => { needs_update.insert(2); }
            ActiveTab::Protocol => { needs_update.insert(3); }
            ActiveTab::Alerts => { needs_update.insert(4); }
            ActiveTab::Tools => { needs_update.insert(5); }
        }
        
        // If there's recent data, update all widgets
        if self.last_update.is_some() {
            // In a real implementation, we'd check how recent the update is
            for idx in 0..self.widget_update_times.len() {
                needs_update.insert(idx);
            }
        }
        
        needs_update
    }
    
    /// Mark a widget as updated
    pub fn mark_widget_updated(&mut self, idx: usize) {
        if idx < self.widget_update_times.len() {
            self.widget_update_times[idx] = Instant::now();
        }
    }
    
    /// Perform a full refresh
    pub fn force_full_refresh(&mut self) {
        self.last_full_refresh = Instant::now();
    }
} 