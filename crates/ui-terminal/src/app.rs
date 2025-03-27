use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use dashboard_core::{
    DashboardData,
    DashboardUpdate,
};
use chrono::{DateTime, Utc};

/// Application state for the Terminal UI
pub struct App {
    /// Dashboard data
    dashboard_data: Option<DashboardData>,
    
    /// Currently selected tab
    selected_tab: usize,
    
    /// Available tabs
    tabs: Vec<String>,
    
    /// Whether the application should quit
    should_quit: bool,
    
    /// Whether help is visible
    show_help: bool,
    
    /// Last tick time for animations
    last_tick: Instant,
    
    /// Dashboard component scroll positions
    scroll_positions: ScrollPositions,
    
    /// Whether dashboard is being updated
    is_updating: bool,
    
    /// Last update time
    last_update: Option<Instant>,
    
    /// Metric history data
    metric_history: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
}

/// Scroll positions for dashboard components
#[derive(Default)]
pub struct ScrollPositions {
    /// Alerts scroll position
    pub alerts: usize,
    
    /// System metrics scroll position
    pub system: usize,
    
    /// Protocol metrics scroll position
    pub protocol: usize,
    
    /// Tool metrics scroll position
    pub tools: usize,
    
    /// Network metrics scroll position
    pub network: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            dashboard_data: None,
            selected_tab: 0,
            tabs: vec![
                "Overview".to_string(),
                "System".to_string(),
                "Protocol".to_string(),
                "Tools".to_string(),
                "Alerts".to_string(),
                "Network".to_string(),
            ],
            should_quit: false,
            show_help: false,
            last_tick: Instant::now(),
            scroll_positions: ScrollPositions::default(),
            is_updating: false,
            last_update: None,
            metric_history: HashMap::new(),
        }
    }
    
    /// Update the dashboard data
    pub fn update_dashboard_data(&mut self, data: DashboardData) {
        self.dashboard_data = Some(data);
        self.last_update = Some(Instant::now());
        self.is_updating = false;
    }
    
    /// Handle a dashboard update
    pub fn handle_update(&mut self, update: DashboardUpdate) {
        match update {
            DashboardUpdate::FullUpdate(data) => {
                self.update_dashboard_data(data);
            },
            DashboardUpdate::MetricsUpdate { metrics, timestamp } => {
                if let Some(data) = &mut self.dashboard_data {
                    for (k, v) in metrics {
                        data.metrics.values.insert(k.clone(), v);
                        
                        // Also update history
                        let history = self.metric_history
                            .entry(k)
                            .or_insert_with(Vec::new);
                            
                        history.push((timestamp, v));
                        
                        // Limit history to 1000 points
                        if history.len() > 1000 {
                            history.remove(0);
                        }
                    }
                    data.timestamp = timestamp;
                    self.last_update = Some(Instant::now());
                }
            },
            DashboardUpdate::SystemUpdate { system, timestamp } => {
                if let Some(data) = &mut self.dashboard_data {
                    // Update CPU history
                    let cpu_history = self.metric_history
                        .entry("system.cpu".to_string())
                        .or_insert_with(Vec::new);
                        
                    cpu_history.push((timestamp, system.cpu_usage));
                    
                    // Limit history to 1000 points
                    if cpu_history.len() > 1000 {
                        cpu_history.remove(0);
                    }
                    
                    // Update memory history
                    let memory_history = self.metric_history
                        .entry("system.memory".to_string())
                        .or_insert_with(Vec::new);
                        
                    memory_history.push((timestamp, system.memory_used as f64));
                    
                    // Limit history to 1000 points
                    if memory_history.len() > 1000 {
                        memory_history.remove(0);
                    }
                    
                    data.system = system;
                    data.timestamp = timestamp;
                    self.last_update = Some(Instant::now());
                }
            },
            DashboardUpdate::NetworkUpdate { network, timestamp } => {
                if let Some(data) = &mut self.dashboard_data {
                    data.network = network;
                    data.timestamp = timestamp;
                    self.last_update = Some(Instant::now());
                }
            },
            DashboardUpdate::AlertUpdate { alert, timestamp } => {
                if let Some(data) = &mut self.dashboard_data {
                    // Find and update the alert if it exists, otherwise add it
                    let found = data.alerts.active.iter_mut().any(|a| {
                        if a.id == alert.id {
                            *a = alert.clone();
                            true
                        } else {
                            false
                        }
                    });
                    
                    if !found {
                        data.alerts.active.push(alert);
                    }
                    
                    data.timestamp = timestamp;
                    self.last_update = Some(Instant::now());
                }
            },
            DashboardUpdate::ConfigUpdate { config: _ } => {
                // Handle config update if needed
            },
            DashboardUpdate::AcknowledgeAlert { alert_id: _, acknowledged_by: _, timestamp: _ } => {
                // This is usually handled by the service, not the UI
            },
        }
    }
    
    /// Tick for animations
    pub fn tick(&mut self) {
        self.last_tick = Instant::now();
    }
    
    /// Handle terminal events
    pub fn handle_event(&mut self, key_event: KeyEvent) -> bool {
        self.handle_key_event(key_event)
    }
    
    /// Handle mouse events
    pub fn handle_mouse(&mut self, _event: crossterm::event::MouseEvent) -> bool {
        // For now, just ignore mouse events
        true
    }
    
    /// Handle terminal resize events
    pub fn handle_resize(&mut self, _width: u16, _height: u16) -> bool {
        // For now, just acknowledge the resize
        true
    }
    
    /// Handle key events
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
        match key_event.code {
            // Quit application (q or Ctrl+c)
            KeyCode::Char('q') => {
                self.should_quit = true;
                false
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                false
            }
            
            // Tab navigation (1-6 or Tab/Shift+Tab)
            KeyCode::Char('1') => {
                self.selected_tab = 0;
                true
            }
            KeyCode::Char('2') => {
                self.selected_tab = 1;
                true
            }
            KeyCode::Char('3') => {
                self.selected_tab = 2;
                true
            }
            KeyCode::Char('4') => {
                self.selected_tab = 3;
                true
            }
            KeyCode::Char('5') => {
                self.selected_tab = 4;
                true
            }
            KeyCode::Char('6') => {
                self.selected_tab = 5;
                true
            }
            KeyCode::Tab => {
                self.next_tab();
                true
            }
            KeyCode::BackTab => {
                self.previous_tab();
                true
            }
            
            // Scroll content (Up/Down or j/k)
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_down();
                true
            }
            
            // Show/hide help (?)
            KeyCode::Char('?') => {
                self.show_help = !self.show_help;
                true
            }
            
            // Refresh dashboard (r)
            KeyCode::Char('r') => {
                self.is_updating = true;
                true
            }
            
            // Unhandled key
            _ => true,
        }
    }
    
    /// Switch to the next tab
    fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
    }
    
    /// Switch to the previous tab
    fn previous_tab(&mut self) {
        if self.selected_tab > 0 {
            self.selected_tab -= 1;
        } else {
            self.selected_tab = self.tabs.len() - 1;
        }
    }
    
    /// Scroll content up
    fn scroll_up(&mut self) {
        match self.selected_tab {
            0 => (), // Overview - no scrolling
            1 => {
                if self.scroll_positions.system > 0 {
                    self.scroll_positions.system -= 1;
                }
            }
            2 => {
                if self.scroll_positions.protocol > 0 {
                    self.scroll_positions.protocol -= 1;
                }
            }
            3 => {
                if self.scroll_positions.tools > 0 {
                    self.scroll_positions.tools -= 1;
                }
            }
            4 => {
                if self.scroll_positions.alerts > 0 {
                    self.scroll_positions.alerts -= 1;
                }
            }
            5 => {
                if self.scroll_positions.network > 0 {
                    self.scroll_positions.network -= 1;
                }
            }
            _ => (),
        }
    }
    
    /// Scroll content down
    fn scroll_down(&mut self) {
        match self.selected_tab {
            0 => (), // Overview - no scrolling
            1 => self.scroll_positions.system += 1,
            2 => self.scroll_positions.protocol += 1,
            3 => self.scroll_positions.tools += 1,
            4 => self.scroll_positions.alerts += 1,
            5 => self.scroll_positions.network += 1,
            _ => (),
        }
    }
    
    /// Get the dashboard data
    pub fn dashboard_data(&self) -> Option<&DashboardData> {
        self.dashboard_data.as_ref()
    }
    
    /// Get the selected alert index
    pub fn alerts_selected_index(&self) -> Option<usize> {
        if self.selected_tab == 4 {  // Alerts tab
            Some(self.scroll_positions.alerts)
        } else {
            None
        }
    }
    
    /// Get the currently selected tab
    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }
    
    /// Get the available tabs
    pub fn tabs(&self) -> &[String] {
        &self.tabs
    }
    
    /// Get whether help is visible
    pub fn show_help(&self) -> bool {
        self.show_help
    }
    
    /// Get the scroll positions
    pub fn scroll_positions(&self) -> &ScrollPositions {
        &self.scroll_positions
    }
    
    /// Get whether the dashboard is being updated
    pub fn is_updating(&self) -> bool {
        self.is_updating
    }
    
    /// Get the time since the last update
    pub fn time_since_update(&self) -> Option<Duration> {
        self.last_update.map(|t| t.elapsed())
    }
    
    /// Get historical data for a specific metric
    pub fn get_metric_history(&self, metric_name: &str) -> Option<&[(DateTime<Utc>, f64)]> {
        self.metric_history
            .get(metric_name)
            .map(|data| data.as_slice())
    }
} 