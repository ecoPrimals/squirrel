use std::collections::HashMap;
use std::sync::Arc;
use std::io;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use std::io::Write;
use futures::executor::block_on;

use dashboard_core::{
    DashboardData, Metrics, CpuMetrics, MemoryMetrics, NetworkMetrics, DiskMetrics, Alert, ProtocolData,
    MetricType,
    update::DashboardUpdate,
    mcp::McpMetrics,
};
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    backend::Backend, 
    Terminal,
    widgets::{Paragraph, Block, Borders},
    layout::Alignment,
    Frame
};

use crate::help::HelpSystem;
use crate::ui::{self, UiState, ActiveTab};
use crate::config::Config;
use crate::widget_manager::WidgetManager;
use crate::widgets::health::HealthCheck;
use crate::adapter::{McpMetricsProviderTrait, ConnectionHealth, ConnectionEvent, ConnectionStatus, McpMetricsConfig, RealMcpMetricsProvider, create_mcp_metrics_provider};
use crate::widgets::{self, Widget, HealthWidget, ConnectionHealthWidget, MetricsWidget, ChartWidget, NetworkWidget, AlertsWidget};
use crate::mcp_client_wrapper::ClientWrapper;
use crate::{
    adapter::{AdapterConfig, ConnectionEvent, ConnectionEventType, ConnectionHealth, ConnectionStatus, McpAdapterError},
    theme::Theme,
    ui::UiApp,
    widgets::*,
};
use log::{debug, error, info, warn};
use tokio::sync::{mpsc, Mutex, RwLock};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;
use uuid::Uuid;

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
    /// Metric history for charts
    pub metric_history: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
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
    /// MCP metrics provider
    pub mcp_metrics_provider: Option<Arc<dyn McpMetricsProviderTrait>>,
    /// Client wrapper for metrics
    pub client_wrapper: ClientWrapper,
    /// Last MCP metrics update time
    pub last_mcp_update: Option<Instant>,
    /// MCP update interval
    pub mcp_update_interval: Duration,
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
            metric_history: HashMap::new(),
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
            mcp_metrics_provider: None,
            client_wrapper: ClientWrapper::default(),
            last_mcp_update: None,
            mcp_update_interval: Duration::from_millis(1000), // Update MCP metrics every second
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
            metric_history: HashMap::new(),
            last_update: None,
            widget_update_times: vec![now; 6], // One for each tab
            last_full_refresh: now,
            full_refresh_interval: Duration::from_secs(10), // Full refresh every 10 seconds
            mcp_metrics_provider: None,
            client_wrapper: ClientWrapper::default(),
            last_mcp_update: None,
            mcp_update_interval: Duration::from_millis(1000), // Update MCP metrics every second
        }
    }
    
    /// Initialize the MCP metrics provider
    pub fn init_mcp_metrics_provider(&mut self, mcp_config: McpMetricsConfig) {
        // Use the factory function to create the provider
        let provider = create_mcp_metrics_provider(mcp_config);
        self.mcp_metrics_provider = Some(Arc::new(provider));
    }
    
    /// Update MCP metrics if needed
    pub async fn update_mcp_metrics(&mut self) {
        // Check if we need to update MCP metrics
        let should_update = match self.last_mcp_update {
            Some(last_update) => last_update.elapsed() >= self.mcp_update_interval,
            None => true,
        };
        
        if !should_update {
            return;
        }
        
        // Update MCP metrics if provider exists
        if let Some(provider) = &self.mcp_metrics_provider {
            match provider.get_metrics().await {
                Ok(mcp_metrics) => {
                    // Update dashboard protocol data if dashboard data exists
                    if let Some(dashboard_data) = &mut self.dashboard_data {
                        // Ensure protocol data exists
                        if dashboard_data.protocol.is_none() {
                            dashboard_data.protocol = Some(ProtocolData::default());
                        }
                        if let Some(protocol) = &mut dashboard_data.protocol {
                            // Convert MCP metrics to dashboard protocol data format
                            protocol.metrics.insert("request_count".to_string(), mcp_metrics.message_stats.total_requests as f64);
                            protocol.metrics.insert("response_count".to_string(), mcp_metrics.message_stats.total_responses as f64);
                            protocol.metrics.insert("request_rate".to_string(), mcp_metrics.message_stats.request_rate);
                            protocol.metrics.insert("response_rate".to_string(), mcp_metrics.message_stats.response_rate);
                            protocol.metrics.insert("transaction_count".to_string(), mcp_metrics.transaction_stats.total_transactions as f64);
                            protocol.metrics.insert("transaction_rate".to_string(), mcp_metrics.transaction_stats.transaction_rate);
                            protocol.metrics.insert("success_rate".to_string(), mcp_metrics.transaction_stats.success_rate);
                            protocol.metrics.insert("error_count".to_string(), mcp_metrics.error_stats.total_errors as f64);
                            protocol.metrics.insert("error_rate".to_string(), mcp_metrics.error_stats.error_rate);
                            protocol.metrics.insert("average_latency".to_string(), mcp_metrics.latency_stats.average_latency_ms);
                            
                            // Update protocol status
                            let connection_status = provider.get_connection_status().await.unwrap_or(ConnectionStatus::Disconnected);
                            protocol.connected = matches!(connection_status, ConnectionStatus::Connected);
                            protocol.status = connection_status.to_string();
                        }
                        
                        // Update timestamp
                        dashboard_data.timestamp = Utc::now();
                        
                        // Update metric history
                        let timestamp = Utc::now();
                        
                        // Update request rate history
                        let request_rate_history = self.metric_history
                            .entry("protocol.request_rate".to_string())
                            .or_insert_with(Vec::new);
                        request_rate_history.push((timestamp, mcp_metrics.message_stats.request_rate));
                        if request_rate_history.len() > 100 {
                            request_rate_history.remove(0);
                        }
                        
                        // Update response rate history
                        let response_rate_history = self.metric_history
                            .entry("protocol.response_rate".to_string())
                            .or_insert_with(Vec::new);
                        response_rate_history.push((timestamp, mcp_metrics.message_stats.response_rate));
                        if response_rate_history.len() > 100 {
                            response_rate_history.remove(0);
                        }
                        
                        // Update transaction rate history
                        let transaction_rate_history = self.metric_history
                            .entry("protocol.transaction_rate".to_string())
                            .or_insert_with(Vec::new);
                        transaction_rate_history.push((timestamp, mcp_metrics.transaction_stats.transaction_rate));
                        if transaction_rate_history.len() > 100 {
                            transaction_rate_history.remove(0);
                        }
                        
                        // Update error rate history
                        let error_rate_history = self.metric_history
                            .entry("protocol.error_rate".to_string())
                            .or_insert_with(Vec::new);
                        error_rate_history.push((timestamp, mcp_metrics.error_stats.error_rate));
                        if error_rate_history.len() > 100 {
                            error_rate_history.remove(0);
                        }
                        
                        // Update latency history
                        let latency_history = self.metric_history
                            .entry("protocol.latency".to_string())
                            .or_insert_with(Vec::new);
                        latency_history.push((timestamp, mcp_metrics.latency_stats.average_latency_ms));
                        if latency_history.len() > 100 {
                            latency_history.remove(0);
                        }
                    }
                },
                Err(e) => {
                    // Log error but continue
                    log::error!("Failed to get MCP metrics: {}", e);
                }
            }
            
            // Update last MCP update time
            self.last_mcp_update = Some(Instant::now());
        }
    }
    
    /// Get metric history for a specific metric
    pub fn get_metric_history(&self, metric_name: &str) -> Option<&[(DateTime<Utc>, f64)]> {
        self.metric_history.get(metric_name).map(|v| v.as_slice())
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
            KeyCode::Char('r') => {
                // Trigger MCP reconnect if on Protocol tab
                if self.active_tab == ActiveTab::Protocol {
                    if let Some(provider) = &self.mcp_metrics_provider {
                        // Use tokio spawn to avoid blocking the UI
                        let provider_clone = provider.clone();
                        tokio::spawn(async move {
                            let _ = provider_clone.reconnect().await;
                        });
                    }
                }
            },
            _ => {}
        }
        
        Ok(true)
    }
    
    /// Render app to terminal
    pub fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        ui::draw_app(terminal, self)
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
            DashboardUpdate::MetricsUpdate { metrics, timestamp } => {
                if let Some(data) = &mut self.dashboard_data {
                    // Update specific metrics based on the map
                    if let Some(cpu_usage) = metrics.get(&MetricType::CpuUsage) { data.metrics.cpu.usage = *cpu_usage; }
                    if let Some(mem_usage) = metrics.get(&MetricType::MemoryUsage) { data.metrics.memory.used = (*mem_usage * data.metrics.memory.total as f64 / 100.0) as u64; }
                    // ... update other specific metrics as needed ...
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
    pub async fn update_dashboard_data(&mut self, data: DashboardData) {
        if let Some(provider) = &self.mcp_metrics_provider {
            let mut dashboard_data = self.dashboard_data.write().await;

            // Update core metrics
            match provider.get_metrics().await {
                Ok(metrics) => dashboard_data.metrics = metrics,
                Err(e) => error!("Failed to get metrics: {}", e),
            }

            // Update protocol data
            match provider.get_protocol_data().await {
                Ok(protocol_data) => {
                    // Check if existing protocol data needs initialization or update
                    if dashboard_data.protocol_data.is_none() { // Check Option
                        info!("Initializing protocol data");
                        dashboard_data.protocol_data = Some(protocol_data); // Assign Some(ProtocolData)
                    } else {
                        // Update existing data if necessary (e.g., merge counts)
                        if let Some(existing_protocol) = &mut dashboard_data.protocol_data { // Access Option correctly
                             *existing_protocol = protocol_data; // Replace with new data (or merge logic)
                        }
                    }
                }
                Err(e) => {
                     error!("Failed to get protocol data: {}", e);
                     // Optionally clear protocol data on error or leave stale?
                     // dashboard_data.protocol_data = None;
                }
            }

            // Update connection status
            let status = provider.get_connection_status().await;
            dashboard_data.connection_status = status;

            // Update connection health
            match provider.get_connection_health().await {
                Ok(health) => dashboard_data.connection_health = Some(health), // Store as Option
                Err(e) => {
                    error!("Failed to get connection health: {}", e);
                    dashboard_data.connection_health = None;
                }
            }

            // Update connection history
            match provider.get_connection_history().await {
                Ok(history) => dashboard_data.connection_history = history,
                Err(e) => error!("Failed to get connection history: {}", e),
            }

            // Update historical metrics (Example for CPU)
            // Directly update the metric field, no HashMap lookup needed here
            let cpu_usage = dashboard_data.metrics.cpu.usage;
            self.metric_history
                .entry("CPU Usage".to_string())
                .or_default()
                .push_back((Utc::now(), cpu_usage));
            // Trim history if needed
             if let Some(history) = self.metric_history.get_mut("CPU Usage") {
                 while history.len() > self.config.metrics_history_size.unwrap_or(100) { // Use config value
                     history.pop_front();
                 }
             }

            // Update historical metrics (Example for Memory)
            // Directly update the metric field
            let memory_usage_bytes = dashboard_data.metrics.memory.used as f64;
            let memory_total_bytes = dashboard_data.metrics.memory.total as f64;
            let memory_usage_percent = if memory_total_bytes > 0.0 {
                 (memory_usage_bytes / memory_total_bytes) * 100.0
             } else {
                 0.0
             };
            self.metric_history
                .entry("Memory Usage".to_string())
                .or_default()
                .push_back((Utc::now(), memory_usage_percent));
             // Trim history
            if let some(history) = self.metric_history.get_mut("Memory Usage") {
                while history.len() > self.config.metrics_history_size.unwrap_or(100) {
                    history.pop_front();
                }
            }

            // Update other historical metrics (Network, Disk) similarly...

        } else {
            debug!("MCP metrics provider not available.");
            // Handle case where provider is None (e.g., clear data or show defaults)
            let mut dashboard_data = self.dashboard_data.write().await;
            dashboard_data.connection_status = ConnectionStatus::Disconnected; // Indicate disconnected
            dashboard_data.protocol_data = None;
            dashboard_data.connection_health = None;
        }
        self.last_update = Some(Instant::now());
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

    /// Create and get a connection health widget
    /// This version returns a widget that owns its data to avoid lifetime issues
    pub fn get_connection_health_widget(&self) -> Option<ConnectionHealthWidget> {
        let provider = self.mcp_metrics_provider.as_ref()?;
        
        // Get connection health using block_on
        let connection_health = match block_on(provider.get_connection_health()) {
            Ok(health) => health,
            Err(e) => {
                log::error!("Failed to get connection health: {}", e);
                return None;
            }
        };
        
        // Get connection history using block_on
        let connection_events = match block_on(provider.get_connection_history()) {
            Ok(events) => events,
            Err(e) => {
                log::error!("Failed to get connection history: {}", e);
                return None;
            }
        };
        
        // Get metric history (remains sync)
        let history_metrics = self.metric_history
            .get("connection_health") // This key might need adjustment
            .cloned()
            .unwrap_or_default();
        
        // Create widget with owned data
        let widget = ConnectionHealthWidget::new()
            .with_title("Connection Health")
            .with_health_owned(connection_health)
            .with_history_owned(connection_events)
            .with_history_metrics_owned(history_metrics);
        
        Some(widget)
    }

    /// Draw the app UI 
    pub fn draw(&mut self, f: &mut Frame) {
        // Create a UiApp snapshot for drawing
        let ui_app = self.create_ui_app_snapshot(); // Assume this function exists

        // Main layout
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.size());

        self.draw_header(f, chunks[0], &ui_app);
        self.draw_main(f, chunks[1], &ui_app); // Pass UiApp
        self.draw_footer(f, chunks[2], &ui_app);

        // Draw popups if active
        if self.show_help {
             // Need to implement or integrate help drawing logic
             // Example: Draw centered popup
             let area = centered_rect(60, 80, f.size());
             f.render_widget(Clear, area); // Clear background
             let help_block = Block::default().title("Help - Keybinds").borders(Borders::ALL);
             // TODO: Get keybinds and render them inside help_block
             f.render_widget(help_block, area);
        }
        // Add other popups like error messages...
    }

    // Placeholder for snapshot function - needs implementation
    fn create_ui_app_snapshot(&self) -> UiApp {
        // This function needs to read relevant fields from App (self)
        // and construct a UiApp instance.
        // It might need read locks on dashboard_data, etc.
        // Example:
        // let dashboard_data = futures::executor::block_on(self.dashboard_data.read()); // Use block_on carefully or make snapshot async
        UiApp {
            // Populate fields from self and dashboard_data
            active_tab: self.active_tab.clone(), // Example
            ui_state: self.ui_state.clone(), // Example
            // ... other fields ...
            // dashboard_data: dashboard_data.clone(), // Need careful cloning/locking
            is_loading: self.is_loading,
            last_update: self.last_update,
            // ... etc
            metrics_history: self.metric_history.clone(), // Clone history
        }
    }

    // Refactored drawing helpers to accept UiApp
    fn draw_header(&self, f: &mut Frame, area: Rect, ui_app: &UiApp) {
        // Draw header content using ui_app data
        // ... implementation ...
    }

    fn draw_main(&self, f: &mut Frame, area: Rect, ui_app: &UiApp) {
        // Draw main content based on active tab using ui_app data
        match ui_app.active_tab {
            // Call specific drawing functions for each tab
            ActiveTab::Dashboard => crate::ui::draw_dashboard(f, area, ui_app),
            ActiveTab::Network => crate::ui::draw_network(f, area, ui_app),
            ActiveTab::Alerts => crate::ui::draw_alerts(f, area, ui_app),
            ActiveTab::Settings => crate::ui::draw_settings(f, area, ui_app),
        }
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect, ui_app: &UiApp) {
        // Draw footer content using ui_app data
        // ... implementation ...
    }
}

// Helper function for centered rect (example)
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

impl Default for App {
    fn default() -> Self {
        Self {
            dashboard_data: None,
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            metric_history: HashMap::new(),
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
            mcp_metrics_provider: None,
            client_wrapper: ClientWrapper::default(),
            last_mcp_update: None,
            mcp_update_interval: Duration::from_millis(1000),
        }
    }
}

impl App {
    pub fn tick_timestamp(&mut self) -> Instant {
        Instant::now()
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

// Implement Clone for App
impl Clone for App {
    fn clone(&self) -> Self {
        // Create a new App with default values
        let mut new_app = App::new();
        
        // Copy simple fields
        new_app.title = self.title.clone();
        new_app.config = self.config.clone();
        new_app.help_system = self.help_system.clone();
        new_app.dashboard_data = self.dashboard_data.clone();
        new_app.active_tab = self.active_tab.clone();
        new_app.show_help = self.show_help;
        new_app.health_checks = self.health_checks.clone();
        new_app.time_series = self.time_series.clone();
        new_app.metric_history = self.metric_history.clone();
        new_app.last_update = self.last_update.clone();
        new_app.ui_state = self.ui_state.clone();
        // widget_managers is not cloned as it contains Box<dyn Trait>
        new_app.running = self.running;
        new_app.widget_update_times = self.widget_update_times.clone();
        new_app.last_full_refresh = self.last_full_refresh;
        new_app.full_refresh_interval = self.full_refresh_interval;
        new_app.mcp_metrics_provider = self.mcp_metrics_provider.clone();
        new_app.client_wrapper = self.client_wrapper.clone();
        new_app.last_mcp_update = self.last_mcp_update.clone();
        new_app.mcp_update_interval = self.mcp_update_interval;
        
        new_app
    }
}

/// Render help screen
pub fn render_help(f: &mut Frame) {
    let area = f.size();
    let paragraph = Paragraph::new("HELP: Press 'q' to quit, 'h' for help, 'tab' to switch tabs")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(paragraph, area);
}

/// Draw help screen
pub fn draw_help(f: &mut Frame) {
    render_help(f);
} 