use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::App;
use crate::widgets::{
    MetricsWidget, AlertsWidget, HealthWidget, 
    NetworkWidget, ChartWidget, ChartType, ProtocolWidget
};

/// Draw the UI
pub fn draw(f: &mut Frame, app: &mut App) {
    // Create base layout (tabs and content)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(f.size());
    
    // Draw tabs
    let tabs = draw_tabs(app);
    f.render_widget(tabs, chunks[0]);
    
    // Draw content based on selected tab
    match app.selected_tab() {
        0 => draw_overview_tab(f, app, chunks[1]),
        1 => draw_system_tab(f, app, chunks[1]),
        2 => draw_protocol_tab(f, app, chunks[1]),
        3 => draw_tools_tab(f, app, chunks[1]),
        4 => draw_alerts_tab(f, app, chunks[1]),
        5 => draw_network_tab(f, app, chunks[1]),
        _ => {}
    }
    
    // Draw status bar
    draw_status_bar(f, app, chunks[2]);
    
    // Draw help if visible
    if app.show_help() {
        draw_help(f);
    }
}

/// Draw the tabs widget
fn draw_tabs(app: &App) -> Tabs {
    let titles = app
        .tabs()
        .iter()
        .map(|t| Line::from(Span::styled(t, Style::default().fg(Color::White))))
        .collect();
    
    Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Dashboard"))
        .select(app.selected_tab())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        )
}

/// Draw the overview tab
fn draw_overview_tab(f: &mut Frame, app: &App, area: Rect) {
    // Create a layout with multiple sections for key metrics
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Top section for system health
            Constraint::Percentage(70), // Bottom section for metrics
        ])
        .split(area);
    
    // Split top section for health
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100), // Health status
        ])
        .split(chunks[0]);
    
    // Split bottom section for metrics
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // System metrics
            Constraint::Percentage(33), // Protocol metrics
            Constraint::Percentage(34), // Network metrics
        ])
        .split(chunks[1]);
    
    // Draw health status
    if let Some(data) = app.dashboard_data() {
        // Create dummy health checks
        let dummy_health_checks: Vec<crate::widgets::health::HealthCheck> = Vec::new();
        
        // Draw health widget
        let health_widget = HealthWidget::new(&dummy_health_checks, "Health");
        health_widget.render(f, top_chunks[0]);
        
        // Draw system metrics
        let system_widget = MetricsWidget::new(&data.metrics, "System");
        system_widget.render(f, bottom_chunks[0]);
        
        // Draw protocol metrics
        let protocol_widget = MetricsWidget::new(&data.metrics, "Protocol");
        protocol_widget.render(f, bottom_chunks[1]);
        
        // Draw network metrics
        let network_widget = NetworkWidget::new(&data.network, "Network");
        network_widget.render(f, bottom_chunks[2]);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading dashboard data...")
            .block(Block::default().borders(Borders::ALL).title("Dashboard"));
        f.render_widget(loading, area);
    }
}

/// Draw the system tab
fn draw_system_tab(f: &mut Frame, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create layout with metrics and charts
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Metrics
                Constraint::Percentage(60), // Charts
            ])
            .split(area);
            
        // Create detailed system metrics widget
        let system_widget = MetricsWidget::new(&data.metrics, "System Metrics");
        system_widget.render(f, chunks[0]);
        
        // Split charts section
        let chart_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // CPU chart
                Constraint::Percentage(50), // Memory chart
            ])
            .split(chunks[1]);
            
        // Get metric history from app
        if let Some(cpu_history) = app.get_metric_history("system.cpu") {
            // Create CPU chart
            let cpu_chart = ChartWidget::new(cpu_history, "CPU Usage")
                .y_label("Usage %")
                .chart_type(ChartType::Line)
                .time_range(600); // 10 minutes
                
            cpu_chart.render(f, chart_chunks[0]);
        }
        
        if let Some(memory_history) = app.get_metric_history("system.memory") {
            // Create memory chart
            let memory_chart = ChartWidget::new(memory_history, "Memory Usage")
                .y_label("Bytes")
                .chart_type(ChartType::Line)
                .time_range(600); // 10 minutes
                
            memory_chart.render(f, chart_chunks[1]);
        }
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading system metrics...")
            .block(Block::default().borders(Borders::ALL).title("System"));
        f.render_widget(loading, area);
    }
}

