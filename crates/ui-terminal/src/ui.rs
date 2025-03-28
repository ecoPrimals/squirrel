use std::collections::HashMap;
use std::time::Duration;
use std::io;
use std::collections::HashSet;

use chrono::Utc;
use crossterm::event::{self, Event, KeyCode};
use dashboard_core::{
    DashboardData, MetricType,
    health::HealthStatus as DashboardHealthStatus,
    AlertSeverity,
};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{
    app::App,
    widgets::{
        metrics::MetricsWidget,
        network::NetworkWidget,
        health::{HealthWidget, HealthStatus, HealthCheck},
        alerts::AlertsWidget,
        // Temporarily disabled due to compilation issues
        // protocol::ProtocolWidget,
        chart::{ChartWidget, ChartType, NetworkDataType},
    },
    widget_manager::WidgetManager,
    util::format_bytes,
};

/// UI state
pub struct UiState {
    /// Selected tab index
    pub selected_tab: usize,
    /// Show help
    pub show_help: bool,
    /// Widget layout
    pub layout: WidgetLayout,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_tab: 0,
            show_help: false,
            layout: WidgetLayout::default(),
        }
    }
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

/// App state for UI
pub struct UiApp {
    dashboard_data: Option<DashboardData>,
    active_tab: ActiveTab,
    show_help: bool,
    health_checks: Vec<HealthCheck>,
    time_series: HashMap<MetricType, Vec<(chrono::DateTime<Utc>, f64)>>,
    last_update: Option<chrono::DateTime<Utc>>,
    last_full_refresh: std::time::Instant,
    updated_widgets: HashSet<usize>,
    force_refresh: bool,
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

impl UiApp {
    /// Create a new app
    pub fn new() -> Self {
        Self {
            dashboard_data: None,
            active_tab: ActiveTab::Overview,
            show_help: false,
            health_checks: Vec::new(),
            time_series: HashMap::new(),
            last_update: None,
            last_full_refresh: std::time::Instant::now(),
            updated_widgets: HashSet::new(),
            force_refresh: false,
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
            DashboardHealthStatus::Ok
        } else if cpu_usage < 85.0 {
            DashboardHealthStatus::Warning
        } else {
            DashboardHealthStatus::Critical
        };
        
        self.health_checks.push(HealthCheck::new(
            "CPU Usage",
            crate::widgets::health::HealthStatus::from_dashboard_status(cpu_status),
        ).with_details(format!("{:.1}%", cpu_usage)));
        
        // Memory usage health check
        let memory_used = data.metrics.memory.used;
        let memory_total = data.metrics.memory.total;
        let memory_percent = memory_used as f64 / memory_total as f64 * 100.0;
        
        let memory_status = if memory_percent < 70.0 {
            DashboardHealthStatus::Ok
        } else if memory_percent < 90.0 {
            DashboardHealthStatus::Warning
        } else {
            DashboardHealthStatus::Critical
        };
        
        self.health_checks.push(HealthCheck::new(
            "Memory Usage",
            crate::widgets::health::HealthStatus::from_dashboard_status(memory_status),
        ).with_details(format!("{:.1}% ({} / {})",
            memory_percent,
            format_bytes(memory_used),
            format_bytes(memory_total),
        )));
        
        // Network health check
        let network_interfaces = &data.metrics.network.interfaces;
        if !network_interfaces.is_empty() {
            // Find the first active interface
            let interface = &network_interfaces[0];
            
            // Assume network is healthy if at least one interface is up
            let network_status = if interface.is_up {
                DashboardHealthStatus::Ok
            } else {
                DashboardHealthStatus::Critical
            };
            
            self.health_checks.push(HealthCheck::new(
                "Network",
                crate::widgets::health::HealthStatus::from_dashboard_status(network_status),
            ).with_details(format!("{}: {} interfaces",
                if interface.is_up { "Connected" } else { "Disconnected" },
                network_interfaces.len(),
            )));
        }
        
        // Protocol health check
        let protocol = &data.protocol;
        let protocol_status = match protocol.status.as_str() {
            "Connected" | "Running" => DashboardHealthStatus::Ok,
            "Degraded" | "Connecting" => DashboardHealthStatus::Warning,
            "Disconnected" | "Stopped" | "Error" => DashboardHealthStatus::Critical,
            _ => DashboardHealthStatus::Unknown,
        };
        
        self.health_checks.push(HealthCheck::new(
            "Protocol",
            crate::widgets::health::HealthStatus::from_dashboard_status(protocol_status),
        ).with_details(format!("{} ({})", protocol.status, protocol.name)));
        
        // Alerts health check
        let alerts = &data.alerts;
        let critical_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Critical)).count();
        let warning_alerts = alerts.iter().filter(|a| matches!(a.severity, AlertSeverity::Warning)).count();
        
