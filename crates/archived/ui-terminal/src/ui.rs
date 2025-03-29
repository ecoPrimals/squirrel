use std::collections::{HashMap, HashSet};
use std::io;
use std::time::Duration;

use ratatui::{
    backend::Backend,
    Terminal,
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap, Tabs, Chart, Dataset, Gauge, GraphType, LegendPosition, List, ListItem, Sparkline, StatefulWidget, Axis, Widget},
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    symbols,
};

use dashboard_core::{
    DashboardData,
    MetricType,
    data::{AlertSeverity, ProtocolMetrics},
    Metrics,
};
use squirrel_mcp::client::MCPClientMetrics;

use chrono::{DateTime, Utc};
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use crate::{
    widgets::{
        chart::{ChartWidget, ChartType, NetworkDataType},
        metrics::MetricsWidget,
        network::NetworkWidget,
        health::{HealthWidget, HealthCheck, HealthStatus},
        alerts::AlertsWidget,
        connection_health::ConnectionHealthWidget,
        Widget,
    },
    widget_manager::WidgetManager,
    util::format_bytes,
};

use dashboard_core::DashboardService;
use tokio::time::Instant;
use futures::executor::block_on;
use log::{error, info, warn};
use std::panic;

use crate::{
    app::{App, ActiveTab},
    widgets::{
        AlertsWidget, ChartWidget, ConnectionHealthWidget, HealthWidget, MetricsWidget, NetworkWidget,
    },
};
use crate::adapter::ConnectionStatus;

/// UI state
#[derive(Debug, Clone, Default)]
pub struct UiState {
    /// Selected tab index
    pub selected_tab: usize,
    /// Show help
    pub show_help: bool,
    /// Widget layout
    pub layout: WidgetLayout,
    /// Is running
    pub running: bool,
}

/// Widget layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetLayout {
    /// Grid layout (2x2)
    Grid,
    /// Vertical layout (stacked)
    Vertical,
    /// Horizontal layout (side by side)
    Horizontal,
    /// Focused layout (one widget takes most space)
    Focused(usize),
}

impl Default for WidgetLayout {
    fn default() -> Self {
        Self::Grid
    }
}

/// UI Application wrapper
#[derive(Clone)]
pub struct UiApp {
    /// Dashboard data
    pub dashboard_data: Option<DashboardData>,
    /// Active tab
    pub active_tab: ActiveTab,
    /// Show help
    pub show_help: bool,
    /// Health checks
    pub health_checks: Vec<crate::widgets::health::HealthCheck>,
    /// Time series data
    pub time_series: HashMap<MetricType, Vec<(DateTime<Utc>, f64)>>,
    /// Metric history for charts
    pub metric_history: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
    /// Last update timestamp
    pub last_update: Option<DateTime<Utc>>,
    /// UI state
    pub ui_state: UiState,
    /// Title for the app
    pub title: String,
    /// Set of widgets that need updating
    pub updated_widgets: HashSet<usize>,
    /// Whether to force a refresh
    pub force_refresh: bool,
    /// Last full refresh time
    pub last_full_refresh: std::time::Instant,
}

/// Active tab enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Overview tab
    Overview,
    /// System tab
    System,
    /// Network tab
    Network,
    /// Protocol tab
    Protocol,
    /// Alerts tab
    Alerts,
    /// Tools tab
    Tools,
}

/// Trait for types that can be rendered like an App
pub trait AppLike {
    /// Get the title
    fn title(&self) -> &str;
    
    /// Get the active tab
    fn active_tab(&self) -> &ActiveTab;
    
    /// Check if help should be shown
    fn show_help(&self) -> bool;
    
    /// Get the dashboard data
    fn dashboard_data(&self) -> Option<&DashboardData>;
    
    /// Get the ui state
    fn ui_state(&self) -> &UiState;
    
    /// Get the time series data
    fn time_series(&self) -> &HashMap<MetricType, Vec<(DateTime<Utc>, f64)>>;
    
    /// Get the last update time
    fn last_update(&self) -> Option<&DateTime<Utc>>;
    