/// Draw the protocol tab
fn draw_protocol_tab(f: &mut Frame, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create detailed protocol metrics widget
        let protocol_widget = ProtocolWidget::new(&data.metrics, "Protocol Metrics");
        protocol_widget.render(f, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading protocol metrics...")
            .block(Block::default().borders(Borders::ALL).title("Protocol"));
        f.render_widget(loading, area);
    }
}

/// Draw the tools tab
fn draw_tools_tab(f: &mut Frame, app: &App, area: Rect) {
    if let Some(_data) = app.dashboard_data() {
        // Create tools metrics widget
        // TODO: Implement detailed tool metrics widget
        let tools_widget = Paragraph::new("Tool metrics will be displayed here.")
            .block(Block::default().borders(Borders::ALL).title("Tools"));
        f.render_widget(tools_widget, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading tool metrics...")
            .block(Block::default().borders(Borders::ALL).title("Tools"));
        f.render_widget(loading, area);
    }
}

/// Draw the alerts tab
fn draw_alerts_tab(f: &mut Frame, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create alerts widget
        let alerts_widget = AlertsWidget::new(&data.alerts.active, "Active Alerts", app.alerts_selected_index());
        alerts_widget.render(f, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading alerts...")
            .block(Block::default().borders(Borders::ALL).title("Alerts"));
        f.render_widget(loading, area);
    }
}

/// Draw the network tab
fn draw_network_tab(f: &mut Frame, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create network widget
        let network_widget = NetworkWidget::new(&data.network, "Network Metrics");
        network_widget.render(f, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading network metrics...")
            .block(Block::default().borders(Borders::ALL).title("Network"));
        f.render_widget(loading, area);
    }
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match (app.is_updating(), app.time_since_update()) {
        (true, _) => {
            Line::from(Span::styled(
                "Updating...",
                Style::default().fg(Color::Yellow)
            ))
        }
        (false, Some(duration)) => {
            let seconds = duration.as_secs();
            
            if seconds < 5 {
                Line::from(Span::styled(
                    format!("Updated: Just now"),
                    Style::default().fg(Color::Green)
                ))
            } else if seconds < 60 {
                Line::from(Span::styled(
                    format!("Updated: {} seconds ago", seconds),
                    Style::default().fg(Color::Green)
                ))
            } else if seconds < 3600 {
                Line::from(Span::styled(
                    format!("Updated: {} minutes ago", seconds / 60),
                    Style::default().fg(Color::Yellow)
                ))
            } else {
                Line::from(Span::styled(
                    format!("Updated: {} hours ago", seconds / 3600),
                    Style::default().fg(Color::Red)
                ))
            }
        }
        (false, None) => {
            Line::from(Span::styled(
                "Not updated yet",
                Style::default().fg(Color::Red)
            ))
        }
    };
    
    // Add help text
    let help_text = Line::from(Span::raw(" Press ? for help"));
    
    // Create a layout for status and help
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);
    
    // Render status
    let status = Paragraph::new(status_text)
        .style(Style::default());
    f.render_widget(status, chunks[0]);
    
    // Render help reminder
    let help = Paragraph::new(help_text)
        .style(Style::default());
    f.render_widget(help, chunks[1]);
}

/// Draw the help screen
fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.size());
    
    let help_text = vec![
        Line::from(Span::styled("Help", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw("Navigation:")),
        Line::from(Span::raw("  Tab, Right - Next tab")),
        Line::from(Span::raw("  Shift+Tab, Left - Previous tab")),
        Line::from(Span::raw("  1-6 - Select tab")),
        Line::from(""),
        Line::from(Span::raw("Scrolling:")),
        Line::from(Span::raw("  Up, Down - Scroll current view")),
        Line::from(Span::raw("  Page Up, Page Down - Scroll page")),
        Line::from(Span::raw("  Home, End - Scroll to top/bottom")),
        Line::from(""),
        Line::from(Span::raw("Other:")),
        Line::from(Span::raw("  ? - Toggle help")),
        Line::from(Span::raw("  q, Esc - Quit")),
        Line::from(Span::raw("  r - Refresh data")),
    ];
    
    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Help");
    
    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(Style::default());
    
    f.render_widget(help_paragraph, area);
}

/// Helper function to create a centered rect
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