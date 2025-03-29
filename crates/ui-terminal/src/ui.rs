use crate::app::{App, ActiveTab};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};
use crate::widgets; // Import widgets module

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_>) {
    // Define main layout: Title, Content, Footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Footer/Status bar
        ].as_ref())
        .split(frame.size());

    render_title_bar::<B>(app, frame, main_chunks[0]);
    render_content::<B>(app, frame, main_chunks[1]);
    render_footer::<B>(app, frame, main_chunks[2]);

    if app.state.show_help {
        render_help_popup::<B>(app, frame, frame.size());
    }
}

fn render_title_bar<B: Backend>(app: &App, frame: &mut Frame<'_>, area: Rect) {
    let titles = ["Overview", "System", "Network", "Protocol", "Alerts"];
    let tabs = Tabs::new(titles.iter().cloned().map(Line::from).collect::<Vec<_>>())
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app.state.active_tab.clone() as usize)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black), // Highlight background
        );

    frame.render_widget(tabs, area);
    // TODO: Add status indicators (connection, errors) to the title bar?
}

fn render_content<B: Backend>(app: &App, frame: &mut Frame<'_>, area: Rect) {
    match app.state.active_tab {
        ActiveTab::Overview => render_overview_tab::<B>(app, frame, area),
        // ActiveTab::System => { /* render system tab */ }
        // ActiveTab::Network => { /* render network tab */ }
        // ActiveTab::Protocol => { /* render protocol tab */ }
        // ActiveTab::Alerts => { /* render alerts tab */ }
        _ => { // Default for unimplemented tabs
            let placeholder = Paragraph::new("Tab content not yet implemented.")
                .block(Block::default().borders(Borders::ALL).title("Placeholder"));
            frame.render_widget(placeholder, area);
        }
    }
}

fn render_overview_tab<B: Backend>(app: &App, frame: &mut Frame<'_>, area: Rect) {
    // Example layout for Overview tab (e.g., 2x2 grid)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    // --- Render Health Widget --- 
    let health_checks = app.get_health_checks();
    widgets::health::render::<B>(frame, left_chunks[0], &health_checks);
    // ---------------------------

    // --- Render Metrics Widget ---
    widgets::metrics::render::<B>(frame, app, left_chunks[1]);
    // ---------------------------

    // --- Render CPU Chart --- 
    widgets::chart::render::<B>(
        frame,
        right_chunks[0], // Area for the chart
        "CPU Usage (%)",
        &app.state.cpu_history, // Pass the CPU history data
    );
    // ------------------------

    // --- Render Memory Chart --- 
    widgets::chart::render::<B>(
        frame,
        right_chunks[1], // Area for the chart
        "Memory Usage (%)",
        &app.state.memory_history, // Pass the Memory history data
    );
    // -------------------------

    // TODO: Call actual widget render functions for charts
    // e.g., widgets::chart::render(frame, app, right_chunks[0], "CPU Usage");
}

fn render_footer<B: Backend>(app: &App, frame: &mut Frame<'_>, area: Rect) {
    let connection_status_text = format!(
        "Connection: {:?}", // Use Debug format
        app.state.connection_status
    );
    let status_text = format!(
        "Status: {} | Last Update: {} | Errors: {} | Press 'h' for help, 'q' to quit",
        connection_status_text,
        app.state.last_update.map_or_else(|| "N/A".to_string(), |ts| ts.format("%H:%M:%S").to_string()),
        app.state.recent_errors.len()
    );
    let footer = Paragraph::new(status_text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, area);
}

fn render_help_popup<B: Backend>(_app: &App, frame: &mut Frame<'_>, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Help", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))),
        Line::from(""),
        Line::from(" [1-5] Switch Tabs"),
        Line::from("   [h] Toggle Help"),
        Line::from("   [q] Quit"),
    ];

    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black)); // Popup background

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(Color::White));

    // Calculate centered rect (use helper from util.rs)
    // let popup_area = crate::util::centered_rect(60, 50, area);
    let popup_area = Rect::new(area.width / 4, area.height / 4, area.width / 2, area.height / 2); // Simple approximation

    frame.render_widget(Clear, popup_area); // Clear underlying area
    frame.render_widget(paragraph, popup_area);
} 