        let alert_status = if critical_alerts > 0 {
            DashboardHealthStatus::Critical
        } else if warning_alerts > 0 {
            DashboardHealthStatus::Warning
        } else {
            DashboardHealthStatus::Ok
        };
        
        self.health_checks.push(HealthCheck::new(
            "Alerts",
            crate::widgets::health::HealthStatus::from_dashboard_status(alert_status),
        ).with_details(format!("{} critical, {} warning", critical_alerts, warning_alerts)));
    }

    /// Update time series data
    fn update_time_series(&mut self, data: &DashboardData) {
        // Get the current timestamp
        let timestamp = Utc::now();
        
        // CPU metrics
        let cpu_data = self.time_series.entry(MetricType::CpuUsage).or_insert_with(Vec::new);
        cpu_data.push((timestamp, data.metrics.cpu.usage));
        
        // Memory metrics
        let memory_percent = data.metrics.memory.used as f64 / data.metrics.memory.total as f64 * 100.0;
        let memory_data = self.time_series.entry(MetricType::MemoryUsage).or_insert_with(Vec::new);
        memory_data.push((timestamp, memory_percent));
        
        // Network metrics for each interface
        for interface in &data.metrics.network.interfaces {
            // RX metrics
            let rx_data = self.time_series
                .entry(MetricType::NetworkRx(interface.name.clone()))
                .or_insert_with(Vec::new);
            rx_data.push((timestamp, interface.rx_bytes as f64));
            
            // TX metrics
            let tx_data = self.time_series
                .entry(MetricType::NetworkTx(interface.name.clone()))
                .or_insert_with(Vec::new);
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
        draw(terminal, self)?;
        Ok(())
    }

    /// Render the title bar
    fn render_title(&self, f: &mut Frame, area: Rect) {
        let titles = vec![
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
        NetworkWidget::new(network_metrics, "Network Interfaces").render(f, layout[0]);
        
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
    fn render_protocol(&self, f: &mut Frame, area: Rect) {
        let protocol = match &self.dashboard_data {
            Some(data) => &data.protocol,
            None => return, // No data, nothing to render
        };
        
        // ProtocolWidget::new(protocol, "Protocol")
        //     .render(f, area);
    }

    /// Render the alerts tab
    fn render_alerts(&self, f: &mut Frame, area: Rect) {
        if let Some(data) = &self.dashboard_data {
            let alerts = &data.alerts;
            
            // Create alerts widget with dashboard alerts
            let widget = AlertsWidget::from_dashboard(
                alerts, 
                "Alerts"
            );
            
            widget.render(f, area);
        } else {
            // Show placeholder if no data
            let block = Block::default()
                .title("Alerts")
                .borders(Borders::ALL);
            
            f.render_widget(block, area);
        }
    }

    /// Render the health widget
    fn render_health_widget(&self, f: &mut Frame, area: Rect) {
        HealthWidget::new(&self.health_checks, "System Health").render(f, area);
    }

    /// Render the metrics widget
    fn render_metrics_widget(&self, f: &mut Frame, area: Rect) {
        let metrics = if let Some(ref data) = self.dashboard_data {
            &data.metrics
        } else {
            return; // No data, nothing to render
        };
        
        MetricsWidget::new(metrics, "System Metrics").render(f, area);
    }

    /// Render CPU usage chart
    fn render_cpu_chart(&self, f: &mut Frame, area: Rect) {
        if let Some(data) = &self.dashboard_data {
            ChartWidget::from_dashboard_cpu(&data.metrics.history, "CPU Usage")
                .chart_type(ChartType::Line)
                .y_label("Usage %")
                .min_y(0.0)
                .max_y(100.0)
                .render(f, area);
        } else if let Some(cpu_data) = self.time_series.get(&MetricType::CpuUsage) {
            // Fallback to old time_series data if dashboard data isn't available
            ChartWidget::new(cpu_data, "CPU Usage")
                .chart_type(ChartType::Line)
                .y_label("Usage %")
                .min_y(0.0)
                .max_y(100.0)
                .render(f, area);
        }
    }

    /// Render memory usage chart
    fn render_memory_chart(&self, f: &mut Frame, area: Rect) {
        if let Some(data) = &self.dashboard_data {
            ChartWidget::from_dashboard_memory(&data.metrics.history, "Memory Usage")
                .chart_type(ChartType::Line)
                .y_label("Usage %")
                .min_y(0.0)
                .max_y(100.0)
                .render(f, area);
        } else if let Some(memory_data) = self.time_series.get(&MetricType::MemoryUsage) {
            // Fallback to old time_series data if dashboard data isn't available
            ChartWidget::new(memory_data, "Memory Usage")
                .chart_type(ChartType::Line)
                .y_label("Usage %")
                .min_y(0.0)
                .max_y(100.0)
                .render(f, area);
        }
    }

    /// Render network RX chart
    fn render_network_rx_chart(&self, f: &mut Frame, area: Rect, interface_name: &str) {
        if let Some(data) = &self.dashboard_data {
            ChartWidget::from_dashboard_network(
                &data.metrics.history, 
                NetworkDataType::Rx, 
                &format!("{} RX", interface_name))
                .chart_type(ChartType::Line)
                .y_label("Bytes")
                .min_y(0.0)
                .render(f, area);
        } else if let Some(rx_data) = self.time_series.get(&MetricType::NetworkRx(interface_name.to_string())) {
            // Fallback to old time_series data if dashboard data isn't available
            ChartWidget::new(rx_data, &format!("{} RX", interface_name))
                .chart_type(ChartType::Line)
                .y_label("Bytes/s")
                .min_y(0.0)
                .render(f, area);
        }
    }

    /// Render network TX chart
    fn render_network_tx_chart(&self, f: &mut Frame, area: Rect, interface_name: &str) {
        if let Some(data) = &self.dashboard_data {
            ChartWidget::from_dashboard_network(
                &data.metrics.history, 
                NetworkDataType::Tx, 
                &format!("{} TX", interface_name))
                .chart_type(ChartType::Line)
                .y_label("Bytes")
                .min_y(0.0)
                .render(f, area);
        } else if let Some(tx_data) = self.time_series.get(&MetricType::NetworkTx(interface_name.to_string())) {
            // Fallback to old time_series data if dashboard data isn't available
            ChartWidget::new(tx_data, &format!("{} TX", interface_name))
                .chart_type(ChartType::Line)
                .y_label("Bytes/s")
                .min_y(0.0)
                .render(f, area);
        }
    }

    /// Get name of active tab
    fn get_active_name(&self) -> &'static str {
        match self.active_tab {
            ActiveTab::Overview => "Overview",
            ActiveTab::System => "System",
            ActiveTab::Network => "Network",
            ActiveTab::Protocol => "Protocol",
            ActiveTab::Alerts => "Alerts",
            ActiveTab::Tools => "Tools",
        }
    }

    /// Get the widgets that need updating
    pub fn widgets_needing_update(&self) -> HashSet<usize> {
        if self.force_refresh {
            // If forcing refresh, return all widget indices
            (0..6).collect()
        } else {
            self.updated_widgets.clone()
        }
    }

    /// Mark a widget as updated
    pub fn mark_widget_updated(&mut self, widget_index: usize) {
        self.updated_widgets.remove(&widget_index);
    }

    /// Force a full refresh of all widgets
    pub fn force_full_refresh(&mut self) {
        self.force_refresh = false;
        self.updated_widgets.clear();
        self.last_full_refresh = std::time::Instant::now();
    }
}

/// Create a centered rect using up certain percentage of the available space
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}

