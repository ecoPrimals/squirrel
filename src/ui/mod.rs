use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::App;
// Removed widget imports as they will be used in submodules

// Declare the UI submodules
mod overview;
mod system;
mod protocol;
mod tools;
mod alerts;
mod network;
mod status_bar;
mod help;

// Import necessary functions from submodules
use self::overview::draw_overview_tab;
use self::system::draw_system_tab;
use self::protocol::draw_protocol_tab;
use self::tools::draw_tools_tab;
use self::alerts::draw_alerts_tab;
use self::network::draw_network_tab;
use self::status_bar::draw_status_bar;
use self::help::draw_help;

/// Draw the main UI layout
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
    
    // Draw content based on selected tab by calling functions from submodules
    match app.selected_tab() {
        0 => draw_overview_tab(f, app, chunks[1]),
        1 => draw_system_tab(f, app, chunks[1]),
        2 => draw_protocol_tab(f, app, chunks[1]),
        3 => draw_tools_tab(f, app, chunks[1]),
        4 => draw_alerts_tab(f, app, chunks[1]),
        5 => draw_network_tab(f, app, chunks[1]),
        _ => {} // Do nothing for unimplemented tabs
    }
    
    // Draw status bar
    draw_status_bar(f, app, chunks[2]);
    
    // Draw help screen if visible
    if app.show_help() {
        draw_help(f);
    }
}

/// Draw the tabs widget (kept in mod.rs as it's closely tied to the main layout)
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