    /// Get the metric history
    fn metric_history(&self) -> &HashMap<String, Vec<(DateTime<Utc>, f64)>>;

    /// Get the health checks
    fn health_checks(&self) -> &Vec<HealthCheck>;
}

impl AppLike for crate::app::App {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn active_tab(&self) -> &ActiveTab {
        &self.active_tab
    }
    
    fn show_help(&self) -> bool {
        self.show_help
    }
    
    fn dashboard_data(&self) -> Option<&DashboardData> {
        self.dashboard_data.as_ref()
    }
    
    fn ui_state(&self) -> &UiState {
        &self.ui_state
    }
    
    fn time_series(&self) -> &HashMap<MetricType, Vec<(DateTime<Utc>, f64)>> {
        &self.time_series
    }
    
    fn last_update(&self) -> Option<&DateTime<Utc>> {
        self.last_update.as_ref()
    }
    
    fn metric_history(&self) -> &HashMap<String, Vec<(DateTime<Utc>, f64)>> {
        &self.metric_history
    }

    fn health_checks(&self) -> &Vec<HealthCheck> {
        &self.health_checks
    }
}

impl AppLike for UiApp {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn active_tab(&self) -> &ActiveTab {
        &self.active_tab
    }
    
    fn show_help(&self) -> bool {
        self.show_help
    }
    
    fn dashboard_data(&self) -> Option<&DashboardData> {
        self.dashboard_data.as_ref()
    }
    
    fn ui_state(&self) -> &UiState {
        &self.ui_state
    }
    
    fn time_series(&self) -> &HashMap<MetricType, Vec<(DateTime<Utc>, f64)>> {
        &self.time_series
    }
    
    fn last_update(&self) -> Option<&DateTime<Utc>> {
        self.last_update.as_ref()
    }
    
    fn metric_history(&self) -> &HashMap<String, Vec<(DateTime<Utc>, f64)>> {
        &self.metric_history
    }

    fn health_checks(&self) -> &Vec<HealthCheck> {
        &self.health_checks
    }
}

impl UiApp {
    /// Create a new app
    pub fn new() -> Self {
        // Create a default DashboardData instance using its fields
        let default_data = DashboardData {
            // Remove non-existent fields: hostname, uptime_seconds, tasks, config, system_info, alert_history
            metrics: Metrics::default(), // Assuming Metrics::default() exists
            protocol: ProtocolData::default(), // Assuming ProtocolData::default() exists
            alerts: Vec::new(),
            timestamp: Utc::now(), // Set timestamp
        };

        Self {
            dashboard_data: Some(default_data),
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            last_update: None,
            last_full_refresh: std::time::Instant::now(),
            updated_widgets: HashSet::new(),
            force_refresh: false,
            metric_history: Default::default(),
            title: "Dashboard".to_string(),
            ui_state: UiState::default(),
        }
    }

