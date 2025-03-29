// crates/ui-terminal/src/widgets/connection_health.rs
// Placeholder for ConnectionHealthWidget implementation

use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::App; // App is no longer generic

pub fn render<B: Backend>(
    _app: &App, // Changed &App<P> to &App
    frame: &mut Frame<'_>,
    area: Rect
) {
    let widget = Paragraph::new("Connection Health Widget Placeholder")
        .block(Block::default().borders(Borders::ALL).title("Connection Health"));
    frame.render_widget(widget, area);
} 