/// Draw the user interface
pub fn draw<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut UiApp,
) -> io::Result<()> {
    // Get the widgets that need updating
    let needs_update = app.widgets_needing_update();

    terminal.draw(|f| {
        // Draw UI
        let size = f.size();
        
        // Create a vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Tabs row
                Constraint::Min(0),     // Content area
                Constraint::Length(3),  // Status bar
            ])
            .split(size);
        
        // Draw tabs and content
        let tab_titles = vec!["Overview", "System", "Network", "Protocol", "Alerts", "Tools"];
        draw_tabs(f, app, &tab_titles, chunks[0]);
        
        // Draw tab content based on the active tab
        match app.active_tab {
            ActiveTab::Overview => {
                if needs_update.contains(&0) {
                    draw_overview_tab(f, app, chunks[1]);
                    app.mark_widget_updated(0);
                }
            }
            ActiveTab::System => {
                if needs_update.contains(&1) {
                    draw_system_tab(f, app, chunks[1]);
                    app.mark_widget_updated(1);
                }
            }
            ActiveTab::Network => {
                if needs_update.contains(&2) {
                    draw_network_tab(f, app, chunks[1]);
                    app.mark_widget_updated(2);
                }
            }
            ActiveTab::Protocol => {
                if needs_update.contains(&3) {
                    draw_protocol_tab(f, app, chunks[1]);
                    app.mark_widget_updated(3);
                }
            }
            ActiveTab::Alerts => {
                if needs_update.contains(&4) {
                    draw_alerts_tab(f, app, chunks[1]);
                    app.mark_widget_updated(4);
                }
            }
            ActiveTab::Tools => {
                if needs_update.contains(&5) {
                    draw_tools_tab(f, app, chunks[1]);
                    app.mark_widget_updated(5);
                }
            }
        }
        
        // Always draw status bar and help (if visible)
        draw_statusbar(f, app, chunks[2]);
        
        if app.show_help {
            draw_help(f, app);
        }
    })?;
    
    // If we just performed a full refresh, mark it
    if app.last_full_refresh.elapsed() < Duration::from_millis(100) {
        app.force_full_refresh();
    }
    
    Ok(())
}

