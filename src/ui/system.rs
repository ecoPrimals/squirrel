use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::{ChartWidget, ChartType, MetricsWidget};

/// Draw the system tab
pub(super) fn draw_system_tab(f: &mut Frame, app: &App, area: Rect) {
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