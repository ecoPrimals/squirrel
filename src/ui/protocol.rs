use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::ProtocolWidget;

/// Draw the protocol tab
pub(super) fn draw_protocol_tab(f: &mut Frame, app: &App, area: Rect) {
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