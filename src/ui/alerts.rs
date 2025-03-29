use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::widgets::AlertsWidget;

/// Draw the alerts tab
pub(super) fn draw_alerts_tab(f: &mut Frame, app: &App, area: Rect) {
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