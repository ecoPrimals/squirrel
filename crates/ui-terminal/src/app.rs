use std::collections::HashMap;
use std::sync::Arc;
use std::io;

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
}

impl App {
    /// Create a new app
    pub fn new() -> Self {
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
        }
    }
    
    /// Create a new app with custom config
    pub fn with_config(
        title: String,
        config: Config,
        help_system: Arc<HelpSystem>,
        widget_managers: Vec<Box<dyn WidgetManager>>,
    ) -> Self {
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
        terminal.draw(|f| {
            ui::draw(
                f,
                &self.title,
                &self.ui_state,
                &self.widget_managers,
                self.dashboard_data.as_ref(),
            )
        })?;
        
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
        // Update app with new dashboard data
        self.update_health_checks(&data);
        self.update_time_series(&data);
        
        // Update widgets if any
        for widget in &mut self.widget_managers {
            widget.update(&data);
        }
        
        self.dashboard_data = Some(data);
        self.last_update = Some(Utc::now());
    }

    /// Update health checks based on dashboard data
    fn update_health_checks(&mut self, _data: &DashboardData) {
        // Implementation that updates health_checks from data
        // This is handled in ui.rs currently
    }

    /// Update time series data based on dashboard data
    fn update_time_series(&mut self, data: &DashboardData) {
        // Add CPU usage to time series
        let now = Utc::now();
        
        // CPU usage
        let cpu_series = self.time_series.entry(MetricType::CpuUsage).or_insert_with(Vec::new);
        cpu_series.push((now, data.metrics.cpu.usage));
        
        // Memory usage
        let memory_used_percent = data.metrics.memory.used as f64 / data.metrics.memory.total as f64 * 100.0;
        let memory_series = self.time_series.entry(MetricType::MemoryUsage).or_insert_with(Vec::new);
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

    /// Performs a tick update for animations and time-based updates
    pub fn tick(&mut self) {
        // Update any time-based animations or data
        // This is called regularly by the main loop
    }

    /// Render the app to the terminal frame
    pub fn render_to_frame(&self, f: &mut ratatui::Frame) {
        // If help is being shown, render the help screen
        if self.show_help {
            ui::draw_help(f, &self.help_system);
            return;
        }
        
        // Otherwise render the normal UI
        ui::draw(
            f,
            &self.title,
            &self.ui_state,
            &self.widget_managers,
            self.dashboard_data.as_ref(),
        );
    }

    /// Handle UI tick for animations
    pub fn on_tick(&mut self) {
        // Update all widget managers
        for widget in &mut self.widget_managers {
            widget.tick();
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
        }
    }
}

impl App {
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
    pub fn on_resize(&mut self, width: u16, height: u16) {
        // Store the new dimensions if needed
        // This is a placeholder for future window size-dependent features
    }
} 