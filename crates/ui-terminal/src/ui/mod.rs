use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs, Paragraph, BorderType},
    prelude::Alignment,
};

use crate::app::{App, AppTab};
use crate::widgets::health::render as render_health_checks;
use crate::widgets::alerts::render as render_alerts;
use crate::widgets::network::render as render_network_stats;
use crate::widgets::protocol::render as render_protocol_stats;
use crate::widgets::system::render as render_system_stats;
use dashboard_core::service::DashboardService;

// Export the chat UI module
pub mod chat;

/// Render the UI based on the current application state
pub fn render<B, S>(
    app: &App<S>, 
    f: &mut Frame
) where 
    B: Backend,
    S: DashboardService + Send + Sync + 'static + ?Sized,
{
    let size = f.size();

    // If help is being shown, just render the help screen over everything
    if app.state.show_help {
        draw_help(f, size);
        return;
    }

    // Create the main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Tab bar
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(size);

    // Draw the tabs
    draw_tabs(app, f, chunks[0]);

    // Draw the tab content based on the active tab
    match app.state.active_tab {
        AppTab::Overview => draw_overview(app, f, chunks[1]),
        AppTab::System => draw_system_tab(app, f, chunks[1]),
        AppTab::Network => draw_network_tab(app, f, chunks[1]),
        AppTab::Protocol => draw_protocol_tab(app, f, chunks[1]),
        AppTab::Alerts => draw_alerts_tab(app, f, chunks[1]),
    }

    // Draw footer
    draw_footer(f, chunks[2]);
}

/// Draw the top tab bar
fn draw_tabs<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    let titles = app.tabs.iter().map(|t| {
        Span::styled(
            format!(" {} ", t),
            Style::default().fg(Color::White)
        )
    }).collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Dashboard"))
        .select(app.state.active_tab as usize)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );

    f.render_widget(tabs, area);
}

/// Draw the overview tab content
fn draw_overview<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    // Split the area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Further split the top and bottom sections
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    // Render widgets in each section
    render_health_checks(f, top_chunks[0], &[]);
    render_system_stats(f, top_chunks[1], app.state.metrics.as_ref());
    render_network_stats(f, bottom_chunks[0], app.state.metrics.as_ref());
    render_protocol_stats(f, bottom_chunks[1], app.state.protocol_data.as_ref());
}

/// Draw the system tab content
fn draw_system_tab<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    let inner_area = Block::default().title("System").borders(Borders::ALL).inner(area);
    render_system_stats(f, inner_area, app.state.metrics.as_ref());
}

/// Draw the network tab content
fn draw_network_tab<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    let inner_area = Block::default().title("Network").borders(Borders::ALL).inner(area);
    render_network_stats(f, inner_area, app.state.metrics.as_ref());
}

/// Draw the protocol tab content
fn draw_protocol_tab<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    let inner_area = Block::default().title("Protocol").borders(Borders::ALL).inner(area);
    render_protocol_stats(f, inner_area, app.state.protocol_data.as_ref());
}

/// Draw the alerts tab content
fn draw_alerts_tab<S: DashboardService + Send + Sync + 'static + ?Sized>(
    app: &App<S>,
    f: &mut Frame,
    area: Rect,
) {
    let inner_area = Block::default().title("Alerts").borders(Borders::ALL).inner(area);
    render_alerts(f, inner_area, app.state.alerts.as_ref());
}

/// Draw the footer area
fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = "Press 'q' to quit, 'h' for help";
    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}

/// Draw help overlay
fn draw_help(f: &mut Frame, area: Rect) {
    let help_area = centered_rect(80, 70, area);
    
    let help_text = vec![
        "Squirrel Dashboard Controls:",
        "",
        "Tab Navigation:",
        "  → / ←     : Navigate between tabs",
        "  1-5       : Jump directly to a tab",
        "",
        "General:",
        "  q, Ctrl+C : Quit",
        "  h         : Toggle help",
        "  r         : Refresh data",
        "",
        "Press any key to close help",
    ].join("\n");
    
    let help_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Yellow));
    
    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    
    f.render_widget(help_paragraph, help_area);
}

/// Helper function to create a centered rect using up certain percentage of the available rect
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

/// Creates a rectangle that excludes the tab navigation area
fn inner_area_below_tabs(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y + 3, // Assuming tab navigation takes 3 vertical rows
        width: area.width,
        height: area.height - 3,
    }
} 