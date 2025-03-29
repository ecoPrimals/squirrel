use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::NetworkWidget;

/// Draw the network tab
pub(super) fn draw_network_tab(f: &mut Frame, app: &App, area: Rect) {
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