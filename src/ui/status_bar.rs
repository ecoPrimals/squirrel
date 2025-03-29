use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;

/// Draw the status bar
pub(super) fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match (app.is_updating(), app.time_since_update()) {
        (true, _) => {
            Line::from(Span::styled(
                "Updating...",
                Style::default().fg(Color::Yellow)
            ))
        }
        (false, Some(duration)) => {
            let seconds = duration.as_secs();
            if seconds < 5 {
                Line::from(Span::styled(
                    format!("Updated: Just now"),
                    Style::default().fg(Color::Green)
                ))
            } else if seconds < 60 {
                Line::from(Span::styled(
                    format!("Updated: {} seconds ago", seconds),
                    Style::default().fg(Color::Green)
                ))
            } else if seconds < 3600 {
                Line::from(Span::styled(
                    format!("Updated: {} minutes ago", seconds / 60),
                    Style::default().fg(Color::Yellow)
                ))
            } else {
                Line::from(Span::styled(
                    format!("Updated: {} hours ago", seconds / 3600),
                    Style::default().fg(Color::Red)
                ))
            }
        }
        (false, None) => {
            Line::from(Span::styled(
                "Not updated yet",
                Style::default().fg(Color::Red)
            ))
        }
    };

    // Add help text
    let help_text = Line::from(Span::raw(" Press ? for help"));

    // Create a layout for status and help
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Render status
    let status = Paragraph::new(status_text)
        .style(Style::default());
    f.render_widget(status, chunks[0]);

    // Render help reminder
    let help = Paragraph::new(help_text)
        .style(Style::default());
    f.render_widget(help, chunks[1]);
} 