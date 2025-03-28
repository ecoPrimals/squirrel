use std::collections::HashMap;
use std::time::Duration;

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
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};

use crate::{
    util::format_bytes,
    widgets::{
        alerts::AlertsWidget,
        chart::{ChartType, ChartWidget, NetworkDataType},
        health::{HealthWidget, HealthCheck},
        metrics::MetricsWidget,
        network::NetworkWidget,
        protocol::ProtocolWidget,
    },
    widget_manager::WidgetManager,
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

pub struct App {
    dashboard_data: Option<DashboardData>,
    active_tab: ActiveTab,
    show_help: bool,
    health_checks: Vec<HealthCheck>,
    time_series: HashMap<MetricType, Vec<(chrono::DateTime<Utc>, f64)>>,
    last_update: Option<chrono::DateTime<Utc>>,
}

/// Active tab enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Overview tab
    Overview,
    /// Network tab
    Network,
    /// Protocol tab
    Protocol,
    /// Alerts tab
    Alerts,
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
                    KeyCode::Char('2') => self.active_tab = ActiveTab::Network,
                    KeyCode::Char('3') => self.active_tab = ActiveTab::Protocol,
                    KeyCode::Char('4') => self.active_tab = ActiveTab::Alerts,
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
        terminal.draw(|f| {
            let size = f.size();
            
            if self.show_help {
                self.render_help(f, size);
                return;
            }
            
            // Create layout
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ].as_ref())
                .split(size);
            
            self.render_title(f, main_layout[0]);
            self.render_footer(f, main_layout[2]);
            
            match self.active_tab {
                ActiveTab::Overview => self.render_overview(f, main_layout[1]),
                ActiveTab::Network => self.render_network(f, main_layout[1]),
                ActiveTab::Protocol => self.render_protocol(f, main_layout[1]),
                ActiveTab::Alerts => self.render_alerts(f, main_layout[1]),
            }
        })?;
        
        Ok(())
    }

    /// Render the title bar
    fn render_title(&self, f: &mut Frame, area: Rect) {
        let titles = vec![
            ("1", "Overview"),
            ("2", "Network"),
            ("3", "Protocol"),
            ("4", "Alerts"),
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
                Span::styled("[1-4]", Style::default().fg(Color::Yellow)),
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
        
        ProtocolWidget::new(protocol, "Protocol")
            .render(f, area);
    }

    /// Render the alerts tab
    fn render_alerts(&self, f: &mut Frame, area: Rect) {
        let alerts = match &self.dashboard_data {
            Some(data) => &data.alerts,
            None => return, // No data, nothing to render
        };
        
        // Create alerts widget with dashboard alerts
        let widget = AlertsWidget::from_dashboard(
            alerts, 
            "System Alerts"
        );
        
        // Render it
        widget.render(f, area);
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
            ActiveTab::Network => "Network",
            ActiveTab::Protocol => "Protocol",
            ActiveTab::Alerts => "Alerts",
        }
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

/// Draw the UI
pub fn draw(
    f: &mut Frame,
    title: &str,
    state: &UiState,
    widget_managers: &[Box<dyn WidgetManager>],
    data: Option<&DashboardData>,
) {
    let size = f.size();
    
    // Draw title bar
    let title_bar_height = 3;
    let title_bar_area = Rect {
        x: 0,
        y: 0,
        width: size.width,
        height: title_bar_height,
    };
    
    let title_text = if let Some(data) = data {
        format!("{} - Last updated: {:?}", title, data.timestamp)
    } else {
        format!("{} - No data", title)
    };
    
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Blue));
        
    f.render_widget(title_block, title_bar_area);
    
    let inner_title_area = Rect {
        x: 1,
        y: 1,
        width: title_bar_area.width - 2,
        height: 1,
    };
    
    let title_paragraph = Paragraph::new(Line::from(vec![
        Span::styled(title_text, Style::default().fg(Color::White))
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(title_paragraph, inner_title_area);
    
    // Draw widget area
    let widget_area = Rect {
        x: 0,
        y: title_bar_height,
        width: size.width,
        height: size.height - title_bar_height,
    };
    
    match state.layout {
        WidgetLayout::Grid => draw_grid_layout(f, widget_area, widget_managers),
        WidgetLayout::Vertical => draw_vertical_layout(f, widget_area, widget_managers),
        WidgetLayout::Horizontal => draw_horizontal_layout(f, widget_area, widget_managers),
        WidgetLayout::Focused(index) => draw_focused_layout(f, widget_area, widget_managers, index),
    }
    
    // Draw help if enabled
    if state.show_help {
        // Create a default HelpSystem to use
        let help_system = crate::help::HelpSystem::default();
        draw_help(f, &help_system);
    }
}

/// Draw grid layout (2x2)
fn draw_grid_layout(
    f: &mut Frame,
    area: Rect,
    widget_managers: &[Box<dyn WidgetManager>],
) {
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    
    let top_left = horizontal_layout[0];
    let top_right = horizontal_layout[1];
    
    let vertical_left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(top_left);
    
    let vertical_right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(top_right);
    
    // Render widgets in each area
    for (i, widget) in widget_managers.iter().enumerate().take(4) {
        if widget.enabled() {
            let widget_area = match i {
                0 => vertical_left[0],
                1 => vertical_left[1],
                2 => vertical_right[0],
                3 => vertical_right[1],
                _ => unreachable!(),
            };
            
            widget.render(f, widget_area);
        }
    }
}

/// Draw vertical layout (stacked)
fn draw_vertical_layout(
    f: &mut Frame,
    area: Rect,
    widget_managers: &[Box<dyn WidgetManager>],
) {
    let enabled_widgets: Vec<_> = widget_managers.iter().filter(|w| w.enabled()).collect();
    
    if enabled_widgets.is_empty() {
        return;
    }
    
    let height_percent = 100 / enabled_widgets.len() as u16;
    let constraints: Vec<_> = (0..enabled_widgets.len())
        .map(|_| Constraint::Percentage(height_percent))
        .collect();
    
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    
    for (i, widget) in enabled_widgets.iter().enumerate() {
        widget.render(f, vertical_layout[i]);
    }
}

/// Draw horizontal layout (side by side)
fn draw_horizontal_layout(
    f: &mut Frame,
    area: Rect,
    widget_managers: &[Box<dyn WidgetManager>],
) {
    let enabled_widgets: Vec<_> = widget_managers.iter().filter(|w| w.enabled()).collect();
    
    if enabled_widgets.is_empty() {
        return;
    }
    
    let width_percent = 100 / enabled_widgets.len() as u16;
    let constraints: Vec<_> = (0..enabled_widgets.len())
        .map(|_| Constraint::Percentage(width_percent))
        .collect();
    
    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);
    
    for (i, widget) in enabled_widgets.iter().enumerate() {
        widget.render(f, horizontal_layout[i]);
    }
}

/// Draw focused layout (one widget takes most space)
fn draw_focused_layout(
    f: &mut Frame,
    area: Rect,
    widget_managers: &[Box<dyn WidgetManager>],
    focused_index: usize,
) {
    let enabled_widgets: Vec<_> = widget_managers.iter().filter(|w| w.enabled()).collect();
    
    if enabled_widgets.is_empty() {
        return;
    }
    
    let focused_widget = if focused_index < enabled_widgets.len() {
        enabled_widgets[focused_index]
    } else {
        enabled_widgets[0]
    };
    
    focused_widget.render(f, area);
}

/// Draw help screen with HelpSystem
pub fn draw_help(f: &mut Frame, help_system: &crate::help::HelpSystem) {
    // Get frame size
    let size = f.size();
    
    // Calculate appropriate help area
    let help_area = centered_rect(80, 90, size);
    
    // Clear the area behind the help
    f.render_widget(Clear, help_area);
    
    // Draw a block around the help
    let help_block = Block::default()
        .title(" Help ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    // Create layout for topics and content
    let help_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(help_block.inner(help_area));
    
    // Draw topic list
    let topic_list = help_system.get_topic_list();
    let topics_paragraph = Paragraph::new(topic_list)
        .block(Block::default()
            .title(" Topics ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Yellow)));
    
    f.render_widget(topics_paragraph, help_layout[0]);
    
    // Draw help content
    let help_content = help_system.get_content();
    let content_paragraph = Paragraph::new(help_content)
        .block(Block::default()
            .title(" Information ")
            .title_alignment(Alignment::Center));
    
    f.render_widget(content_paragraph, help_layout[1]);
    
    // Draw the main border around everything
    f.render_widget(help_block, help_area);
} 