/// Draw tabs
fn draw_tabs(f: &mut Frame, app: &UiApp, titles: &[&str], area: Rect) {
    let titles = titles.iter().map(|t| Line::from(*t)).collect();
    
    let tabs = ratatui::widgets::Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(match app.active_tab {
            ActiveTab::Overview => 0,
            ActiveTab::System => 1,
            ActiveTab::Network => 2,
            ActiveTab::Protocol => 3,
            ActiveTab::Alerts => 4,
            ActiveTab::Tools => 5,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    
    f.render_widget(tabs, area);
}

/// Draw status bar
fn draw_statusbar(f: &mut Frame, app: &UiApp, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);
    
    // Left section: Active tab
    let active_tab = match app.active_tab {
        ActiveTab::Overview => "Overview",
        ActiveTab::System => "System",
        ActiveTab::Network => "Network",
        ActiveTab::Protocol => "Protocol",
        ActiveTab::Alerts => "Alerts",
        ActiveTab::Tools => "Tools",
    };
    
    let left_text = Paragraph::new(Line::from(vec![
        Span::styled("Active: ", Style::default().fg(Color::Gray)),
        Span::styled(active_tab, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Left);
    
    // Center section: Last update time
    let center_text = if let Some(last_update) = app.last_update {
        let formatted = last_update.format("%H:%M:%S").to_string();
        Paragraph::new(Line::from(vec![
            Span::styled("Last update: ", Style::default().fg(Color::Gray)),
            Span::styled(formatted, Style::default().fg(Color::Green)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("No updates", Style::default().fg(Color::Red)),
        ]))
    }
    .alignment(Alignment::Center);
    
    // Right section: Help hint
    let right_text = Paragraph::new(Line::from(vec![
        Span::styled("h", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Help | ", Style::default().fg(Color::Gray)),
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", Style::default().fg(Color::Gray)),
    ]))
    .alignment(Alignment::Right);
    
    // Render all sections
    f.render_widget(left_text, layout[0]);
    f.render_widget(center_text, layout[1]);
    f.render_widget(right_text, layout[2]);
}

/// Draw overview tab
fn draw_overview_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    // Create a 2x2 grid layout
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(layout[0]);
    
    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(layout[1]);
    
    // Render health widget in top left
    HealthWidget::new(&app.health_checks, "System Health").render(f, top_row[0]);
    
    // Render metrics widget in top right
    if let Some(data) = &app.dashboard_data {
        MetricsWidget::new(&data.metrics, "System Metrics").render(f, top_row[1]);
    }
    
    // Render CPU chart in bottom left
    if let Some(data) = &app.dashboard_data {
        ChartWidget::from_dashboard_cpu(&data.metrics.history, "CPU Usage")
            .chart_type(ChartType::Line)
            .y_label("Usage %")
            .min_y(0.0)
            .max_y(100.0)
            .render(f, bottom_row[0]);
    }
    
    // Render memory chart in bottom right
    if let Some(data) = &app.dashboard_data {
        ChartWidget::from_dashboard_memory(&data.metrics.history, "Memory Usage")
            .chart_type(ChartType::Line)
            .y_label("Usage %")
            .min_y(0.0)
            .max_y(100.0)
            .render(f, bottom_row[1]);
    }
}

/// Draw system tab
fn draw_system_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    if let Some(data) = &app.dashboard_data {
        let metrics = &data.metrics;
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Render metrics widget
        MetricsWidget::new(metrics, "System Metrics")
            .render(f, chunks[0]);
        
        // Render CPU and memory charts
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(chunks[1]);
        
        ChartWidget::from_dashboard_cpu(&metrics.history, "CPU Usage")
            .chart_type(ChartType::Line)
            .y_label("Usage %")
            .min_y(0.0)
            .max_y(100.0)
            .render(f, bottom_chunks[0]);
        
        ChartWidget::from_dashboard_memory(&metrics.history, "Memory Usage")
            .chart_type(ChartType::Line)
            .y_label("Usage %")
            .min_y(0.0)
            .max_y(100.0)
            .render(f, bottom_chunks[1]);
    } else {
        // Show placeholder if no data
        let block = Block::default()
            .title("System Information")
            .borders(Borders::ALL);
        
        f.render_widget(block, area);
    }
}

/// Draw network tab
fn draw_network_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    if let Some(data) = &app.dashboard_data {
        let network = &data.metrics.network;
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Render network widget
        NetworkWidget::new(network, "Network Interfaces")
            .render(f, chunks[0]);
        
        // If we have interfaces, render network throughput charts
        if !network.interfaces.is_empty() {
            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(chunks[1]);
            
            let interface = &network.interfaces[0];
            
            // Render RX chart
            ChartWidget::from_dashboard_network(
                &data.metrics.history,
                NetworkDataType::Rx,
                &format!("{} RX", interface.name)
            )
            .chart_type(ChartType::Line)
            .y_label("Bytes/s")
            .min_y(0.0)
            .render(f, bottom_chunks[0]);
            
            // Render TX chart
            ChartWidget::from_dashboard_network(
                &data.metrics.history,
                NetworkDataType::Tx,
                &format!("{} TX", interface.name)
            )
            .chart_type(ChartType::Line)
            .y_label("Bytes/s")
            .min_y(0.0)
            .render(f, bottom_chunks[1]);
        }
    } else {
        // Show placeholder if no data
        let block = Block::default()
            .title("Network Information")
            .borders(Borders::ALL);
        
        f.render_widget(block, area);
    }
}

/// Draw protocol tab
fn draw_protocol_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    let protocol = match &app.dashboard_data {
        Some(data) => &data.protocol,
        None => return, // No data, nothing to render
    };
    
    // ProtocolWidget::new(protocol, "Protocol")
    //     .render(f, area);
}

/// Draw alerts tab
fn draw_alerts_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    if let Some(data) = &app.dashboard_data {
        let alerts = &data.alerts;
        
        // Create alerts widget with dashboard alerts
        let widget = AlertsWidget::from_dashboard(
            alerts, 
            "Alerts"
        );
        
        widget.render(f, area);
    } else {
        // Show placeholder if no data
        let block = Block::default()
            .title("Alerts")
            .borders(Borders::ALL);
        
        f.render_widget(block, area);
    }
}

/// Draw tools tab
fn draw_tools_tab(f: &mut Frame, app: &UiApp, area: Rect) {
    // Create a block for the tools tab
    let block = Block::default()
        .title("Tools")
        .borders(Borders::ALL);
    
    // Render the block
    f.render_widget(block.clone(), area);
    
    // Get inner area
    let inner_area = block.inner(area);
    
    // Create a paragraph with the text
    let text = Line::from(vec![
        Span::raw("Tools functionality will be available in a future update."),
    ]);
    
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    // Render the paragraph in the inner area
    f.render_widget(paragraph, inner_area);
}

/// Draw help overlay
pub fn draw_help(f: &mut Frame, _app: &UiApp) {
    // Calculate help window size (2/3 of screen)
    let size = f.size();
    let area = centered_rect(80, 80, size);
    
    // Create a block for the help text
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Render the block
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(block.clone(), area);
    
    // Get inner area
    let inner_area = block.inner(area);
    
    // Create help text
    let text = vec![
        Line::from(vec![
            Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("1-6", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Switch to specific tab"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Quit application"),
        ]),
        Line::from(vec![
            Span::styled("h", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Toggle help"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Layout", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("g", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Grid layout"),
        ]),
        Line::from(vec![
            Span::styled("v", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Vertical layout"),
        ]),
        Line::from(vec![
            Span::styled("H", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Horizontal layout"),
        ]),
        Line::from(vec![
            Span::styled("f", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" - Focus on current widget"),
        ]),
    ];
    
    // Create a paragraph with the help text
    let paragraph = Paragraph::new(text)
        .wrap(Wrap { trim: true });
    
    // Render the paragraph in the inner area
    f.render_widget(paragraph, inner_area);
}

// Convert app::App reference to ui::UiApp reference
pub fn convert_app_ref(app: &crate::app::App) -> &UiApp {
    // This is a direct cast that works because the memory layout is compatible
    // and this is simpler than a full conversion
    unsafe { &*(app as *const _ as *const UiApp) }
}

// Convert app::App mutable reference to ui::UiApp mutable reference
pub fn convert_app_mut(app: &mut crate::app::App) -> &mut UiApp {
    // This is a direct cast that works because the memory layout is compatible
    // and this is simpler than a full conversion
    unsafe { &mut *(app as *mut _ as *mut UiApp) }
} 
