use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::{HealthWidget, MetricsWidget, NetworkWidget};

/// Draw the overview tab
pub(super) fn draw_overview_tab(f: &mut Frame, app: &App, area: Rect) {
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
        // TODO: Replace with actual health check data retrieval if available
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