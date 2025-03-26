use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
    Frame,
};

use crate::app::App;
use crate::widgets::metrics::MetricsWidget;
use crate::widgets::alerts::AlertsWidget;
use crate::widgets::health::HealthWidget;
use crate::widgets::network::NetworkWidget;

/// Draw the UI
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
        .map(|t| Spans::from(Span::styled(t, Style::default().fg(Color::White))))
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
fn draw_overview_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
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
        // Draw health widget
        let health_widget = HealthWidget::new(&data.health);
        f.render_widget(health_widget, top_chunks[0]);
        
        // Draw system metrics
        let system_widget = MetricsWidget::new(&data.system_metrics, "System");
        f.render_widget(system_widget, bottom_chunks[0]);
        
        // Draw protocol metrics
        let protocol_widget = MetricsWidget::new_protocol(&data.protocol_metrics, "Protocol");
        f.render_widget(protocol_widget, bottom_chunks[1]);
        
        // Draw network metrics
        let network_widget = NetworkWidget::new(&data.network, "Network");
        f.render_widget(network_widget, bottom_chunks[2]);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading dashboard data...")
            .block(Block::default().borders(Borders::ALL).title("Dashboard"));
        f.render_widget(loading, area);
    }
}

/// Draw the system tab
fn draw_system_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create detailed system metrics widget
        let system_widget = MetricsWidget::new_detailed(&data.system_metrics, "System Metrics");
        f.render_widget(system_widget, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading system metrics...")
            .block(Block::default().borders(Borders::ALL).title("System"));
        f.render_widget(loading, area);
    }
}

/// Draw the protocol tab
fn draw_protocol_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create detailed protocol metrics widget
        let protocol_widget = MetricsWidget::new_protocol_detailed(&data.protocol_metrics, "Protocol Metrics");
        f.render_widget(protocol_widget, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading protocol metrics...")
            .block(Block::default().borders(Borders::ALL).title("Protocol"));
        f.render_widget(loading, area);
    }
}

/// Draw the tools tab
fn draw_tools_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
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
fn draw_alerts_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create alerts widget
        let alerts_widget = AlertsWidget::new(&data.alerts, "Alerts", app.scroll_positions().alerts);
        f.render_widget(alerts_widget, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading alerts...")
            .block(Block::default().borders(Borders::ALL).title("Alerts"));
        f.render_widget(loading, area);
    }
}

/// Draw the network tab
fn draw_network_tab<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    if let Some(data) = app.dashboard_data() {
        // Create detailed network metrics widget
        let network_widget = NetworkWidget::new_detailed(&data.network, "Network Metrics");
        f.render_widget(network_widget, area);
    } else {
        // Show loading message if no data available
        let loading = Paragraph::new("Loading network metrics...")
            .block(Block::default().borders(Borders::ALL).title("Network"));
        f.render_widget(loading, area);
    }
}

/// Draw the status bar
fn draw_status_bar<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    // Format update time
    let update_time = match app.time_since_update() {
        Some(duration) => {
            let seconds = duration.as_secs();
            if seconds < 60 {
                format!("{}s ago", seconds)
            } else if seconds < 3600 {
                format!("{}m {}s ago", seconds / 60, seconds % 60)
            } else {
                format!("{}h {}m ago", seconds / 3600, (seconds % 3600) / 60)
            }
        }
        None => "Never".to_string(),
    };
    
    // Create status message
    let status = if app.is_updating() {
        format!("Updating... | Last update: {}", update_time)
    } else {
        format!("Press '?' for help | Last update: {}", update_time)
    };
    
    // Draw status bar
    let status_style = Style::default().fg(Color::White);
    let status_bar = Paragraph::new(status).style(status_style);
    f.render_widget(status_bar, area);
}

/// Draw the help popup
fn draw_help<B: Backend>(f: &mut Frame<B>, ) {
    // Create help popup in the center of the screen
    let area = centered_rect(60, 40, f.size());
    
    // Create help text
    let text = vec![
        Spans::from(Span::styled("Dashboard Help", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Spans::from(""),
        Spans::from(Span::styled("Navigation", Style::default().fg(Color::Green))),
        Spans::from("  1-6        - Select tab"),
        Spans::from("  Tab        - Next tab"),
        Spans::from("  Shift+Tab  - Previous tab"),
        Spans::from(""),
        Spans::from(Span::styled("Scrolling", Style::default().fg(Color::Green))),
        Spans::from("  j / Down   - Scroll down"),
        Spans::from("  k / Up     - Scroll up"),
        Spans::from(""),
        Spans::from(Span::styled("Actions", Style::default().fg(Color::Green))),
        Spans::from("  r          - Refresh data"),
        Spans::from("  ?          - Toggle help"),
        Spans::from("  q / Ctrl+c - Quit"),
    ];
    
    // Draw help popup
    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Left);
    
    f.render_widget(help, area);
}

/// Helper function to create a centered rect using up certain percentage of the available area
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