    /// Handle input events
    pub fn handle_input<B: Backend>(&mut self, _terminal: &mut Terminal<B>) -> Result<bool, std::io::Error> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(false),
                    KeyCode::Char('h') => self.show_help = !self.show_help,
                    KeyCode::Char('1') => self.active_tab = ActiveTab::Overview,
                    KeyCode::Char('2') => self.active_tab = ActiveTab::System,
                    KeyCode::Char('3') => self.active_tab = ActiveTab::Network,
                    KeyCode::Char('4') => self.active_tab = ActiveTab::Protocol,
                    KeyCode::Char('5') => self.active_tab = ActiveTab::Alerts,
                    KeyCode::Char('6') => self.active_tab = ActiveTab::Tools,
                    _ => {}
                }
            }
        }
        Ok(true)
    }

    /// Update app state with new dashboard data
    pub fn update(&mut self, data: DashboardData) {
        // Update health checks
        self.update_health_checks(&data);

        // Update time series data
        self.update_time_series(&data);

        // Store dashboard data
        self.dashboard_data = Some(data);
        self.last_update = Some(Utc::now());
    }

    /// Update health checks based on dashboard data
    fn update_health_checks(&mut self, data: &DashboardData) {
        // Clear existing health checks
        self.health_checks.clear();
        
        // CPU usage health check
        let cpu_usage = data.metrics.cpu.usage;
        let cpu_status = if cpu_usage < 60.0 {
            TerminalDashboardHealthStatus::Healthy
        } else if cpu_usage < 85.0 {
            TerminalDashboardHealthStatus::Warning
        } else {
            TerminalDashboardHealthStatus::Critical
        };
        
        let cpu_message = format!("{:.1}%", cpu_usage);
        
        self.health_checks.push(HealthCheck::new("CPU", crate::widgets::health::HealthStatus::Unknown)
            .with_percentage(cpu_usage)
            .with_status(
                crate::widgets::health::HealthStatus::from(cpu_status),
            )
            .with_message(cpu_message));
        
        // Memory usage health check
        let memory_used = data.metrics.memory.used;
        let memory_total = data.metrics.memory.total;
        let memory_percent = memory_used as f64 / memory_total as f64 * 100.0;
        
        let memory_status = if memory_percent < 70.0 {
            TerminalDashboardHealthStatus::Healthy
        } else if memory_percent < 90.0 {
            TerminalDashboardHealthStatus::Warning
        } else {
            TerminalDashboardHealthStatus::Critical
        };
        
        let memory_message = format!("{:.1}% ({} / {})",
            memory_percent,
            format_bytes(memory_used),
            format_bytes(memory_total),
        );
        
        self.health_checks.push(HealthCheck::new("Memory", crate::widgets::health::HealthStatus::Unknown)
            .with_percentage(memory_percent)
            .with_status(
                crate::widgets::health::HealthStatus::from(memory_status),
            )
            .with_message(memory_message));
        
        // Network health check
        let network_interfaces = &data.metrics.network.interfaces;
        if !network_interfaces.is_empty() {
            // Find the first active interface
            let interface = &network_interfaces[0];
            
            // Assume network is healthy if at least one interface is up
            let network_status = if interface.is_up {
                TerminalDashboardHealthStatus::Healthy
            } else {
                TerminalDashboardHealthStatus::Critical
            };
            
            let network_usage = if interface.is_up { 100.0 } else { 0.0 };
            let network_message = format!("{}: {} interfaces",
                if interface.is_up { "Connected" } else { "Disconnected" },
                network_interfaces.len(),
            );
            
            self.health_checks.push(HealthCheck::new("Network", crate::widgets::health::HealthStatus::Unknown)
                .with_percentage(network_usage)
                .with_status(
                    crate::widgets::health::HealthStatus::from(network_status),
                )
                .with_message(network_message));
        }
        
        // Protocol health check
        let protocol = &data.protocol;
        let protocol_status = match protocol.status.as_str() {
            "Connected" | "Running" => TerminalDashboardHealthStatus::Healthy,
            "Degraded" | "Connecting" => TerminalDashboardHealthStatus::Warning,
            "Disconnected" | "Stopped" | "Error" => TerminalDashboardHealthStatus::Critical,
            _ => TerminalDashboardHealthStatus::Unknown,
        };
        
        let protocol_usage = match protocol.status.as_str() {
            "Connected" | "Running" => 100.0,
            "Degraded" | "Connecting" => 50.0,
            _ => 0.0,
        };
        let protocol_message = format!("{} ({})", protocol.status, protocol.name);
        
        self.health_checks.push(HealthCheck::new("Protocol", crate::widgets::health::HealthStatus::Unknown)
            .with_percentage(protocol_usage)
            .with_status(
                crate::widgets::health::HealthStatus::from(protocol_status),
            )
            .with_message(protocol_message));
        
        // Alerts health check
        let alerts = &data.alerts;
        let critical_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let warning_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Warning)).count();
        
        let alert_status = if critical_alerts > 0 {
            TerminalDashboardHealthStatus::Critical
        } else if warning_alerts > 0 {
            TerminalDashboardHealthStatus::Warning
        } else {
            TerminalDashboardHealthStatus::Healthy
        };
        
        let alert_usage = if critical_alerts > 0 { 100.0 } else if warning_alerts > 0 { 50.0 } else { 0.0 };
        let alert_message = format!("{} critical, {} warning", critical_alerts, warning_alerts);
        
        self.health_checks.push(HealthCheck::new("Alerts", crate::widgets::health::HealthStatus::Unknown)
            .with_percentage(alert_usage)
            .with_status(
                crate::widgets::health::HealthStatus::from(alert_status),
            )
            .with_message(alert_message));
    }

    /// Update time series data
    fn update_time_series(&mut self, data: &DashboardData) {
        // Get the current timestamp
        let timestamp = Utc::now();
        
        // CPU metrics
        let cpu_data = self.time_series.entry(MetricType::CpuUsage).or_default();
        cpu_data.push((timestamp, data.metrics.cpu.usage));
        
        // Memory metrics
        let memory_percent = data.metrics.memory.used as f64 / data.metrics.memory.total as f64 * 100.0;
        let memory_data = self.time_series.entry(MetricType::MemoryUsage).or_default();
        memory_data.push((timestamp, memory_percent));
        
        // Network metrics for each interface
        for interface in &data.metrics.network.interfaces {
            // RX metrics
            let rx_data = self.time_series
                .entry(MetricType::NetworkRx(interface.name.clone()))
                .or_default();
            rx_data.push((timestamp, interface.rx_bytes as f64));
            
            // TX metrics
            let tx_data = self.time_series
                .entry(MetricType::NetworkTx(interface.name.clone()))
                .or_default();
            tx_data.push((timestamp, interface.tx_bytes as f64));
        }
        
        // Trim history if too long
        const MAX_HISTORY_POINTS: usize = 100;
        for (_, data) in self.time_series.iter_mut() {
            if data.len() > MAX_HISTORY_POINTS {
                let trim_count = data.len() - MAX_HISTORY_POINTS;
                data.drain(0..trim_count);
            }
        }
    }

    /// Render UI
    pub fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<(), std::io::Error> {
        terminal.draw(|f| draw(f, self))?;
        Ok(())
    }

    /// Render the title bar
    fn render_title(&self, f: &mut Frame, area: Rect) {
        let titles = [
            ("1", "Overview"),
            ("2", "System"),
            ("3", "Network"),
            ("4", "Protocol"),
            ("5", "Alerts"),
            ("6", "Tools"),
        ];
        
        let title_spans: Vec<Line> = titles
            .iter()
            .map(|(key, name)| {
                let spans = vec![
                    Span::styled(
                        format!("[{}]", key),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        name.to_string(),
                        if self.get_active_name() == *name {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::raw("   "),
                ];
                
                Line::from(spans)
            })
            .collect();
        
        let last_update = if let Some(last_update) = self.last_update {
            format!("Last update: {}", last_update.format("%H:%M:%S"))
        } else {
            "No updates yet".to_string()
        };
        
        let title_paragraph = ratatui::widgets::Paragraph::new(title_spans)
            .style(Style::default().bg(Color::Black))
            .alignment(Alignment::Left);
        
        let status_paragraph = ratatui::widgets::Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    last_update,
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  "),
                Span::styled(
                    "[h] Help",
                    Style::default().fg(Color::Blue),
                ),
                Span::raw("  "),
                Span::styled(
                    "[q] Quit",
                    Style::default().fg(Color::Red),
                ),
            ]),
        ])
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Right);
        
        let title_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(area);
        
        f.render_widget(title_paragraph, title_layout[0]);
        f.render_widget(status_paragraph, title_layout[1]);
    }

    /// Render the footer
    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = match self.active_tab {
            ActiveTab::Overview => "Overview: System health and resources",
            ActiveTab::Network => "Network: Interface status and throughput",
            ActiveTab::Protocol => "Protocol: Protocol status and statistics",
            ActiveTab::Alerts => "Alerts: System alerts and notifications",
            ActiveTab::System => "System: System metrics and performance",
            ActiveTab::Tools => "Tools: Utility tools and configuration",
        };
        
        let footer = ratatui::widgets::Paragraph::new(Line::from(vec![
            Span::styled(
                footer_text,
                Style::default().fg(Color::White),
            ),
        ]))
        .style(Style::default().bg(Color::Black))
        .alignment(Alignment::Center);
        
        f.render_widget(footer, area);
    }

    /// Render the help screen
    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(vec![
                Span::styled(
                    "Dashboard Help",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Navigation:",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[1-6]", Style::default().fg(Color::Yellow)),
                Span::raw(" - Switch between tabs"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[h]", Style::default().fg(Color::Yellow)),
                Span::raw(" - Toggle help screen"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("[q]", Style::default().fg(Color::Yellow)),
                Span::raw(" - Quit application"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Tabs:",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Overview", Style::default().fg(Color::White)),
                Span::raw(" - System health and resources"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("System", Style::default().fg(Color::White)),
                Span::raw(" - System metrics and performance"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Network", Style::default().fg(Color::White)),
                Span::raw(" - Network interfaces and traffic"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Protocol", Style::default().fg(Color::White)),
                Span::raw(" - Protocol status and statistics"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Alerts", Style::default().fg(Color::White)),
                Span::raw(" - System alerts and notifications"),
            ]),
        ];
        
        let help_block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .style(Style::default().bg(Color::Black))
            .title("Help");
        
        let help_paragraph = ratatui::widgets::Paragraph::new(help_text)
            .style(Style::default().bg(Color::Black))
            .block(help_block)
            .alignment(Alignment::Left);
        
        let help_area = centered_rect(60, 70, area);
        f.render_widget(Clear, help_area);
        f.render_widget(help_paragraph, help_area);
    }

    /// Render the overview tab
    fn render_overview(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .margin(1)
            .split(area);
        
        let top_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(layout[0]);
        
        let bottom_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .split(layout[1]);
        
        // Render widgets
        self.render_health_widget(f, top_layout[0]);
        self.render_metrics_widget(f, top_layout[1]);
        self.render_cpu_chart(f, bottom_layout[0]);
        self.render_memory_chart(f, bottom_layout[1]);
    }

    /// Render the network tab
    fn render_network(&self, f: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ].as_ref())
            .margin(1)
            .split(area);
        
        // Create a NetworkMetrics struct from dashboard data
        let network_metrics = if let Some(ref data) = self.dashboard_data {
            // Use the existing network metrics
            &data.metrics.network
        } else {
            // Early return if no data
            return;
        };
        
        // Render network widget
        NetworkWidget::new(Some(network_metrics), "Network Interfaces").render(f, layout[0]);
        
        // Render network charts if we have interfaces
        if !network_metrics.interfaces.is_empty() {
            let bottom_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ].as_ref())
                .split(layout[1]);
            
            self.render_network_rx_chart(f, bottom_layout[0], &network_metrics.interfaces[0].name);
            self.render_network_tx_chart(f, bottom_layout[1], &network_metrics.interfaces[0].name);
        }
    }

    /// Render the protocol tab
    fn render_protocol(&self, _f: &mut Frame, _area: Rect) {
        let _protocol = match &self.dashboard_data {
            Some(data) => &data.protocol,
            None => return,
        };
        
        // Protocol rendering would go here
    }

    /// Render the alerts tab
    fn render_alerts(&self, f: &mut Frame, area: Rect) {
        // Create layout for alerts tab
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(100), // Show current alerts only for now
                // Constraint::Min(10),    // Alert history - removed
            ])
            .split(area);

        // Create alerts widget using data.alerts
        let alerts = self.dashboard_data.as_ref()
            .map(|data| data.alerts.as_slice()) // Get alerts as slice
            .unwrap_or_default(); // Use empty slice if no data

        // Correct AlertsWidget initialization (takes Option<&[Alert]>, &str)
        let alerts_widget = AlertsWidget::new(Some(alerts), "Active Alerts");
        f.render_widget(alerts_widget, chunks[0]);

        // Remove alert history widget section
        // let alert_history = self.dashboard_data
        //     .map(|data| &data.alert_history) // alert_history doesn't exist
        //     .unwrap_or_default();

        // let history_widget = AlertsWidget::new(alert_history)
        //     .title("Alert History");
        // f.render_widget(history_widget, chunks[1]);
    }

    /// Render the health widget
    fn render_health_widget(&self, f: &mut Frame, area: Rect) {
        let health_widget = HealthWidget::new(&self.health_checks, "System Health");
        f.render_widget(health_widget, area);
    }

    /// Render the metrics widget
    fn render_metrics_widget(&self, f: &mut Frame, area: Rect) {
        let metrics = self.dashboard_data.as_ref().map(|data| &data.metrics);
        let metrics_widget = MetricsWidget::new(metrics, "System Metrics");
        f.render_widget(metrics_widget, area);
    }

    fn render_cpu_chart(&self, f: &mut Frame, area: Rect) {
        let data = self.time_series.get(&MetricType::CpuUsage).cloned().unwrap_or_default();
        let chart = ChartWidget::new("CPU Usage (%)".to_string(), ChartType::Line, data);
        f.render_widget(chart, area);
    }
    
    fn render_memory_chart(&self, f: &mut Frame, area: Rect) {
        let data = self.time_series.get(&MetricType::MemoryUsage).cloned().unwrap_or_default();
        let chart = ChartWidget::new("Memory Usage (%)".to_string(), ChartType::Line, data);
        f.render_widget(chart, area);
    }

    fn render_network_rx_chart(&self, f: &mut Frame, area: Rect, interface_name: &str) {
        let data = self.time_series
            .get(&MetricType::NetworkRx(interface_name.to_string()))
            .cloned()
            .unwrap_or_default();
        let chart = ChartWidget::new(format!("RX ({})", interface_name), ChartType::Line, data)
                     .data_type(NetworkDataType::Bytes);
        f.render_widget(chart, area);
    }

    fn render_network_tx_chart(&self, f: &mut Frame, area: Rect, interface_name: &str) {
        let data = self.time_series
            .get(&MetricType::NetworkTx(interface_name.to_string()))
            .cloned()
            .unwrap_or_default();
        let chart = ChartWidget::new(format!("TX ({})", interface_name), ChartType::Line, data)
                     .data_type(NetworkDataType::Bytes);
        f.render_widget(chart, area);
    }
    
    fn get_active_name(&self) -> &str {
        match self.active_tab {
            ActiveTab::Overview => "Overview",
            ActiveTab::System => "System",
            ActiveTab::Network => "Network",
            ActiveTab::Protocol => "Protocol",
            ActiveTab::Alerts => "Alerts",
            ActiveTab::Tools => "Tools",
        }
    }
}

