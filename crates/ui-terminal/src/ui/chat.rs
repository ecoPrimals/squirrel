use ratatui::{
    backend::Backend,
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    prelude::Alignment,
};

use crate::app::chat::ChatApp;
use crate::widgets::chat;
use dashboard_core::service::DashboardService;

/// Render the chat UI for a given app state
pub fn render<B: Backend, S: DashboardService + ?Sized>(f: &mut Frame, app: &ChatApp<S>) {
    let size = f.size();

    // If help is being shown, just render the help screen over everything
    if app.show_help {
        draw_help::<B>(f, size);
        return;
    }

    // Create the main layout with header, chat area, and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(1),     // Chat area 
            Constraint::Length(1),  // Footer
        ])
        .split(size);

    // Render header
    render_header::<B>(f, chunks[0]);
    
    // Render chat area using the chat widget
    crate::widgets::chat::render::<B>(f, chunks[1], &app.state);
    
    // Render footer
    render_footer::<B>(f, chunks[2]);
}

/// Render the header with title and status
fn render_header<B: Backend>(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Squirrel AI Chat")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    // Create title and status text
    let title = Line::from(vec![
        Span::styled(
            "Squirrel AI Chat Terminal",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ]);

    let paragraph = Paragraph::new(title)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Render the footer with key bindings
fn render_footer<B: Backend>(f: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" Send | "),
        Span::styled("Esc", Style::default().fg(Color::Green)),
        Span::raw(" Clear | "),
        Span::styled("?", Style::default().fg(Color::Green)),
        Span::raw(" Help | "),
        Span::styled("q", Style::default().fg(Color::Green)),
        Span::raw(" Quit"),
    ]);

    let paragraph = Paragraph::new(text)
        .style(Style::default().bg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw help overlay
fn draw_help<B: Backend>(f: &mut Frame, area: Rect) {
    let help_area = centered_rect(80, 70, area);
    
    let help_text = vec![
        "Squirrel AI Chat Controls:",
        "",
        "Message Input:",
        "  Enter     : Send message",
        "  Backspace : Delete character",
        "  ← / →     : Move cursor",
        "  Ctrl+A    : Move cursor to start",
        "  Ctrl+E    : Move cursor to end",
        "",
        "General:",
        "  q, Ctrl+C : Quit",
        "  h         : Toggle help",
        "",
        "Press any key to close help",
    ].join("\n");
    
    let help_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Thick)
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