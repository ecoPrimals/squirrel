use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App;

/// Draw the tools tab
pub(super) fn draw_tools_tab(f: &mut Frame, app: &App, area: Rect) {
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