pub fn draw(f: &mut Frame, app: &UiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ].as_ref())
        .split(f.size());

    let title_area = chunks[0];
    let content_area = chunks[1];
    let footer_area = chunks[2];

    app.render_title(f, title_area);

    match app.active_tab {
        ActiveTab::Overview => app.render_overview(f, content_area),
        ActiveTab::System => { /* Render System tab */ }
        ActiveTab::Network => app.render_network(f, content_area),
        ActiveTab::Protocol => app.render_protocol(f, content_area),
        ActiveTab::Alerts => app.render_alerts(f, content_area),
        ActiveTab::Tools => { /* Render Tools tab */ }
    }

    app.render_footer(f, footer_area);

    if app.show_help {
        app.render_help(f, f.size());
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalDashboardHealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

impl From<TerminalDashboardHealthStatus> for crate::widgets::health::HealthStatus {
    fn from(status: TerminalDashboardHealthStatus) -> Self {
        match status {
            TerminalDashboardHealthStatus::Healthy => crate::widgets::health::HealthStatus::Healthy,
            TerminalDashboardHealthStatus::Warning => crate::widgets::health::HealthStatus::Warning,
            TerminalDashboardHealthStatus::Critical => crate::widgets::health::HealthStatus::Critical,
            TerminalDashboardHealthStatus::Unknown => crate::widgets::health::HealthStatus::Unknown,
        }
    }
}
