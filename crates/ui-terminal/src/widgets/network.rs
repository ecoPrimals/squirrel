// crates/ui-terminal/src/widgets/network.rs
// Placeholder for NetworkWidget implementation

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
    let widget = Paragraph::new("Network Widget Placeholder")
        .block(Block::default().borders(Borders::ALL).title("Network Interfaces"));
    frame.render_widget(widget, area);
} 